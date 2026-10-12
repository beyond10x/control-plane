//! Every test fixture under the tree's `.scratch` is deleted when its test ends: a test holds its
//! `TempDir` guard for its own duration and never keeps the directory. Only `xtask mutate --keep`
//! keeps one, and that is source, not a test.

use std::{
    fs,
    path::{Path, PathBuf},
};

/// Every Rust source under `dir`, recursively.
fn sources(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
}

#[test]
fn tests_keep_no_scratch_fixtures() {
    // Spelled apart so this file does not match itself.
    let kept = [concat!(".", "keep()"), concat!(".", "into_path()")];
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut files = Vec::new();
    for krate in fs::read_dir(&crates).unwrap() {
        let tests = krate.unwrap().path().join("tests");
        if tests.is_dir() {
            sources(&tests, &mut files);
        }
    }
    assert!(
        !files.is_empty(),
        "no test sources under {}",
        crates.display()
    );
    let mut keeping = Vec::new();
    for file in files {
        for (index, line) in fs::read_to_string(&file).unwrap().lines().enumerate() {
            if kept.iter().any(|needle| line.contains(needle)) {
                keeping.push(format!("{}:{}: {}", file.display(), index + 1, line.trim()));
            }
        }
    }
    assert!(
        keeping.is_empty(),
        "tests keep their fixture directories:\n{}",
        keeping.join("\n")
    );
}
