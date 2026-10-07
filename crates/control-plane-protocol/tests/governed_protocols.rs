use std::cell::Cell;

use control_plane_protocol::canon::model::{Case, ClaimId, Decision, EvidenceRecord, Truth};
use control_plane_protocol::{
    AttestedEvidence, EvaluationContext, EvidenceOrigin, Protocol, ProtocolKind,
};

fn context() -> EvaluationContext<'static> {
    EvaluationContext {
        at: "2026-10-05T20:00:00Z",
        implementor_context: Some("implementor-1"),
    }
}

fn case(kind: ProtocolKind, revision: &str) -> Case {
    let (protocol, artifact) = match kind {
        ProtocolKind::Planning => ("engineering.plan", "plan"),
        ProtocolKind::VerifiedMerge => ("software.change.merge", "implementation"),
    };
    serde_yaml_ng::from_str(&format!(
        "format: canon-case/1\nid: case-1\nprotocol: {protocol}\nartifacts:\n  {artifact}: {{revision: {revision}}}\n"
    )).unwrap()
}

fn evidence(
    id: &str,
    kind: &str,
    result: &str,
    subject: &str,
    revision: &str,
    origin: EvidenceOrigin,
) -> AttestedEvidence {
    let record: EvidenceRecord = serde_yaml_ng::from_str(&format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\nresult: {result}\nsubject: {subject}\nsubject_revision: {revision}\n"
    )).unwrap();
    AttestedEvidence { record, origin }
}

fn validation(revision: &str, pass: bool) -> AttestedEvidence {
    evidence(
        "validation-1",
        "plan_validation",
        if pass { "pass" } else { "fail" },
        "plan",
        revision,
        EvidenceOrigin::PlanValidator {
            producer: "validator".into(),
            observation: "validation-output-1".into(),
            revision: revision.into(),
            succeeded: pass,
        },
    )
}

fn tests(revision: &str, pass: bool) -> AttestedEvidence {
    evidence(
        if pass { "tests-pass" } else { "tests-fail" },
        "test_result",
        if pass { "pass" } else { "fail" },
        "implementation",
        revision,
        EvidenceOrigin::TestRunner {
            producer: "test-runner".into(),
            observation: "process-1".into(),
            revision: revision.into(),
            exit_code: if pass { 0 } else { 1 },
        },
    )
}

fn review(revision: &str, approved: bool) -> AttestedEvidence {
    evidence(
        if approved {
            "review-approved"
        } else {
            "review-rejected"
        },
        "independent_code_review",
        if approved { "approved" } else { "rejected" },
        "implementation",
        revision,
        EvidenceOrigin::IndependentReviewer {
            producer: "review-adapter".into(),
            observation: "review-output-1".into(),
            revision: revision.into(),
            execution_context: "reviewer-1".into(),
        },
    )
}

fn merge(revision: &str) -> AttestedEvidence {
    evidence(
        "merge-1",
        "merge_observation",
        "merged",
        "implementation",
        revision,
        EvidenceOrigin::RemoteObserver {
            producer: "repository-adapter".into(),
            observation: "remote-receipt-1".into(),
            candidate: revision.into(),
        },
    )
}

fn claim(decision: &Decision, name: &str) -> Truth {
    decision.claims.get(&ClaimId::new(name)).unwrap().value
}

fn accepted(decision: &Decision) -> bool {
    decision.outcomes.as_ref().unwrap()["accepted"]["status"] == "legitimate"
}

fn merge_status(decision: &Decision) -> &str {
    decision.actions.as_ref().unwrap()["repository.merge"]["status"]
        .as_str()
        .unwrap()
}

#[test]
fn planning_requires_current_validation() {
    let protocol = Protocol::compile(ProtocolKind::Planning).unwrap();
    let case = case(ProtocolKind::Planning, "p2");
    for evidence in [vec![], vec![validation("p1", true)]] {
        let decision = protocol.evaluate(&case, &evidence, context()).unwrap();
        assert_eq!(claim(&decision, "plan.validated"), Truth::Unknown);
        assert!(!accepted(&decision));
    }
    let failed = protocol
        .evaluate(&case, &[validation("p2", false)], context())
        .unwrap();
    assert_eq!(claim(&failed, "plan.validated"), Truth::False);
    assert!(!accepted(&failed));
    assert!(accepted(
        &protocol
            .evaluate(&case, &[validation("p2", true)], context())
            .unwrap()
    ));
}

