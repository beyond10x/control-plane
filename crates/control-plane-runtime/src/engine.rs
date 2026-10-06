//! One bounded planning commission. Loom proposes; trusted effects validate and mutate.
use crate::refusal::InputRefusal;
use crate::{
    AgentModel, ModelRequest, RuntimeConfig,
    model::{PlannerAction, critique_schema, planner_schema},
    process::ProcessRunner,
};
use anyhow::{Context, Result, bail, ensure};
use control_plane_protocol::{AttestedEvidence, EvidenceOrigin};
use loom_sdk::commission::{
    model::{
        behaviour::Generated,
        json as wire,
        primitives::{Timestamp, Uuid},
        responsibility::*,
    },
    outcome::RunStore,
    ports::{
        authority::{AuthorityProvider, AuthorityProviderError},
        effect::{AdmittedRequest, EffectError, EffectPort},
    },
};
use loom_sdk::loom::{
    arguments::ArgumentContext,
    model::run::{CatalogueEntry, SelectionStrategy},
    selection::{Choice, SelectionContext, SelectorError},
};
use loom_sdk::{ActionSelector, ArgumentGenerator, Loom, LoopContext, run_until_blocked};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
};

pub type ProgressHook = Arc<dyn Fn(&str, &Value) -> Result<()> + Send + Sync>;

pub struct EngineInput {
    pub path: PathBuf,
    pub goal: Value,
    pub namespace: String,
    pub config: RuntimeConfig,
    pub runner: ProcessRunner,
    pub progress: ProgressHook,
}

pub struct EngineOutput {
    pub stories: Vec<String>,
    pub receipt: Value,
}

struct State {
    revision: i64,
    transcript: Vec<String>,
    unseen: Vec<String>,
    pending: Option<PlannerAction>,
    selected: Vec<String>,
    evidence: Vec<AttestedEvidence>,
    plan_revision: String,
    failure: Option<String>,
    syntax_failures: usize,
    review_attempts: usize,
    rejected_revision: Option<i64>,
    memory: crate::context::ActionMemory,
}

enum OperationResult {
    Completed,
    SyntaxFeedback,
    ReviewFeedback,
    ValidationFeedback,
    Refused(String),
}

struct Planning<'a> {
    input: &'a EngineInput,
    model: &'a dyn AgentModel,
    governor: crate::governance::HostGovernor,
    case_id: CaseId,
    state: Mutex<State>,
    refusals: crate::refusal::RefusalBudget,
    started: std::time::Instant,
}

pub fn run(input: EngineInput, model: Arc<dyn AgentModel>) -> Result<EngineOutput> {
    (input.progress)("prepare-branch", &json!({"namespace":input.namespace}))?;
    if input
        .runner
        .command(&input.path, "git", &["branch", "--show-current"])?
        .trim()
        .is_empty()
    {
        input.runner.command(
            &input.path,
            "git",
            &[
                "switch",
                "-c",
                &format!("control-plane/{}", input.namespace),
            ],
        )?;
    }
    let inspection = inspect(&input.path, &input.runner)?;
    if !input.path.join(".engineering/project.yaml").exists() {
        (input.progress)("adopt", &json!({"source":input.config.aep_protocols}))?;
        input.runner.command(
            &input.path,
            "aep",
            &[
                "plan",
                "reverse",
                "init",
                "--protocols",
                &input.config.aep_protocols,
                "--profile",
                "development.standard",
            ],
        )?;
    }
    let scan = input.runner.command(
        &input.path,
        "aep",
        &["plan", "reverse", "scan", "--format", "json"],
    )?;
    let backlog = input.runner.command(
        &input.path,
        "aep",
        &["plan", "artifact", "list", "--format", "json"],
    )?;
    let planning = Planning {
        governor: crate::governance::open(
            control_plane_protocol::PLANNING_YAML,
            "engineering-plan@1",
            &CaseId(input.namespace.clone()),
            std::collections::BTreeMap::from([("plan".into(), input.namespace.clone())]),
            input.progress.clone(),
        )?,
        case_id: CaseId(input.namespace.clone()),
        input: &input,
        model: model.as_ref(),
        refusals: Default::default(),
        started: std::time::Instant::now(),
        state: Mutex::new(State {
            revision: 1,
            unseen: Vec::new(),
            transcript: vec![
                inspection,
                crate::context::scan(&scan)?,
                crate::context::backlog(&backlog)?,
            ],
            pending: None,
            selected: Vec::new(),
            evidence: Vec::new(),
            plan_revision: input.namespace.clone(),
            failure: None,
            syntax_failures: 0,
            review_attempts: 0,
            rejected_revision: None,
            memory: crate::context::ActionMemory::default(),
        }),
    };
    let executor = Loom::new(&planning, &planning, text(&input.goal, "objective")?);
    let commission = Commission::new(CommissionData {
        commission_id: CommissionId(id()),
        agent_revision_id: AgentRevisionId(id()),
        case_id: planning.case_id.clone(),
        principal: PrincipalId("control-plane-supervisor".into()),
        authority_context: AuthorityContext(wire::Value::Null),
    });
    let mut runs = Generated::new(RunStore::new(|| RunId(id())));
    let mut context = ContextClock {
        max_steps: input.config.max_steps,
    };
    let result = run_until_blocked(
        &planning.governor,
        &executor,
        &NoAuthority,
        &planning,
        &commission,
        &mut runs,
        &mut context,
    );
    let state = planning
        .state
        .lock()
        .map_err(|_| anyhow::anyhow!("planner state poisoned"))?;
    if let Some(error) = &state.failure {
        bail!("{error}");
    }
    let end = result?;
    ensure!(
        matches!(end.outcome, RunOutcome::Completed(_)),
        "planner ended without a validated plan: {:?}",
        end.outcome
    );
    Ok(EngineOutput {
        stories: state.selected.clone(),
        receipt: json!({"namespace":input.namespace,"run_id":end.run_id.0.0,"revision":state.plan_revision,"transcript":state.transcript,"steps":end.steps,"action_history":state.memory.prompt()}),
    })
}

