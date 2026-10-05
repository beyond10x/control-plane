//! Product-local protocols evaluated by Canon, with host-attested evidence.

pub use b10x_canon as canon;

use canon::model::{Case, Decision, EvidenceRecord};

#[derive(Debug, Clone, Copy)]
pub enum ProtocolKind {
    Planning,
    VerifiedMerge,
}

/// Provenance supplied by trusted host adapters, never deserialized from model output.
#[derive(Debug, Clone)]
pub enum EvidenceOrigin {
    PlanValidator { producer: String, observation: String, succeeded: bool },
    TestRunner { producer: String, observation: String, exit_code: i32 },
    IndependentReviewer { producer: String, observation: String, execution_context: String },
    RemoteObserver { producer: String, observation: String, candidate: String },
    Model { producer: String },
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
pub struct Protocol;

impl Protocol {
    pub fn compile(_kind: ProtocolKind) -> Result<Self, String> {
        Err("protocol adapter is not implemented".into())
    }

    pub fn evaluate(&self, _case: &Case, _evidence: &[AttestedEvidence], _context: EvaluationContext<'_>) -> Result<Decision, String> {
        Err("protocol adapter is not implemented".into())
    }

    pub fn evaluate_effect(
        &self,
        _case: &Case,
        _evidence: &[AttestedEvidence],
        _context: EvaluationContext<'_>,
        _action: &str,
        _authority: impl FnMut(&str) -> Result<bool, String>,
    ) -> Result<Decision, String> {
        Err("protocol adapter is not implemented".into())
    }
}
