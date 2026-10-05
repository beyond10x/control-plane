//! Bounded host-owned processes. No shell, inherited working directory or ignored exit status.
use anyhow::{Context, Result, bail, ensure};
use nix::{
    sys::signal::{Signal, killpg},
    unistd::Pid,
};
use std::{
    io::{Read, Write},
    os::unix::process::CommandExt,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;

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
        let mut child = Command::new(program)
            .args(args)
            .current_dir(cwd)
            .envs(self.environment.iter().cloned())
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
        let mut interrupted = false;
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if self.cancel.is_cancelled() || started.elapsed() >= self.timeout {
                interrupted = true;
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
        ensure!(
            !interrupted,
            "{program} cancelled or exceeded process timeout"
        );
        ensure!(
            status.success(),
            "{program} {args:?} exited {status}: {stderr}"
        );
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
        bail!("process output exceeded 2 MiB; refusing a truncated observation");
    }
    String::from_utf8(bytes).context("process output is not UTF-8")
}