fn id() -> Uuid {
    Uuid(uuid::Uuid::new_v4().to_string())
}
fn instant() -> String {
    time::OffsetDateTime::now_utc()
        .replace_nanosecond(0)
        .expect("zero nanos")
        .format(&time::format_description::well_known::Rfc3339)
        .expect("UTC timestamp")
}
pub fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key]
        .as_str()
        .with_context(|| format!("missing text field {key}"))
}

impl Planning<'_> {
    fn respond(&self, request: ModelRequest) -> Result<Value> {
        (self.input.progress)(
            "activity",
            &json!({"action":"model.requested","role":request.role,"detail":format!("Waiting for {} response from {}",request.role,request.model),"status":"running"}),
        )?;
        let progress = self.input.progress.clone();
        let role = request.role.clone();
        let continuation = if role == "planner" {
            let mut state = self
                .state
                .lock()
                .map_err(|_| anyhow::anyhow!("planner state poisoned"))?;
            let new = std::mem::take(&mut state.unseen);
            Some(format!(
                "New trusted host observations:\n{}",
                new.join("\n")
            ))
        } else {
            None
        };
        let answer = self.model.respond_in(&request, &crate::ModelEnvironment {
            workspace: self.input.path.clone(),
            max_turns: self.input.config.max_steps as u64,
            cancel: self.input.runner.cancel.clone(),
            progress: Arc::new(move |event| progress("activity", &json!({"action":"loom.event","role":role,"detail":event,"status":"running"}))),
            continuation,
        });
        let (action, status, detail) = match &answer {
            Ok(_) => (
                "model.completed",
                "completed",
                format!("{} response received", request.role),
            ),
            Err(error) => (
                "model.failed",
                "failed",
                format!("{}: {error:#}", request.role),
            ),
        };
        // This also rechecks operator authority after a model call, before any response effect.
        (self.input.progress)(
            "activity",
            &json!({"action":action,"role":request.role,"detail":detail,"status":status}),
        )?;
        answer
    }

    fn remaining(&self) -> Result<std::time::Duration> {
        let seconds = self.input.goal["max_minutes"]
            .as_u64()
            .context("invalid goal time limit")?
            .checked_mul(60)
            .context("goal time limit overflow")?;
        let remaining = std::time::Duration::from_secs(seconds)
            .checked_sub(self.started.elapsed())
            .context("planner exceeded goal time budget")?;
        ensure!(!remaining.is_zero(), "planner exhausted goal time budget");
        Ok(remaining)
    }
    fn prompt(&self) -> Result<String> {
        let state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("planner state poisoned"))?;
        let text = format!(
            "Goal: {}\nAcceptance: {}\nRegistered workspace directories: {}\nRead context files using workspace:<directory_id>/<relative-file>. These directories are read-only; every edit stays inside this planning worktree.\nContext is a bounded working set. Read returns a first page; use read_range with path, start_line and line_count for more. Older observations may leave this working set; retrieve relevant files again when needed. Do not repeat unchanged reads without a reason.\nObserved repository context:\n{}",
            text(&self.input.goal, "objective")?,
            text(&self.input.goal, "acceptance")?,
            self.input.goal["directories"],
            format_args!("{}\n{}", state.memory.prompt(), state.transcript.join("\n"))
        );
        ensure!(
            text.len() <= 160 * 1024,
            "goal and registered directory metadata exceed planner input budget"
        );
        Ok(text)
    }
    fn record(&self, message: String) -> Result<()> {
        let receipt = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| anyhow::anyhow!("planner state poisoned"))?;
            state.unseen.push(message.clone());
            crate::context::push(&mut state.transcript, message);
            json!({"namespace":self.input.namespace,"revision":state.revision,"transcript":state.transcript,"action_history":state.memory.prompt()})
        };
        (self.input.progress)("observation", &receipt)
    }
    fn read_record(&self, key: &str, label: &str, message: String) -> Result<()> {
        let count = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| anyhow::anyhow!("planner state poisoned"))?;
            let count = state
                .memory
                .observed(key.to_owned(), digest(&json!(message)));
            state
                .memory
                .note(&format!("observed {label}; unchanged-result count {count}"));
            count
        };
        self.record(format!("Observed {label}:\n{message}"))?;
        if count > 1 {
            let feedback = format!(
                "Repeated unchanged observation ({count}): {label}. Choose new evidence, make an authorized change, or Finish with the authoritative stories. Re-reading this unchanged result is not progress."
            );
            self.state
                .lock()
                .map_err(|_| anyhow::anyhow!("planner state poisoned"))?
                .memory
                .note(&feedback);
            self.record(feedback)?;
        }
        ensure!(
            count < 4,
            "planner stalled after 4 unchanged observations: {label}"
        );
        Ok(())
    }

    fn syntax_feedback(&self, args: &[String], error: &str) -> Result<OperationResult> {
        let message = format!(
            "AEP syntax feedback: command was rejected, no successful mutation is claimed.\nAttempted args: {}\n{}\nAdmitted grammar: {}",
            crate::context::excerpt(&json!(args).to_string(), 2048),
            crate::context::excerpt(error, 4096),
            AEP_GRAMMAR
        );
        let failures = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| anyhow::anyhow!("planner state poisoned"))?;
            state.syntax_failures += 1;
            let failures = state.syntax_failures;
            state.memory.note(&format!(
                "syntax rejected ({failures}/3): {}",
                crate::context::excerpt(error, 512)
            ));
            state.unseen.push(message.clone());
            crate::context::push(&mut state.transcript, message.clone());
            state.syntax_failures
        };
        (self.input.progress)(
            "activity",
            &json!({"action":"aep.syntax_rejected","role":"planner","status":"failed","detail":message}),
        )?;
        ensure!(
            failures < 3,
            "AEP syntax error budget exhausted after 3 rejected commands"
        );
        Ok(OperationResult::SyntaxFeedback)
    }

    fn perform(&self, action: PlannerAction) -> Result<OperationResult> {
        let path = &self.input.path;
        let mut bounded = self.input.runner.clone();
        bounded.timeout = bounded.timeout.min(self.remaining()?);
        let runner = &bounded;
        let action_value = serde_json::to_value(&action)?;
        let action_key = digest(&action_value);
        let label = crate::context::action_label(&action_value);
        self.state
            .lock()
            .map_err(|_| anyhow::anyhow!("planner state poisoned"))?
            .memory
            .attempted(&label);
        (self.input.progress)("intent", &serde_json::to_value(&action)?)?;
        match action {
            PlannerAction::EssSchema { pointer } => {
                let root = spec_root(path)?;
                let root = if root.is_dir() {
                    root.as_path()
                } else {
                    path.as_path()
                };
                let observation = match crate::ess_reference::lookup(root, runner, &pointer) {
                    Ok(reference) => reference,
                    Err(error) => format!(
                        "ESS reference unavailable: {error:#}. No alternate schema is inferred."
                    ),
                };
                self.read_record(&action_key, &label, observation)?;
            }
            PlannerAction::Read { paths } => {
                if paths.len() > 32 {
                    return Err(InputRefusal::new(
                        "too_many_reads",
                        format!("{} paths requested", paths.len()),
                        "Read at most 32 files per request.",
                    )
                    .into());
                }
                for name in &paths {
                    crate::read_request::parse(name)?;
                }
                for name in paths {
                    let observation = self.read_observation(&name, 256 * 1024, |contents| {
                        crate::context::page(&name, contents, 1, 160)
                    })?;
                    self.read_record(&format!("{action_key}:{name}"), &label, observation)?;
                }
            }
            PlannerAction::ReadRange {
                path: name,
                start_line,
                line_count,
            } => {
                let observation = self.read_observation(&name, 2 * 1024 * 1024, |contents| {
                    crate::context::page(&name, contents, start_line, line_count)
                })?;
                self.read_record(&action_key, &label, observation)?;
            }
            PlannerAction::ReadBytes {
                path: name,
                start_byte,
                byte_count,
            } => {
                let observation = self.read_observation(&name, 2 * 1024 * 1024, |contents| {
                    crate::context::bytes(&name, contents, start_byte, byte_count)
                })?;
                self.read_record(&action_key, &label, observation)?;
            }
            PlannerAction::WriteSpecification {
                path: name,
                contents,
            } => {
                let root = spec_root(path)?;
                let refuse = |code, reason: String| -> anyhow::Error {
                    InputRefusal::new(code, reason, SPECIFICATION_WRITE_HELP).into()
                };
                if crate::read_request::parse(&name)
                    .map_or(true, |(directory, _)| directory.is_some())
                {
                    return Err(refuse(
                        "write_path_syntax",
                        format!("{name} is not a normalized worktree-relative path"),
                    ));
                }
                let file = confined(path, &name, true)?;
                if !file.starts_with(&root) {
                    return Err(refuse(
                        "write_outside_specification_root",
                        format!("{name} is outside the specification root"),
                    ));
                }
                if !specification_path(path, &root, &file) {
                    return Err(refuse(
                        "write_not_specification_source",
                        format!("{name} is not an admitted ESS source"),
                    ));
                }
                if !matches!(
                    file.extension().and_then(|s| s.to_str()),
                    Some("yaml" | "yml")
                ) {
                    return Err(refuse(
                        "write_not_yaml",
                        format!("{name} is not a YAML specification source"),
                    ));
                }
                if contents.len() > 256 * 1024 {
                    return Err(refuse(
                        "write_too_large",
                        format!("{name} would have {} bytes", contents.len()),
                    ));
                }
                match std::fs::read(&file) {
                    Ok(existing) if existing == contents.as_bytes() => {
                        self.read_record(&format!("unchanged-write:{name}"), &label, format!("No change: {name} already contains exactly these bytes. Create the missing domain file, make a substantive correction, or finish; repeating this write is not progress."))?;
                        return Ok(OperationResult::Completed);
                    }
                    Ok(_) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
                }
                std::fs::create_dir_all(file.parent().context("specification has no parent")?)?;
                std::fs::write(&file, contents)?;
                self.changed()?;
                self.record(format!("wrote specification {name}"))?;
            }
            PlannerAction::Aep { args, body } => {
                let canonical = canonical_aep_args(&args);
                let mutation = match validate_aep_args(canonical) {
                    Ok(mutation) => mutation,
                    Err(error) if error.is::<AepSyntax>() => {
                        return self.syntax_feedback(&args, &error.to_string());
                    }
                    Err(error) => return Err(error),
                };
                if mutation {
                    validate_spec(path, runner)?;
                }
                let command = [vec!["plan".into(), "artifact".into()], canonical.to_vec()].concat();
                let output = match runner.run(path, "aep", &command, body.as_deref()) {
                    Ok(output) => output,
                    Err(error) if aep_cli_syntax(&error) => {
                        return self.syntax_feedback(&args, &error.to_string());
                    }
                    Err(error) => return Err(error),
                };
                if mutation {
                    self.changed()?;
                    self.record(format!("Observed aep {}:\n{output}", json!(canonical)))?;
                } else {
                    let label =
                        crate::context::action_label(&json!({"action":"aep","args":canonical}));
                    self.read_record(&digest(&json!(canonical)), &label, output)?;
                }
            }
            PlannerAction::Finish { stories, summary } => {
                // An invalid draft or selection is feedback, not an independent review attempt.
                let validation = validate_spec(path, runner)?;
                let refuse = |code, reason: String| -> anyhow::Error {
                    InputRefusal::new(code, reason, FINISH_HELP).into()
                };
                let unique: BTreeSet<_> = stories.iter().collect();
                if unique.len() != stories.len() {
                    return Err(refuse(
                        "duplicate_story_selection",
                        format!("{} selects a story twice", json!(stories)),
                    ));
                }
                let mut reviewed = Vec::new();
                for story in &stories {
                    if !story.starts_with("story:") || story.contains(char::is_whitespace) {
                        return Err(refuse(
                            "selection_not_a_story",
                            format!("{story} is not an AEP story id"),
                        ));
                    }
                    let shown = match runner.command(
                        path,
                        "aep",
                        &["plan", "artifact", "show", story, "--format", "json"],
                    ) {
                        Err(error) if error.is::<crate::process::ProcessExit>() => {
                            return Err(refuse(
                                "selection_unknown_story",
                                format!("{story} is not in this planning store"),
                            ));
                        }
                        shown => shown?,
                    };
                    let item: Value = serde_json::from_str(&shown)?;
                    if !matches!(text(&item, "status")?, "draft" | "proposed" | "active") {
                        return Err(refuse(
                            "story_not_available",
                            format!("{story} is {}", text(&item, "status")?),
                        ));
                    }
                    if !item["scope"].as_array().is_some_and(|s| !s.is_empty()) {
                        return Err(refuse(
                            "story_without_scope",
                            format!("{story} has no machine-readable scope"),
                        ));
                    }
                    reviewed.push(item);
                }
                {
                    let mut state = self
                        .state
                        .lock()
                        .map_err(|_| anyhow::anyhow!("planner state poisoned"))?;
                    if state.rejected_revision == Some(state.revision) {
                        return Err(refuse(
                            "plan_unchanged_since_rejection",
                            "independent plan critique rejected this unchanged revision".into(),
                        ));
                    }
                    ensure!(
                        state.review_attempts < 2,
                        "independent plan review budget exhausted"
                    );
                    state.review_attempts += 1;
                }
                let plan_validation =
                    runner.command(path, "aep", &["plan", "artifact", "validate"])?;
                self.record(format!("Observed ESS validation:\n{validation}\nObserved AEP validation:\n{plan_validation}"))?;
                let critic_context = format!("critic-{}", uuid::Uuid::new_v4());
                let critique=self.respond(ModelRequest { role:"critic".into(),execution_context:critic_context.clone(),model:text(&self.input.goal,"reviewer_model")?.into(),instructions:"Independently review this proposed plan against the standing goal, repository observations and ESS. Reject duplicate backlog, missing named conformance scenarios, unsafe scope, unsupported dependencies or goal claims unsupported by evidence. You cannot execute tools or grant authority.".into(),prompt:format!("{}\nSelected stories: {}\nSummary: {summary}\nActual validations:\n{validation}\n{plan_validation}",self.prompt()?,serde_json::to_string(&reviewed)?),schema:critique_schema(),timeout:self.remaining()? })?;
                ensure!(
                    critique["reason"]
                        .as_str()
                        .is_some_and(|s| !s.trim().is_empty()),
                    "critic supplied no reasoning"
                );
                if critique["approved"] != true {
                    let attempts = {
                        let mut state = self
                            .state
                            .lock()
                            .map_err(|_| anyhow::anyhow!("planner state poisoned"))?;
                        state.rejected_revision = Some(state.revision);
                        state.evidence.clear();
                        state.review_attempts
                    };
                    (self.input.progress)(
                        "activity",
                        &json!({"action":"plan.review_rejected","role":"critic","status":"failed","detail":critique["reason"],"context":critic_context}),
                    )?;
                    self.record(format!("Independent plan review rejected revision (attempt {attempts}/2): {}. Revise the scoped story/specification to address this feedback before requesting Finish again. An unchanged plan cannot be resubmitted. No assignment has been admitted.", critique["reason"]))?;
                    ensure!(
                        attempts < 2,
                        "independent plan critique rejected after two reviews: {}",
                        critique["reason"]
                    );
                    return Ok(OperationResult::ReviewFeedback);
                }
                (self.input.progress)(
                    "plan-approved",
                    &json!({"context":critic_context,"critique":critique}),
                )?;
                for item in &reviewed {
                    let story = text(item, "id")?;
                    (self.input.progress)(
                        "accept-story",
                        &json!({"story":story,"context":critic_context}),
                    )?;
                    if item["status"] == "draft" {
                        runner.command(
                            path,
                            "aep",
                            &["plan", "artifact", "move", story, "--to", "proposed"],
                        )?;
                    }
                    if item["status"] != "active" {
                        runner.command(
                            path,
                            "aep",
                            &["plan", "artifact", "move", story, "--to", "active"],
                        )?;
                    }
                }
                runner.command(path, "aep", &["plan", "artifact", "validate"])?;
                let ready = ready_stories(path, runner)?;
                ensure!(
                    stories.iter().all(|story| ready.contains(story)),
                    "selected story has unresolved dependencies or blockers"
                );
                let selected = stories;
                self.changed()?;
                self.record(format!(
                    "Plan validated; critic {critic_context}: {critique}"
                ))?;
                let mut state = self
                    .state
                    .lock()
                    .map_err(|_| anyhow::anyhow!("planner state poisoned"))?;
                state.selected = selected;
                let revision = state.plan_revision.clone();
                state.evidence = vec![AttestedEvidence {
                    record: serde_json::from_value(
                        json!({"format":"canon-evidence/1","id":format!("validation-{}",uuid::Uuid::new_v4()),"kind":"plan_validation","result":"pass","subject":"plan","subject_revision":revision}),
                    )?,
                    origin: EvidenceOrigin::PlanValidator {
                        producer: "control-plane-validator".into(),
                        observation: critic_context,
                        revision,
                        succeeded: true,
                    },
                }];
            }
        }
        Ok(OperationResult::Completed)
    }
    fn changed(&self) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("planner state poisoned"))?;
        state.revision += 1;
        state.plan_revision = format!("{}-{}", self.input.namespace, state.revision);
        state.evidence.clear();
        state.memory.changed();
        Ok(())
    }
    fn read_observation(
        &self,
        name: &str,
        limit: u64,
        render: impl FnOnce(&str) -> Result<String>,
    ) -> Result<String> {
        // Confinement, registered roots and symlinks are checked before handling
        // absence as ordinary tool feedback. Other I/O failures stay fatal.
        let file = context_path(&self.input.path, name, &self.input.goal, true)?;
        let read = || -> Result<String> {
            let size = std::fs::metadata(&file)?.len();
            if size > limit {
                return Err(InputRefusal::new(
                    "read_too_large",
                    format!("{name} has {size} bytes"),
                    "Use read_range or read_bytes for a part of a large file.",
                )
                .into());
            }
            String::from_utf8(std::fs::read(&file)?).map_err(|_| {
                InputRefusal::new(
                    "read_not_utf8",
                    format!("{name} is not UTF-8 text"),
                    "Binary and non-UTF-8 files cannot be read.",
                )
                .into()
            })
        };
        match read() {
            Ok(contents) => render(&contents),
            Err(error)
                if error
                    .downcast_ref::<std::io::Error>()
                    .is_some_and(|io| io.kind() == std::io::ErrorKind::NotFound) =>
            {
                Ok(format!(
                    "File not found: {name}. No contents were read. If this is a required ESS source, create it with write_specification before reading or validating it."
                ))
            }
            Err(error) => Err(error),
        }
    }
}

