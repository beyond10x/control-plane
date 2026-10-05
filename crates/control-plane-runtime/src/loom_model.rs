//! Host bindings to Loom's native loop and session store, and LLM's single-turn port.
//! No model retry, compaction, provider wire, or effect execution lives here.
use crate::{AgentModel, ModelEnvironment, ModelRequest};
use anyhow::{Context, Result, bail, ensure};
use loom_sdk::loom::{
    harness::{
        turn_loop::{
            AgentLoop, Budget, DenyAll, LoopConfig, LoopEvent, LoopSink, OutputSchema, RunLedger,
        },
        wire,
    },
    model::{
        primitives::Uuid,
        run::{CommissionRunId, RunEnding, SessionData, SessionId},
    },
    session::SessionFile,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};

pub struct CodexAgentModel {
    pub timeout: Duration,
    pub sessions: PathBuf,
    provider: Option<Arc<dyn llm_core::Model>>,
}
impl CodexAgentModel {
    pub fn new(sessions: PathBuf) -> Self {
        Self {
            timeout: Duration::from_secs(600),
            sessions,
            provider: None,
        }
    }
    /// Bind an already configured foundation provider (also the offline integration seam).
    pub fn with_provider(sessions: PathBuf, provider: Arc<dyn llm_core::Model>) -> Self {
        Self {
            provider: Some(provider),
            ..Self::new(sessions)
        }
    }
}
impl AgentModel for CodexAgentModel {
    fn respond(&self, _: &ModelRequest) -> Result<Value> {
        bail!("Loom model execution requires a host environment")
    }
    fn respond_in(&self, request: &ModelRequest, environment: &ModelEnvironment) -> Result<Value> {
        let provider: Arc<dyn llm_core::Model> = match &self.provider {
            Some(provider) => provider.clone(),
            None => Arc::new(b10x_llm_tool_call::codex_model(&request.model)?),
        };
        run(self, provider, request, environment)
    }
}

fn run(
    host: &CodexAgentModel,
    provider: Arc<dyn llm_core::Model>,
    request: &ModelRequest,
    environment: &ModelEnvironment,
) -> Result<Value> {
    ensure!(
        !environment.cancel.is_cancelled(),
        "model execution cancelled"
    );
    let mut port = ProviderPort::new(provider, host.timeout.min(request.timeout), environment)?;
    let id = SessionId(Uuid(format!(
        "{:x}",
        Sha256::digest(request.execution_context.as_bytes())
    )));
    std::fs::create_dir_all(&host.sessions)?;
    // File existence only selects open/resume. Every load/claim failure is propagated, never
    // treated as an absent session and never repaired by resetting its spend.
    let mut session = if host
        .sessions
        .join(format!("{}.json", id.0.0))
        .try_exists()?
    {
        SessionFile::resume(&host.sessions, &id, &port.wire, &environment.workspace)?
    } else {
        let mut opened = SessionFile::open(
            &SessionData {
                session_id: id.clone(),
                commission_run: CommissionRunId(Uuid(request.execution_context.clone())),
                wire: port.wire.as_str().into(),
            },
            &request.model,
            "llm:codex",
            &environment.workspace,
        )?;
        // Establish a durable record before a model call can cost anything.
        opened.file(&host.sessions, RunEnding::Stopped)?;
        SessionFile::resume(&host.sessions, &id, &port.wire, &environment.workspace)?
    };
    let result = (|| {
        ensure!(session.model == request.model, "session model changed");
        let remaining_turns = environment
            .max_turns
            .checked_sub(session.turns)
            .filter(|n| *n > 0)
            .context("Loom session exhausted the model turn budget")?;
        let mut schema = request.schema.clone();
        schema
            .as_object_mut()
            .context("response schema is not an object")?
            .insert("type".into(), json!("object"));
        let config = LoopConfig::new(&request.model, &request.instructions)
            .with_output_schema(Some(OutputSchema::named(wire::ToolName::new("propose")?, "Return one proposed action or review in this schema. A proposal grants no authority and does not execute an effect.", schema)?))
            .with_context_window(Some(port.provider.capabilities().context_window))
            .with_budget(Budget { max_turns: Some(remaining_turns), max_duration_ms: Some(u64::try_from(request.timeout.as_millis())?), ..Default::default() });
        let mut tools = NoEffects;
        let mut approvals = DenyAll;
        let mut sink = Events {
            environment,
            role: &request.role,
            error: None,
            cancel: port.cancel.clone(),
        };
        let mut items = std::mem::take(&mut session.items);
        let mut spent = RunLedger::default();
        let outcome = AgentLoop::new(&mut port, &mut tools, &mut approvals, config).run_in(
            &mut items,
            &mut spent,
            environment
                .continuation
                .as_deref()
                .unwrap_or(&request.prompt),
            &mut sink,
        );
        session.items = items;
        session.spent(&spent);
        if let Some(error) = sink.error {
            return Err(error);
        }
        let outcome = outcome?;
        ensure!(
            outcome.stop.is_completed(),
            "Loom stopped: {:?}",
            outcome.stop
        );
        outcome
            .structured
            .context("Loom completed without a validated proposal")
    })();
    // A filing failure prevents the caller from seeing an executable proposal.
    session.file(
        &host.sessions,
        if result.is_ok() {
            RunEnding::Answered
        } else {
            RunEnding::Failed
        },
    )?;
    result
}