#[test]
fn merge_requires_current_test_and_independent_review() {
    let protocol = Protocol::compile(ProtocolKind::VerifiedMerge).unwrap();
    let case = case(ProtocolKind::VerifiedMerge, "r2");
    for evidence in [
        vec![],
        vec![tests("r2", true)],
        vec![review("r2", true)],
        vec![tests("r2", true), review("r1", true)],
        vec![tests("r1", true), review("r2", true)],
        vec![tests("r2", true), review("r2", false)],
    ] {
        let decision = protocol
            .evaluate_effect(&case, &evidence, context(), "repository.merge", |_| {
                Ok(true)
            })
            .unwrap();
        assert_eq!(merge_status(&decision), "blocked");
    }
    let evidence = [tests("r2", true), review("r2", true)];
    let ready = protocol.evaluate(&case, &evidence, context()).unwrap();
    assert_eq!(claim(&ready, "implementation.verified"), Truth::True);
    assert_eq!(merge_status(&ready), "approval-required");
    let authorized = protocol
        .evaluate_effect(&case, &evidence, context(), "repository.merge", |_| {
            Ok(true)
        })
        .unwrap();
    assert_eq!(merge_status(&authorized), "admissible");
    assert!(!accepted(&authorized));
}

#[test]
fn stale_evidence_cannot_complete() {
    let protocol = Protocol::compile(ProtocolKind::VerifiedMerge).unwrap();
    let evidence = [tests("r1", true), review("r1", true), merge("r1")];
    assert!(accepted(
        &protocol
            .evaluate(
                &case(ProtocolKind::VerifiedMerge, "r1"),
                &evidence,
                context()
            )
            .unwrap()
    ));
    let decision = protocol
        .evaluate(
            &case(ProtocolKind::VerifiedMerge, "r2"),
            &evidence,
            context(),
        )
        .unwrap();
    assert_eq!(claim(&decision, "implementation.verified"), Truth::Unknown);
    assert_eq!(claim(&decision, "repository.merged"), Truth::Unknown);
    assert!(!accepted(&decision));
}

#[test]
fn merge_observation_required() {
    let protocol = Protocol::compile(ProtocolKind::VerifiedMerge).unwrap();
    let case = case(ProtocolKind::VerifiedMerge, "r2");
    let mut evidence = vec![tests("r2", true), review("r2", true)];
    assert!(!accepted(
        &protocol.evaluate(&case, &evidence, context()).unwrap()
    ));
    evidence.push(merge("r1"));
    assert!(!accepted(
        &protocol.evaluate(&case, &evidence, context()).unwrap()
    ));
    evidence.pop();
    evidence.push(merge("r2"));
    assert!(accepted(
        &protocol.evaluate(&case, &evidence, context()).unwrap()
    ));
}

#[test]
fn authority_is_checked_at_effect() {
    let protocol = Protocol::compile(ProtocolKind::VerifiedMerge).unwrap();
    let case = case(ProtocolKind::VerifiedMerge, "r2");
    let evidence = [tests("r2", true), review("r2", true)];
    let allowed = Cell::new(true);
    let calls = Cell::new(0);
    let check = |capability: &str| {
        assert_eq!(capability, "repository.merge");
        calls.set(calls.get() + 1);
        Ok(allowed.get())
    };
    assert_eq!(
        merge_status(
            &protocol
                .evaluate_effect(&case, &evidence, context(), "repository.merge", check)
                .unwrap()
        ),
        "admissible"
    );
    allowed.set(false);
    assert_eq!(
        merge_status(
            &protocol
                .evaluate_effect(&case, &evidence, context(), "repository.merge", check)
                .unwrap()
        ),
        "blocked"
    );
    assert_eq!(calls.get(), 2);
    assert!(
        protocol
            .evaluate_effect(&case, &evidence, context(), "repository.merge", |_| Err(
                "authority store unavailable".into()
            ))
            .is_err()
    );
}

#[test]
fn model_cannot_supply_host_evidence() {
    for (kind, mut item) in [
        (ProtocolKind::Planning, validation("p1", true)),
        (ProtocolKind::VerifiedMerge, tests("r1", true)),
        (ProtocolKind::VerifiedMerge, review("r1", true)),
        (ProtocolKind::VerifiedMerge, merge("r1")),
    ] {
        item.origin = EvidenceOrigin::Model {
            producer: "model".into(),
        };
        let protocol = Protocol::compile(kind).unwrap();
        assert!(
            protocol
                .evaluate(&case(kind, "r1"), &[item], context())
                .unwrap_err()
                .contains("model")
        );
    }
}

#[test]
fn same_or_unknown_execution_context_is_not_independent() {
    let protocol = Protocol::compile(ProtocolKind::VerifiedMerge).unwrap();
    for run in ["implementor-1", "", " "] {
        let mut item = review("r1", true);
        if let EvidenceOrigin::IndependentReviewer {
            execution_context, ..
        } = &mut item.origin
        {
            *execution_context = run.into();
        }
        assert!(
            protocol
                .evaluate(&case(ProtocolKind::VerifiedMerge, "r1"), &[item], context())
                .is_err()
        );
    }
    assert!(
        protocol
            .evaluate(
                &case(ProtocolKind::VerifiedMerge, "r1"),
                &[review("r1", true)],
                EvaluationContext {
                    implementor_context: None,
                    ..context()
                }
            )
            .is_err()
    );
}