impl ActionSelector for &Planning<'_> {
    fn select(&self, _: &SelectionContext, _: &[CatalogueEntry]) -> Result<Choice, SelectorError> {
        let answer = (|| -> Result<PlannerAction> {
            let response = self.respond(ModelRequest {
                role: "planner".into(),
                execution_context: self.input.namespace.clone(),
                model: text(&self.input.goal, "planner_model")?.into(),
                instructions: format!(
                    "{PLANNER_INSTRUCTIONS}\n{}\n{AEP_GRAMMAR}",
                    crate::read_request::PATH_HELP
                ),
                prompt: self.prompt()?,
                schema: planner_schema(),
                timeout: self.remaining()?,
            })?;
            Ok(serde_json::from_value(response)?)
        })()
        .map_err(|e| SelectorError::Unavailable(e.to_string()))?;
        let action = answer.protocol_action().into();
        self.state
            .lock()
            .map_err(|_| SelectorError::Unavailable("planner state poisoned".into()))?
            .pending = Some(answer);
        Ok(Choice {
            action,
            confidence: None,
        })
    }
    fn strategy(&self) -> SelectionStrategy {
        SelectionStrategy::ReasoningModel
    }
}
impl ArgumentGenerator for &Planning<'_> {
    fn generate(&self, _: &ArgumentContext, _: &CatalogueEntry) -> Result<wire::Value, String> {
        let state = self
            .state
            .lock()
            .map_err(|_| "planner state poisoned".to_owned())?;
        let value = serde_json::to_string(state.pending.as_ref().ok_or("no pending selection")?)
            .map_err(|e| e.to_string())?;
        wire::parse(&value).map_err(|e| format!("{e:?}"))
    }
}
impl EffectPort for Planning<'_> {
    fn performs(&self, action: &str) -> bool {
        matches!(action, "repository.inspect" | "plan.edit" | "plan.validate")
    }
    fn invoke(
        &self,
        _: &Commission<commission_state::Assigned>,
        request: &AdmittedRequest,
    ) -> Result<EffectOutcome, EffectError> {
        let action = self
            .state
            .lock()
            .map_err(|_| EffectError::new("planner state poisoned"))?
            .pending
            .take()
            .ok_or_else(|| EffectError::new("missing selected arguments"))?;
        if action.protocol_action() != request.data().action {
            return Err(EffectError::new("selected action changed"));
        }
        let performed = self.perform(action).or_else(|error| {
            if let Some(refusal) = crate::refusal::refusal_of(&error) {
                self.refusals.refused(&refusal)?;
                let observation = refusal.observation();
                self.record(format!("Refused action: {observation}"))?;
                Ok(OperationResult::Refused(observation))
            } else if ess_validation_refusal(&error) {
                self.read_record("ess-validation", "ESS validation refusal", format!("{error}\nNo plan validation evidence or AEP mutation was produced. Correct the specification before retrying."))?;
                Ok(OperationResult::ValidationFeedback)
            } else {
                Err(error)
            }
        }).and_then(|result| {
            let state = self
                .state
                .lock()
                .map_err(|_| anyhow::anyhow!("planner state poisoned"))?;
            self.governor
                .update_revision(&self.case_id, "plan", &state.plan_revision)
                .map_err(|e| anyhow::anyhow!("{e:?}"))?;
            for item in &state.evidence {
                crate::governance::submit(&self.governor, &self.case_id, item, None)?;
            }
            Ok(result)
        });
        match performed {
            Ok(OperationResult::Refused(reason)) => {
                Ok(EffectOutcome::Refused(EffectOutcomeRefused { reason }))
            }
            Ok(result) => {
                self.refusals.admitted();
                Ok(EffectOutcome::Performed(EffectOutcomePerformed {
                    report: wire::Value::Text(
                        match result {
                            OperationResult::Completed => "host operation completed",
                            OperationResult::SyntaxFeedback => {
                                "command syntax rejected; corrective feedback recorded"
                            }
                            OperationResult::ReviewFeedback => {
                                "plan review rejected; revision feedback recorded"
                            }
                            OperationResult::ValidationFeedback => {
                                "ESS validation refused; corrective diagnostics recorded"
                            }
                            OperationResult::Refused(_) => {
                                unreachable!("handled as refused effect")
                            }
                        }
                        .into(),
                    ),
                }))
            }
            Err(error) => {
                let message = error.to_string();
                if let Ok(mut state) = self.state.lock() {
                    state.failure = Some(message.clone());
                }
                Err(EffectError::new(message))
            }
        }
    }
}
pub(crate) struct NoAuthority;
impl AuthorityProvider for NoAuthority {
    fn decide(
        &self,
        _: &CommissionData,
        _: &str,
    ) -> Result<AuthorityVerdict, AuthorityProviderError> {
        Ok(AuthorityVerdict::Deny(AuthorityVerdictDeny {
            reason: "planner has no publication authority".into(),
        }))
    }
}
struct ContextClock {
    max_steps: usize,
}
impl LoopContext for ContextClock {
    fn action_request_id(&mut self) -> ActionRequestId {
        ActionRequestId(id())
    }
    fn observation_id(&mut self) -> ObservationId {
        ObservationId(id())
    }
    fn now(&mut self) -> Timestamp {
        Timestamp(instant())
    }
    fn step_budget(&self) -> Option<usize> {
        Some(self.max_steps)
    }
}

