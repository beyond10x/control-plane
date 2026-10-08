//! Adversarial cases for story:terminal-goal-edits, wave 2 pass 2 (attacks 1c58835).
//!
//! The fix lets an UpdateGoal on a Satisfied or Cancelled goal skip the host field checks, so the
//! generated `satisfied` / `cancelled` refusal answers it. These cases probe what that skip could
//! open: an invalid value reaching a stored row, a refusal that does not replay, an open goal that
//! lost a check, and an unknown goal that behaves differently from base.
use control_plane_core::{Actor, Store};
use serde_json::{Value, json};

fn identity(outcome: &Value, field: &str) -> String {
    outcome["published"][0]["payload"][field]
        .as_str()
        .unwrap()
        .to_owned()
}

fn limits() -> Value {
    json!({"objective":"deliver change","acceptance":"tests and independent review",
        "max_workers":3,"max_attempts":3,"max_minutes":60,"planner_model":"scripted",
        "implementor_model":"scripted","reviewer_model":"scripted","merge_authority":true})
}

fn version(store: &Store) -> u64 {
    *store.subscribe().borrow()
}

/// Every value the host field checks refuse, one at a time and all together.
fn invalid_edits(goal: &str) -> Vec<(&'static str, Value)> {
    let edit = |field: &str, value: Value| {
        let mut body = limits();
        body["goal_id"] = json!(goal);
        body[field] = value;
        body
    };
    let mut all = limits();
    all["goal_id"] = json!(goal);
    all["max_workers"] = json!(0);
    all["max_attempts"] = json!(-1);
    all["max_minutes"] = json!(0);
    all["objective"] = json!("");
    all["acceptance"] = json!("   ");
    vec![
        (
            "max_workers must be positive",
            edit("max_workers", json!(0)),
        ),
        (
            "max_attempts must be positive",
            edit("max_attempts", json!(0)),
        ),
        (
            "max_minutes must be positive",
            edit("max_minutes", json!(-1)),
        ),
        ("goal objective is empty", edit("objective", json!(""))),
        (
            "goal acceptance is empty",
            edit("acceptance", json!(" \t ")),
        ),
        ("max_workers must be positive", all),
    ]
}

async fn workspace_and_goal(store: &mut Store, root: &std::path::Path) -> (String, String) {
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
    let mut body = limits();
    body["workspace_id"] = json!(ws);
    let goal = identity(
        &store
            .execute("CreateGoal", body, Actor::Operator)
            .await
            .unwrap(),
        "goal_id",
    );
    (ws, goal)
}

/// Every field the host checks, alone and together, on a Satisfied and on a Cancelled goal:
/// each edit takes the declared refusal, records exactly one refusal decision, stores no value
/// in the goal row, and the store reopens to the same row.
#[tokio::test]
async fn every_field_check_yields_to_the_declared_refusal_on_a_finished_goal() {
    for (command, body, actor, refusal) in [
        (
            "SatisfyGoal",
            json!({"satisfaction_receipt":json!({"kind":"goal_acceptance","goal_revision":1}).to_string(),"receipt_revision":1}),
            Actor::Supervisor,
            "satisfied",
        ),
        ("CancelGoal", json!({}), Actor::Operator, "cancelled"),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let db = temp.path().join("state.sqlite");
        let mut store = Store::open(&db).await.unwrap();
        let (_, goal) = workspace_and_goal(&mut store, temp.path()).await;
        store
            .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
            .await
            .unwrap();
        let mut finish = body.clone();
        finish["goal_id"] = json!(goal);
        assert_eq!(
            store.execute(command, finish, actor).await.unwrap()["outcome"],
            "applied"
        );
        let before = store.query("GoalList").unwrap()[0].clone();
        for (rule, edit) in invalid_edits(&goal) {
            let at = version(&store);
            let answer = store
                .execute("UpdateGoal", edit.clone(), Actor::Operator)
                .await
                .unwrap_or_else(|error| panic!("{refusal} / {rule}: {error:#}"));
            assert_eq!(answer["outcome"], refusal, "{rule}: {answer}");
            assert_eq!(
                answer["error"], "controlplane.host.GoalStateConflict",
                "{rule}: {answer}"
            );
            assert_eq!(
                version(&store),
                at + 1,
                "{refusal} / {rule}: one refusal recorded"
            );
            assert_eq!(store.query("GoalList").unwrap()[0], before, "{rule}");
        }
        let recorded = version(&store);
        drop(store);
        let reopened = Store::open(&db).await.unwrap_or_else(|error| {
            panic!("{refusal}: recorded refusals do not replay: {error:#}")
        });
        assert_eq!(version(&reopened), recorded);
        assert_eq!(reopened.query("GoalList").unwrap()[0], before, "{refusal}");
    }
}

