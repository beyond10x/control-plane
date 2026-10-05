//! The actual durable contract adapter, reopened between commands and observations.
//! Only successfully committed outcome envelopes enter the observed event history.
use crate::codec;
use anyhow::{Context, Result};
use control_plane_core::{Actor, contract::ContractStore};
use ess_conformance::scenario::OutcomeRef;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, CountReport, Runner};
use ess_primitives::{consistency::ConsistencyToken, node::Node};
use std::{
    cell::{Cell, RefCell},
    path::{Path, PathBuf},
};

pub struct DurableTarget {
    runtime: tokio::runtime::Runtime,
    scratch: PathBuf,
    live: RefCell<Option<tempfile::TempDir>>,
    events: RefCell<Vec<ObservedEvent>>,
    committed: Cell<u64>,
}

impl DurableTarget {
    pub fn new(scratch: &Path) -> Result<Self> {
        std::fs::create_dir_all(scratch)?;
        Ok(Self {
            runtime: tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?,
            scratch: scratch.into(),
            live: RefCell::new(None),
            events: RefCell::new(Vec::new()),
            committed: Cell::new(0),
        })
    }
    fn path(&self) -> Result<PathBuf, TargetError> {
        self.live
            .borrow()
            .as_ref()
            .map(|dir| dir.path().join("host.sqlite"))
            .ok_or_else(|| TargetError::unavailable("database", "no active scenario"))
    }
}

fn unavailable(error: impl std::fmt::Display) -> TargetError {
    TargetError::unavailable("durable contract", error.to_string())
}

// Core currently preserves dispatcher status and body in its error. Decode that exact
// response; never consult grants or guess a result before actually invoking the command.
fn command_error(error: anyhow::Error) -> TargetError {
    let detail = error.to_string();
    if let Some(body) = detail.strip_prefix("generated command refused (403): ")
        && let Ok(response) = serde_json::from_str::<serde_json::Value>(body)
        && response["refused"] == "not granted"
    {
        return TargetError::not_granted(response["actor"].as_str());
    }
    unavailable(error)
}

impl ConformanceTarget for DurableTarget {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(
            "control-plane-core::contract::ContractStore",
            env!("CARGO_PKG_VERSION"),
        ))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        *self.live.borrow_mut() = Some(tempfile::tempdir_in(&self.scratch).map_err(unavailable)?);
        self.events.borrow_mut().clear();
        self.committed.set(0);
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.live.borrow_mut().take();
        self.events.borrow_mut().clear();
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        if request.caller.is_some() {
            return Err(TargetError::unsupported(
                "caller attributes",
                "contract declares no caller attributes",
            ));
        }
        let actor = match request.actor.as_ref().map(ToString::to_string).as_deref() {
            Some("controlplane.host.Operator") => Actor::Operator,
            Some("controlplane.host.Supervisor") => Actor::Supervisor,
            _ => {
                return Err(TargetError::unsupported(
                    "actor",
                    "contract API requires one declared actor",
                ));
            }
        };
        let input = codec::to_json(&Node::Map(request.input)).map_err(unavailable)?;
        let path = self.path()?;
        let command = request.command.to_string();
        let answer = self
            .runtime
            .block_on(async {
                let mut store = ContractStore::open(&path).await?;
                store.execute(&command, input, actor).await
            })
            .map_err(command_error)?;
        self.committed.set(self.committed.get() + 1);
        let outcome = answer["outcome"]
            .as_str()
            .ok_or_else(|| unavailable("no actual outcome"))?;
        let mut result = SemanticCommandResult::took(OutcomeRef::new(
            request.command,
            outcome.parse().map_err(unavailable)?,
        ));
        if let Some(error) = answer["error"].as_str() {
            let mut declared = DeclaredErrorValue::new(error.parse().map_err(unavailable)?);
            if let Some(payload) = answer.get("payload") {
                declared.fields = codec::fields(payload).map_err(unavailable)?;
            }
            result = result.with_error(declared);
        }
        let mut history = self.events.borrow_mut();
        for published in answer["published"]
            .as_array()
            .ok_or_else(|| unavailable("no actual publication array"))?
        {
            let name = published["event"]
                .as_str()
                .ok_or_else(|| unavailable("no actual event name"))?;
            let mut event = ObservedEvent::new(name.parse().map_err(unavailable)?)
                .in_activity(request.correlation.clone())
                .at(history.len() as u64 + 1);
            event.payload = codec::fields(&published["payload"]).map_err(unavailable)?;
            history.push(event.clone());
            result = result.emitting(event);
        }
        // Opening the next operation replays Eventlog; the target retains no domain state.
        Ok(result.with_consistency(
            ConsistencyToken::new(format!("committed:{}", self.committed.get()))
                .map_err(unavailable)?,
        ))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let path = self.path()?;
        if let Some(token) = request.consistency.token() {
            let revision = token
                .as_str()
                .strip_prefix("committed:")
                .and_then(|s| s.parse::<u64>().ok())
                .ok_or_else(|| unavailable("unrecognized consistency token"))?;
            if revision > self.committed.get() {
                return Err(unavailable("requested commit has not completed"));
            }
        }
        let rows = self
            .runtime
            .block_on(async {
                ContractStore::open(path)
                    .await?
                    .query(&request.view.to_string())
            })
            .map_err(unavailable)?;
        let rows = rows
            .as_array()
            .ok_or_else(|| unavailable("view returned no rows"))?;
        Ok(SemanticViewResult::of(
            rows.iter()
                .map(codec::fields)
                .collect::<Result<Vec<_>>>()
                .map_err(unavailable)?,
        ))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .events
            .borrow()
            .iter()
            .filter(|event| {
                event.event == request.event
                    && event.correlation.as_ref() == Some(&request.correlation)
            })
            .cloned()
            .collect())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "external outcome",
            "no external outcomes declared",
        ))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "redelivery",
            "no bindings declared",
        ))
    }
}

