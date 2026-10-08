//! Adversarial cases for story:typed-satisfaction-receipt, wave 7 pass 2: replay of recorded
//! `stale-revision` refusals and the order of refusals around an edit.
use control_plane_core::{Actor, Store};
use serde_json::{Value, json};
use std::path::Path;

fn identity(outcome: &Value, field: &str) -> String {
    outcome["published"][0]["payload"][field]
        .as_str()
        .unwrap()
        .to_owned()
}

fn edit(goal: &str, acceptance: &str) -> Value {
    json!({"goal_id":goal,"objective":"deliver change","acceptance":acceptance,
        "max_workers":3,"max_attempts":3,"max_minutes":60,"planner_model":"scripted",
        "implementor_model":"scripted","reviewer_model":"scripted","merge_authority":true})
}

fn satisfy(goal: &str, receipt: &str, revision: i64) -> Value {
    json!({"goal_id":goal,"satisfaction_receipt":receipt,"receipt_revision":revision})
}

fn receipt(revision: i64) -> String {
    json!({"kind":"goal_acceptance","goal_revision":revision}).to_string()
}

fn goal_row(store: &Store, goal: &str) -> Value {
    store
        .query("GoalList")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["goal_id"] == goal)
        .unwrap()
        .clone()
}

fn version(store: &Store) -> u64 {
    *store.subscribe().borrow()
}

/// A workspace with one repository and a Running goal at revision 2 with one queued
/// assignment, the state the recorded-history script is in when it asks for `stale-revision`.
async fn running_with_unfinished_assignment(store: &mut Store, root: &Path) -> String {
    let repo = root.join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    assert!(
        std::process::Command::new("git")
            .args(["init", "--quiet", "--initial-branch=main"])
            .arg(&repo)
            .status()
            .unwrap()
            .success()
    );
    let ws = identity(
        &store
            .execute(
                "RegisterWorkspace",
                json!({"path":root,"name":"attack"}),
                Actor::Operator,
            )
            .await
            .unwrap(),
        "workspace_id",
    );
    let repository = identity(&store.execute("RegisterRepository",json!({"workspace_id":ws,"path":repo,"name":"repo","common_dir":"untrusted","base_branch":"main","test_command":"cargo test","publish_command":"configured-helper"}),Actor::Operator).await.unwrap(),"repository_id");
    let goal = identity(&store.execute("CreateGoal",json!({"workspace_id":ws,"objective":"deliver change","acceptance":"tests and independent review","max_workers":3,"max_attempts":3,"max_minutes":60,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":true}),Actor::Operator).await.unwrap(),"goal_id");
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    store
        .execute("UpdateGoal", edit(&goal, "edited"), Actor::Operator)
        .await
        .unwrap();
    store.execute("QueueAssignment",json!({"goal_id":goal,"repository_id":repository,"story_id":"story:attack","case_id":"case","worktree_id":"","candidate":"","attempt":0,"reason":"","implementor_run":"","reviewer_run":"","goal_revision":2}),Actor::Supervisor).await.unwrap();
    let row = goal_row(store, &goal);
    assert_eq!(
        (row["state"].clone(), row["revision"].clone()),
        (json!("Running"), json!(2))
    );
    goal
}

/// Item 3. A `stale-revision` refusal answered while host rules also refuse the call (an
/// unfinished assignment, a receipt that is not JSON) is recorded once, replays to the same
/// views, and does not open a way past the host rules: the current revision is still refused
/// for the unfinished assignment, before and after replay, and records nothing.
#[tokio::test]
async fn stale_refusal_over_host_refusals_is_recorded_once_and_replays() {
    let temp = tempfile::tempdir().unwrap();
    let db = temp.path().join("state.sqlite");
    let mut store = Store::open(&db).await.unwrap();
    let goal = running_with_unfinished_assignment(&mut store, temp.path()).await;
    let goals = store.query("GoalList").unwrap();
    let assignments = store.query("AssignmentList").unwrap();

    let at = version(&store);
    let answer = store
        .execute(
            "SatisfyGoal",
            satisfy(&goal, "not json", 1),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(
        answer,
        json!({"outcome":"stale-revision","error":"controlplane.host.GoalStateConflict",
            "payload":{"state":"Running"},"published":[]})
    );
    assert_eq!(version(&store), at + 1);
    assert_eq!(store.query("GoalList").unwrap(), goals);
    assert_eq!(store.query("AssignmentList").unwrap(), assignments);

    for reopen in [false, true] {
        if reopen {
            drop(store);
            store = Store::open(&db).await.unwrap();
            assert_eq!(store.query("GoalList").unwrap(), goals);
            assert_eq!(store.query("AssignmentList").unwrap(), assignments);
        }
        let at = version(&store);
        let error = store
            .execute(
                "SatisfyGoal",
                satisfy(&goal, &receipt(2), 2),
                Actor::Supervisor,
            )
            .await
            .expect_err("the current revision satisfied a goal with an unfinished assignment");
        assert_eq!(format!("{error:#}"), "goal has unfinished assignments");
        assert_eq!(version(&store), at, "reopened {reopen}");
        assert_eq!(goal_row(&store, &goal)["state"], "Running");
    }
}

/// Items 3 and 4. Acceptance races two edits: a satisfaction at revision 1 after the first edit
/// and at revision 2 after the second are both `stale-revision`; the one at revision 3 applies.
/// Replayed, the goal is Satisfied at revision 3 with the revision-3 receipt, and every refusal
/// decision replays to the outcome recorded for it.
#[tokio::test]
async fn stale_refusals_around_edits_replay_in_order() {
    let temp = tempfile::tempdir().unwrap();
    let db = temp.path().join("state.sqlite");
    let mut store = Store::open(&db).await.unwrap();
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
    let goal = identity(&store.execute("CreateGoal",json!({"workspace_id":ws,"objective":"deliver change","acceptance":"tests and independent review","max_workers":3,"max_attempts":3,"max_minutes":60,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":true}),Actor::Operator).await.unwrap(),"goal_id");
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    let mut answers = Vec::new();
    for (checked, acceptance) in [(1, "first edit"), (2, "second edit")] {
        store
            .execute("UpdateGoal", edit(&goal, acceptance), Actor::Operator)
            .await
            .unwrap();
        answers.push(
            store
                .execute(
                    "SatisfyGoal",
                    satisfy(&goal, &receipt(checked), checked),
                    Actor::Supervisor,
                )
                .await
                .unwrap()["outcome"]
                .clone(),
        );
    }
    answers.push(
        store
            .execute(
                "SatisfyGoal",
                satisfy(&goal, &receipt(3), 3),
                Actor::Supervisor,
            )
            .await
            .unwrap()["outcome"]
            .clone(),
    );
    assert_eq!(
        answers,
        [
            json!("stale-revision"),
            json!("stale-revision"),
            json!("applied")
        ]
    );
    let satisfied = goal_row(&store, &goal);
    assert_eq!(satisfied["state"], "Satisfied");
    assert_eq!(satisfied["revision"], 3);
    assert_eq!(satisfied["satisfaction_receipt"], receipt(3));
    let at = version(&store);
    drop(store);
    let store = Store::open(&db).await.unwrap();
    assert_eq!(version(&store), at);
    assert_eq!(goal_row(&store, &goal), satisfied);
}
