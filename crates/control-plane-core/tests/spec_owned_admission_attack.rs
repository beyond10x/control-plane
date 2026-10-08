//! Adversarial cases for story:spec-owned-admission, pass 1.
//!
//! The change moved CreateGoal's limit check from the host guard (`as_i64().is_some_and(|n| n > 0)`)
//! to the declared `workers-invalid`, `attempts-invalid` and `minutes-invalid` refusals, and moved
//! RepairAssignment's empty-base check to the declared `base-missing` refusal.
use control_plane_core::{Actor, Store};
use serde_json::{Value, json};

fn identity(outcome: &Value, field: &str) -> String {
    outcome["published"][0]["payload"][field]
        .as_str()
        .unwrap()
        .to_owned()
}

fn goal_body(workspace: &str) -> Value {
    json!({"workspace_id":workspace,"objective":"deliver change","acceptance":"tests",
        "max_workers":3,"max_attempts":3,"max_minutes":60,"planner_model":"scripted",
        "implementor_model":"scripted","reviewer_model":"scripted","merge_authority":true})
}

fn refused(answer: &anyhow::Result<Value>) -> bool {
    answer
        .as_ref()
        .map_or(true, |answer| answer.get("error").is_some())
}

/// A limit the host guard refused before the change (not an exact positive i64) is still refused
/// on the console's path, and creates no goal.
#[tokio::test]
async fn non_integer_goal_limits_are_still_refused() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let ws = identity(
        &store
            .execute(
                "RegisterWorkspace",
                json!({"path":temp.path(),"name":"attack"}),
                Actor::Operator,
            )
            .await
            .unwrap(),
        "workspace_id",
    );
    let mut admitted = Vec::new();
    for field in ["max_workers", "max_attempts", "max_minutes"] {
        for value in [
            json!(1.5),
            json!(2.0),
            json!(u64::MAX),
            json!("3"),
            json!(-0.5),
        ] {
            let mut body = goal_body(&ws);
            body[field] = value.clone();
            let answer = store.execute("CreateGoal", body, Actor::Operator).await;
            if !refused(&answer) {
                admitted.push(format!("{field}={value}: {answer:?}"));
            }
        }
    }
    assert!(admitted.is_empty(), "admitted: {admitted:#?}");
    assert_eq!(store.query("GoalList").unwrap(), json!([]));
}