pub fn run(suite: &str, target: &impl ConformanceTarget) -> Result<(CountReport, String)> {
    let admitted = AdmittedSuite::from_json(suite)?;
    let executed = Runner::for_suite(admitted.suite()).run_admitted(&admitted, target);
    let report = CountReport::from_run(&executed, &admitted)?;
    Ok((report, serde_json::to_string_pretty(&*executed)?))
}

pub fn gate(report: &CountReport) -> Result<()> {
    anyhow::ensure!(
        report.conformance_status() == ess_conformance::CountStatus::Passed,
        "ESS coverage qualification is not passed"
    );
    let c = report.counts();
    anyhow::ensure!(
        c.total >= 152
            && c.passed == c.total
            && c.failed == 0
            && c.error == 0
            && c.unsupported == 0
            && c.skipped == 0,
        "incomplete scenario execution: {}",
        report.to_canonical_json()?
    );
    Ok(())
}

pub fn conformance(root: &Path) -> Result<()> {
    let scratch = root.join(".scratch/conformance");
    let suite = std::fs::read_to_string(root.join("generated/conformance.json"))?;
    let target = DurableTarget::new(&scratch)?;
    let (report, diagnostics) = run(&suite, &target)?;
    std::fs::write(scratch.join("report.json"), report.to_canonical_json()?)?;
    std::fs::write(scratch.join("diagnostics.json"), diagnostics)?;
    gate(&report).context("see .scratch/conformance/report.json and diagnostics.json")?;
    println!(
        "{} scenarios passed; 0 failed/error/unsupported/skipped; coverage qualification: {:?}",
        report.counts().passed,
        report.conformance_status()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // This fault is introduced after the real durable view call: all commands, grant
    // checks, Eventlog appends, replay and other observations still execute normally.
    struct CorruptArchivedState(DurableTarget);
    impl ConformanceTarget for CorruptArchivedState {
        fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
            self.0.identity()
        }
        fn begin_scenario(&self, s: &ScenarioContext) -> Result<(), TargetError> {
            self.0.begin_scenario(s)
        }
        fn end_scenario(&self, s: &ScenarioContext) -> Result<(), TargetError> {
            self.0.end_scenario(s)
        }
        fn execute_command(
            &self,
            r: SemanticCommandRequest,
        ) -> Result<SemanticCommandResult, TargetError> {
            self.0.execute_command(r)
        }
        fn query_view(&self, r: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
            let corrupt = r.view.to_string() == "controlplane.host.WorkspaceList";
            let mut answer = self.0.query_view(r)?;
            if corrupt {
                for row in &mut answer.rows {
                    if row.get("state") == Some(&Node::Text("Archived".into())) {
                        row.insert("state".into(), Node::Text("Registered".into()));
                    }
                }
            }
            Ok(answer)
        }
        fn observe_events(
            &self,
            r: EventObservationRequest,
        ) -> Result<Vec<ObservedEvent>, TargetError> {
            self.0.observe_events(r)
        }
        fn configure_external_outcome(&self, r: ExternalOutcomeControl) -> Result<(), TargetError> {
            self.0.configure_external_outcome(r)
        }
        fn redeliver_event(&self, r: RedeliveryRequest) -> Result<(), TargetError> {
            self.0.redeliver_event(r)
        }
    }

    #[test]
    fn real_durable_suite_passes_and_wrong_archived_state_fails_by_name() -> Result<()> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let suite = std::fs::read_to_string(root.join("generated/conformance.json"))?;
        let scratch = root.join(".scratch/conformance-tests");
        let target = DurableTarget::new(&scratch)?;
        let (baseline, _) = run(&suite, &target)?;
        gate(&baseline)?;
        let mutant = CorruptArchivedState(DurableTarget::new(&scratch)?);
        let (report, diagnostics) = run(&suite, &mutant)?;
        std::fs::write(
            scratch.join("negative-report.json"),
            report.to_canonical_json()?,
        )?;
        std::fs::write(scratch.join("negative-diagnostics.json"), diagnostics)?;
        assert!(gate(&report).is_err());
        assert_eq!(
            report.statuses()["controlplane.host.Workspace/transition/archive/by/controlplane.host.ArchiveWorkspace/applied"],
            "failed"
        );
        assert_eq!(report.counts().error, 0);
        assert_eq!(report.counts().unsupported, 0);
        Ok(())
    }
}