struct NoEffects;
impl wire::ToolPort for NoEffects {
    fn specs(&self) -> &[wire::ToolSpec] {
        &[]
    }
    fn call(&mut self, _: &wire::ToolCall) -> wire::ToolOutcome {
        wire::ToolOutcome::failed("effects require Commission admission")
    }
}
struct Events<'a> {
    environment: &'a ModelEnvironment,
    role: &'a str,
    error: Option<anyhow::Error>,
    cancel: llm_core::Cancel,
}
impl LoopSink for Events<'_> {
    fn emit(&mut self, event: LoopEvent) {
        if self.error.is_some() {
            return;
        }
        // Runtime counters and lifecycle events are durable UI input. Raw reasoning and partial
        // argument bytes are private model material and are not operator progress messages.
        let summary = match &event {
            LoopEvent::Usage(usage) => format!(
                "{} turn usage: {} input / {} output tokens",
                self.role, usage.input_tokens, usage.output_tokens
            ),
            LoopEvent::TurnStarted { .. } => format!("{} model turn started", self.role),
            LoopEvent::Finished { stop, .. } => {
                format!("{} proposal turn stopped: {stop:?}", self.role)
            }
            LoopEvent::Compacted { .. } => format!("{} context compacted by Loom", self.role),
            _ => return,
        };
        let result = serde_json::to_value(event)
            .map_err(Into::into)
            .and_then(|event| {
                (self.environment.progress)(
                    json!({"role":self.role,"event":event,"summary":summary}),
                )
            });
        if let Err(error) = result {
            self.cancel.cancel();
            self.error = Some(error);
        }
    }
}

