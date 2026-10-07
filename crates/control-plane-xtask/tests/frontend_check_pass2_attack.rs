//! Adversarial cases for `frontend-check` (story:console-component-tests, pass 2, head 956ab67).
//!
//! The harness is the one `tests/frontend_check.rs` uses: copy `frontend/` without
//! `node_modules` into a canonical temporary root, link each installed package and `.bin`, change
//! the copy only and run `frontend::run(root, false)`, which is what `frontend-check` runs. The
//! real `frontend/` is only read. Run `npm ci --prefix frontend` first.
#[allow(dead_code)]
#[path = "../src/frontend.rs"]
mod frontend;
#[allow(dead_code)]
#[path = "../src/generation.rs"]
mod generation;

use anyhow::{Context, Result, ensure};
use std::{
    fs,
    path::{Path, PathBuf},
};

const ACCEPTANCE: &str = "test('app_renders_a_snapshot_payload'";
const GOAL: &str = "Deliver the recorded change";

fn copy(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        let kind = entry.file_type()?;
        if entry.file_name() == "node_modules" {
            continue;
        } else if kind.is_dir() {
            copy(&entry.path(), &target)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn frontend_copy(prefix: &str) -> Result<(tempfile::TempDir, PathBuf)> {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../frontend");
    let modules = fs::canonicalize(source.join("node_modules"))
        .context("frontend/node_modules is missing; run npm ci --prefix frontend first")?;
    let temporary = tempfile::Builder::new().prefix(prefix).tempdir()?;
    let root = fs::canonicalize(temporary.path())?;
    copy(&source, &root.join("frontend"))?;
    let linked = root.join("frontend/node_modules");
    fs::create_dir(&linked)?;
    for entry in fs::read_dir(&modules)? {
        let name = entry?.file_name();
        if name == ".bin" || !name.to_string_lossy().starts_with('.') {
            std::os::unix::fs::symlink(modules.join(&name), linked.join(&name))?;
        }
    }
    Ok((temporary, root))
}

/// Remove every file Vitest's default include would collect (`*.test.*` and `*.spec.*`) under
/// `dir`, so the case holds however many suites the console has; returns how many were removed.
fn remove_component_tests(dir: &Path) -> Result<usize> {
    let mut removed = 0;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.file_type()?.is_dir() {
            removed += remove_component_tests(&entry.path())?;
        } else if name.contains(".test.") || name.contains(".spec.") {
            fs::remove_file(entry.path())?;
            removed += 1;
        }
    }
    Ok(removed)
}

fn rewrite(root: &Path, file: &str, from: &str, to: &str) -> Result<()> {
    let path = root.join("frontend").join(file);
    let text = fs::read_to_string(&path)?;
    ensure!(text.contains(from), "{file} no longer contains {from}");
    fs::write(&path, text.replacen(from, to, 1))?;
    Ok(())
}

/// The gate must fail, and fail in the component suite rather than in the build or drift check.
fn refused_by_component_tests(root: &Path, planted: &str) -> Result<String> {
    let message = format!(
        "{:#}",
        frontend::run(root, false)
            .err()
            .with_context(|| format!("frontend-check passed although {planted}"))?
    );
    ensure!(
        message.contains("Vue component tests failed"),
        "frontend-check failed outside the component suite ({planted}):\n{message}"
    );
    Ok(message)
}

/// `test.fails` inverts a test: Vitest reports it `passed` exactly when its body throws, and the
/// JSON report carries no trace of the inversion. The control proves the planted payload breaks
/// the acceptance test as written; the gate must still refuse once that test is marked
/// `test.fails`, because the console no longer renders the recorded goal.
#[test]
fn inverted_acceptance_test_fails_the_gate() -> Result<()> {
    let (_temporary, root) = frontend_copy("frontend-check-pass2-")?;
    rewrite(
        &root,
        "src/fixtures/overview-operations.sse",
        GOAL,
        "A goal the acceptance test never expects",
    )?;
    refused_by_component_tests(
        &root,
        "the recorded goal is gone from the payload (control: the acceptance test as written must fail)",
    )?;
    rewrite(
        &root,
        "src/App.test.js",
        ACCEPTANCE,
        "test.fails('app_renders_a_snapshot_payload'",
    )?;
    refused_by_component_tests(
        &root,
        "app_renders_a_snapshot_payload is marked test.fails, so a console that no longer renders the recorded goal passes it",
    )?;
    Ok(())
}

/// The `no test passed` refusal is reached only when Vitest exits 0 having run nothing, which
/// `passWithNoTests` allows. No other case reaches that branch.
#[test]
fn passing_with_no_tests_fails_the_gate() -> Result<()> {
    let (_temporary, root) = frontend_copy("frontend-check-pass2-")?;
    ensure!(
        remove_component_tests(&root.join("frontend/src"))? > 0,
        "the copy held no component test file to remove"
    );
    rewrite(
        &root,
        "vitest.config.js",
        "allowOnly: false",
        "allowOnly: false, passWithNoTests: true",
    )?;
    let message = refused_by_component_tests(
        &root,
        "passWithNoTests is set and no test file exists, so no test ran",
    )?;
    ensure!(
        message.contains("no test passed"),
        "the gate refused for another reason than that no test passed:\n{message}"
    );
    Ok(())
}

/// The report path is passed through `npm run test -- --outputFile.json=<path>`; a repository
/// root with spaces must still reach Vitest as one argument and be read back.
#[test]
fn report_is_read_back_under_a_root_with_spaces() -> Result<()> {
    let (_temporary, root) = frontend_copy("frontend check pass2 ")?;
    frontend::run(&root, false).context("the unchanged copy must pass under a root with spaces")?;
    rewrite(
        &root,
        "src/App.test.js",
        ACCEPTANCE,
        "test.skip('app_renders_a_snapshot_payload'",
    )?;
    let message = refused_by_component_tests(
        &root,
        "app_renders_a_snapshot_payload is skipped under a root with spaces",
    )?;
    ensure!(
        message.contains("src/App.test.js > app_renders_a_snapshot_payload (skipped)"),
        "the gate did not name the skipped test from its report:\n{message}"
    );
    Ok(())
}

/// A passing suite plus one TypeScript spec file whose only suite is skipped: the default
/// pattern must collect the file and the report check must name its test.
#[test]
fn passing_suite_plus_skipped_spec_file_fails_the_gate() -> Result<()> {
    let (_temporary, root) = frontend_copy("frontend-check-pass2-")?;
    fs::write(
        root.join("frontend/src/Later.spec.ts"),
        "import { describe, test } from 'vitest'\ndescribe.skip('later', () => {\n  test('pending work', () => {})\n})\n",
    )?;
    let message = refused_by_component_tests(
        &root,
        "src/Later.spec.ts holds only a skipped suite beside the passing App.test.js",
    )?;
    ensure!(
        message.contains("src/Later.spec.ts > later > pending work (skipped)"),
        "the gate did not name the skipped spec test:\n{message}"
    );
    Ok(())
}

/// With `include` gone Vitest's root is still `frontend/`: a failing test file in the xtask
/// fixture directory or in the repository's `.scratch/` must not be collected.
#[test]
fn tests_outside_the_frontend_are_not_collected() -> Result<()> {
    let (_temporary, root) = frontend_copy("frontend-check-pass2-")?;
    let failing = "import { expect, test } from 'vitest'\ntest('collected from outside frontend', () => { expect('outside').toBe('never') })\n";
    for file in [
        "crates/control-plane-xtask/tests/fixtures/frontend/outside.test.js",
        ".scratch/outside.test.js",
    ] {
        let path = root.join(file);
        fs::create_dir_all(path.parent().context("planted file has a parent")?)?;
        fs::write(path, failing)?;
    }
    frontend::run(&root, false)
        .context("frontend-check collected a test file outside frontend/")?;
    Ok(())
}
