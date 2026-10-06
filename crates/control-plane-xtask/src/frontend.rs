//! Reproducible Vue asset build; serving the committed bundle needs no Node runtime.
use anyhow::{Context, Result, ensure};
use std::{fs, path::Path, process::Command};

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
    }
    Ok(())
}
