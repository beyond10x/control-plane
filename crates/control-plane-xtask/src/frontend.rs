//! Reproducible Vue asset build; serving the committed bundle needs no Node runtime. The check
//! also runs the Vue component suite once the embedded assets match their source.
use anyhow::{Context, Result, ensure};
use serde_json::Value;
use std::{ffi::OsString, fs, path::Path, process::Command};

pub fn run(root: &Path, install: bool) -> Result<()> {
    let frontend = root.join("frontend");
    let scratch = root.join(".scratch");
    fs::create_dir_all(&scratch)?;
    let temporary = tempfile::Builder::new()
        .prefix("frontend-build-")
        .tempdir_in(scratch)?;
    let expected = temporary.path().join("dist");
    let output = Command::new("npm")
        .current_dir(&frontend)
        .args(["run", "build", "--", "--outDir"])
        .arg(if install {
            frontend.join("dist")
        } else {
            expected.clone()
        })
        .arg("--emptyOutDir")
        .output()
        .context("start Vue build; install Node and run npm ci --prefix frontend first")?;
    ensure!(
        output.status.success(),
        "Vue build failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    print!("{}", String::from_utf8_lossy(&output.stdout));
    if !install {
        crate::generation::compare(&expected, &frontend.join("dist"))?;
        println!("embedded Vue assets match the source build");
        component_tests(&frontend, &temporary.path().join("vitest-blob.json"))?;
    }
    Ok(())
}

/// `npm run test` is `vitest run`, which fails on a failing test, on a test file without tests
/// and when it finds no test file. It exits 0 when a test or suite is skipped or todo, and when a
/// `test.fails` test fails as expected. Its JSON report records that last case as passed, so the
/// check reads the blob report instead, which holds the run's own task tree: every suite and test
/// in it must have run as written, every test must have passed, and at least one must exist.
fn component_tests(frontend: &Path, report: &Path) -> Result<()> {
    let mut report_option = OsString::from("--outputFile.blob=");
    report_option.push(report);
    let output = Command::new("npm")
        .current_dir(frontend)
        .args(["run", "test", "--", "--reporter=default", "--reporter=blob"])
        .arg(report_option)
        .output()
        .context("start Vue component tests; run npm ci --prefix frontend first")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    ensure!(
        output.status.success(),
        "Vue component tests failed:\n{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let outcomes = outcomes(frontend, report)?;
    ensure!(
        outcomes.refused.is_empty(),
        "Vue component tests failed: Vitest exited 0, but these did not run and pass as written:\n  {}\n{stdout}",
        outcomes.refused.join("\n  ")
    );
    ensure!(
        outcomes.passed > 0,
        "Vue component tests failed: Vitest exited 0, but no test passed\n{stdout}"
    );
    print!("{stdout}");
    println!("Vue component tests passed");
    Ok(())
}

#[derive(Default)]
struct Outcomes {
    passed: usize,
    /// `file > suite > title (status)` for every task that did not run and pass as written.
    refused: Vec<String>,
}

/// Vitest's blob report, encoded by `flatted`: the root is the first entry, and every string
/// inside an object or array is the index of the entry holding the value. Other values are inline.
struct Blob(Vec<Value>);

impl Blob {
    fn at<'a>(&'a self, value: &'a Value) -> Result<&'a Value> {
        match value {
            Value::String(index) => self
                .0
                .get(
                    index
                        .parse::<usize>()
                        .context("blob reference is not an index")?,
                )
                .context("blob reference points past the report"),
            inline => Ok(inline),
        }
    }

    fn field<'a>(&'a self, object: &'a Value, key: &str) -> Result<Option<&'a Value>> {
        match object.get(key) {
            None | Some(Value::Null) => Ok(None),
            Some(value) => self.at(value).map(Some),
        }
    }

    fn text<'a>(&'a self, object: &'a Value, key: &str) -> Result<Option<&'a str>> {
        self.field(object, key)?
            .map(|value| {
                value
                    .as_str()
                    .with_context(|| format!("blob field {key} is not text"))
            })
            .transpose()
    }

    fn items<'a>(&'a self, value: Option<&'a Value>) -> Result<Vec<&'a Value>> {
        value
            .map_or(&[][..], |value| {
                value.as_array().map_or(&[][..], Vec::as_slice)
            })
            .iter()
            .map(|item| self.at(item))
            .collect()
    }

    /// Records `task` and everything beneath it; `title` is the task's own `file > … > name`.
    fn walk(&self, task: &Value, title: &str, outcomes: &mut Outcomes) -> Result<()> {
        let kind = self.text(task, "type")?;
        let state = match self.field(task, "result")? {
            Some(result) => self.text(result, "state")?,
            None => None,
        };
        let refusal = match self.text(task, "mode")? {
            Some("skip") => Some("skipped"),
            Some("todo") => Some("todo"),
            _ if task.get("fails").and_then(Value::as_bool) == Some(true) => Some("expected fail"),
            _ if kind != Some("test") => None,
            _ if state == Some("pass") => {
                outcomes.passed += 1;
                None
            }
            _ if state == Some("skip") => Some("skipped"),
            _ => Some(state.unwrap_or("not run")),
        };
        if let Some(status) = refusal {
            outcomes.refused.push(format!("{title} ({status})"));
        }
        match kind {
            Some("test") => {}
            Some("suite") => {
                for child in self.items(self.field(task, "tasks")?)? {
                    let name = self.text(child, "name")?.unwrap_or("<untitled>");
                    self.walk(child, &format!("{title} > {name}"), outcomes)?;
                }
            }
            other => outcomes
                .refused
                .push(format!("{title} (unknown task type {other:?})")),
        }
        Ok(())
    }
}

/// Every test file, suite and test in Vitest's blob report, as passed or refused.
fn outcomes(frontend: &Path, report: &Path) -> Result<Outcomes> {
    let blob = Blob(
        serde_json::from_slice(
            &fs::read(report).context("Vitest exited 0 without writing its blob report")?,
        )
        .context("parse Vitest's blob report")?,
    );
    let root = blob.0.first().context("Vitest's blob report is empty")?;
    let base = fs::canonicalize(frontend)?;
    let mut outcomes = Outcomes::default();
    for file in blob.items(Some(blob.at(&root[1])?))? {
        let path = Path::new(blob.text(file, "filepath")?.unwrap_or("<unnamed file>"));
        let path = path.strip_prefix(&base).unwrap_or(path);
        blob.walk(file, &path.display().to_string(), &mut outcomes)?;
    }
    for error in blob.items(Some(blob.at(&root[2])?))? {
        let message = blob.text(error, "message")?.unwrap_or("<no message>");
        outcomes
            .refused
            .push(format!("unhandled error ({message})"));
    }
    Ok(outcomes)
}
