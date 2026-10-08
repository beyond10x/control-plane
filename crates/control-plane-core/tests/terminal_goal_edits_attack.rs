//! Adversarial cases for story:terminal-goal-edits, wave 2 pass 1.
//!
//! Acceptance: "UpdateGoal on a Satisfied or a Cancelled goal is refused with a declared outcome".
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

/// A workspace and one goal, started and then finished by `finish`.
async fn finished(
    store: &mut Store,
    root: &std::path::Path,
    finish: (&str, Value, Actor),
) -> String {
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
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    let (command, mut body, actor) = finish;
    body["goal_id"] = json!(goal);
    assert_eq!(
        store.execute(command, body, actor).await.unwrap()["outcome"],
        "applied"
    );
    goal
}

/// The CLI forwards an operator's limit as given (`control-plane goal edit <id> --max-workers 0`),
/// so an edit of a satisfied goal can carry a non-positive limit. The acceptance says the edit of
/// a satisfied goal is refused with a declared outcome; here the host's field guard
/// (`guards.rs`, `CreateGoal | UpdateGoal`) answers first with an undeclared error, so the
/// declared `satisfied` refusal is never returned and no refusal decision is recorded.
#[tokio::test]
async fn invalid_edit_of_satisfied_goal_answers_the_declared_refusal() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let satisfy = (
        "SatisfyGoal",
        json!({"satisfaction_receipt":json!({"kind":"goal_acceptance","goal_revision":1}).to_string(),"receipt_revision":1}),
        Actor::Supervisor,
    );
    let goal = finished(&mut store, temp.path(), satisfy).await;
    let before = store.query("GoalList").unwrap()[0].clone();

    let mut edit = limits();
    edit["goal_id"] = json!(goal);
    edit["objective"] = json!("a different objective");
    edit["max_workers"] = json!(0);
    let answer = store.execute("UpdateGoal", edit, Actor::Operator).await;
    assert_eq!(store.query("GoalList").unwrap()[0], before);
    let answer = answer.unwrap_or_else(|error| {
        panic!("an edit of a Satisfied goal was refused by an undeclared host error instead of the declared `satisfied` outcome: {error:#}")
    });
    assert_eq!(answer["outcome"], "satisfied", "{answer}");
    assert_eq!(
        answer["error"], "controlplane.host.GoalStateConflict",
        "{answer}"
    );
}
