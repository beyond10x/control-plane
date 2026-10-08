//! Adversarial cases for story:typed-satisfaction-receipt, wave 7 pass 1.
//!
//! SatisfyGoal gains `receipt_revision: Optional<Integer>`; the generated `stale-revision`
//! refusal fires only when it is defined and the goal is Running, and the host guard requires an
//! integer `receipt_revision` for a Running goal. These cases pin the edges of that claim the
//! unit's suite does not exercise.
use control_plane_core::{Actor, Store, contract::ContractStore};
use serde_json::{Value, json};
use std::path::Path;

fn identity(outcome: &Value, field: &str) -> String {
    outcome["published"][0]["payload"][field]
        .as_str()
        .unwrap()
        .to_owned()
}

fn limits(ws: &str) -> Value {
    json!({"workspace_id":ws,"objective":"deliver change","acceptance":"tests and independent review",
        "max_workers":3,"max_attempts":3,"max_minutes":60,"planner_model":"scripted",
        "implementor_model":"scripted","reviewer_model":"scripted","merge_authority":true})
}

fn edit(goal: &str, acceptance: &str) -> Value {
    json!({"goal_id":goal,"objective":"deliver change","acceptance":acceptance,
        "max_workers":3,"max_attempts":3,"max_minutes":60,"planner_model":"scripted",
        "implementor_model":"scripted","reviewer_model":"scripted","merge_authority":true})
}

fn receipt(goal_revision: Value) -> String {
    json!({"kind":"goal_acceptance","goal_revision":goal_revision}).to_string()
}

async fn workspace(store: &mut Store, root: &Path) -> String {
    identity(
        &store
            .execute(
                "RegisterWorkspace",
                json!({"path":root,"name":"attack"}),
                Actor::Operator,
            )
            .await
            .unwrap(),
        "workspace_id",
    )
}

async fn goal(store: &mut Store, ws: &str) -> String {
    identity(
        &store
            .execute("CreateGoal", limits(ws), Actor::Operator)
            .await
            .unwrap(),
        "goal_id",
    )
}

/// A Running goal moved to revision 2 by an operator edit.
async fn running_at_two(store: &mut Store, ws: &str) -> String {
    let goal = goal(store, ws).await;
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    store
        .execute("UpdateGoal", edit(&goal, "edited"), Actor::Operator)
        .await
        .unwrap();
    let row = row(store, &goal);
    assert_eq!(row["revision"], 2, "{row}");
    assert_eq!(row["state"], "Running", "{row}");
    goal
}

