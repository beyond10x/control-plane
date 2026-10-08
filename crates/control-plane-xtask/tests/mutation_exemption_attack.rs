//! Adversarial cases for the mutation audit's unwitnessed exemption (story:typed-satisfaction-
//! receipt, wave 7 pass 2). `run_verb` now admits a failed `ess --collect` whenever `verdict`
//! accepts a report holding the exempted mutant and the baseline passed, so `verdict` alone is
//! the judge of such a report. These cases drive `verdict` with reports it should refuse.
//!
//! The gate modules are compiled in by path so the cases call `verdict` exactly as the binary
//! does; run them with the `exemption_attack` filter.
#[allow(dead_code)]
#[path = "../src/codec.rs"]
mod codec;
#[allow(dead_code)]
#[path = "../src/mutation.rs"]
mod mutation;
#[allow(dead_code)]
#[path = "../src/target.rs"]
mod target;

use mutation::{UNWITNESSED_EXEMPTION, verdict};
use serde_json::{Value, json};

fn killed() -> Value {
    json!({"id": "guard-negate/controlplane.host.CreateGoal/workers-invalid",
        "class": "guard-negate", "verdict": "killed"})
}

fn exempted() -> Value {
    let (id, code, scenario) = UNWITNESSED_EXEMPTION;
    json!({"id": id, "class": "guard-negate", "verdict": "unwitnessed",
        "added_refusals": [{"code": code, "scenario": scenario,
            "subject": "outcome controlplane.host.SatisfyGoal/stale-revision"}]})
}

fn report(mutants: Vec<Value>, counts: Value) -> Value {
    json!({"format": "ess-mutation-report/3", "counts": counts, "mutants": mutants})
}

/// Control: the shape the unit admits is still admitted.
#[test]
fn exemption_attack_control_report_is_admitted() {
    let admitted = verdict(&report(
        vec![killed(), exempted()],
        json!({"mutants": 2, "killed": 1, "survived": 0, "inconclusive": 0, "unwitnessed": 1,
            "equivalent": 0, "stillborn": 0}),
    ))
    .unwrap();
    assert_eq!((admitted.killed, admitted.exempt), (1, 1));
}

/// `Verdict::exempt` is documented as "0 or 1" and the constant names one mutant, yet the same
/// exempted record listed twice, with `unwitnessed: 2`, is admitted with two exemptions.
#[test]
fn exemption_attack_duplicate_exempted_mutant_is_refused() {
    let answer = verdict(&report(
        vec![killed(), exempted(), exempted()],
        json!({"mutants": 3, "killed": 1, "survived": 0, "inconclusive": 0, "unwitnessed": 2,
            "equivalent": 0, "stillborn": 0}),
    ));
    assert!(
        answer.is_err(),
        "a report exempting the one mutant twice was admitted: {answer:?}"
    );
}

/// A report whose counts say a mutant was stillborn or equivalent, beside the exempted one, is
/// admitted: `verdict` never reads those counts, and the counts it does read need not add up to
/// `mutants`. With the exemption, `--collect`'s own exit status no longer backs it.
#[test]
fn exemption_attack_stillborn_or_equivalent_count_is_refused() {
    for key in ["stillborn", "equivalent"] {
        let mut counts = json!({"mutants": 2, "killed": 1, "survived": 0, "inconclusive": 0,
            "unwitnessed": 1, "equivalent": 0, "stillborn": 0});
        counts[key] = json!(1);
        let answer = verdict(&report(vec![killed(), exempted()], counts.clone()));
        assert!(
            answer.is_err(),
            "{key}: 1 beside the exemption was admitted: {counts}"
        );
    }
}