// Returns whether the admitted command can mutate the planning store.
fn validate_aep_args(args: &[String]) -> Result<bool> {
    ensure!(args.len() <= 128, "invalid AEP argument count");
    if args.is_empty() {
        return Err(AepSyntax("missing artifact verb; request --help for grammar").into());
    }
    let admitted_verb = |verb: &str| {
        matches!(
            verb,
            "new"
                | "body"
                | "scope"
                | "relate"
                | "unrelate"
                | "show"
                | "list"
                | "kinds"
                | "lifecycle"
                | "relations"
        )
    };
    let help_flag = |arg: &str| matches!(arg, "--help" | "-h");
    let help = match args {
        [flag] => help_flag(flag),
        [verb, flag] => admitted_verb(verb) && help_flag(flag),
        _ => false,
    };
    if help {
        // Clap exits after printing help. No artifact kind, operands, overrides
        // or mutation flags are admitted with this exception.
        return Ok(false);
    }
    // Authority and confinement violations are always fatal, including when a
    // malformed help request or other syntax error is present as well.
    for (index, arg) in args.iter().enumerate() {
        ensure!(
            !["--store", "--root", "--findings"]
                .iter()
                .any(|flag| arg == flag || arg.starts_with(&format!("{flag}="))),
            "AEP path override is forbidden"
        );
        if arg == "--from" {
            ensure!(
                args.get(index + 1).is_some_and(|value| value == "-"),
                "AEP bodies must use stdin"
            );
        }
        ensure!(
            !arg.starts_with("--from="),
            "use --from followed by stdin marker"
        );
    }
    if !admitted_verb(&args[0]) {
        ensure!(
            !matches!(
                args[0].as_str(),
                "move"
                    | "set"
                    | "evidence"
                    | "findings"
                    | "review-value"
                    | "review"
                    | "approve"
                    | "approval"
                    | "publish"
                    | "merge"
                    | "delete"
                    | "remove"
            ),
            "model cannot perform this AEP operation"
        );
        // Unknown/noun-first grammar is never invoked or reinterpreted. Return
        // bounded corrective feedback so the model can choose an admitted verb.
        return Err(AepSyntax("unknown artifact verb or argument order; use [show, story:<id>] to inspect a story, or --help for admitted CLI syntax").into());
    }
    if args[0] == "new" {
        if args.len() == 1 {
            return Err(AepSyntax("new requires an admitted artifact kind").into());
        }
        ensure!(
            matches!(
                args[1].as_str(),
                "story" | "epic" | "task" | "executable-system-specification"
            ),
            "model cannot manufacture approval, review or evidence artifacts"
        );
    }
    if args.iter().any(|arg| help_flag(arg)) {
        return Err(AepSyntax(
            "AEP help requires exactly --help, -h, or an admitted verb followed by a help flag",
        )
        .into());
    }
    Ok(!matches!(
        args[0].as_str(),
        "list" | "show" | "kinds" | "lifecycle" | "relations"
    ))
}

