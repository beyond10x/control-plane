//! Bounded host-owned processes. No shell, inherited working directory or ignored exit status.
use anyhow::{Context, Result, ensure};
use nix::{
    sys::signal::{Signal, killpg},
    unistd::Pid,
};
use std::{
    ffi::OsString,
    io::{Read, Write},
    os::unix::process::CommandExt,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;

/// An observed unsuccessful process exit, distinct from cancellation and host failures.
#[derive(Debug)]
pub struct ProcessExit {
    pub program: String,
    pub args: Vec<String>,
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl std::fmt::Display for ProcessExit {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{} {:?} exited {:?}:\nstdout:\n{}\nstderr:\n{}",
            self.program, self.args, self.code, self.stdout, self.stderr
        )
    }
}

impl std::error::Error for ProcessExit {}

/// A process stopped at a host limit. Distinct from cancellation, which stays fatal.
#[derive(Debug)]
pub enum ProcessLimit {
    TimedOut { program: String, seconds: u64 },
    OutputTooLarge,
    OutputNotUtf8,
}

impl ProcessLimit {
    pub fn code(&self) -> &'static str {
        match self {
            Self::TimedOut { .. } => "command_timed_out",
            Self::OutputTooLarge => "command_output_too_large",
            Self::OutputNotUtf8 => "command_output_not_utf8",
        }
    }
}

impl std::fmt::Display for ProcessLimit {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TimedOut { program, seconds } => {
                write!(
                    formatter,
                    "{program} exceeded its {seconds} s process timeout and was killed"
                )
            }
            Self::OutputTooLarge => formatter
                .write_str("process output exceeded 2 MiB; refusing a truncated observation"),
            Self::OutputNotUtf8 => formatter.write_str("process output is not UTF-8"),
        }
    }
}

impl std::error::Error for ProcessLimit {}

/// Service variables a host-started process inherits: program lookup, user directories,
/// locale and toolchain or build caches. Everything else is withheld, because test and
/// tool commands run model-written code. Trusted commit and publish commands receive
/// credentials explicitly through `RuntimeConfig::credentials`.
pub const INHERITED_ENVIRONMENT: &[&str] = &[
    "PATH",
    "HOME",
    "USER",
    "LOGNAME",
    "LANG",
    "LANGUAGE",
    "LC_ALL",
    "LC_CTYPE",
    "LC_MESSAGES",
    "TZ",
    "TMPDIR",
    "XDG_CONFIG_HOME",
    "XDG_CACHE_HOME",
    "XDG_DATA_HOME",
    "XDG_STATE_HOME",
    "CARGO_HOME",
    "RUSTUP_HOME",
    "RUSTUP_TOOLCHAIN",
    "GOPATH",
    "GOCACHE",
    "GOMODCACHE",
    "GOROOT",
    "GOTOOLCHAIN",
    "GOFLAGS",
];

/// The complete environment of a host-started process: allowlisted parent variables,
/// then configured entries, which override inherited ones of the same name.
pub fn child_environment(
    parent: impl IntoIterator<Item = (OsString, OsString)>,
    configured: &[(String, String)],
) -> Vec<(OsString, OsString)> {
    let mut environment: Vec<(OsString, OsString)> = parent
        .into_iter()
        .filter(|(name, _)| {
            name.to_str()
                .is_some_and(|name| INHERITED_ENVIRONMENT.contains(&name))
        })
        .filter(|(name, _)| !configured.iter().any(|(key, _)| name == key.as_str()))
        .collect();
    environment.extend(
        configured
            .iter()
            .map(|(key, value)| (key.into(), value.into())),
    );
    environment
}

#[derive(Clone, Debug)]
pub struct ProcessRunner {
    pub environment: Vec<(String, String)>,
    pub timeout: Duration,
    pub cancel: CancellationToken,
}

