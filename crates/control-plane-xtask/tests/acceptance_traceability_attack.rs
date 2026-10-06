//! Adversarial cases for `acceptance-check` (story:acceptance-traceability, wave 2, pass 1).
//!
//! Each case drives the gate module directly with a hand-built story body, Rust source or
//! frontend test file and asserts what the story's Scenario name rule says should happen.
#[allow(dead_code)]
#[path = "../src/acceptance.rs"]
mod acceptance;

use anyhow::Result;
use std::fs;

fn story(id: &str, status_line: &str, acceptance: &str) -> String {
    format!(
        "---\nformat: aep.planning-md/3\nid: {id}\nkind: story\n{status_line}\ntitle: fixture\n---\n## Outcome\n\nFixture.\n\n## Acceptance\n\n{acceptance}\n\n## Scope\n\nInferred: scope_section_is_not_acceptance.\n"
    )
}

/// A shell comment inside a fenced block is not a Markdown heading, so it must not end the
/// Acceptance section: the names listed after the fence are still the story's names.
#[test]
fn fenced_shell_comment_does_not_end_acceptance() {
    let body = "## Outcome\n\nFixture.\n\n## Acceptance\n\n\
                ```console\n\
                # run from the repository root\n\
                cargo run --locked -p control-plane-xtask -- acceptance-check\n\
                ```\n\n\
                - `name_listed_after_the_fence`: still a scenario of this story.\n\n\
                ## Scope\n\nNone.\n";
    let section = acceptance::acceptance(body).expect("the body has an Acceptance section");
    assert_eq!(
        acceptance::scenario_names(&section),
        ["name_listed_after_the_fence"],
        "section seen by the extractor: {section:?}"
    );
}

/// YAML allows a single-quoted scalar; a story whose status is `'active'` is active and its
/// names must be checked, not silently skipped.
#[test]
fn single_quoted_status_is_still_checked() -> Result<()> {
    let dir = tempfile::tempdir()?;
    fs::write(
        dir.path().join("quoted.md"),
        story(
            "story:quoted",
            "status: 'active'",
            "- `quoted_status_scenario`: listed.",
        ),
    )?;
    let ids: Vec<String> = acceptance::stories(dir.path())?
        .into_iter()
        .map(|story| story.id)
        .collect();
    assert_eq!(ids, ["story:quoted"]);
    Ok(())
}

/// `proposed` and `archived` stories are not checked; only `active` and `implemented` are.
/// Green against the unit; the unit's own fixtures cover `draft` only, so a filter of
/// `status != "draft"` would pass its suite.
#[test]
fn proposed_and_archived_stories_are_not_checked() -> Result<()> {
    let dir = tempfile::tempdir()?;
    for (slug, status) in [
        ("proposed", "proposed"),
        ("archived", "archived"),
        ("active", "active"),
        ("implemented", "implemented"),
        ("dquoted", "\"implemented\""),
    ] {
        fs::write(
            dir.path().join(format!("{slug}.md")),
            story(
                &format!("story:{slug}"),
                &format!("status: {status}"),
                "- `some_listed_scenario`",
            ),
        )?;
    }
    let ids: Vec<String> = acceptance::stories(dir.path())?
        .into_iter()
        .map(|story| story.id)
        .collect();
    assert_eq!(ids, ["story:active", "story:dquoted", "story:implemented"]);
    Ok(())
}

/// A `#[test]` the compiler never builds is not a test: neither a test inside `#[cfg(any())]`
/// nor one inside a `macro_rules!` body that is never expanded runs in `cargo test`.
#[test]
fn tests_the_compiler_never_builds_do_not_resolve() {
    let source = r#"
#[cfg(any())]
mod disabled {
    #[test]
    fn disabled_module_scenario() {}
}

#[cfg(any())]
#[test]
fn disabled_function_scenario() {}

macro_rules! never_expanded {
    () => {
        #[test]
        fn unexpanded_macro_scenario() {}
    };
}

#[test]
fn compiled_scenario_runs() {}
"#;
    assert_eq!(
        acceptance::rust_test_names(source),
        ["compiled_scenario_runs"]
    );
}

/// The same name with call parentheses is a name whether or not it is backticked: plain prose
/// `goal_drives_plan()` is extracted, so the backticked form must be too.
#[test]
fn backticked_name_with_call_parentheses_is_a_name() {
    let plain = acceptance::scenario_names("- goal_drives_plan(): plain.");
    let backticked = acceptance::scenario_names("- `goal_drives_plan()`: backticked.");
    assert_eq!(plain, ["goal_drives_plan"]);
    assert_eq!(backticked, plain);
}

/// A regular-expression literal holding a quote is not the start of a string: the title after it
/// is a test, and a `test("…")` inside a later string literal is not.
#[test]
fn quote_in_a_regex_literal_neither_hides_nor_invents_titles() {
    let source = r#"import { expect, test } from 'vitest'

test('apostrophe_regex_scenario', () => {
  expect(label()).not.toMatch(/'/)
  const note = 'see test("phantom_title_scenario") in the plan'
})

test('title_after_the_regex', () => {})
"#;
    assert_eq!(
        acceptance::frontend_test_titles(source),
        ["apostrophe_regex_scenario", "title_after_the_regex"]
    );
}

/// Names in nested bullets, links, bold and multi-line list items, in an Acceptance section that
/// is the body's last section. Green against the unit.
#[test]
fn names_in_markdown_decorations_are_extracted() {
    let body = "## Outcome\n\nFixture.\n\n## Acceptance\n\n\
                - Parent item\n  - `nested_bullet_scenario`: nested.\n\
                - [linked_scenario_name](#anchor-in-the-doc) and **bold_scenario_name**.\n\
                - a list item that wraps onto\n  continued_line_scenario on its second line.";
    let section = acceptance::acceptance(body).expect("the body has an Acceptance section");
    assert_eq!(
        acceptance::scenario_names(&section),
        [
            "nested_bullet_scenario",
            "linked_scenario_name",
            "bold_scenario_name",
            "continued_line_scenario",
        ]
    );
}