/// Lossless type projection. LLM owns credentials, provider I/O and binding provenance;
/// Loom owns continuation, compaction and retry decisions.
struct ProviderPort<'a> {
    provider: Arc<dyn llm_core::Model>,
    wire: wire::WireId,
    deadline: Instant,
    environment: &'a ModelEnvironment,
    cancel: llm_core::Cancel,
}
impl<'a> ProviderPort<'a> {
    fn new(
        provider: Arc<dyn llm_core::Model>,
        timeout: Duration,
        environment: &'a ModelEnvironment,
    ) -> Result<Self> {
        let binding = serde_json::to_vec(provider.provenance())?;
        Ok(Self {
            provider,
            wire: wire::WireId::new(format!("llm-{:x}", Sha256::digest(binding)))?,
            deadline: Instant::now() + timeout,
            environment,
            cancel: llm_core::Cancel::new(),
        })
    }
    fn request(
        &self,
        request: &wire::TurnRequest,
    ) -> Result<llm_core::TurnRequest, wire::WireError> {
        let items = request
            .items
            .iter()
            .map(|item| match item {
                wire::Item::Opaque { wire, payload } => {
                    if wire != &self.wire {
                        return Err(wire::WireError::unsupported("foreign continuation wire"));
                    }
                    // Preserve the full original LLM item, including its exact serving binding.
                    let item: llm_core::Item = decode(payload.clone())?;
                    match &item {
                        llm_core::Item::Opaque { provenance, .. }
                            if provenance == self.provider.provenance() =>
                        {
                            Ok(item)
                        }
                        _ => Err(wire::WireError::unsupported(
                            "foreign or unattributed continuation binding",
                        )),
                    }
                }
                _ => decode(serde_json::to_value(item).map_err(projection)?),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let tools = request
            .tools
            .iter()
            .map(|tool| {
                Ok(llm_core::ToolSpec {
                    name: llm_core::ToolName::new(tool.name.as_str()).map_err(projection)?,
                    description: tool.description.clone(),
                    input_schema: tool.input_schema.clone(),
                })
            })
            .collect::<Result<Vec<_>, wire::WireError>>()?;
        Ok(llm_core::TurnRequest {
            model: request.model.clone(),
            instructions: request.instructions.clone(),
            items,
            tools,
            max_output_tokens: request.max_output_tokens,
            sampling: decode(serde_json::to_value(&request.sampling).map_err(projection)?)?,
            tool_choice: decode(serde_json::to_value(&request.tool_choice).map_err(projection)?)?,
        })
    }
}
fn projection(error: impl std::fmt::Display) -> wire::WireError {
    wire::WireError::protocol(format!("LLM/Loom projection: {error}"))
}
fn decode<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, wire::WireError> {
    serde_json::from_value(value).map_err(projection)
}

impl wire::ModelPort for ProviderPort<'_> {
    fn wire(&self) -> &wire::WireId {
        &self.wire
    }
    fn turn(
        &mut self,
        request: &wire::TurnRequest,
        sink: &mut dyn wire::StreamSink,
    ) -> Result<wire::TurnOutcome, wire::WireError> {
        if self.environment.cancel.is_cancelled() || self.cancel.is_cancelled() {
            return Err(wire::WireError::cancelled());
        }
        let remaining = self
            .deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(wire::WireError::cancelled)?;
        let request = self.request(request)?;
        // A scoped bridge lets LLM's Send async sink forward to Loom's synchronous sink without
        // buffering the entire stream, introducing another provider client, or dropping events.
        let result = std::thread::scope(|scope| {
            let (tx, rx) = mpsc::sync_channel(16);
            let cancel = self.cancel.clone();
            let provider = &self.provider;
            let worker = scope.spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(projection)?;
                runtime.block_on(async {
                    let mut forward = Forward(tx);
                    tokio::time::timeout(remaining, provider.turn(&request, &mut forward, &cancel))
                        .await
                        .map_err(|_| {
                            wire::WireError::new(
                                wire::WireErrorCode::Cancelled,
                                "model deadline exceeded",
                                false,
                            )
                        })?
                        .map_err(|error| {
                            wire::WireError::new(
                                match error.code {
                                    llm_core::ErrorCode::Unauthorized => {
                                        wire::WireErrorCode::Unauthorized
                                    }
                                    llm_core::ErrorCode::Cancelled
                                    | llm_core::ErrorCode::Deadline => {
                                        wire::WireErrorCode::Cancelled
                                    }
                                    llm_core::ErrorCode::TooLarge => wire::WireErrorCode::TooLarge,
                                    llm_core::ErrorCode::RateLimited => {
                                        wire::WireErrorCode::RateLimited
                                    }
                                    llm_core::ErrorCode::Transport
                                    | llm_core::ErrorCode::Unavailable => {
                                        wire::WireErrorCode::Transport
                                    }
                                    llm_core::ErrorCode::Unsupported => {
                                        wire::WireErrorCode::Unsupported
                                    }
                                    llm_core::ErrorCode::Refused => wire::WireErrorCode::Refused,
                                    _ => wire::WireErrorCode::Protocol,
                                },
                                error.to_string(),
                                false,
                            )
                        }) // Product attempt permits no automatic paid resend.
                })
            });
            let mut streamed = 0_u64;
            let mut last_visible = Instant::now();
            loop {
                if self.environment.cancel.is_cancelled() {
                    self.cancel.cancel();
                }
                match rx.recv_timeout(Duration::from_millis(25)) {
                    Ok(event) => {
                        streamed += 1;
                        if streamed == 1 || last_visible.elapsed() >= Duration::from_secs(1) {
                            (self.environment.progress)(json!({"summary":format!("Receiving model response ({streamed} streamed events)"),"event":{"kind":"model-stream","events":streamed}})).map_err(|error| {
                                self.cancel.cancel();
                                projection(error)
                            })?;
                            last_visible = Instant::now();
                        }
                        if !matches!(event, llm_core::StreamEvent::ToolCallStarted { .. }) {
                            sink.emit(decode(serde_json::to_value(event).map_err(projection)?)?);
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
            worker
                .join()
                .map_err(|_| wire::WireError::protocol("LLM turn panicked"))?
        })?;
        (self.environment.progress)(json!({"summary":"Provider completed this turn; observed usage recorded","event":{"kind":"provider-observation","observation":result.observation}})).map_err(projection)?;
        let items = result
            .items
            .into_iter()
            .map(|item| match item {
                llm_core::Item::Opaque { .. } => Ok(wire::Item::Opaque {
                    wire: self.wire.clone(),
                    payload: serde_json::to_value(item).map_err(projection)?,
                }),
                llm_core::Item::UnattributedOpaque { .. } => Err(wire::WireError::unsupported(
                    "provider returned unattributed opaque state",
                )),
                _ => decode(serde_json::to_value(item).map_err(projection)?),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let usage = result.observation.usage.as_ref().and_then(|u| {
            Some(wire::Usage {
                model: result
                    .observation
                    .upstream_model
                    .as_ref()
                    .map_or(self.provider.provenance().model.as_str(), |m| m.as_str())
                    .into(),
                input_tokens: u.input_tokens?,
                output_tokens: u.output_tokens?,
                cached_input_tokens: u.cached_input_tokens?,
                cache_creation_input_tokens: u.cache_creation_input_tokens,
            })
        });
        Ok(wire::TurnOutcome {
            stop_reason: decode(serde_json::to_value(result.stop_reason).map_err(projection)?)?,
            items,
            usage,
        })
    }
}
struct Forward(mpsc::SyncSender<llm_core::StreamEvent>);
impl llm_core::StreamSink for Forward {
    fn emit(
        &mut self,
        event: llm_core::StreamEvent,
    ) -> llm_core::BoxFuture<'_, Result<(), llm_core::Error>> {
        Box::pin(async move {
            self.0.send(event).map_err(|_| {
                llm_core::Error::new(llm_core::ErrorCode::Cancelled, "Loom stream closed")
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llm_core::{BoxFuture, Capabilities, Model, Provenance, TurnObservation};
    use std::sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    };
    struct Scripted {
        binding: Provenance,
        caps: Capabilities,
        calls: AtomicUsize,
    }
    impl Scripted {
        fn new() -> Self {
            Self { binding: serde_json::from_value(json!({"protocol":"responses","provider":"fixture","account":"test","endpoint":"local","model":"fixture","binding_revision":"one"})).unwrap(), caps: Capabilities { tools:true, tool_choice:true, temperature:false, top_p:false, reasoning_efforts:vec![], context_window:128000, max_output_tokens:32000 }, calls: AtomicUsize::new(0) }
        }
    }
    impl Model for Scripted {
        fn provenance(&self) -> &Provenance {
            &self.binding
        }
        fn capabilities(&self) -> &Capabilities {
            &self.caps
        }
        fn turn<'a>(
            &'a self,
            request: &'a llm_core::TurnRequest,
            _: &'a mut dyn llm_core::StreamSink,
            _: &'a llm_core::Cancel,
        ) -> BoxFuture<'a, Result<llm_core::TurnOutcome, llm_core::Error>> {
            Box::pin(async move {
                request.validate_for(&self.binding, &self.caps)?;
                let n = self.calls.fetch_add(1, Ordering::SeqCst);
                if n > 0 {
                    assert!(request.items.iter().any(|item| matches!(item, llm_core::Item::Opaque { provenance, payload } if provenance == &self.binding && payload == &json!({"retained":"verbatim"}))));
                    assert!(
                        request
                            .items
                            .iter()
                            .any(|item| matches!(item, llm_core::Item::ToolResult { .. })),
                        "previous proposal tool result must survive"
                    );
                    assert!(request.items.iter().any(|item| matches!(item, llm_core::Item::UserText { text } if text.contains("File not found"))), "actual refusal observation reaches continuation");
                }
                Ok(llm_core::TurnOutcome {
                    stop_reason: llm_core::StopReason::ToolCalls,
                    items: vec![
                        llm_core::Item::Opaque {
                            provenance: self.binding.clone(),
                            payload: json!({"retained":"verbatim"}),
                        },
                        llm_core::Item::ToolCall(llm_core::ToolCall {
                            call_id: llm_core::CallId::new(format!("call-{n}")).unwrap(),
                            name: llm_core::ToolName::new("propose").unwrap(),
                            arguments: json!({"action":if n==0 {"read"} else {"write"}}),
                        }),
                    ],
                    observation: TurnObservation {
                        usage: Some(llm_core::Usage {
                            input_tokens: Some(12),
                            output_tokens: Some(5),
                            cached_input_tokens: Some(0),
                            ..Default::default()
                        }),
                        final_usage: true,
                        ..TurnObservation::new(self.binding.clone())
                    },
                })
            })
        }
    }
    #[test]
    fn loom_session_retains_continuation_refusals_and_spent_turns_across_restart() -> Result<()> {
        let root = tempfile::tempdir()?;
        let workspace = root.path().join("workspace");
        std::fs::create_dir(&workspace)?;
        let sessions = root.path().join("sessions");
        let host = CodexAgentModel::new(sessions.clone());
        let events = Arc::new(Mutex::new(Vec::new()));
        let saved = events.clone();
        let env = ModelEnvironment {
            workspace,
            max_turns: 2,
            continuation: None,
            cancel: Default::default(),
            progress: Arc::new(move |e| {
                saved.lock().unwrap().push(e);
                Ok(())
            }),
        };
        let provider = Arc::new(Scripted::new());
        let mut request = ModelRequest {
            role: "planner".into(),
            execution_context: "fixture-run".into(),
            model: "fixture".into(),
            instructions: "Propose".into(),
            prompt: "Read missing file".into(),
            schema: json!({"type":"object","properties":{"action":{"type":"string"}},"required":["action"],"additionalProperties":false}),
            timeout: Duration::from_secs(5),
        };
        assert_eq!(
            run(&host, provider.clone(), &request, &env)?["action"],
            "read"
        );
        drop(host);
        let host = CodexAgentModel::new(sessions);
        request.prompt = "File not found: session.yaml. Create the file before reading it.".into();
        assert_eq!(
            run(&host, provider.clone(), &request, &env)?["action"],
            "write"
        );
        assert!(
            run(&host, provider.clone(), &request, &env)
                .unwrap_err()
                .to_string()
                .contains("turn budget")
        );
        assert_eq!(provider.calls.load(Ordering::SeqCst), 2);
        assert!(
            events
                .lock()
                .unwrap()
                .iter()
                .any(|e| e["event"]["kind"] == "usage")
        );
        Ok(())
    }
    #[test]
    fn session_storage_refusal_prevents_any_provider_call() -> Result<()> {
        let root = tempfile::tempdir()?;
        let provider = Arc::new(Scripted::new());
        let host = CodexAgentModel::new(root.path().join("inside-workspace"));
        let env = ModelEnvironment {
            workspace: root.path().into(),
            max_turns: 1,
            continuation: None,
            cancel: Default::default(),
            progress: Arc::new(|_| Ok(())),
        };
        let request = ModelRequest {
            role: "planner".into(),
            execution_context: "blocked".into(),
            model: "fixture".into(),
            instructions: "".into(),
            prompt: "".into(),
            schema: json!({"type":"object"}),
            timeout: Duration::from_secs(1),
        };
        assert!(run(&host, provider.clone(), &request, &env).is_err());
        assert_eq!(provider.calls.load(Ordering::SeqCst), 0);
        Ok(())
    }
    #[test]
    fn streaming_progress_failure_cancels_the_provider_before_deadline() -> Result<()> {
        use std::sync::atomic::AtomicBool;
        struct Waiting {
            inner: Scripted,
            cancelled: AtomicBool,
        }
        impl Model for Waiting {
            fn provenance(&self) -> &Provenance {
                self.inner.provenance()
            }
            fn capabilities(&self) -> &Capabilities {
                self.inner.capabilities()
            }
            fn turn<'a>(
                &'a self,
                _: &'a llm_core::TurnRequest,
                sink: &'a mut dyn llm_core::StreamSink,
                cancel: &'a llm_core::Cancel,
            ) -> BoxFuture<'a, Result<llm_core::TurnOutcome, llm_core::Error>> {
                Box::pin(async move {
                    sink.emit(llm_core::StreamEvent::TextDelta {
                        text: "stream opened".into(),
                    })
                    .await?;
                    cancel.cancelled().await;
                    self.cancelled.store(true, Ordering::SeqCst);
                    Err(llm_core::Error::new(
                        llm_core::ErrorCode::Cancelled,
                        "cancel observed",
                    ))
                })
            }
        }
        for write_fails in [true, false] {
            let root = tempfile::tempdir()?;
            let workspace = root.path().join("workspace");
            std::fs::create_dir(&workspace)?;
            let cancel = llm_core::Cancel::new();
            let signal = cancel.clone();
            let env = ModelEnvironment {
                workspace,
                max_turns: 2,
                continuation: None,
                cancel,
                progress: Arc::new(move |event| {
                    if event["event"]["kind"] == "model-stream" {
                        if write_fails {
                            bail!("injected durable progress failure");
                        }
                        signal.cancel();
                    }
                    Ok(())
                }),
            };
            let provider = Arc::new(Waiting {
                inner: Scripted::new(),
                cancelled: AtomicBool::new(false),
            });
            let host = CodexAgentModel::new(root.path().join("sessions"));
            let request = ModelRequest {
                role: "planner".into(),
                execution_context: "waiting".into(),
                model: "fixture".into(),
                instructions: "".into(),
                prompt: "".into(),
                schema: json!({"type":"object"}),
                timeout: Duration::from_secs(2),
            };
            assert!(run(&host, provider.clone(), &request, &env).is_err());
            assert!(
                provider.cancelled.load(Ordering::SeqCst),
                "provider must observe cancellation, not merely expire at its timeout; write_fails={write_fails}"
            );
        }
        Ok(())
    }
}
