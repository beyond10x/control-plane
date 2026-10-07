//! Product persistence/evidence bindings. Frontier and completion belong to Loom's governor.
use crate::engine::ProgressHook;
use anyhow::{Result, anyhow};
use control_plane_protocol::{AttestedEvidence, EvaluationContext, validate_attestation};
use loom_sdk::commission::{
    model::{
        json as wire,
        primitives::{Timestamp, Uuid},
        responsibility::*,
    },
    ports::{evidence::submit_evidence, governor::Governor},
};
use loom_sdk::{
    CanonGovernor,
    governor::{CaseState, FallibleCaseStore},
};
use serde_json::json;
use std::{collections::BTreeMap, sync::Mutex};

type Clock = fn(&CaseId) -> Result<Option<String>, GovernorError>;
pub(crate) type HostGovernor = CanonGovernor<HostCases, Clock>;

pub(crate) fn open(
    yaml: &str,
    protocol: &str,
    case: &CaseId,
    artifacts: BTreeMap<String, String>,
    progress: ProgressHook,
) -> Result<HostGovernor> {
    let governor = CanonGovernor::new(HostCases {
        cases: Mutex::new(BTreeMap::new()),
        observations: Mutex::new(Vec::new()),
        progress,
    })
    .with_protocol_yaml(protocol, yaml)?
    .with_evaluation_time(clock as Clock);
    governor.open_case(case.clone(), protocol, artifacts)?;
    Ok(governor)
}
fn clock(_: &CaseId) -> Result<Option<String>, GovernorError> {
    Ok(Some(now()))
}
pub(crate) fn now() -> String {
    time::OffsetDateTime::now_utc()
        .replace_nanosecond(0)
        .unwrap()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap()
}
fn id() -> Uuid {
    Uuid(uuid::Uuid::new_v4().to_string())
}

pub(crate) fn submit(
    governor: &HostGovernor,
    case: &CaseId,
    evidence: &AttestedEvidence,
    implementor: Option<&str>,
) -> Result<()> {
    validate_attestation(
        evidence,
        EvaluationContext {
            at: &now(),
            implementor_context: implementor,
        },
    )
    .map_err(anyhow::Error::msg)?;
    let record = &evidence.record;
    let mut document = json!({"format":record.format,"id":record.id.as_str(),"kind":record.kind.as_str(),"subject":record.subject.as_str(),"subject_revision":record.subject_revision.as_str()});
    if let Some(result) = &record.result {
        document["result"] = json!(result);
    }
    if let Some(at) = &record.observed_at {
        document["observed_at"] = json!(at.as_str());
    }
    let upstream: BTreeMap<_, _> = record
        .upstream_revisions
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();
    document["upstream_revisions"] = json!(upstream);
    let facts = wire::parse(&document.to_string()).map_err(|e| anyhow!("{e:?}"))?;
    submit_evidence(
        governor,
        "control-plane-trusted-receipt-adapter",
        EvidenceData {
            evidence_id: EvidenceId(id()),
            case_id: case.clone(),
            kind: evidence.record.kind.as_str().into(),
            subject_revision: governor
                .current_revision(case)
                .map_err(|e| anyhow!("{e:?}"))?,
            producer: String::new(),
            observation_ids: vec![ObservationId(id())],
            facts,
            provenance: wire::Value::Text(format!("{:?}", evidence.origin)),
        },
    )?;
    Ok(())
}

/// Cases belong to this bounded attempt. Product recovery opens a fresh case and revalidates
/// retained candidate/check/publication state; it never resumes an uncertain effect from a trace.
/// Every state change is journalled through the durable host hook before becoming visible here.
pub(crate) struct HostCases {
    cases: Mutex<BTreeMap<String, CaseState>>,
    observations: Mutex<Vec<ObservationData>>,
    progress: ProgressHook,
}
impl HostCases {
    fn persist(&self, state: &CaseState) -> Result<()> {
        (self.progress)(
            "governor-state",
            &json!({"case_id":state.id.0,"case_revision":state.revision,"protocol":state.protocol,"artifacts":state.artifacts,"evidence_count":state.evidence.len(),"recovery":"fresh-case-with-trusted-revalidation"}),
        )
    }
}
impl FallibleCaseStore for HostCases {
    type Error = anyhow::Error;
    fn insert(&self, state: CaseState) -> Result<bool> {
        let mut cases = self
            .cases
            .lock()
            .map_err(|_| anyhow!("case store poisoned"))?;
        if cases.contains_key(&state.id.0) {
            return Ok(false);
        }
        self.persist(&state)?;
        cases.insert(state.id.0.clone(), state);
        Ok(true)
    }
    fn get(&self, case: &CaseId) -> Result<Option<CaseState>> {
        Ok(self
            .cases
            .lock()
            .map_err(|_| anyhow!("case store poisoned"))?
            .get(&case.0)
            .cloned())
    }
    fn update<T>(
        &self,
        case: &CaseId,
        change: impl FnOnce(&mut CaseState) -> T,
    ) -> Result<Option<T>> {
        let mut cases = self
            .cases
            .lock()
            .map_err(|_| anyhow!("case store poisoned"))?;
        let Some(current) = cases.get_mut(&case.0) else {
            return Ok(None);
        };
        let mut candidate = current.clone();
        let result = change(&mut candidate);
        self.persist(&candidate)?;
        *current = candidate;
        Ok(Some(result))
    }
    fn observe(&self, observation: ObservationData) -> Result<()> {
        (self.progress)(
            "loom-observation",
            &json!({"observation":format!("{observation:?}")}),
        )?;
        self.observations
            .lock()
            .map_err(|_| anyhow!("observations poisoned"))?
            .push(observation);
        Ok(())
    }
    fn observations(&self) -> Result<Vec<ObservationData>> {
        Ok(self
            .observations
            .lock()
            .map_err(|_| anyhow!("observations poisoned"))?
            .clone())
    }
}

pub(crate) struct ContextClock {
    pub max_steps: usize,
}
impl loom_sdk::LoopContext for ContextClock {
    fn action_request_id(&mut self) -> ActionRequestId {
        ActionRequestId(id())
    }
    fn observation_id(&mut self) -> ObservationId {
        ObservationId(id())
    }
    fn now(&mut self) -> Timestamp {
        Timestamp(now())
    }
    fn step_budget(&self) -> Option<usize> {
        Some(self.max_steps)
    }
}
