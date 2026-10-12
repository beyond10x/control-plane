//! Adversarial cases for the mutation audit's verdict. The one-id unwitnessed exemption of
//! story:typed-satisfaction-receipt is gone (story:remove-stale-revision-mutation-exemption):
//! `run_verb` admits an audit only when `ess --collect` passed and `verdict` accepts its report,
//! which admits killed mutants and `equivalent` ones carrying ESS's evidence, nothing else. These
//! cases drive `verdict` with reports it should refuse.
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

use mutation::verdict;
use serde_json::{Value, json};

fn killed() -> Value {
    json!({"id": "guard-negate/controlplane.host.CreateGoal/workers-invalid",
        "class": "guard-negate", "verdict": "killed"})
}

/// The record ESS 0.56.0 wrote for the negated `stale-revision` guard, once exempted.
fn formerly_exempted() -> Value {
    json!({"id": "guard-negate/controlplane.host.SatisfyGoal/stale-revision",
        "class": "guard-negate", "verdict": "unwitnessed",
        "added_refusals": [{"code": "ESS-SYNTH-003",
            "scenario": "controlplane.host.SatisfyGoal/outcome/stale-revision",
            "subject": "outcome controlplane.host.SatisfyGoal/stale-revision"}]})
}

/// The record ESS 0.59.0 writes for that mutant.
fn equivalent() -> Value {
    json!({"id": "guard-negate/controlplane.host.SatisfyGoal/stale-revision",
        "class": "guard-negate", "verdict": "equivalent",
        "unsatisfiable_guard": "(not (defined(receipt_revision)) and (state == Running and revision != input.receipt_revision))",
        "added_refusals": [{"code": "ESS-SYNTH-003",
            "scenario": "controlplane.host.SatisfyGoal/outcome/stale-revision",
            "subject": "outcome controlplane.host.SatisfyGoal/stale-revision"}]})
}

fn report(mutants: Vec<Value>, counts: Value) -> Value {
    json!({"format": "ess-mutation-report/5", "counts": counts, "mutants": mutants})
}

/// Control: a killed mutant beside an evidenced equivalent one is admitted.
#[test]
fn exemption_attack_control_report_is_admitted() {
    let admitted = verdict(&report(
        vec![killed(), equivalent()],
        json!({"mutants": 2, "killed": 1, "survived": 0, "inconclusive": 0, "unwitnessed": 0,
            "equivalent": 1, "stillborn": 0}),
    ))
    .unwrap();
    assert_eq!((admitted.killed, admitted.equivalent), (1, 1));
}

/// The once-exempted record, listed once or twice, is refused: no mutant is admitted unwitnessed.
#[test]
fn exemption_attack_formerly_exempted_mutant_is_refused() {
    for copies in [1, 2] {
        let mut mutants = vec![killed()];
        mutants.extend(std::iter::repeat_n(formerly_exempted(), copies));
        let answer = verdict(&report(
            mutants,
            json!({"mutants": 1 + copies, "killed": 1, "survived": 0, "inconclusive": 0,
                "unwitnessed": copies, "equivalent": 0, "stillborn": 0}),
        ));
        assert!(
            answer.is_err(),
            "{copies} unwitnessed record(s) were admitted: {answer:?}"
        );
    }
}

/// A report whose counts say a mutant was stillborn or equivalent, with no such record listed, is
/// refused: the counts must add up to the records.
#[test]
fn exemption_attack_stillborn_or_equivalent_count_is_refused() {
    for key in ["stillborn", "equivalent"] {
        let mut counts = json!({"mutants": 2, "killed": 1, "survived": 0, "inconclusive": 0,
            "unwitnessed": 0, "equivalent": 1, "stillborn": 0});
        counts[key] = json!(counts[key].as_u64().unwrap() + 1);
        let answer = verdict(&report(vec![killed(), equivalent()], counts.clone()));
        assert!(
            answer.is_err(),
            "{key} counted beyond its records was admitted: {counts}"
        );
    }
}
