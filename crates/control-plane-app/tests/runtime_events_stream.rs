//! The workspace live stream carries what the native Loom runtime actually emitted: a scripted
//! provider behind the production model binding (`CodexAgentModel`) drives a real planner,
//! implementor and review run through Loom sessions and Commission, and the stream served by
//! `/workspaces/{id}/events` is read as a browser reads it. No store row is written by hand.
#[allow(dead_code)]
#[path = "../../control-plane-runtime/tests/support/fleet_fixture.rs"]
mod fleet_fixture;

use axum::{body::Body, http::Request};
use control_plane_app::{AppState, router};
use control_plane_runtime::{CodexAgentModel, Supervisor};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::sync::Notify;
use tower::ServiceExt;

/// Streamed by the provider on every turn; private model material that no frame may carry.
const REASONING: &str = "RAW_REASONING_MUST_NOT_REACH_THE_STREAM";

/// One scripted provider for every role. The implementor's first proposal reads an absolute
/// path, which the host refuses as malformed (`input.refused`); it then writes the answer and
/// finishes. Each turn waits long enough for the stream to send a frame between turns.
struct Provider {
    binding: llm_core::Provenance,
    caps: llm_core::Capabilities,
    absolute: String,
    implementor_turns: AtomicUsize,
}
impl llm_core::Model for Provider {
    fn provenance(&self) -> &llm_core::Provenance {
        &self.binding
    }
    fn capabilities(&self) -> &llm_core::Capabilities {
        &self.caps
    }
    fn turn<'a>(
        &'a self,
        request: &'a llm_core::TurnRequest,
        sink: &'a mut dyn llm_core::StreamSink,
        _: &'a llm_core::Cancel,
    ) -> llm_core::BoxFuture<'a, Result<llm_core::TurnOutcome, llm_core::Error>> {
        Box::pin(async move {
            request.validate_for(&self.binding, &self.caps)?;
            sink.emit(llm_core::StreamEvent::ReasoningDelta {
                text: REASONING.into(),
            })
            .await?;
            std::thread::sleep(Duration::from_millis(400));
            let previous = request
                .items
                .iter()
                .filter(|item| matches!(item, llm_core::Item::ToolCall(_)))
                .count();
            let arguments = if request
                .instructions
                .starts_with("Implement the accepted AEP story")
            {
                self.implementor_turns.fetch_add(1, Ordering::SeqCst);
                match previous {
                    0 => json!({"action":"read","paths":[self.absolute]}),
                    1 => {
                        json!({"action":"write","path":"src/lib.rs","contents":"pub fn answer() -> u32 { 42 }\n"})
                    }
                    _ => {
                        json!({"action":"finish","summary":"Candidate ready for actual checks and independent review"})
                    }
                }
            } else if request.instructions.starts_with("Independently") {
                json!({"approved":true,"reason":"Trusted test output and exact diff establish the accepted answer"})
            } else {
                json!({"action":"finish","stories":["story:deliver"],"summary":"Select existing accepted scope"})
            };
            Ok(llm_core::TurnOutcome {
                stop_reason: llm_core::StopReason::ToolCalls,
                items: vec![
                    llm_core::Item::Opaque {
                        provenance: self.binding.clone(),
                        payload: json!({"retained":true}),
                    },
                    llm_core::Item::ToolCall(llm_core::ToolCall {
                        call_id: llm_core::CallId::new(format!("call-{previous}")).unwrap(),
                        name: request.tools[0].name.clone(),
                        arguments,
                    }),
                ],
                observation: llm_core::TurnObservation {
                    usage: Some(llm_core::Usage {
                        input_tokens: Some(20),
                        output_tokens: Some(10),
                        cached_input_tokens: Some(0),
                        ..Default::default()
                    }),
                    final_usage: true,
                    ..llm_core::TurnObservation::new(self.binding.clone())
                },
            })
        })
    }
}

