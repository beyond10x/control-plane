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
        component_tests(&frontend, &temporary.path().join("vitest.json"))?;
    }
    Ok(())
}

/// `npm run test` is `vitest run`, which fails on a failing test, on a test file without tests
/// and when it finds no test file. It exits 0 when a test is skipped or todo, so the JSON report
/// it also writes must show that every collected test passed, and that at least one did.
fn component_tests(frontend: &Path, report: &Path) -> Result<()> {
    let mut report_option = OsString::from("--outputFile.json=");
    report_option.push(report);
    let output = Command::new("npm")
        .current_dir(frontend)
        .args(["run", "test", "--", "--reporter=default", "--reporter=json"])
        .arg(report_option)
        .output()
        .context("start Vue component tests; run npm ci --prefix frontend first")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    ensure!(
        output.status.success(),
        "Vue component tests failed:\n{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let (passed, unrun) = outcomes(frontend, report)?;
    ensure!(
        unrun.is_empty(),
        "Vue component tests failed: Vitest exited 0, but these tests did not run:\n  {}\n{stdout}",
        unrun.join("\n  ")
    );
    ensure!(
        passed > 0,
        "Vue component tests failed: Vitest exited 0, but no test passed\n{stdout}"
    );
    print!("{stdout}");
    println!("Vue component tests passed");
    Ok(())
}

/// The number of passed tests in Vitest's JSON report, and every other test as
/// `file > suite > title (status)`.
fn outcomes(frontend: &Path, report: &Path) -> Result<(usize, Vec<String>)> {
    let report: Value = serde_json::from_slice(
        &fs::read(report).context("Vitest exited 0 without writing its JSON report")?,
    )
    .context("parse Vitest's JSON report")?;
    let base = fs::canonicalize(frontend)?;
    let (mut passed, mut unrun) = (0, Vec::new());
    for file in report["testResults"]
        .as_array()
        .context("Vitest's JSON report has no testResults")?
    {
        let path = Path::new(file["name"].as_str().unwrap_or("<unnamed file>"));
        let path = path.strip_prefix(&base).unwrap_or(path);
        for test in file["assertionResults"]
            .as_array()
            .context("Vitest's JSON report has a file without assertionResults")?
        {
            let status = test["status"].as_str().unwrap_or("without status");
            if status == "passed" {
                passed += 1;
                continue;
            }
            let mut title: Vec<&str> = test["ancestorTitles"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect();
            title.push(test["title"].as_str().unwrap_or("<untitled>"));
            unrun.push(format!(
                "{} > {} ({status})",
                path.display(),
                title.join(" > ")
            ));
        }
    }
    Ok((passed, unrun))
}