impl ProcessRunner {
    pub fn run(
        &self,
        cwd: &Path,
        program: &str,
        args: &[String],
        stdin: Option<&str>,
    ) -> Result<String> {
        ensure!(
            !self.cancel.is_cancelled(),
            "execution cancelled before {program}"
        );
        if program == "aep" {
            confine_aep_store(cwd)?;
        }
        let mut child = Command::new(program)
            .args(args)
            .current_dir(cwd)
            .env_clear()
            .envs(child_environment(std::env::vars_os(), &self.environment))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
            .spawn()
            .with_context(|| format!("start {program}"))?;
        let pid = Pid::from_raw(i32::try_from(child.id())?);
        let stdout = child.stdout.take().context("stdout not piped")?;
        let stderr = child.stderr.take().context("stderr not piped")?;
        let out = thread::spawn(move || collect(stdout));
        let err = thread::spawn(move || collect(stderr));
        let input = stdin.unwrap_or("").as_bytes().to_vec();
        let pipe = child.stdin.take().context("stdin not piped")?;
        let writer = thread::spawn(move || {
            let mut pipe = pipe;
            pipe.write_all(&input)
        });
        let started = Instant::now();
        let mut cancelled = false;
        let mut timed_out = false;
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            cancelled = self.cancel.is_cancelled();
            timed_out = started.elapsed() >= self.timeout;
            if cancelled || timed_out {
                let _ = killpg(pid, Signal::SIGKILL);
                break child.wait()?;
            }
            thread::sleep(Duration::from_millis(10));
        };
        // Descendants may retain a pipe even after the direct child exited.
        let _ = killpg(pid, Signal::SIGKILL);
        let stdout = out
            .join()
            .map_err(|_| anyhow::anyhow!("stdout reader panicked"))??;
        let stderr = err
            .join()
            .map_err(|_| anyhow::anyhow!("stderr reader panicked"))??;
        let written = writer
            .join()
            .map_err(|_| anyhow::anyhow!("stdin writer panicked"))?;
        ensure!(!cancelled, "{program} cancelled during execution");
        if timed_out {
            return Err(ProcessLimit::TimedOut {
                program: program.to_owned(),
                seconds: self.timeout.as_secs(),
            }
            .into());
        }
        if !status.success() {
            return Err(ProcessExit {
                program: program.to_owned(),
                args: args.to_vec(),
                code: status.code(),
                stdout,
                stderr,
            }
            .into());
        }
        written.context("write command input")?;
        Ok(stdout)
    }

    pub fn command(&self, cwd: &Path, program: &str, args: &[&str]) -> Result<String> {
        self.run(
            cwd,
            program,
            &args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>(),
            None,
        )
    }
}

