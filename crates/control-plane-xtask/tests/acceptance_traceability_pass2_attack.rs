//! Adversarial cases for `acceptance-check` (story:acceptance-traceability, wave 2, pass 2).
//!
//! Pass 2 attacks the fixes 180d232 made for pass 1: fence handling, `macro_rules!` skipping and
//! the cases the unit's suite does not pin down. Each case drives the gate module directly.
#[allow(dead_code)]
#[path = "../src/acceptance.rs"]
mod acceptance;

use anyhow::Result;
use std::{collections::BTreeSet, fs};

/// A disposable repository whose planning store holds `stories` (id, status, whole body).
fn repository(stories: &[(&str, &str, &str)]) -> Result<tempfile::TempDir> {
    let dir = tempfile::tempdir()?;
    let store = dir.path().join(acceptance::STORIES);
    fs::create_dir_all(&store)?;
    for (id, status, body) in stories {
        let slug = id.strip_prefix("story:").unwrap();
        fs::write(
            store.join(format!("{slug}.md")),
            format!("---\nid: {id}\nkind: story\nstatus: {status}\ntitle: {slug}\n---\n{body}"),
        )?;
    }
    fs::create_dir_all(dir.path().join("crates"))?;
    Ok(dir)
}

/// A fence left open earlier in the body, or a four-backtick fence closed with three, swallows the
/// `## Acceptance` heading. At dea51c3 the gate still read that section and refused the absent
/// names; since 180d232 the story is silently exempt and the gate passes. A gate that meets
/// unbalanced fences must refuse, not pass.
#[test]
fn unbalanced_fence_does_not_exempt_a_story() -> Result<()> {
    let unclosed = "## Outcome\n\nFixture.\n\n## Evidence\n\n\
                    ```console\n$ task check\nFAIL: three names unresolved\n\n\
                    ## Acceptance\n\n\
                    - `absent_after_unclosed_fence`: no test has this name.\n";
    let nested = "## Outcome\n\nA nested example, its outer fence closed with three backticks:\n\n\
                  ````markdown\n```console\ntask check\n```\n```\n\n\
                  ## Acceptance\n\n\
                  - `absent_after_nested_fence`: no test has this name.\n";
    let dir = repository(&[
        ("story:unclosed", "active", unclosed),
        ("story:nested", "implemented", nested),
    ])?;
    let outcome = acceptance::check(dir.path());
    assert!(
        outcome.is_err(),
        "the gate passed two checked stories whose Acceptance names no test: {outcome:?}"
    );
    Ok(())
}

/// CommonMark closes a fenced block when its list item ends (`markdown-it` renders the second
/// bullet below as a list item). The extractor keeps the fence open to the end of the section, so
/// every name listed after it is dropped without error.
#[test]
fn fence_in_a_list_item_ends_with_the_item() {
    let section = "\n- `first_listed_scenario`: run\n  ```console\n  task check\n\
                   - `second_listed_scenario`: the next item.\n";
    assert_eq!(
        acceptance::scenario_names(section),
        ["first_listed_scenario", "second_listed_scenario"]
    );
}

/// A `macro_rules!` body that is invoked expands to a real test, which `cargo test` runs. The fix
/// for pass 1 skips every `macro_rules!` body, invoked or not, so the test no longer resolves.
#[test]
fn test_in_an_invoked_macro_still_resolves() {
    let source = r#"
macro_rules! scenario {
    () => {
        #[test]
        fn invoked_macro_scenario() {}
    };
}

scenario!();

#[test]
fn ordinary_scenario_runs() {}
"#;
    let found: BTreeSet<String> = acceptance::rust_test_names(source).into_iter().collect();
    let expected: BTreeSet<String> = ["invoked_macro_scenario", "ordinary_scenario_runs"]
        .map(str::to_owned)
        .into();
    assert_eq!(found, expected);
}

/// Fence edges the unit's suite does not exercise: a decoy `## Acceptance` inside an earlier
/// fence, a four-backtick fence holding a three-backtick block and a `# ` line, a tilde fence
/// holding a backtick line, a line with an info string inside a fence, and an indented fence in a
/// list item. Green against 180d232; each of five one-line mutants of `Fence` or `acceptance`
/// that the existing suite survives turns it red.
#[test]
fn fence_edges_follow_commonmark() {
    let body = "## Outcome\n\n\
                ```markdown\n## Acceptance\n- `decoy_inside_outcome_fence`\n```\n\n\
                ## Acceptance\n\n\
                - `before_the_fences_scenario`\n\n\
                ````markdown\n```console\n# a comment inside the inner block\n```\n\
                ## Scope inside the outer fence\nnot_a_scenario_inside_fence\n````\n\n\
                ~~~text\n```\nstill_inside_the_tilde_fence\n~~~\n\n\
                ```text\n```rust is an info string, so this line does not close the fence\n\
                inside_after_info_string_line\n```\n\n\
                - `after_the_fences_scenario`: listed after every fence.\n  \
                ```console\n  cargo run -p control-plane-xtask\n  ```\n  \
                ~~~console\n  cargo run -p control-plane-app\n  ~~~\n\n\
                ## Scope\n\nInferred: scope_section_is_not_acceptance.\n";
    let section = acceptance::acceptance(body).expect("the body has an Acceptance section");
    assert_eq!(
        acceptance::scenario_names(&section),
        ["before_the_fences_scenario", "after_the_fences_scenario"],
        "section seen by the extractor: {section:?}"
    );
}