const AEP_GRAMMAR: &str = "AEP args are artifact arguments: --help, or one of new/body/scope/relate/unrelate/show/list/kinds/lifecycle/relations followed by its arguments. For verb help use [verb, --help]. Optional leading prefixes artifact, plan artifact, or aep plan artifact are normalized. New artifact kinds: story, epic, task, executable-system-specification. Bodies use --from - with body. Store/root overrides, evidence, approval and status authority remain forbidden. Correct the syntax and continue the standing goal; do not repeat the same rejected command.";

#[derive(Debug)]
struct AepSyntax(&'static str);
impl std::fmt::Display for AepSyntax {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for AepSyntax {}

fn canonical_aep_args(args: &[String]) -> &[String] {
    for prefix in [
        &["aep", "plan", "artifact"][..],
        &["plan", "artifact"][..],
        &["artifact"][..],
    ] {
        if args.len() >= prefix.len()
            && args
                .iter()
                .zip(prefix)
                .all(|(actual, expected)| actual == expected)
        {
            return &args[prefix.len()..];
        }
    }
    args
}

fn ess_validation_refusal(error: &anyhow::Error) -> bool {
    error
        .downcast_ref::<crate::process::ProcessExit>()
        .is_some_and(|exit| {
            exit.program == "ess"
                && exit.code == Some(1)
                && exit
                    .args
                    .starts_with(&["specify".into(), "validate".into()])
                && exit.stderr.contains(" was refused:\n")
        })
}

fn aep_cli_syntax(error: &anyhow::Error) -> bool {
    error
        .downcast_ref::<crate::process::ProcessExit>()
        .is_some_and(|exit| {
            exit.program == "aep"
                && exit.code == Some(2)
                && exit.stdout.is_empty()
                && [
                    "error: unexpected argument ",
                    "error: the following required arguments were not provided:",
                    "error: invalid value ",
                    "error: unrecognized subcommand ",
                ]
                .iter()
                .any(|prefix| exit.stderr.starts_with(prefix))
                && exit.stderr.contains("\nUsage: aep plan artifact")
                && exit.stderr.contains("For more information, try '--help'.")
        })
}

pub fn spec_root(root: &Path) -> Result<PathBuf> {
    let mut found = BTreeSet::new();
    for relative in ["ess", "spec", "systems", ""] {
        let path = root.join(relative);
        if path.join("system.yaml").is_file() || path.join("ess-inputs.yaml").is_file() {
            found.insert(path);
        }
    }
    ensure!(
        found.len() <= 1,
        "multiple specification roots need explicit operator selection"
    );
    Ok(found.into_iter().next().unwrap_or_else(|| root.join("ess")))
}
pub fn specification_path(repository: &Path, spec: &Path, file: &Path) -> bool {
    let Ok(relative) = file.strip_prefix(spec) else {
        return false;
    };
    if relative
        .components()
        .any(|part| part.as_os_str().to_string_lossy().starts_with('.'))
    {
        return false;
    }
    if spec != repository {
        return true;
    }
    matches!(
        relative.to_str(),
        Some("system.yaml" | "ess-inputs.yaml" | "components.yaml")
    ) || relative.starts_with("domains")
        || relative.starts_with("contracts")
        || relative.starts_with("conformance")
}
pub fn validate_spec(root: &Path, runner: &ProcessRunner) -> Result<String> {
    let spec = spec_root(root)?;
    ensure!(
        spec.join("system.yaml").is_file() || spec.join("ess-inputs.yaml").is_file(),
        "write an ESS specification before AEP mutations"
    );
    runner.command(
        root,
        "ess",
        &[
            "specify",
            "validate",
            "--path",
            spec.to_str().context("specification path is not UTF-8")?,
            "--strict-requires",
        ],
    )
}
const SPECIFICATION_WRITE_HELP: &str = "write_specification takes a worktree-relative YAML path of an admitted ESS source beneath the specification root, at most 256 KiB.";
const FINISH_HELP: &str = "Finish selects distinct existing story ids (story:<name>) in draft, proposed or active status with machine-readable scope; after a rejected review, revise the plan before finishing again.";

pub fn confined(root: &Path, name: &str, missing: bool) -> Result<PathBuf> {
    ensure!(
        root.canonicalize()? == root,
        "tool root changed or contains a symlink"
    );
    let relative = Path::new(name);
    ensure!(
        !relative.as_os_str().is_empty()
            && relative
                .components()
                .all(|part| matches!(part, Component::Normal(_))),
        "path must be a normalized repository-relative path"
    );
    ensure!(
        !relative.starts_with(".git"),
        "Git administrative paths are not tool inputs"
    );
    let mut path = root.to_path_buf();
    for part in relative.components() {
        path.push(part);
        match std::fs::symlink_metadata(&path) {
            Ok(meta) => ensure!(
                !meta.file_type().is_symlink(),
                "symlink paths are not tool inputs"
            ),
            Err(error) if missing && error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(path)
}
pub(crate) fn context_path(
    root: &Path,
    name: &str,
    goal: &Value,
    missing: bool,
) -> Result<PathBuf> {
    let (directory, relative) = crate::read_request::parse(name)?;
    if let Some(id) = directory {
        let directory = goal["directories"]
            .as_array()
            .context("workspace context unavailable")?
            .iter()
            .find(|d| d["directory_id"] == id && d["state"] == "Registered")
            .ok_or_else(|| {
                InputRefusal::new(
                    "context_directory_unknown",
                    format!("{id} is not a registered workspace directory"),
                    "Use a directory_id from the registered workspace directories.",
                )
            })?;
        confined(Path::new(text(directory, "path")?), relative, missing)
    } else {
        confined(root, name, missing)
    }
}
fn inspect(root: &Path, runner: &ProcessRunner) -> Result<String> {
    let files = runner.command(root, "git", &["ls-files"])?;
    let mut context = String::new();
    let spec = spec_root(root)?;
    let relative = spec.strip_prefix(root)?.to_string_lossy();
    let relative = if relative.is_empty() { "." } else { &relative };
    context.push_str(&format!("\nAdmitted specification root: {relative}. write_specification paths are repository-relative, confined to this root. Planning markdown belongs in AEP, never in an invented specifications directory.\n"));
    if !spec.join("system.yaml").exists() && !spec.join("ess-inputs.yaml").exists() {
        context.push_str(&format!("No ESS exists. Bootstrap these two files before AEP mutations. Rename example nouns to the task's domain; retain the ESS keys and typed structure. Do not invent prose keys such as behaviors or security. Begin with the smallest domain; acceptance scenarios belong in the story body.\n{relative}/system.yaml:\nformat: ess/22\nsystem: example\nversion: v1\ndomains: [example.session]\n\n{relative}/domains/session.yaml:\ndomain: example.session\nentities:\n  - name: example.session.Session\n    identity:\n      name: session_id\n      type: Uuid\n    fields:\n      - name: username\n        type: String\n    lifecycle:\n      initial: Active\n      states: [Active]\n      terminal: [Active]\n"));
    }
    for name in ["AGENTS.md", "README.md", "TODO.md", "PLAN.md"] {
        if root.join(name).is_file() {
            let path = confined(root, name, false)?;
            ensure!(
                std::fs::metadata(&path)?.len() <= 256 * 1024,
                "instruction file too large"
            );
            context.push_str(&format!(
                "\n{name}:\n{}",
                crate::context::excerpt(&std::fs::read_to_string(path)?, 8 * 1024)
            ));
        }
    }
    context.push_str("\nRead relevant specification/domain files before editing. Existing TODO/PLAN sources must be migrated with citations, never duplicated or deleted.");
    context.push_str(&format!(
        "\nFile index (read relevant files):\n{}",
        crate::context::excerpt(&files, 8 * 1024)
    ));
    Ok(context)
}
fn ready_stories(root: &Path, runner: &ProcessRunner) -> Result<BTreeSet<String>> {
    let stories: Vec<Value> = serde_json::from_str(&runner.command(
        root,
        "aep",
        &["plan", "artifact", "list", "--format", "json"],
    )?)?;
    Ok(stories
        .iter()
        .filter(|item| {
            item["kind"] == "story"
                && item["status"] == "active"
                && item["blocked_by"].as_array().is_none_or(|b| b.is_empty())
                && item["relations"].as_array().is_none_or(|relations| {
                    relations
                        .iter()
                        .filter(|edge| edge["relation"] == "depends_on")
                        .all(|edge| {
                            stories.iter().any(|target| {
                                target["id"] == edge["target"] && target["status"] == "implemented"
                            })
                        })
                })
        })
        .filter_map(|item| item["id"].as_str().map(str::to_owned))
        .collect())
}
pub fn digest(value: &Value) -> String {
    format!("{:x}", Sha256::digest(value.to_string().as_bytes()))
}

const PLANNER_INSTRUCTIONS: &str = concat!(
    "You are the control-plane planner in an isolated managed worktree. ",
    "The standing goal's objective and acceptance define this task. Existing backlog is context only: reuse stories only when they directly serve that goal. ",
    "Build the smallest validated plan for the goal; do not complete unrelated project backlog. ",
    "Backend and tooling added to a beyond10x repository is Rust; CLIs use clap derive. Frontend JS/TS/JSX/TSX/Vue assets may use explicitly accepted frontend/ or web/ scope, excluding backend/server/tools/scripts subdirectories; this does not authorize a JavaScript backend or tooling. Follow repository AGENTS.md. ",
    "Inspect existing ESS and AEP before changes. Migrate relevant written legacy backlog preserving sources and citing source locations. ",
    "New typed behavior belongs in ESS before any story. Use normal readable YAML. ",
    "For exact authoring syntax use ess_schema with pointer \"\" for the property/definition index, or an RFC6901 pointer such as /definitions/RawEntitySpec. Follow returned $ref pointers. This read-only, version-matched foundation resource is bounded; semantic checks still use ESS. ",
    "Only write_specification may write specification files; only aep may mutate planning artifacts, always through the AEP CLI. ",
    "Use --from - and body for prose; record machine-readable scope. Acceptance must name conformance scenarios. ",
    "Use the recent action journal and unchanged-result feedback to choose new evidence, an authorized change or Finish; repeating unchanged reads is not progress. ",
    "Never fabricate check results, approvals, merge evidence or authority. ",
    "AEP help displays the CLI's complete surface, but only the admitted verbs below are available to you. Do not call move or evidence. ",
    "Finish may select a scoped draft story: the host performs independent review, validation and lifecycle admission. ",
    "Finish selects authoritative story ids and a summary; a separate critic and real validators decide acceptance. An empty selection never means goal completion."
);

#[cfg(test)]
mod aep_help_tests {
    use super::validate_aep_args;

    #[test]
    fn ess_feedback_requires_observed_validation_diagnostics() {
        let refusal = "ess was refused:\n  - [dead_end_state] invalid lifecycle\n";
        let error = |program: &str, code, args: &[&str], stderr: &str| {
            anyhow::Error::new(crate::process::ProcessExit {
                program: program.into(),
                args: args.iter().map(|s| (*s).into()).collect(),
                code,
                stdout: String::new(),
                stderr: stderr.into(),
            })
        };
        let args = ["specify", "validate", "--path", "ess"];
        assert!(super::ess_validation_refusal(&error(
            "ess",
            Some(1),
            &args,
            refusal
        )));
        for (program, code, arguments, message) in [
            ("ess", None, args.as_slice(), refusal),
            ("ess", Some(2), args.as_slice(), refusal),
            ("aep", Some(1), args.as_slice(), refusal),
            ("ess", Some(1), ["--help"].as_slice(), refusal),
            ("ess", Some(1), args.as_slice(), "permission denied"),
        ] {
            assert!(!super::ess_validation_refusal(&error(
                program, code, arguments, message
            )));
        }
        assert!(!super::ess_validation_refusal(&anyhow::anyhow!(refusal)));
    }

    #[test]
    fn syntax_recovery_requires_typed_aep_clap_failure() {
        let stderr = "error: unexpected argument '--typo' found\n\nUsage: aep plan artifact list [OPTIONS]\n\nFor more information, try '--help'.\n";
        let error = |program: &str, code, message: &str| {
            anyhow::Error::new(crate::process::ProcessExit {
                program: program.into(),
                args: vec!["plan".into(), "artifact".into(), "list".into()],
                code,
                stdout: String::new(),
                stderr: message.into(),
            })
        };
        assert!(super::aep_cli_syntax(&error("aep", Some(2), stderr)));
        assert!(!super::aep_cli_syntax(&error("aep", Some(1), stderr)));
        assert!(!super::aep_cli_syntax(&error("git", Some(2), stderr)));
        assert!(!super::aep_cli_syntax(&error("aep", None, stderr)));
        assert!(!super::aep_cli_syntax(&anyhow::anyhow!(stderr.to_owned())));
        assert!(!super::aep_cli_syntax(&error(
            "aep",
            Some(2),
            "error: validation refused\nUsage: aep plan artifact\nFor more information, try '--help'."
        )));
    }

    #[test]
    fn exact_help_forms_are_read_only_for_every_admitted_verb() {
        for flag in ["--help", "-h"] {
            assert!(!validate_aep_args(&[flag.into()]).unwrap());
            for verb in [
                "new",
                "body",
                "scope",
                "relate",
                "unrelate",
                "show",
                "list",
                "kinds",
                "lifecycle",
                "relations",
            ] {
                assert!(!validate_aep_args(&[verb.into(), flag.into()]).unwrap());
            }
        }
        assert!(validate_aep_args(&["new".into(), "story".into(), "example".into()]).unwrap());
    }

    #[test]
    fn help_does_not_admit_overrides_evidence_or_additional_authority() {
        for args in [
            vec!["--help", "--store", "elsewhere"],
            vec!["new", "--help", "--root=elsewhere"],
            vec!["new", "evidence", "--help"],
            vec!["new", "review"],
            vec!["evidence", "--help"],
            vec!["move", "--help"],
            vec!["show", "story:example", "--help"],
            vec!["scope", "--help", "--add", "src/"],
            vec!["help", "new"],
        ] {
            let args = args.into_iter().map(str::to_owned).collect::<Vec<_>>();
            assert!(validate_aep_args(&args).is_err(), "{args:?}");
        }
    }
}
