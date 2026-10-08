//! Adversarial cases for story:spec-owned-admission, pass 2.
//!
//! `guard_mutants_are_killed` reads the committed `--collect` report and judges it with
//! `mutation::verdict`, which reads only the `counts` object and the class of each mutant. These
//! cases drive the real `verdict` (the module is compiled here from `src/mutation.rs`; its runner
//! dependency is stubbed, since no case runs a suite) with the committed report altered the way a
//! stale or hand-edited report would be.
use serde_json::Value;
use std::{fs, path::PathBuf};

/// Stands in for `crate::target`, which `mutation.rs` imports for `audit`; no case calls it.
#[allow(dead_code)]
mod target {
    use std::path::Path;
    pub struct DurableTarget;
    impl DurableTarget {
        pub fn new(_: &Path) -> anyhow::Result<Self> {
            anyhow::bail!("not used by these cases")
        }
    }
    pub struct Report;
    impl Report {
        pub fn to_canonical_json(&self) -> anyhow::Result<String> {
            anyhow::bail!("not used by these cases")
        }
    }
    pub fn run(_: &str, _: &DurableTarget) -> anyhow::Result<(Report, String)> {
        anyhow::bail!("not used by these cases")
    }
}

#[allow(dead_code)]
#[path = "../src/mutation.rs"]
mod mutation;

fn recorded() -> Value {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/guard-mutation-report.json");
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

/// A report whose per-mutant record says a guard mutant survived is refused, whatever its
/// `counts` say. Today `verdict` reads only `counts`, so the committed report with one mutant
/// marked `survived` passes `guard_mutants_are_killed`'s checks.
#[test]
fn verdict_refuses_a_report_whose_mutant_survived() {
    let mut report = recorded();
    report["mutants"][0]["verdict"] = "survived".into();
    report["mutants"][0]["killers"] = Value::Array(vec![]);
    let judged = mutation::verdict(&report);
    assert!(
        judged.is_err(),
        "a report listing mutant {} as survived was accepted: {judged:?}",
        report["mutants"][0]["id"]
    );
}

/// A report whose `counts` disagree with its `mutants` list is refused. Today a report that keeps
/// one guard mutant of twelve, with the counts still claiming twelve killed, is accepted.
#[test]
fn verdict_refuses_counts_that_disagree_with_the_mutants_listed() {
    let mut report = recorded();
    let first = report["mutants"][0].clone();
    report["mutants"] = Value::Array(vec![first]);
    let judged = mutation::verdict(&report);
    assert!(
        judged.is_err(),
        "a report listing 1 mutant while counting {} was accepted: {judged:?}",
        report["counts"]["mutants"]
    );
}

/// `task mutation` (the CI job) passes on `verdict` alone. An audit in which no guard mutant was
/// killed, every one `stillborn`, is not an audit that "reports guard-class mutants and no
/// survivor" in any useful sense, and is refused.
#[test]
fn verdict_refuses_an_audit_that_killed_no_guard_mutant() {
    let mut report = recorded();
    let total = report["counts"]["mutants"].as_u64().unwrap();
    report["counts"]["killed"] = 0.into();
    report["counts"]["stillborn"] = total.into();
    for mutant in report["mutants"].as_array_mut().unwrap() {
        mutant["verdict"] = "stillborn".into();
        mutant["killers"] = Value::Array(vec![]);
    }
    let judged = mutation::verdict(&report);
    assert!(
        judged.is_err(),
        "an audit with {total} stillborn and 0 killed mutants was accepted: {judged:?}"
    );
}