/// The `data:` payloads of the `operations` events in one body frame.
fn projections(frame: &str) -> Vec<Value> {
    frame
        .split("\n\n")
        .filter(|event| event.lines().any(|line| line == "event: operations"))
        .filter_map(|event| event.lines().find_map(|line| line.strip_prefix("data: ")))
        .map(|data| serde_json::from_str(data).unwrap())
        .collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn native_runtime_events_reach_the_workspace_stream() {
    let fixture = fleet_fixture::fixture(1).await;
    let workspace = fixture.store.lock().await.query("WorkspaceList").unwrap()[0]["workspace_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let state = AppState::new(
        fixture.store.clone(),
        "127.0.0.1:8787".parse().unwrap(),
        Arc::new(Notify::new()),
    );
    let response = router(state)
        .oneshot(
            Request::builder()
                .uri(format!("/workspaces/{workspace}/events"))
                .header("host", "127.0.0.1:8787")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_success(), "{}", response.status());
    let mut body = response.into_body();

    let provider = Arc::new(Provider {
        binding: serde_json::from_value(json!({"protocol":"responses","provider":"fixture","account":"test","endpoint":"offline","model":"scripted","binding_revision":"one"})).unwrap(),
        caps: llm_core::Capabilities {
            tools: true,
            tool_choice: true,
            temperature: false,
            top_p: false,
            reasoning_efforts: vec![],
            context_window: 128000,
            max_output_tokens: 32000,
        },
        absolute: fixture.root.join("repos/repo0/Cargo.toml").display().to_string(),
        implementor_turns: AtomicUsize::new(0),
    });
    let sessions = tempfile::tempdir().unwrap();
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config.clone(),
        Arc::new(CodexAgentModel::with_provider(
            sessions.path().into(),
            provider.clone(),
        )),
    );
    let mut run = tokio::spawn(async move {
        let planned = supervisor.tick().await?;
        anyhow::ensure!(planned.queued == 1, "{planned:?}");
        supervisor.fleet_tick().await
    });

    // Every frame the stream sends while the run goes on, and those that follow its end.
    let mut frames = Vec::new();
    let mut report = None;
    loop {
        let wait = if report.is_some() {
            Duration::from_millis(1500)
        } else {
            Duration::from_secs(120)
        };
        tokio::select! {
            finished = &mut run, if report.is_none() => {
                report = Some(finished.unwrap().unwrap());
            }
            frame = tokio::time::timeout(wait, body.frame()) => match frame {
                Ok(Some(frame)) => {
                    if let Ok(data) = frame.unwrap().into_data() {
                        frames.push(String::from_utf8(data.to_vec()).unwrap());
                    }
                }
                Ok(None) => panic!("the workspace stream closed"),
                Err(_) if report.is_some() => break,
                Err(_) => panic!("the run sent no frame for two minutes"),
            },
        }
    }
    let report = report.unwrap();
    assert!(report.blockers.is_empty(), "{report:?}");
    assert_eq!(provider.implementor_turns.load(Ordering::SeqCst), 3);

    for frame in &frames {
        assert!(
            !frame.contains(REASONING),
            "raw reasoning reached the stream"
        );
    }
    // The goal's activity in the order it happened: each projection lists its newest entries
    // first, and an entry keeps its id across projections.
    let mut seen = Vec::<Value>::new();
    for projection in frames.iter().flat_map(|frame| projections(frame)) {
        let goal = &projection["goals"][0];
        for entry in goal["activity"].as_array().unwrap().iter().rev() {
            if !seen.iter().any(|known| known["id"] == entry["id"]) {
                seen.push(entry.clone());
            }
        }
    }
    let position = |from: usize, action: &str, detail: &str| {
        seen.iter()
            .enumerate()
            .skip(from)
            .find(|(_, entry)| {
                entry["action"] == action && entry["detail"].as_str().unwrap().contains(detail)
            })
            .map(|(index, _)| index)
    };
    let listed = || {
        seen.iter()
            .map(|entry| format!("{} {} {}", entry["action"], entry["role"], entry["detail"]))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let started = position(0, "loom.event", "implementor model turn started")
        .unwrap_or_else(|| panic!("no implementor turn started:\n{}", listed()));
    let usage = position(
        started,
        "loom.event",
        "implementor turn usage: 20 input / 10 output",
    )
    .unwrap_or_else(|| panic!("no usage after the turn started:\n{}", listed()));
    let finished = position(usage, "loom.event", "implementor proposal turn stopped")
        .unwrap_or_else(|| panic!("no finished event after usage:\n{}", listed()));
    let refused = position(finished, "input.refused", "")
        .unwrap_or_else(|| panic!("no input.refused after the turn finished:\n{}", listed()));
    let next = position(started + 1, "loom.event", "implementor model turn started")
        .unwrap_or_else(|| panic!("no second implementor turn:\n{}", listed()));
    assert!(
        started < usage && usage < finished && finished < refused && refused < next,
        "runtime events out of order:\n{}",
        listed()
    );
    assert_eq!(seen[refused]["role"], "implementor");
}