fn row(store: &Store, goal: &str) -> Value {
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

/// Attack 2, precedence. A Paused, a Satisfied and a Cancelled goal at revision 2 answer their
/// declared `wrong-state` refusal to a SatisfyGoal naming an older revision, never
/// `stale-revision`, whatever the receipt text holds. Each answer is recorded and replays.
#[tokio::test]
async fn finished_or_paused_goals_answer_wrong_state_not_stale_revision() {
    let temp = tempfile::tempdir().unwrap();
    let db = temp.path().join("state.sqlite");
    let mut store = Store::open(&db).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;

    let paused = running_at_two(&mut store, &ws).await;
    store
        .execute("PauseGoal", json!({"goal_id":paused}), Actor::Operator)
        .await
        .unwrap();
    let cancelled = running_at_two(&mut store, &ws).await;
    store
        .execute("CancelGoal", json!({"goal_id":cancelled}), Actor::Operator)
        .await
        .unwrap();
    // Only one Running goal per workspace: satisfy this one at its current revision.
    let satisfied = running_at_two(&mut store, &ws).await;
    let answer = store
        .execute(
            "SatisfyGoal",
            json!({"goal_id":satisfied,"satisfaction_receipt":receipt(json!(2)),"receipt_revision":2}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(answer["outcome"], "applied", "{answer}");

    for (goal, state) in [
        (&paused, "Paused"),
        (&cancelled, "Cancelled"),
        (&satisfied, "Satisfied"),
    ] {
        let before = row(&store, goal);
        assert_eq!(before["state"], state, "{before}");
        assert_eq!(before["revision"], 2, "{before}");
        for (text, revision) in [
            (receipt(json!(1)), json!(1)),
            (receipt(json!(2)), json!(1)),
            ("not json".to_owned(), json!(1)),
            (receipt(json!(1)), json!(-1)),
            (receipt(json!(1)), json!(i64::MAX)),
        ] {
            let version = *store.subscribe().borrow();
            let answer = store
                .execute(
                    "SatisfyGoal",
                    json!({"goal_id":goal,"satisfaction_receipt":text,"receipt_revision":revision}),
                    Actor::Supervisor,
                )
                .await
                .unwrap_or_else(|error| panic!("{state} {text} {revision}: {error:#}"));
            assert_eq!(
                answer,
                json!({"outcome":"wrong-state","error":"controlplane.host.GoalStateConflict",
                    "payload":{"state":state},"published":[]}),
                "{state} {text} {revision}"
            );
            assert_eq!(
                *store.subscribe().borrow(),
                version + 1,
                "{state} {text} {revision}"
            );
            assert_eq!(row(&store, goal), before, "{state} {text} {revision}");
        }
    }
    let views = store.query("GoalList").unwrap();
    drop(store);
    let reopened = Store::open(&db).await.unwrap();
    assert_eq!(reopened.query("GoalList").unwrap(), views);
}

/// Attack 1, optional input bypass. For a Running goal at revision 2, no `receipt_revision` that
/// is absent, null, a string, a float, a boolean, an object or beyond i64 satisfies the goal,
/// and none of them records a decision, even when the receipt text names the current revision.
#[tokio::test]
async fn no_malformed_receipt_revision_satisfies_a_running_goal() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let goal = running_at_two(&mut store, &ws).await;
    let before = row(&store, &goal);
    let version = *store.subscribe().borrow();
    for revision in [
        None,
        Some(Value::Null),
        Some(json!("2")),
        Some(json!(2.0)),
        Some(json!(true)),
        Some(json!({"goal_revision":2})),
        Some(json!([2])),
        Some(json!(u64::MAX)),
        Some(json!(9_223_372_036_854_775_808_u64)),
        Some(json!(1e20)),
    ] {
        let mut body = json!({"goal_id":goal,"satisfaction_receipt":receipt(json!(2))});
        if let Some(revision) = &revision {
            body["receipt_revision"] = revision.clone();
        }
        let answer = store.execute("SatisfyGoal", body, Actor::Supervisor).await;
        assert!(answer.is_err(), "{revision:?} answered {answer:?}");
        assert_eq!(*store.subscribe().borrow(), version, "{revision:?}");
        assert_eq!(row(&store, &goal), before, "{revision:?}");
    }
}

/// Attack 1, integer boundaries. A Running goal at revision 2 refuses every integer other than
/// 2 that the receipt also names, including 0, negatives and the i64 extremes, with the declared
/// `stale-revision` outcome. The goal stays Running at revision 2 without a receipt.
#[tokio::test]
async fn every_other_integer_revision_is_stale() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let goal = running_at_two(&mut store, &ws).await;
    let before = row(&store, &goal);
    for revision in [1_i64, 3, 0, -1, -2, i64::MIN, i64::MAX, 20, 12] {
        let version = *store.subscribe().borrow();
        let answer = store
            .execute(
                "SatisfyGoal",
                json!({"goal_id":goal,"satisfaction_receipt":receipt(json!(revision)),
                    "receipt_revision":revision}),
                Actor::Supervisor,
            )
            .await
            .unwrap_or_else(|error| panic!("{revision}: {error:#}"));
        assert_eq!(
            answer,
            json!({"outcome":"stale-revision","error":"controlplane.host.GoalStateConflict",
                "payload":{"state":"Running"},"published":[]}),
            "{revision}"
        );
        assert_eq!(*store.subscribe().borrow(), version + 1, "{revision}");
        assert_eq!(row(&store, &goal), before, "{revision}");
    }
}

/// Attack 1, receipt and input disagree. `receipt_revision` names the current revision and the
/// receipt text names another, or the other way round. Neither satisfies the goal: the first is
/// a host refusal that records nothing, the second the declared `stale-revision`.
#[tokio::test]
async fn receipt_and_receipt_revision_must_agree_on_the_current_revision() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let goal = running_at_two(&mut store, &ws).await;
    let before = row(&store, &goal);

    for named in [json!(1), json!(3), json!(2.0), json!("2"), Value::Null] {
        let version = *store.subscribe().borrow();
        let answer = store
            .execute(
                "SatisfyGoal",
                json!({"goal_id":goal,"satisfaction_receipt":receipt(named.clone()),
                    "receipt_revision":2}),
                Actor::Supervisor,
            )
            .await;
        assert!(answer.is_err(), "{named}: {answer:?}");
        assert_eq!(*store.subscribe().borrow(), version, "{named}");
        assert_eq!(row(&store, &goal), before, "{named}");
    }

    let version = *store.subscribe().borrow();
    let answer = store
        .execute(
            "SatisfyGoal",
            json!({"goal_id":goal,"satisfaction_receipt":receipt(json!(2)),"receipt_revision":1}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(answer["outcome"], "stale-revision", "{answer}");
    assert_eq!(*store.subscribe().borrow(), version + 1);
    assert_eq!(row(&store, &goal), before);
}

/// Attack 1 and 4, through `ContractStore::admit`, the admission "as the console and runtime
/// send it". A SatisfyGoal without `receipt_revision` for a Running goal is refused and records
/// nothing; a stale one gets the declared refusal; the current one applies.
#[tokio::test]
async fn contract_admission_requires_and_compares_the_revision() {
    let temp = tempfile::tempdir().unwrap();
    let db = temp.path().join("state.sqlite");
    let goal = {
        let mut store = Store::open(&db).await.unwrap();
        let ws = workspace(&mut store, temp.path()).await;
        running_at_two(&mut store, &ws).await
    };
    let mut contract = ContractStore::open(&db).await.unwrap();
    let before = contract.query("GoalList").unwrap();
    let missing = contract
        .admit(
            "controlplane.host.SatisfyGoal",
            json!({"goal_id":goal,"satisfaction_receipt":receipt(json!(2))}),
            Actor::Supervisor,
        )
        .await;
    assert!(missing.is_err(), "{missing:?}");
    assert_eq!(contract.query("GoalList").unwrap(), before);

    let stale = contract
        .admit(
            "controlplane.host.SatisfyGoal",
            json!({"goal_id":goal,"satisfaction_receipt":receipt(json!(1)),"receipt_revision":1}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(stale["outcome"], "stale-revision", "{stale}");
    assert_eq!(contract.query("GoalList").unwrap(), before);

    let applied = contract
        .admit(
            "controlplane.host.SatisfyGoal",
            json!({"goal_id":goal,"satisfaction_receipt":receipt(json!(2)),"receipt_revision":2}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(applied["outcome"], "applied", "{applied}");
    drop(contract);
    let reopened = Store::open(&db).await.unwrap();
    let after = reopened.query("GoalList").unwrap();
    let after = after
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["goal_id"] == goal)
        .unwrap();
    assert_eq!(after["state"], "Satisfied");
    assert_eq!(after["revision"], 2);
}

/// Attack 5. A recorded `stale-revision` refusal changes no view: every declared view is equal
/// before the refusal, after it and after replay.
#[tokio::test]
async fn recorded_stale_refusal_changes_no_view() {
    const VIEWS: [&str; 6] = [
        "AssignmentList",
        "GoalList",
        "PublicationIntentList",
        "RepositoryRegistrationList",
        "WorkspaceDirectoryList",
        "WorkspaceList",
    ];
    let temp = tempfile::tempdir().unwrap();
    let db = temp.path().join("state.sqlite");
    let mut store = Store::open(&db).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let goal = running_at_two(&mut store, &ws).await;
    let before: Vec<Value> = VIEWS.iter().map(|v| store.query(v).unwrap()).collect();
    let answer = store
        .execute(
            "SatisfyGoal",
            json!({"goal_id":goal,"satisfaction_receipt":receipt(json!(1)),"receipt_revision":1}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(answer["outcome"], "stale-revision", "{answer}");
    let after: Vec<Value> = VIEWS.iter().map(|v| store.query(v).unwrap()).collect();
    assert_eq!(after, before);
    drop(store);
    let reopened = Store::open(&db).await.unwrap();
    let replayed: Vec<Value> = VIEWS.iter().map(|v| reopened.query(v).unwrap()).collect();
    assert_eq!(replayed, before);
}
