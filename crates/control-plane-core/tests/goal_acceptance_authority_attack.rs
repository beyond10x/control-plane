//! Adversarial cases for story:goal-acceptance-authority, wave 3 pass 1 (attacks 95274aa).
//!
//! The SatisfyGoal admission check admits a Running goal only when the receipt's integer
//! `goal_revision` equals the stored revision, and leaves every other goal to its declared
//! refusal. These cases pin the parts of that claim the unit's suite does not exercise.
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

fn receipt(goal_revision: Value) -> String {
    json!({"kind":"goal_acceptance","goal_revision":goal_revision}).to_string()
}

/// Equality, not "at least": a receipt naming a revision the goal has not reached is refused,
/// as is one whose revision is not an integer. A Cancelled goal and an unknown goal answer
/// their declared refusals whatever the receipt holds. Only the declared refusals are recorded.
#[tokio::test]
async fn only_the_exact_revision_is_admitted_and_other_goals_keep_declared_refusals() {
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
    let create = || {
        let mut body = limits();
        body["workspace_id"] = json!(ws);
        body
    };
    let running = identity(
        &store
            .execute("CreateGoal", create(), Actor::Operator)
            .await
            .unwrap(),
        "goal_id",
    );
    store
        .execute("StartGoal", json!({"goal_id":running}), Actor::Operator)
        .await
        .unwrap();
    let before = store.query("GoalList").unwrap();
    let version = *store.subscribe().borrow();
    // story:typed-satisfaction-receipt: a revision that is not an integer, named in
    // `receipt_revision` or left out of it, is refused by the host and records nothing.
    for (named, message) in [
        (
            json!(1.0),
            "goal satisfaction names no integer receipt_revision; the goal is at revision 1",
        ),
        (
            json!(null),
            "goal satisfaction names no integer receipt_revision; the goal is at revision 1",
        ),
    ] {
        let error = store
            .execute(
                "SatisfyGoal",
                json!({"goal_id":running,"satisfaction_receipt":receipt(named.clone()),
                    "receipt_revision":named.clone()}),
                Actor::Supervisor,
            )
            .await
            .expect_err(&named.to_string());
        assert_eq!(format!("{error:#}"), message);
    }
    assert_eq!(*store.subscribe().borrow(), version);
    assert_eq!(store.query("GoalList").unwrap(), before);
    // A revision the goal has not reached gets the declared `stale-revision` refusal, recorded
    // as one decision; the goal is unchanged.
    let answer = store
        .execute(
            "SatisfyGoal",
            json!({"goal_id":running,"satisfaction_receipt":receipt(json!(2)),"receipt_revision":2}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(answer["outcome"], "stale-revision", "{answer}");
    assert_eq!(answer["error"], "controlplane.host.GoalStateConflict");
    assert_eq!(*store.subscribe().borrow(), version + 1);
    assert_eq!(store.query("GoalList").unwrap(), before);

    let cancelled = identity(
        &store
            .execute("CreateGoal", create(), Actor::Operator)
            .await
            .unwrap(),
        "goal_id",
    );
    store
        .execute("CancelGoal", json!({"goal_id":cancelled}), Actor::Operator)
        .await
        .unwrap();
    let unknown = "00000000-0000-4000-8000-000000000000";
    for (goal, outcome, error) in [
        (
            cancelled.as_str(),
            "wrong-state",
            "controlplane.host.GoalStateConflict",
        ),
        (unknown, "not-found", "controlplane.host.GoalNotFound"),
    ] {
        for text in ["not json", "[1]", &receipt(json!(99))] {
            let answer = store
                .execute(
                    "SatisfyGoal",
                    json!({"goal_id":goal,"satisfaction_receipt":text}),
                    Actor::Supervisor,
                )
                .await
                .unwrap_or_else(|refusal| panic!("{goal} {text}: {refusal:#}"));
            assert_eq!(answer["outcome"], outcome, "{goal} {text}: {answer}");
            assert_eq!(answer["error"], error, "{goal} {text}: {answer}");
        }
    }
}
