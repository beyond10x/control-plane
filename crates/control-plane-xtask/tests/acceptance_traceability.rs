//! `acceptance-check`: the scenario names in active and implemented stories resolve to tests.
//!
//! The gate module is compiled in by path so the extraction and resolution cases drive it
//! directly; `unresolved_name_fails_gate` runs the built binary, so the exit status and the
//! printed `<story id>: <name>` lines are the ones `task check` sees.
#[allow(dead_code)]
#[path = "../src/acceptance.rs"]
mod acceptance;

use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/acceptance")
}

/// A disposable repository holding `stories` (id, status, acceptance text) in the planning store
/// and `files` (path, contents) beneath its root.
fn repository(stories: &[(&str, &str, &str)], files: &[(&str, &str)]) -> Result<tempfile::TempDir> {
    let dir = tempfile::tempdir()?;
    let store = dir.path().join(acceptance::STORIES);
    fs::create_dir_all(&store)?;
    for (id, status, text) in stories {
        let slug = id.strip_prefix("story:").unwrap();
        fs::write(
            store.join(format!("{slug}.md")),
            format!(
                "---\nid: {id}\nkind: story\nstatus: {status}\ntitle: {slug}\n---\n## Outcome\n\nFixture.\n\n## Acceptance\n\n{text}\n"
            ),
        )?;
    }
    fs::create_dir_all(dir.path().join("crates"))?;
    for (path, contents) in files {
        let path = dir.path().join(path);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, contents)?;
    }
    Ok(dir)
}

fn listed(lines: &str) -> Vec<&str> {
    lines
        .lines()
        .filter(|line| line.starts_with("story:"))
        .collect()
}

#[test]
fn scenario_names_follow_the_rule() -> Result<()> {
    let found: Vec<(String, Vec<String>)> = acceptance::stories(&fixture())?
        .into_iter()
        .map(|story| (story.id, story.names))
        .collect();
    let expected: Vec<(&str, Vec<&str>)> = vec![
        (
            "story:backticked-list",
            vec![
                "unacknowledged_spec_change_fails_gate",
                "progress_decision_stays_under_16_kib",
                "restart_open_is_bounded",
                "recorded_history_replays",
            ],
        ),
        (
            "story:hyphen-names",
            vec![
                "shared-governed-executor",
                "no-change-is-not-progress",
                "reject-missing-name",
                "name-after-a-tilde-fence",
            ],
        ),
        ("story:no-acceptance", vec![]),
        (
            "story:plain-prose",
            vec![
                "goal_drives_plan",
                "existing_backlog_is_not_duplicated",
                "goal_completion_requires_evidence",
                "model_wait_is_visible_before_response",
                "planner_activity_survives_restart",
            ],
        ),
    ];
    let expected: Vec<(String, Vec<String>)> = expected
        .into_iter()
        .map(|(id, names)| {
            (
                id.to_owned(),
                names.into_iter().map(str::to_owned).collect(),
            )
        })
        .collect();
    assert_eq!(found, expected);
    Ok(())
}