// AEP may touch an artifact, evidence or journal selected by its store. Check the
// complete existing tree before reads as well as writes, including adoption.
fn confine_aep_store(cwd: &Path) -> Result<()> {
    fn visit(path: &Path, remaining: &mut usize) -> Result<()> {
        ensure!(
            *remaining > 0,
            "AEP store confinement exceeds 100000 entries"
        );
        *remaining -= 1;
        let metadata = std::fs::symlink_metadata(path)?;
        ensure!(
            !metadata.file_type().is_symlink(),
            "AEP store symlink refused: {}",
            path.display()
        );
        ensure!(
            metadata.is_file() || metadata.is_dir(),
            "AEP store special file refused: {}",
            path.display()
        );
        if metadata.is_dir() {
            for entry in std::fs::read_dir(path)? {
                visit(&entry?.path(), remaining)?;
            }
        }
        Ok(())
    }
    let store = cwd.join(".engineering");
    match std::fs::symlink_metadata(&store) {
        Ok(_) => visit(&store, &mut 100_000),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn collect(mut input: impl Read) -> Result<String> {
    const LIMIT: usize = 2 * 1024 * 1024;
    let mut bytes = Vec::new();
    let mut oversized = false;
    let mut chunk = [0; 8192];
    loop {
        let n = input.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        if bytes.len() + n <= LIMIT {
            bytes.extend_from_slice(&chunk[..n]);
        } else {
            oversized = true;
        }
    }
    if oversized {
        return Err(ProcessLimit::OutputTooLarge.into());
    }
    String::from_utf8(bytes).map_err(|_| ProcessLimit::OutputNotUtf8.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_exit_retains_actual_status_for_syntax_recovery() {
        let cwd = tempfile::tempdir().unwrap();
        let runner = ProcessRunner {
            environment: Vec::new(),
            timeout: Duration::from_secs(10),
            cancel: CancellationToken::new(),
        };
        let error = runner
            .command(cwd.path(), "aep", &["plan", "artifact", "show"])
            .unwrap_err();
        let observed = error
            .downcast_ref::<ProcessExit>()
            .expect("process exit lost its structured status");
        assert_eq!(observed.program, "aep");
        assert_eq!(observed.code, Some(2));
        assert!(observed.stderr.contains("Usage:"));
        assert!(observed.stderr.contains("--help"));

        runner.cancel.cancel();
        let cancelled = runner.command(cwd.path(), "aep", &["--help"]).unwrap_err();
        assert!(
            cancelled.downcast_ref::<ProcessExit>().is_none(),
            "cancellation must not become syntax feedback"
        );
    }

    #[test]
    fn child_environment_keeps_only_allowlisted_and_configured_variables() {
        let parent = [
            ("PATH", "/usr/bin"),
            ("HOME", "/home/operator"),
            ("OPENAI_API_KEY", "service-secret"),
            ("SSH_AUTH_SOCK", "/run/agent"),
            ("XDG_STATE_HOME", "/home/operator/.local/state"),
        ]
        .map(|(name, value)| (OsString::from(name), OsString::from(value)));
        let configured = [
            ("XDG_STATE_HOME".to_owned(), "/scratch/state".to_owned()),
            ("CONTROL_PLANE_TARGET".to_owned(), "main".to_owned()),
        ];
        let mut environment = child_environment(parent, &configured)
            .into_iter()
            .map(|(name, value)| (name.into_string().unwrap(), value.into_string().unwrap()))
            .collect::<Vec<_>>();
        environment.sort();
        assert_eq!(
            environment,
            [
                ("CONTROL_PLANE_TARGET", "main"),
                ("HOME", "/home/operator"),
                ("PATH", "/usr/bin"),
                ("XDG_STATE_HOME", "/scratch/state"),
            ]
            .map(|(name, value)| (name.to_owned(), value.to_owned()))
        );
    }

    #[test]
    fn execution_limits_are_typed_and_stop_the_process_group() {
        let cwd = tempfile::tempdir().unwrap();
        let runner = ProcessRunner {
            environment: Vec::new(),
            timeout: Duration::from_millis(300),
            cancel: CancellationToken::new(),
        };
        let marker = cwd.path().join("background.pid");
        let script = format!("sleep 30 & echo $! > {}; wait", marker.display());
        let error = runner
            .command(cwd.path(), "sh", &["-c", &script])
            .unwrap_err();
        assert!(matches!(
            error.downcast_ref::<ProcessLimit>(),
            Some(ProcessLimit::TimedOut { .. })
        ));
        let pid = std::fs::read_to_string(&marker).unwrap();
        let status = format!("/proc/{}/status", pid.trim());
        let stopped = (0..100).any(|_| {
            let gone =
                std::fs::read_to_string(&status).map_or(true, |state| state.contains("State:\tZ"));
            if !gone {
                thread::sleep(Duration::from_millis(10));
            }
            gone
        });
        assert!(stopped, "background process survived its group's timeout");

        let runner = ProcessRunner {
            timeout: Duration::from_secs(10),
            ..runner
        };
        let large = runner
            .command(cwd.path(), "head", &["-c", "3000000", "/dev/zero"])
            .unwrap_err();
        assert!(matches!(
            large.downcast_ref::<ProcessLimit>(),
            Some(ProcessLimit::OutputTooLarge)
        ));
        let encoded = runner
            .command(cwd.path(), "printf", &["\\377"])
            .unwrap_err();
        assert!(matches!(
            encoded.downcast_ref::<ProcessLimit>(),
            Some(ProcessLimit::OutputNotUtf8)
        ));
        runner.cancel.cancel();
        let cancelled = runner.command(cwd.path(), "true", &[]).unwrap_err();
        assert!(cancelled.downcast_ref::<ProcessLimit>().is_none());
    }

    #[test]
    fn processes_start_from_allowlisted_environment() {
        let runner = ProcessRunner {
            environment: vec![("CONTROL_PLANE_PROBE".into(), "configured".into())],
            timeout: Duration::from_secs(10),
            cancel: CancellationToken::new(),
        };
        let cwd = tempfile::tempdir().unwrap();
        let observed = runner.command(cwd.path(), "env", &[]).unwrap();
        let names = observed
            .lines()
            .filter_map(|line| line.split_once('=').map(|(name, _)| name))
            .collect::<Vec<_>>();
        assert!(names.contains(&"PATH"), "{observed}");
        assert!(observed.contains("CONTROL_PLANE_PROBE=configured"));
        for name in names {
            assert!(
                INHERITED_ENVIRONMENT.contains(&name) || name == "CONTROL_PLANE_PROBE",
                "child process inherited {name}"
            );
        }
    }
}
