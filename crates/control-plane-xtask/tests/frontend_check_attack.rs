//! Adversarial cases for `frontend-check` (story:console-component-tests, pass 1).
//!
//! Each case copies `frontend/` (without `node_modules`) into a temporary root, links the
//! installed `frontend/node_modules` into it, plants one change in the copy only and runs
//! `frontend::run(root, false)`, which is what `frontend-check` runs. The real suite under
//! `frontend/src` is never touched. Run `npm ci --prefix frontend` first.
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

/// The `## Acceptance` names of story:console-component-tests.
const SCENARIOS: [&str; 3] = [
    "component_tests_run_in_the_gate",
    "failing_component_test_fails_the_gate",
    "app_renders_a_snapshot_payload",
];

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

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

fn frontend_copy() -> Result<(tempfile::TempDir, PathBuf)> {
    let source = repository().join("frontend");
    let modules = fs::canonicalize(source.join("node_modules"))
        .context("frontend/node_modules is missing; run npm ci --prefix frontend first")?;
    let temporary = tempfile::Builder::new()
        .prefix("frontend-check-attack-")
        .tempdir()?;
    let root = fs::canonicalize(temporary.path())?;
    copy(&source, &root.join("frontend"))?;
    // Link each package and `.bin`, not the directory, so Vite's and Vitest's caches
    // (`.vite`, `.vite-temp`) land in the copy instead of the real `node_modules`.
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

fn rewrite_app_test(root: &Path, from: &str, to: &str) -> Result<()> {
    let path = root.join("frontend/src/App.test.js");
    let text = fs::read_to_string(&path)?;
    ensure!(text.contains(from), "App.test.js no longer contains {from}");
    fs::write(&path, text.replacen(from, to, 1))?;
    Ok(())
}

#[test]
fn skipped_acceptance_test_fails_the_gate() -> Result<()> {
    let (_temporary, root) = frontend_copy()?;
    rewrite_app_test(
        &root,
        ACCEPTANCE,
        "test.skip('app_renders_a_snapshot_payload'",
    )?;
    refused_by_component_tests(
        &root,
        "the acceptance test app_renders_a_snapshot_payload is marked test.skip, so nothing observes it",
    )?;
    Ok(())
}

#[test]
fn todo_only_suite_fails_the_gate() -> Result<()> {
    let (_temporary, root) = frontend_copy()?;
    fs::write(
        root.join("frontend/src/App.test.js"),
        "import { test } from 'vitest'\ntest.todo('app_renders_a_snapshot_payload')\n",
    )?;
    refused_by_component_tests(
        &root,
        "the only component test left is test.todo('app_renders_a_snapshot_payload'), so no test ran",
    )?;
    Ok(())
}

#[test]
fn failing_spec_file_fails_the_gate() -> Result<()> {
    let (_temporary, root) = frontend_copy()?;
    fs::write(
        root.join("frontend/src/GoalForm.spec.js"),
        "import { expect, test } from 'vitest'\ntest('planted spec assertion that does not hold', () => { expect('rendered').toBe('never rendered') })\n",
    )?;
    refused_by_component_tests(
        &root,
        "frontend/src/GoalForm.spec.js holds a failing test (vitest.config.js include is src/**/*.test.js only)",
    )?;
    Ok(())
}

#[test]
fn removed_suite_fails_the_gate() -> Result<()> {
    let (_temporary, root) = frontend_copy()?;
    fs::remove_file(root.join("frontend/src/App.test.js"))?;
    refused_by_component_tests(&root, "frontend/src holds no component test file")?;
    Ok(())
}

#[test]
fn test_file_without_tests_fails_the_gate() -> Result<()> {
    let (_temporary, root) = frontend_copy()?;
    fs::write(
        root.join("frontend/src/Empty.test.js"),
        "import { test } from 'vitest'\n",
    )?;
    refused_by_component_tests(&root, "src/Empty.test.js declares no test")?;
    Ok(())
}

#[test]
fn test_file_failing_at_import_fails_the_gate() -> Result<()> {
    let (_temporary, root) = frontend_copy()?;
    fs::write(
        root.join("frontend/src/Broken.test.js"),
        "import { test } from 'vitest'\nthrow new Error('planted import failure')\ntest('never collected', () => {})\n",
    )?;
    let message = refused_by_component_tests(&root, "src/Broken.test.js throws at import")?;
    ensure!(
        message.contains("planted import failure"),
        "the gate failed without naming the import error:\n{message}"
    );
    Ok(())
}

#[test]
fn focused_test_fails_the_gate() -> Result<()> {
    let (_temporary, root) = frontend_copy()?;
    rewrite_app_test(
        &root,
        ACCEPTANCE,
        "test.only('app_renders_a_snapshot_payload'",
    )?;
    refused_by_component_tests(&root, "App.test.js carries test.only")?;
    Ok(())
}

fn sources(dir: &Path, extension: &str, out: &mut String) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_name() == "node_modules" || entry.file_name() == "target" {
            continue;
        }
        if entry.file_type()?.is_dir() {
            sources(&path, extension, out)?;
        } else if path.to_string_lossy().ends_with(extension) {
            out.push_str(&fs::read_to_string(&path)?);
            out.push('\n');
        }
    }
    Ok(())
}

/// A scenario resolves when a Rust test function under `crates/` has its name, or a component
/// test under `frontend/src` has it as its `test(...)` or `it(...)` title.
#[test]
fn every_acceptance_scenario_names_a_test() -> Result<()> {
    let root = repository();
    let (mut rust, mut frontend) = (String::new(), String::new());
    sources(&root.join("crates"), ".rs", &mut rust)?;
    sources(&root.join("frontend/src"), ".test.js", &mut frontend)?;
    let unresolved: Vec<_> = SCENARIOS
        .into_iter()
        .filter(|name| {
            !rust.contains(&format!("fn {name}("))
                && !["test('", "it('", "test(\"", "it(\""]
                    .iter()
                    .any(|call| frontend.contains(&format!("{call}{name}")))
        })
        .collect();
    ensure!(
        unresolved.is_empty(),
        "story:console-component-tests names scenarios that no test carries: {unresolved:?}"
    );
    Ok(())
}