/// A Paused and a Running goal keep every field check: each invalid edit is refused by the
/// host with its own message, nothing is recorded and the row is unchanged.
#[tokio::test]
async fn open_goals_keep_every_field_check() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let (_, goal) = workspace_and_goal(&mut store, temp.path()).await;
    for state in ["Paused", "Running"] {
        if state == "Running" {
            store
                .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
                .await
                .unwrap();
        }
        let before = store.query("GoalList").unwrap()[0].clone();
        assert_eq!(before["state"], state);
        let mut missing = limits();
        missing["goal_id"] = json!(goal);
        missing.as_object_mut().unwrap().remove("objective");
        let mut cases = invalid_edits(&goal);
        cases.push(("objective must be a string", missing));
        for (rule, edit) in cases {
            let at = version(&store);
            let error = store
                .execute("UpdateGoal", edit, Actor::Operator)
                .await
                .expect_err(&format!("{state}: {rule} was admitted"));
            assert_eq!(format!("{error:#}"), rule, "{state}");
            assert_eq!(version(&store), at, "{state} / {rule}: nothing recorded");
            assert_eq!(
                store.query("GoalList").unwrap()[0],
                before,
                "{state} / {rule}"
            );
        }
    }
}

/// An edit of an unknown goal, and of a goal that was deleted, behaves as at base: invalid
/// fields are refused by the host with nothing recorded; a valid edit answers `not-found`.
#[tokio::test]
async fn edit_of_unknown_or_deleted_goal_behaves_as_at_base() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let (_, deleted) = workspace_and_goal(&mut store, temp.path()).await;
    store
        .execute("CancelGoal", json!({"goal_id":deleted}), Actor::Operator)
        .await
        .unwrap();
    assert_eq!(
        store
            .execute("DeleteGoal", json!({"goal_id":deleted}), Actor::Operator)
            .await
            .unwrap()["outcome"],
        "applied"
    );
    for goal in [
        "7f0c2a9e-5b1d-4c3e-9a8f-0123456789ab".to_owned(),
        deleted.clone(),
    ] {
        for (rule, edit) in invalid_edits(&goal) {
            let at = version(&store);
            let error = store
                .execute("UpdateGoal", edit, Actor::Operator)
                .await
                .expect_err(&format!("{goal}: {rule} was admitted"));
            assert_eq!(format!("{error:#}"), rule, "{goal}");
            assert_eq!(version(&store), at, "{goal} / {rule}: nothing recorded");
        }
        let mut edit = limits();
        edit["goal_id"] = json!(goal);
        let at = version(&store);
        let answer = store
            .execute("UpdateGoal", edit, Actor::Operator)
            .await
            .unwrap();
        assert_eq!(answer["outcome"], "not-found", "{answer}");
        assert_eq!(
            answer["error"], "controlplane.host.GoalNotFound",
            "{answer}"
        );
        assert_eq!(version(&store), at + 1);
    }
    assert_eq!(
        store.query("GoalList").unwrap().as_array().unwrap().len(),
        0
    );
}

/// The skip covers the field checks, not decoding: a malformed edit of a finished goal (a
/// missing field, a limit sent as text, a fractional limit) is still refused before anything is
/// recorded, and the goal row is unchanged.
#[tokio::test]
async fn malformed_edit_of_finished_goal_records_nothing() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let (_, goal) = workspace_and_goal(&mut store, temp.path()).await;
    store
        .execute("CancelGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    let before = store.query("GoalList").unwrap()[0].clone();
    let mut missing = limits();
    missing["goal_id"] = json!(goal);
    missing.as_object_mut().unwrap().remove("objective");
    let mut text = limits();
    text["goal_id"] = json!(goal);
    text["max_workers"] = json!("3");
    let mut fraction = limits();
    fraction["goal_id"] = json!(goal);
    fraction["max_minutes"] = json!(1.5);
    for edit in [missing, text, fraction] {
        let at = version(&store);
        let answer = store
            .execute("UpdateGoal", edit.clone(), Actor::Operator)
            .await;
        assert!(answer.is_err(), "{edit} was answered {answer:?}");
        assert_eq!(version(&store), at, "{edit}: nothing recorded");
        assert_eq!(store.query("GoalList").unwrap()[0], before);
    }
}