#[test]
fn unresolved_name_fails_gate() -> Result<()> {
    let names = "- `present_unit_scenario`: an inline unit test.\n\
                 - `present_async_scenario`: a tokio test with arguments and a second attribute.\n\
                 - present-integration-scenario, named with hyphens, in an integration test.\n\
                 - `absent_scenario_name`: nothing has this name.\n\
                 - `helper_is_not_a_test`: a function without a test attribute.\n\
                 - `commented_out_scenario`: a test inside a comment.\n\
                 - `scenario_in_a_string`: a test inside a string literal.\n\
                 - `false_cfg_scenario`: a test the compiler never builds.\n\
                 - `inner_cfg_scenario`: a test in a module an inner attribute switches off.";
    let source = r##"pub fn helper_is_not_a_test() {}

#[cfg(test)]
mod tests {
    #[test]
    fn present_unit_scenario() {}

    #[cfg(false)]
    #[test]
    fn false_cfg_scenario() {}

    mod switched_off {
        #![cfg(any())]
        #[test]
        fn inner_cfg_scenario() {}
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "fixture"]
    async fn present_async_scenario() {}

    // #[test]
    // fn commented_out_scenario() {}

    const SOURCE: &str = r#"#[test] fn scenario_in_a_string() {}"#;
}
"##;
    let integration = "#[test]\nfn present_integration_scenario() {}\n";
    let files = [
        ("crates/demo/src/lib.rs", source),
        ("crates/demo/tests/integration.rs", integration),
    ];
    let dir = repository(
        &[
            ("story:demo", "active", names),
            ("story:draft-demo", "draft", "- `draft_only_scenario`"),
        ],
        &files,
    )?;
    let run = |root: &Path| {
        Command::new(env!("CARGO_BIN_EXE_control-plane-xtask"))
            .args(["acceptance-check", "--root"])
            .arg(root)
            .output()
    };

    let output = run(dir.path())?;
    let stderr = String::from_utf8(output.stderr)?;
    assert!(!output.status.success(), "the gate passed: {stderr}");
    assert_eq!(
        listed(&stderr),
        [
            "story:demo: absent_scenario_name",
            "story:demo: helper_is_not_a_test",
            "story:demo: commented_out_scenario",
            "story:demo: scenario_in_a_string",
            "story:demo: false_cfg_scenario",
            "story:demo: inner_cfg_scenario",
        ],
        "{stderr}"
    );

    let resolved = "- `present_unit_scenario`\n- `present_async_scenario`\n\
                    - present-integration-scenario";
    let dir = repository(
        &[
            ("story:demo", "implemented", resolved),
            ("story:draft-demo", "draft", "- `draft_only_scenario`"),
        ],
        &files,
    )?;
    let output = run(dir.path())?;
    let stdout = String::from_utf8(output.stdout)?;
    assert!(
        output.status.success(),
        "{stdout}{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        stdout.trim(),
        "3 scenario names in 1 active and implemented stories resolve to tests"
    );
    Ok(())
}

#[test]
fn frontend_test_titles_resolve() -> Result<()> {
    let component = r#"import { describe, expect, it, test } from 'vitest'

describe('title_of_a_describe_block', () => {
  test('app_renders_a_snapshot_payload', async () => {
    expect(1).toBe(1)
    const half = total / 2, slash = '/'
  })
  const quote = () => { return /"/ }
  it("shows-the-goal-card", () => {})
  test(`computed_${'title'}_is_not_static`, () => {})
})
// test('commented_out_title_scenario', () => {})
"#;
    assert_eq!(
        acceptance::frontend_test_titles(component),
        ["app_renders_a_snapshot_payload", "shows-the-goal-card"]
    );

    let names = "- `app_renders_a_snapshot_payload`: a component test title.\n\
                 - `shows-the-goal-card`: a hyphenated `it` title.\n\
                 - `title_of_a_describe_block`: a describe title is not a test.\n\
                 - `commented_out_title_scenario`: a commented-out test.\n\
                 - `title_outside_a_test_file`: a call in a module that is not a test file.";
    let files = [
        ("frontend/src/components/App.test.js", component),
        (
            "frontend/src/App.js",
            "test('title_outside_a_test_file', () => {})\n",
        ),
    ];
    let dir = repository(&[("story:console", "active", names)], &files)?;
    let refusal = acceptance::check(dir.path()).unwrap_err().to_string();
    assert_eq!(
        listed(&refusal),
        [
            "story:console: title_of_a_describe_block",
            "story:console: commented_out_title_scenario",
            "story:console: title_outside_a_test_file",
        ],
        "{refusal}"
    );

    let resolved = "- `app_renders_a_snapshot_payload`\n- shows-the-goal-card";
    let dir = repository(&[("story:console", "active", resolved)], &files)?;
    assert_eq!(
        acceptance::check(dir.path())?,
        "2 scenario names in 1 active and implemented stories resolve to tests"
    );
    Ok(())
}