#[test]
fn receipts_must_match_evidence_claims() {
    let protocol = Protocol::compile(ProtocolKind::VerifiedMerge).unwrap();
    let mut item = tests("r1", true);
    item.record.result = Some("fail".into());
    assert!(
        protocol
            .evaluate(&case(ProtocolKind::VerifiedMerge, "r1"), &[item], context())
            .is_err()
    );
    let mut item = merge("r1");
    item.record.subject_revision = "r2".into();
    assert!(
        protocol
            .evaluate(&case(ProtocolKind::VerifiedMerge, "r2"), &[item], context())
            .is_err()
    );
    let mut item = review("r1", true);
    item.origin = tests("r1", true).origin;
    assert!(
        protocol
            .evaluate(&case(ProtocolKind::VerifiedMerge, "r1"), &[item], context())
            .is_err()
    );
}

#[test]
fn contradictory_current_evidence_stays_unknown() {
    let protocol = Protocol::compile(ProtocolKind::VerifiedMerge).unwrap();
    let case = case(ProtocolKind::VerifiedMerge, "r2");
    for evidence in [
        vec![tests("r2", true), tests("r2", false), review("r2", true)],
        vec![tests("r2", true), review("r2", true), review("r2", false)],
    ] {
        let decision = protocol.evaluate(&case, &evidence, context()).unwrap();
        assert_eq!(claim(&decision, "implementation.verified"), Truth::Unknown);
        assert!(!accepted(&decision));
        assert_eq!(merge_status(&decision), "blocked");
    }
    let decision = protocol
        .evaluate(
            &case,
            &[tests("r1", false), tests("r2", true), review("r2", true)],
            context(),
        )
        .unwrap();
    assert_eq!(claim(&decision, "implementation.verified"), Truth::True);
}

#[test]
fn unknown_actions_and_blocked_preconditions_cannot_consume_authority() {
    let protocol = Protocol::compile(ProtocolKind::VerifiedMerge).unwrap();
    let case = case(ProtocolKind::VerifiedMerge, "r1");
    let unexpected =
        |_: &str| -> Result<bool, String> { panic!("authority must not be consulted") };
    assert!(
        protocol
            .evaluate_effect(&case, &[], context(), "repository.deploy", unexpected)
            .is_err()
    );
    assert_eq!(
        merge_status(
            &protocol
                .evaluate_effect(&case, &[], context(), "repository.merge", unexpected)
                .unwrap()
        ),
        "blocked"
    );
    protocol
        .evaluate_effect(&case, &[], context(), "repository.inspect", unexpected)
        .unwrap();
}

#[test]
fn all_evidence_origins_require_producer_and_observation() {
    for field in ["producer", "observation"] {
        for value in ["", " padded", "line\nbreak"] {
            let items = [
                (ProtocolKind::Planning, validation("r1", true)),
                (ProtocolKind::VerifiedMerge, tests("r1", true)),
                (ProtocolKind::VerifiedMerge, review("r1", true)),
                (ProtocolKind::VerifiedMerge, merge("r1")),
            ];
            for (kind, mut item) in items {
                let (producer, observation) = match &mut item.origin {
                    EvidenceOrigin::PlanValidator {
                        producer,
                        observation,
                        ..
                    }
                    | EvidenceOrigin::TestRunner {
                        producer,
                        observation,
                        ..
                    }
                    | EvidenceOrigin::IndependentReviewer {
                        producer,
                        observation,
                        ..
                    }
                    | EvidenceOrigin::RemoteObserver {
                        producer,
                        observation,
                        ..
                    } => (producer, observation),
                    EvidenceOrigin::Model { .. } => unreachable!(),
                };
                if field == "producer" {
                    *producer = value.into();
                } else {
                    *observation = value.into();
                }
                assert!(
                    Protocol::compile(kind)
                        .unwrap()
                        .evaluate(&case(kind, "r1"), &[item], context())
                        .is_err()
                );
            }
        }
    }
}

#[test]
fn canon_refusals_are_not_reinterpreted_as_decisions() {
    let protocol = Protocol::compile(ProtocolKind::VerifiedMerge).unwrap();
    let case = case(ProtocolKind::VerifiedMerge, "r1");
    let item = tests("r1", true);
    assert!(
        protocol
            .evaluate(&case, &[item.clone(), item.clone()], context())
            .is_err()
    );
    assert!(
        protocol
            .evaluate(
                &case,
                &[item],
                EvaluationContext {
                    at: "not-an-instant",
                    ..context()
                }
            )
            .is_err()
    );
    let planning = Protocol::compile(ProtocolKind::Planning).unwrap();
    assert!(planning.evaluate(&case, &[], context()).is_err());
}

#[test]
fn no_receipt_can_be_relabelled_to_another_revision() {
    for (kind, mut item) in [
        (ProtocolKind::Planning, validation("r1", true)),
        (ProtocolKind::VerifiedMerge, tests("r1", true)),
        (ProtocolKind::VerifiedMerge, review("r1", true)),
        (ProtocolKind::VerifiedMerge, merge("r1")),
    ] {
        item.record.subject_revision = "r2".into();
        assert!(
            Protocol::compile(kind)
                .unwrap()
                .evaluate(&case(kind, "r2"), &[item], context())
                .is_err()
        );
    }
}
