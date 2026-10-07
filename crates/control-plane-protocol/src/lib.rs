//! Product-local protocols evaluated by Canon, with host-attested evidence.
//!
//! This crate is a trusted host boundary, not an untrusted evidence ingestion API. The caller
//! obtains receipts from its validators, process runner, isolated reviewer and repository adapter.
//! It checks their actual origin and observed repository/target, then constructs [`EvidenceOrigin`].
//! Never deserialize model output into those attestations. This crate checks their consistency;
//! it cannot authenticate a process or inspect Git. Raw model claims are explicitly rejected.
//!
//! [`Protocol::evaluate`] supplies no authority. [`Protocol::evaluate_effect`] asks a live host
//! callback for the requested action's capabilities, with no cached grant. The caller holds its
//! repository lease and supplies the current candidate snapshot immediately before the effect;
//! a returned decision does not itself publish anything or authorize a different candidate.

pub use b10x_canon as canon;

use canon::eval::{Supplied, evaluate_with};
use canon::ir::{Ir, compile};
use canon::model::{ActionId, Case, Decision, EvidenceRecord};

pub const PLANNING_YAML: &str = include_str!("../protocols/engineering-plan.yaml");
pub const VERIFIED_MERGE_YAML: &str = include_str!("../protocols/software-change-merge.yaml");

#[derive(Debug, Clone, Copy)]
pub enum ProtocolKind {
    Planning,
    VerifiedMerge,
}

/// Provenance supplied by trusted host adapters, never deserialized from model output.
#[derive(Debug, Clone)]
pub enum EvidenceOrigin {
    PlanValidator {
        producer: String,
        observation: String,
        revision: String,
        succeeded: bool,
    },
    TestRunner {
        producer: String,
        observation: String,
        revision: String,
        exit_code: i32,
    },
    IndependentReviewer {
        producer: String,
        observation: String,
        revision: String,
        execution_context: String,
    },
    RemoteObserver {
        producer: String,
        observation: String,
        candidate: String,
    },
    Model {
        producer: String,
    },
}

#[derive(Debug, Clone)]
pub struct AttestedEvidence {
    pub record: EvidenceRecord,
    pub origin: EvidenceOrigin,
}

#[derive(Debug, Clone, Copy)]
pub struct EvaluationContext<'a> {
    pub at: &'a str,
    pub implementor_context: Option<&'a str>,
}

#[derive(Debug)]
pub struct Protocol {
    ir: Ir,
}

impl Protocol {
    pub fn compile(kind: ProtocolKind) -> Result<Self, String> {
        let yaml = match kind {
            ProtocolKind::Planning => PLANNING_YAML,
            ProtocolKind::VerifiedMerge => VERIFIED_MERGE_YAML,
        };
        let model = canon::model::parse(yaml).map_err(|error| error.to_string())?;
        let ir = compile(&model).map_err(|problems| {
            problems
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; ")
        })?;
        // Commission's frontier represents one capability per action. Refuse a future protocol
        // change that the product's existing governor bridge could not represent faithfully.
        if ir.actions.values().any(|action| action.requires.len() > 1) {
            return Err("the governor bridge supports at most one capability per action".into());
        }
        Ok(Self { ir })
    }

    /// The actual Canon IR, available to the host's Commission frontier bridge.
    pub fn ir(&self) -> &Ir {
        &self.ir
    }

    /// Evaluate trusted current state without granting any authority.
    pub fn evaluate(
        &self,
        case: &Case,
        evidence: &[AttestedEvidence],
        context: EvaluationContext<'_>,
    ) -> Result<Decision, String> {
        self.evaluate_with_authority(case, evidence, context, None)
    }