/// Before the change a repair whose `base_revision` is JSON null named no base (guards.rs:
/// `body.get("base_revision").is_some_and(|base| !base.is_null())`) and kept the assignment's.
/// It must still apply without a new base, not answer `base-missing` or `rebased`.
#[tokio::test]
async fn repair_with_null_base_keeps_its_base() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    assert!(
        std::process::Command::new("git")
            .args(["init", "--initial-branch=main"])
            .arg(&repo)
            .output()
            .unwrap()
            .status
            .success()
    );
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let ws = identity(
        &store
            .execute(
                "RegisterWorkspace",
                json!({"path":temp.path(),"name":"attack"}),
                Actor::Operator,
            )
            .await
            .unwrap(),
        "workspace_id",
    );
    let repository = match store
        .query("RepositoryRegistrationList")
        .unwrap()
        .as_array()
        .unwrap()
        .first()
    {
        Some(row) => row["repository_id"].as_str().unwrap().to_owned(),
        None => identity(
            &store
                .execute(
                    "RegisterRepository",
                    json!({"workspace_id":ws,"path":repo,"name":"repo","common_dir":"",
                        "base_branch":"main","test_command":"t","publish_command":"p"}),
                    Actor::Operator,
                )
                .await
                .unwrap(),
            "repository_id",
        ),
    };
    let goal = identity(
        &store
            .execute("CreateGoal", goal_body(&ws), Actor::Operator)
            .await
            .unwrap(),
        "goal_id",
    );
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    let id = identity(
        &store
            .execute(
                "QueueAssignment",
                json!({"goal_id":goal,"repository_id":repository,"story_id":"story:a",
                    "case_id":"case","worktree_id":"","candidate":"","attempt":0,"reason":"",
                    "implementor_run":"","reviewer_run":"","goal_revision":1}),
                Actor::Supervisor,
            )
            .await
            .unwrap(),
        "assignment_id",
    );
    let claimed = store
        .execute(
            "ClaimAssignment",
            json!({"assignment_id":id,"worktree_id":"tree","implementor_run":"impl-1",
                "base_revision":"base"}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(claimed["outcome"], "applied", "{claimed}");
    let blocked = store
        .execute(
            "BlockAssignment",
            json!({"assignment_id":id,"reason":"blocked"}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(blocked["outcome"], "applied", "{blocked}");
    let repaired = store
        .execute(
            "RepairAssignment",
            json!({"assignment_id":id,"reason":"retry","implementor_run":"impl-2",
                "base_revision":null}),
            Actor::Supervisor,
        )
        .await;
    let row = store.query("AssignmentList").unwrap()[0].clone();
    assert!(
        repaired
            .as_ref()
            .is_ok_and(|answer| answer["outcome"] == "applied"),
        "repair with a null base: {repaired:?}"
    );
    assert_eq!(row["base_revision"], "base", "{row}");
}

/// Every decision in `source`, with `edit` applied to each, appended to a fresh history at `to`
/// exactly as `Store` appends its own (`lib.rs`, `commit`).
async fn rewrite_history(
    source: &std::path::Path,
    to: &std::path::Path,
    edit: impl Fn(&mut Value),
) {
    use eventlog_core::{CommandMeta, EventStore, Expected, NewEvent, StreamId, TenantId};
    let stream = StreamId::new(TenantId::new("local").unwrap(), "host", "state").unwrap();
    let from = eventlog_sqlite::SqliteEventStore::open(source.to_str().unwrap(), "control_plane")
        .await
        .unwrap();
    let slice = from.read_stream(&stream, 0, 1000).await.unwrap();
    let into = eventlog_sqlite::SqliteEventStore::open(to.to_str().unwrap(), "control_plane")
        .await
        .unwrap();
    let mut version = 0;
    for event in slice.events {
        let mut data = event.data.clone();
        edit(&mut data);
        let id = uuid::Uuid::new_v4().to_string();
        let meta = CommandMeta {
            idempotency_key: id.clone(),
            request_hash: eventlog_core::request_hash(&[data.clone()]).unwrap(),
            subject: "local-operator".into(),
            actor: data["actor"].as_str().unwrap().into(),
            request_id: id.clone(),
            trace_id: id,
            causation_id: None,
            causation_depth: 0,
            occurred_at: time::OffsetDateTime::now_utc(),
            claim: None,
        };
        let expected = if version == 0 {
            Expected::NoStream
        } else {
            Expected::Exact(version)
        };
        version = into
            .append(
                &stream,
                expected,
                &[NewEvent::new("HostDecision", 1, data).unwrap()],
                &meta,
            )
            .await
            .unwrap()
            .last_version;
    }
}

/// At the base commit an assignment command naming an unknown assignment reached the generated
/// decision whatever its evidence fields held (the host's evidence checks ran only for a known
/// assignment, `guards.rs` `guard_assignment`), so `CompleteAssignment` with an empty receipt for
/// an unknown assignment was recorded as `not-found`. The added `receipt-missing` input guard now
/// answers that call first, so a store holding such a decision no longer opens.
#[tokio::test]
async fn history_recorded_before_the_added_input_guards_still_opens() {
    let temp = tempfile::tempdir().unwrap();
    let recorded = temp.path().join("recorded.sqlite");
    let unknown = "00000000-0000-4000-8000-000000000000";
    {
        let mut store = Store::open(&recorded).await.unwrap();
        let answer = store
            .execute(
                "CompleteAssignment",
                json!({"assignment_id":unknown,"merge_receipt":"receipt"}),
                Actor::Supervisor,
            )
            .await
            .unwrap();
        assert_eq!(answer["outcome"], "not-found", "{answer}");
    }
    // Control: the rewrite alone keeps a history that opens.
    let copy = temp.path().join("copy.sqlite");
    rewrite_history(&recorded, &copy, |_| {}).await;
    Store::open(&copy).await.unwrap();
    // The same decision as the base commit recorded it: an empty receipt, answered `not-found`.
    let base = temp.path().join("base.sqlite");
    rewrite_history(&recorded, &base, |decision| {
        decision["body"]["merge_receipt"] = json!("");
    })
    .await;
    let opened = Store::open(&base).await;
    assert!(
        opened.is_ok(),
        "a history the base commit recorded no longer opens: {:?}",
        opened.err()
    );
}