    /// Evaluate immediately before a requested effect, resolving authority afresh.
    ///
    /// `authority` reads current host policy; its failure is propagated, never treated as allow.
    /// A blocked precondition does not request authority. Inspect the requested action's status
    /// in the returned Canon decision; successful evaluation alone is not admission.
    pub fn evaluate_effect(
        &self,
        case: &Case,
        evidence: &[AttestedEvidence],
        context: EvaluationContext<'_>,
        action: &str,
        mut authority: impl FnMut(&str) -> Result<bool, String>,
    ) -> Result<Decision, String> {
        let declared = self
            .ir
            .actions
            .get(&ActionId::new(action))
            .ok_or_else(|| format!("undeclared action `{action}`"))?;
        let initial = self.evaluate(case, evidence, context)?;
        let status = initial
            .actions
            .as_ref()
            .and_then(|actions| actions.get(action))
            .and_then(|entry| entry.get("status"))
            .and_then(|status| status.as_str());
        if status == Some("blocked") || declared.requires.is_empty() {
            return Ok(initial);
        }
        let mut decisions = Vec::new();
        for capability in &declared.requires {
            let granted = authority(capability.as_str())?;
            decisions.push(serde_json::json!({
                "capability": capability.as_str(),
                "decision": if granted { "granted" } else { "denied" }
            }));
        }
        let authority = serde_json::to_string(&decisions).map_err(|error| error.to_string())?;
        self.evaluate_with_authority(case, evidence, context, Some(&authority))
    }

    fn evaluate_with_authority(
        &self,
        case: &Case,
        evidence: &[AttestedEvidence],
        context: EvaluationContext<'_>,
        authority: Option<&str>,
    ) -> Result<Decision, String> {
        for item in evidence {
            validate_attestation(item, context)?;
        }
        let records: Vec<_> = evidence.iter().map(|item| item.record.clone()).collect();
        evaluate_with(
            &self.ir,
            case,
            &records,
            Supplied {
                authority,
                at: Some(context.at),
                decisions: None,
            },
        )
        .map_err(|error| error.to_string())
    }
}

fn named(value: &str) -> bool {
    !value.is_empty() && value.trim() == value && !value.chars().any(char::is_control)
}

pub fn validate_attestation(
    item: &AttestedEvidence,
    context: EvaluationContext<'_>,
) -> Result<(), String> {
    let record = &item.record;
    let result = record.result.as_deref();
    let (producer, observation, revision, kind, subject) = match &item.origin {
        EvidenceOrigin::PlanValidator {
            producer,
            observation,
            revision,
            succeeded,
        } => {
            if result != Some(if *succeeded { "pass" } else { "fail" }) {
                return Err("plan evidence disagrees with its validator receipt".into());
            }
            (producer, observation, revision, "plan_validation", "plan")
        }
        EvidenceOrigin::TestRunner {
            producer,
            observation,
            revision,
            exit_code,
        } => {
            if result != Some(if *exit_code == 0 { "pass" } else { "fail" }) {
                return Err("test evidence disagrees with its process receipt".into());
            }
            (
                producer,
                observation,
                revision,
                "test_result",
                "implementation",
            )
        }
        EvidenceOrigin::IndependentReviewer {
            producer,
            observation,
            revision,
            execution_context,
        } => {
            let implementor = context
                .implementor_context
                .filter(|value| named(value))
                .ok_or("independent review requires a known implementor execution context")?;
            if !named(execution_context) || execution_context == implementor {
                return Err("review requires a distinct named execution context".into());
            }
            if !matches!(result, Some("approved" | "rejected")) {
                return Err("review evidence has no recognized verdict".into());
            }
            (
                producer,
                observation,
                revision,
                "independent_code_review",
                "implementation",
            )
        }
        EvidenceOrigin::RemoteObserver {
            producer,
            observation,
            candidate,
        } => {
            if !matches!(result, Some("merged" | "not_merged")) {
                return Err("merge evidence has no recognized observation result".into());
            }
            (
                producer,
                observation,
                candidate,
                "merge_observation",
                "implementation",
            )
        }
        EvidenceOrigin::Model { .. } => return Err("model output is not host evidence".into()),
    };
    if !named(producer) || !named(observation) {
        return Err("evidence requires a named producer and observation receipt".into());
    }
    if !named(revision) || revision != record.subject_revision.as_str() {
        return Err("evidence is not bound to the revision in its observation receipt".into());
    }
    if record.kind.as_str() != kind || record.subject.as_str() != subject {
        return Err("evidence kind or subject does not match its trusted producer".into());
    }
    Ok(())
}
