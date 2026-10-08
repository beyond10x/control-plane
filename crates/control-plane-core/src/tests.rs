use super::*;
use serde_json::json;

/// Whether the store refused a command: a host rule's error, or a refusal the specification declares.
fn refused(answer: &Result<Value>) -> bool {
    answer
        .as_ref()
        .map_or(true, |answer| answer.get("error").is_some())
}

fn scratch() -> tempfile::TempDir {
    let root = std::path::PathBuf::from(std::env::var_os("HOME").unwrap())
        .join(".cache/control-plane-host");
    std::fs::create_dir_all(&root).unwrap();
    tempfile::tempdir_in(root).unwrap()
}

#[tokio::test]
async fn durable_state_survives_restart() {
    let temp = scratch();
    let db = temp.path().join("state.sqlite");
    let mut store = Store::open(&db).await.unwrap();
    store
        .execute(
            "RegisterWorkspace",
            json!({"path":temp.path(),"name":"test"}),
            Actor::Operator,
        )
        .await
        .unwrap();
    let before = store.query("WorkspaceList").unwrap();
    assert_eq!(before.as_array().unwrap().len(), 1);
    drop(store);
    let reopened = Store::open(&db).await.unwrap();
    assert_eq!(before, reopened.query("WorkspaceList").unwrap());
}

#[tokio::test]
async fn workspace_registration_is_idempotent() {
    let temp = scratch();
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let first = store
        .execute(
            "RegisterWorkspace",
            json!({"path":temp.path(),"name":"first"}),
            Actor::Operator,
        )
        .await
        .unwrap();
    let again = store
        .execute(
            "RegisterWorkspace",
            json!({"path":temp.path().join("."),"name":"second"}),
            Actor::Operator,
        )
        .await
        .unwrap();
    assert_eq!(first, again);
    assert_eq!(
        store
            .query("WorkspaceList")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn second_service_is_refused_until_first_exits() {
    let temp = scratch();
    let db = temp.path().join("state.sqlite");
    let first = Store::open(&db).await.unwrap();
    assert!(Store::open(&db).await.is_err());
    drop(first);
    assert!(Store::open(&db).await.is_ok());
}

fn identity(outcome: &Value, field: &str) -> String {
    outcome["published"][0]["payload"][field]
        .as_str()
        .unwrap()
        .to_owned()
}
fn goal_body(workspace: &str) -> Value {
    json!({"workspace_id":workspace,"objective":"deliver change","acceptance":"tests and independent review","max_workers":3,"max_attempts":3,"max_minutes":60,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":true})
}

#[tokio::test]
async fn cancelled_goal_can_be_deleted_without_deleting_its_workspace() {
    let temp = tempfile::tempdir().unwrap();
    let db = temp.path().join("delete.sqlite");
    let mut store = Store::open(&db).await.unwrap();
    let workspace = workspace(&mut store, temp.path()).await;
    let goal = goal(&mut store, &workspace).await;
    let refused = store
        .execute("DeleteGoal", json!({"goal_id":goal}), Actor::Operator)
        .await;
    assert!(refused.is_err() || refused.unwrap()["outcome"] != "applied");
    store
        .execute("CancelGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    let deleted = store
        .execute("DeleteGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    assert_eq!(deleted["outcome"], "applied");
    assert!(
        store
            .query("GoalList")
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
    drop(store);
    let reopened = Store::open(db).await.unwrap();
    assert!(
        reopened
            .query("GoalList")
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        reopened.query("WorkspaceList").unwrap()[0]["workspace_id"],
        workspace
    );
}
async fn workspace(store: &mut Store, path: &Path) -> String {
    identity(
        &store
            .execute(
                "RegisterWorkspace",
                json!({"path":path,"name":"workspace"}),
                Actor::Operator,
            )
            .await
            .unwrap(),
        "workspace_id",
    )
}
async fn goal(store: &mut Store, workspace: &str) -> String {
    identity(
        &store
            .execute("CreateGoal", goal_body(workspace), Actor::Operator)
            .await
            .unwrap(),
        "goal_id",
    )
}
fn repository(path: &Path) {
    std::fs::create_dir_all(path).unwrap();
    assert!(
        std::process::Command::new("git")
            .arg("init")
            .arg("--initial-branch=main")
            .arg(path)
            .output()
            .unwrap()
            .status
            .success()
    );
}
async fn register_repository(store: &mut Store, workspace: &str, path: &Path) -> String {
    identity(&store.execute("RegisterRepository",json!({"workspace_id":workspace,"path":path,"name":"repo","common_dir":"untrusted","base_branch":"main","test_command":"cargo test","publish_command":"configured-helper"}),Actor::Operator).await.unwrap(),"repository_id")
}
async fn assignment(store: &mut Store, goal: &str, repository: &str, story: &str) -> String {
    identity(&store.execute("QueueAssignment",json!({"goal_id":goal,"repository_id":repository,"story_id":story,"case_id":"case","worktree_id":"","candidate":"","attempt":0,"reason":"","implementor_run":"","reviewer_run":"","goal_revision":1}),Actor::Supervisor).await.unwrap(),"assignment_id")
}
async fn claim(store: &mut Store, id: &str) -> Result<Value> {
    store.execute("ClaimAssignment",json!({"assignment_id":id,"worktree_id":format!("tree-{id}"),"implementor_run":format!("impl-{id}"),"base_revision":"base"}),Actor::Supervisor).await
}

#[tokio::test]
async fn storage_failure_stops_effects() {
    let temp = scratch();
    let db = temp.path().join("state.sqlite");
    let mut store = Store::open(&db).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let goal = goal(&mut store, &ws).await;
    let before = store.query("GoalList").unwrap();
    store.version += 1; // Real Eventlog SQLite CAS failure, after generated memory staging.
    let mut effects = 0;
    if store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .is_ok()
    {
        effects += 1;
    }
    assert_eq!(effects, 0);
    assert_eq!(store.query("GoalList").unwrap(), before);
    drop(store);
    assert_eq!(
        Store::open(&db).await.unwrap().query("GoalList").unwrap(),
        before
    );
}

#[tokio::test]
async fn one_active_goal_per_workspace() {
    let temp = scratch();
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let first = goal(&mut store, &ws).await;
    let second = goal(&mut store, &ws).await;
    store
        .execute("StartGoal", json!({"goal_id":first}), Actor::Operator)
        .await
        .unwrap();
    assert!(
        store
            .execute("StartGoal", json!({"goal_id":second}), Actor::Operator)
            .await
            .is_err()
    );
    store
        .execute("PauseGoal", json!({"goal_id":first}), Actor::Operator)
        .await
        .unwrap();
    store
        .execute("StartGoal", json!({"goal_id":second}), Actor::Operator)
        .await
        .unwrap();
}

#[tokio::test]
async fn repository_execution_is_exclusive_across_workspaces_and_restart() {
    let temp = scratch();
    let db = temp.path().join("state.sqlite");
    let repo = temp.path().join("repo");
    repository(&repo);
    let mut store = Store::open(&db).await.unwrap();
    let mut assignments = Vec::new();
    for name in ["one", "two"] {
        let path = temp.path().join(name);
        std::fs::create_dir_all(&path).unwrap();
        let ws = workspace(&mut store, &path).await;
        let repo = register_repository(&mut store, &ws, &repo).await;
        let goal = goal(&mut store, &ws).await;
        store
            .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
            .await
            .unwrap();
        assignments.push(assignment(&mut store, &goal, &repo, "story:change").await);
    }
    claim(&mut store, &assignments[0]).await.unwrap();
    assert!(claim(&mut store, &assignments[1]).await.is_err());
    drop(store);
    let mut store = Store::open(&db).await.unwrap();
    assert!(claim(&mut store, &assignments[1]).await.is_err());
    store
        .execute(
            "CancelAssignment",
            json!({"assignment_id":assignments[0]}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    claim(&mut store, &assignments[1]).await.unwrap();
}

#[test]
fn discovery_accepts_pwd_and_immediate_children_only() {
    let temp = scratch();
    repository(&temp.path().join("one"));
    repository(&temp.path().join("two"));
    repository(&temp.path().join("nested/deep"));
    std::fs::create_dir_all(temp.path().join("one/src")).unwrap();
    let parent = discover(temp.path()).unwrap();
    assert_eq!(parent.repositories.len(), 2);
    let pwd = discover(&temp.path().join("one/src")).unwrap();
    assert_eq!(pwd.repositories.len(), 1);
    assert_eq!(pwd.path, temp.path().join("one"));
}

#[tokio::test]
async fn public_actor_cannot_mutate_supervisor_state() {
    let temp = scratch();
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    for command in [
        "QueueAssignment",
        "ClaimAssignment",
        "PreparePublication",
        "SatisfyGoal",
    ] {
        assert!(
            store
                .execute(command, json!({}), Actor::Operator)
                .await
                .unwrap_err()
                .to_string()
                .contains("actor")
        );
    }
}

#[tokio::test]
async fn review_and_publication_require_current_independent_evidence() {
    let temp = scratch();
    let repo_path = temp.path().join("repo");
    repository(&repo_path);
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let repo = register_repository(&mut store, &ws, &repo_path).await;
    let goal = goal(&mut store, &ws).await;
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    let assignment = assignment(&mut store, &goal, &repo, "story:evidence").await;
    claim(&mut store, &assignment).await.unwrap();
    store
        .execute(
            "ReviewAssignment",
            json!({"assignment_id":assignment,"candidate":"candidate","test_revision":"candidate"}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert!(refused(&store.execute("ReadyAssignment",json!({"assignment_id":assignment,"reviewer_run":format!("impl-{assignment}"),"review_revision":"candidate"}),Actor::Supervisor).await));
    assert!(refused(&store.execute("ReadyAssignment",json!({"assignment_id":assignment,"reviewer_run":"reviewer","review_revision":"stale"}),Actor::Supervisor).await));
    store.execute("ReadyAssignment",json!({"assignment_id":assignment,"reviewer_run":"reviewer","review_revision":"candidate"}),Actor::Supervisor).await.unwrap();
    assert!(
        store
            .execute(
                "MergeAssignment",
                json!({"assignment_id":assignment}),
                Actor::Supervisor
            )
            .await
            .is_err()
    );
    let publication = identity(&store.execute("PreparePublication",json!({"assignment_id":assignment,"candidate":"candidate","target":"main","expected_base":"base"}),Actor::Supervisor).await.unwrap(),"publication_id");
    store
        .execute(
            "MergeAssignment",
            json!({"assignment_id":assignment}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    store
        .execute(
            "MarkPublicationUncertain",
            json!({"publication_id":publication}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    store
        .execute(
            "BlockAssignment",
            json!({"assignment_id":assignment,"reason":"connection lost"}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert!(
        store
            .execute(
                "RepairAssignment",
                json!({"assignment_id":assignment,"reason":"retry","implementor_run":"new"}),
                Actor::Supervisor
            )
            .await
            .is_err()
    );
    assert!(
        store
            .execute(
                "ReconcileAssignment",
                json!({"assignment_id":assignment,"merge_receipt":"unobserved"}),
                Actor::Supervisor
            )
            .await
            .is_err()
    );
    store
        .execute(
            "ConfirmPublication",
            json!({"publication_id":publication,"receipt":"observed-target"}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    store
        .execute(
            "ReconcileAssignment",
            json!({"assignment_id":assignment,"merge_receipt":"observed-target"}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(store.query("AssignmentList").unwrap()[0]["state"], "Merged");
}

/// Claim, review, ready and publish `assignment`, then record the publication as the fleet does
/// when the publisher returned without an observed merge: intent Uncertain, assignment Blocked.
async fn unresolved_publication(store: &mut Store, assignment: &str) -> String {
    claim(store, assignment).await.unwrap();
    let candidate = format!("candidate-{assignment}");
    for (command, body) in [
        (
            "ReviewAssignment",
            json!({"assignment_id":assignment,"candidate":candidate,"test_revision":candidate}),
        ),
        (
            "ReadyAssignment",
            json!({"assignment_id":assignment,"reviewer_run":"reviewer","review_revision":candidate}),
        ),
    ] {
        store
            .execute(command, body, Actor::Supervisor)
            .await
            .unwrap();
    }
    let publication = identity(
        &store
            .execute(
                "PreparePublication",
                json!({"assignment_id":assignment,"candidate":candidate,"target":"main","expected_base":"base"}),
                Actor::Supervisor,
            )
            .await
            .unwrap(),
        "publication_id",
    );
    for (command, body) in [
        ("MergeAssignment", json!({"assignment_id":assignment})),
        (
            "MarkPublicationUncertain",
            json!({"publication_id":publication}),
        ),
        (
            "BlockAssignment",
            json!({"assignment_id":assignment,"reason":"publisher returned without an observed merge"}),
        ),
    ] {
        assert_eq!(
            store
                .execute(command, body, Actor::Supervisor)
                .await
                .unwrap()["outcome"],
            "applied"
        );
    }
    publication
}

const NOT_PUBLISHED: &str =
    "The publisher exited and target main does not contain the candidate; closed as not published";

/// story:publication-exit, the review's probe sequence: an uncertain publication refuses repair,
/// cancellation and reconciliation, and after its goal is cancelled it still holds the repository
/// in every workspace. Closing it as not published releases the assignment and the repository.
#[tokio::test]
async fn unpublished_intent_closes_and_frees_repository() {
    let temp = scratch();
    let repo_path = temp.path().join("repo");
    repository(&repo_path);
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let mut goals = Vec::new();
    let mut assignments = Vec::new();
    for name in ["one", "two"] {
        let path = temp.path().join(name);
        std::fs::create_dir_all(&path).unwrap();
        let ws = workspace(&mut store, &path).await;
        let repo = register_repository(&mut store, &ws, &repo_path).await;
        let goal = goal(&mut store, &ws).await;
        store
            .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
            .await
            .unwrap();
        assignments.push(assignment(&mut store, &goal, &repo, "story:change").await);
        goals.push(goal);
    }
    let (first, second) = (&assignments[0], &assignments[1]);
    let publication = unresolved_publication(&mut store, first).await;
    for (command, body) in [
        (
            "RepairAssignment",
            json!({"assignment_id":first,"reason":"retry","implementor_run":"new"}),
        ),
        ("CancelAssignment", json!({"assignment_id":first})),
        (
            "ReconcileAssignment",
            json!({"assignment_id":first,"merge_receipt":"unobserved"}),
        ),
    ] {
        assert!(
            store
                .execute(command, body, Actor::Supervisor)
                .await
                .is_err(),
            "{command} admitted while the publication is unresolved"
        );
    }
    store
        .execute("CancelGoal", json!({"goal_id":goals[0]}), Actor::Operator)
        .await
        .unwrap();
    let held = claim(&mut store, second).await.unwrap_err().to_string();
    assert!(
        held.contains("repository already has an active change"),
        "{held}"
    );

    let operator = store
        .execute(
            "ClosePublication",
            json!({"publication_id":publication,"reason":NOT_PUBLISHED}),
            Actor::Operator,
        )
        .await
        .unwrap_err()
        .to_string();
    assert!(operator.contains("actor"), "{operator}");
    let closed = store
        .execute(
            "ClosePublication",
            json!({"publication_id":publication,"reason":NOT_PUBLISHED}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(closed["outcome"], "applied", "{closed}");
    let intent = store.query("PublicationIntentList").unwrap()[0].clone();
    assert_eq!(intent["state"], "NotPublished", "{intent}");
    assert_eq!(intent["reason"], NOT_PUBLISHED, "{intent}");

    let cancelled = store
        .execute(
            "CancelAssignment",
            json!({"assignment_id":first}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(cancelled["outcome"], "applied", "{cancelled}");
    let claimed = claim(&mut store, second).await.unwrap();
    assert_eq!(claimed["outcome"], "applied", "{claimed}");
}

/// A publication closed as not published can be retried: the assignment is repaired and a new
/// intent for it is admitted, while the closed one stays closed and keeps its reason.
#[tokio::test]
async fn closed_publication_admits_repair_and_a_new_intent() {
    let temp = scratch();
    let repo_path = temp.path().join("repo");
    repository(&repo_path);
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let repo = register_repository(&mut store, &ws, &repo_path).await;
    let goal = goal(&mut store, &ws).await;
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    let assignment = assignment(&mut store, &goal, &repo, "story:retry").await;
    let first = unresolved_publication(&mut store, &assignment).await;
    let empty = store
        .execute(
            "ClosePublication",
            json!({"publication_id":first,"reason":" "}),
            Actor::Supervisor,
        )
        .await;
    assert!(empty.is_err(), "closed without a reason: {empty:?}");
    store
        .execute(
            "ClosePublication",
            json!({"publication_id":first,"reason":NOT_PUBLISHED}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    let again = store
        .execute(
            "ClosePublication",
            json!({"publication_id":first,"reason":"closed twice"}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(again["outcome"], "wrong-state", "{again}");
    let confirmed = store
        .execute(
            "ConfirmPublication",
            json!({"publication_id":first,"receipt":"late receipt"}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(confirmed["outcome"], "wrong-state", "{confirmed}");

    let candidate = "candidate-retry";
    for (command, body) in [
        (
            "RepairAssignment",
            json!({"assignment_id":assignment,"reason":"publication closed as not published","implementor_run":"implementor-retry"}),
        ),
        (
            "ReviewAssignment",
            json!({"assignment_id":assignment,"candidate":candidate,"test_revision":candidate}),
        ),
        (
            "ReadyAssignment",
            json!({"assignment_id":assignment,"reviewer_run":"reviewer-retry","review_revision":candidate}),
        ),
        (
            "PreparePublication",
            json!({"assignment_id":assignment,"candidate":candidate,"target":"main","expected_base":"base"}),
        ),
        ("MergeAssignment", json!({"assignment_id":assignment})),
    ] {
        let answer = store.execute(command, body, Actor::Supervisor).await;
        assert!(
            answer.as_ref().is_ok_and(|answer| matches!(
                answer["outcome"].as_str(),
                Some("applied" | "created")
            )),
            "{command} after the close: {answer:?}"
        );
    }
    let intents = store.query("PublicationIntentList").unwrap();
    let state = |id: &str| {
        intents
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["publication_id"] == id)
            .unwrap()["state"]
            .clone()
    };
    assert_eq!(intents.as_array().unwrap().len(), 2, "{intents}");
    assert_eq!(state(&first), "NotPublished");
    assert_eq!(
        store.query("AssignmentList").unwrap()[0]["state"],
        "Merging"
    );
}

/// ConfirmPublication's receipt check, like ClosePublication's reason check, guards only an open
/// intent: an unknown or settled one answers its declared `not-found` or `wrong-state`.
#[tokio::test]
async fn confirm_receipt_check_leaves_declared_refusals_first() {
    let temp = scratch();
    let repo_path = temp.path().join("repo");
    repository(&repo_path);
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let unknown = store
        .execute(
            "ConfirmPublication",
            json!({"publication_id":"00000000-0000-4000-8000-000000000000","receipt":""}),
            Actor::Supervisor,
        )
        .await;
    assert!(
        unknown
            .as_ref()
            .is_ok_and(|answer| answer["outcome"] == "not-found"),
        "{unknown:?}"
    );
    let ws = workspace(&mut store, temp.path()).await;
    let repo = register_repository(&mut store, &ws, &repo_path).await;
    let goal = goal(&mut store, &ws).await;
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    let assignment = assignment(&mut store, &goal, &repo, "story:receipt").await;
    let publication = unresolved_publication(&mut store, &assignment).await;
    let empty = store
        .execute(
            "ConfirmPublication",
            json!({"publication_id":publication,"receipt":""}),
            Actor::Supervisor,
        )
        .await;
    assert!(
        empty.is_err(),
        "an open intent confirmed without a receipt: {empty:?}"
    );
    store
        .execute(
            "ClosePublication",
            json!({"publication_id":publication,"reason":NOT_PUBLISHED}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    let settled = store
        .execute(
            "ConfirmPublication",
            json!({"publication_id":publication,"receipt":""}),
            Actor::Supervisor,
        )
        .await;
    assert!(
        settled
            .as_ref()
            .is_ok_and(|answer| answer["outcome"] == "wrong-state"),
        "{settled:?}"
    );
}

/// Only a publication closed as not published releases its assignment: one the remote confirmed
/// still refuses repair and cancellation until the assignment is reconciled.
#[tokio::test]
async fn confirmed_publication_still_holds_repair_and_cancellation() {
    let temp = scratch();
    let repo_path = temp.path().join("repo");
    repository(&repo_path);
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let repo = register_repository(&mut store, &ws, &repo_path).await;
    let goal = goal(&mut store, &ws).await;
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    let assignment = assignment(&mut store, &goal, &repo, "story:confirmed").await;
    let publication = unresolved_publication(&mut store, &assignment).await;
    store
        .execute(
            "ConfirmPublication",
            json!({"publication_id":publication,"receipt":"observed-target"}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    for (command, body) in [
        (
            "RepairAssignment",
            json!({"assignment_id":assignment,"reason":"retry","implementor_run":"new"}),
        ),
        ("CancelAssignment", json!({"assignment_id":assignment})),
    ] {
        assert!(
            store
                .execute(command, body, Actor::Supervisor)
                .await
                .is_err(),
            "{command} admitted after the publication was confirmed"
        );
    }
}

#[tokio::test]
async fn changed_goal_revokes_existing_assignments() {
    let temp = scratch();
    let repo_path = temp.path().join("repo");
    repository(&repo_path);
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let repo = register_repository(&mut store, &ws, &repo_path).await;
    let goal = goal(&mut store, &ws).await;
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    let assignment = assignment(&mut store, &goal, &repo, "story:change").await;
    let mut update = goal_body(&ws);
    update.as_object_mut().unwrap().remove("workspace_id");
    update["goal_id"] = json!(goal);
    update["merge_authority"] = json!(false);
    store
        .execute("UpdateGoal", update, Actor::Operator)
        .await
        .unwrap();
    assert!(claim(&mut store, &assignment).await.is_err());
    assert_eq!(store.query("GoalList").unwrap()[0]["revision"], 2);
}

/// story:terminal-goal-edits probe sequence: create, plan, finish the goal, then edit it.
/// The edit is refused with the declared outcome named `refusal`, and the goal row, including
/// objective, acceptance receipt and revision, stays as it was, also after replay.
async fn finished_goal_refuses_edit(finish: (&str, Value, Actor), refusal: &str) -> Value {
    let temp = tempfile::tempdir().unwrap();
    let db = temp.path().join("state.sqlite");
    let mut store = Store::open(&db).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let goal = goal(&mut store, &ws).await;
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    store.execute("RecordPlanningProgress",json!({"goal_id":goal,"planning_revision":1,"planning_fingerprint":"plan","planning_repository":"","planning_worktree_id":"tree","planning_worktree_path":"tree","planning_phase":"Planning","planning_reason":"","planning_receipt":"{}"}),Actor::Supervisor).await.unwrap();
    let (command, mut body, actor) = finish;
    body["goal_id"] = json!(goal);
    let finished = store.execute(command, body, actor).await.unwrap();
    assert_eq!(finished["outcome"], "applied", "{finished}");
    let before = store.query("GoalList").unwrap()[0].clone();

    let mut edit = goal_body(&ws);
    edit.as_object_mut().unwrap().remove("workspace_id");
    edit["goal_id"] = json!(goal);
    edit["objective"] = json!("a different objective");
    edit["acceptance"] = json!("a different acceptance");
    let refused = store
        .execute("UpdateGoal", edit.clone(), Actor::Operator)
        .await
        .unwrap();
    assert_eq!(refused["outcome"], refusal, "{refused}");
    // An edit the field checks would refuse still takes the declared refusal.
    for (field, value) in [("max_workers", json!(0)), ("objective", json!(" "))] {
        let mut invalid = edit.clone();
        invalid[field] = value;
        let refused = store
            .execute("UpdateGoal", invalid, Actor::Operator)
            .await
            .unwrap();
        assert_eq!(refused["outcome"], refusal, "{refused}");
        assert_eq!(refused["error"], "controlplane.host.GoalStateConflict");
    }
    let after = store.query("GoalList").unwrap()[0].clone();
    assert_eq!(after["objective"], "deliver change");
    assert_eq!(after["revision"], 1);
    assert_eq!(after, before);
    drop(store);
    let reopened = Store::open(&db).await.unwrap();
    assert_eq!(reopened.query("GoalList").unwrap()[0], before);
    after
}

#[tokio::test]
async fn satisfied_goal_refuses_edit() {
    let receipt = json!({"kind":"goal_acceptance","goal_revision":1}).to_string();
    let satisfy = json!({"satisfaction_receipt":receipt,"receipt_revision":1});
    let goal =
        finished_goal_refuses_edit(("SatisfyGoal", satisfy, Actor::Supervisor), "satisfied").await;
    assert_eq!(goal["state"], "Satisfied");
    assert_eq!(goal["satisfaction_receipt"], receipt);
}

/// A goal accepted at the revision it was checked at: an operator edit before acceptance moved
/// it to revision 2, acceptance read revision 2, and the receipt it sends names revision 2.
#[tokio::test]
async fn unchanged_goal_is_satisfied_after_acceptance() {
    let temp = tempfile::tempdir().unwrap();
    let db = temp.path().join("state.sqlite");
    let mut store = Store::open(&db).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let goal = goal(&mut store, &ws).await;
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    let mut edit = goal_body(&ws);
    edit.as_object_mut().unwrap().remove("workspace_id");
    edit["goal_id"] = json!(goal);
    edit["objective"] = json!("deliver the edited change");
    store
        .execute("UpdateGoal", edit, Actor::Operator)
        .await
        .unwrap();
    let checked = store.query("GoalList").unwrap()[0].clone();
    assert_eq!(checked["state"], "Running");
    assert_eq!(checked["revision"], 2);
    let receipt = json!({"kind":"goal_acceptance","goal_revision":checked["revision"]}).to_string();
    let satisfied = store
        .execute(
            "SatisfyGoal",
            json!({"goal_id":goal,"satisfaction_receipt":receipt,"receipt_revision":checked["revision"]}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(satisfied["outcome"], "applied", "{satisfied}");
    let after = store.query("GoalList").unwrap()[0].clone();
    assert_eq!(after["state"], "Satisfied");
    assert_eq!(after["revision"], 2);
    let named: Value =
        serde_json::from_str(after["satisfaction_receipt"].as_str().unwrap()).unwrap();
    assert_eq!(named["goal_revision"], 2);
    drop(store);
    let reopened = Store::open(&db).await.unwrap();
    assert_eq!(reopened.query("GoalList").unwrap()[0], after);
}

/// SatisfyGoal is admitted only for the revision it names. Goal acceptance reads the goal at
/// revision 1 and an operator edit moves it to 2. A satisfaction naming revision 1 gets the
/// declared `stale-revision` refusal, which is recorded. A satisfaction that names no integer
/// `receipt_revision`, or whose receipt does not name that same revision, is refused by the host
/// and nothing is recorded. Either way the goal stays Running at revision 2 without a receipt. A
/// goal that cannot be satisfied keeps its declared `wrong-state` refusal, whatever it is sent.
#[tokio::test]
async fn satisfaction_receipt_must_name_the_current_goal_revision() {
    let temp = tempfile::tempdir().unwrap();
    let db = temp.path().join("state.sqlite");
    let mut store = Store::open(&db).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let goal = goal(&mut store, &ws).await;
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    let checked = store.query("GoalList").unwrap()[0]["revision"].clone();
    assert_eq!(checked, 1);
    let stale = json!({"kind":"goal_acceptance","goal_revision":checked}).to_string();
    let mut edit = goal_body(&ws);
    edit.as_object_mut().unwrap().remove("workspace_id");
    edit["goal_id"] = json!(goal);
    edit["acceptance"] = json!("tests, independent review and documentation");
    store
        .execute("UpdateGoal", edit, Actor::Operator)
        .await
        .unwrap();
    let edited = store.query("GoalList").unwrap()[0].clone();
    assert_eq!(edited["revision"], 2);

    let version = store.version;
    let refused = store
        .execute(
            "SatisfyGoal",
            json!({"goal_id":goal,"satisfaction_receipt":stale,"receipt_revision":checked}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(
        refused,
        json!({"outcome":"stale-revision","error":"controlplane.host.GoalStateConflict","payload":{"state":"Running"},"published":[]})
    );
    assert_eq!(
        store.version,
        version + 1,
        "the declared refusal is recorded"
    );
    assert_eq!(store.query("GoalList").unwrap()[0], edited);

    let version = store.version;
    let current = json!({"kind":"goal_acceptance","goal_revision":2}).to_string();
    for (receipt, revision, message) in [
        (
            current.clone(),
            Value::Null,
            "goal satisfaction names no integer receipt_revision; the goal is at revision 2",
        ),
        (
            current.clone(),
            json!("2"),
            "goal satisfaction names no integer receipt_revision; the goal is at revision 2",
        ),
        (
            stale,
            json!(2),
            "goal satisfaction receipt names goal revision 1 but receipt_revision is 2",
        ),
        (
            json!({"kind":"goal_acceptance","goal_revision":"2"}).to_string(),
            json!(2),
            "goal satisfaction receipt names goal revision \"2\" but receipt_revision is 2",
        ),
        (
            json!({"kind":"goal_acceptance"}).to_string(),
            json!(2),
            "goal satisfaction receipt names no goal_revision; receipt_revision is 2",
        ),
        (
            "acceptance-verified".to_owned(),
            json!(2),
            "goal satisfaction receipt is not a JSON object naming its goal_revision; receipt_revision is 2",
        ),
        (
            "2".to_owned(),
            json!(2),
            "goal satisfaction receipt is not a JSON object naming its goal_revision; receipt_revision is 2",
        ),
    ] {
        let mut body = json!({"goal_id":goal,"satisfaction_receipt":receipt});
        if !revision.is_null() {
            body["receipt_revision"] = revision.clone();
        }
        let error = store
            .execute("SatisfyGoal", body, Actor::Supervisor)
            .await
            .expect_err(&format!("{receipt} {revision}"));
        assert_eq!(format!("{error:#}"), message, "{receipt} {revision}");
        assert_eq!(store.version, version, "{receipt} {revision}");
        assert_eq!(store.query("GoalList").unwrap()[0], edited, "{receipt}");
    }
    assert_eq!(edited["state"], "Running");
    assert_eq!(edited["satisfaction_receipt"], "");
    drop(store);
    let mut store = Store::open(&db).await.unwrap();
    assert_eq!(store.query("GoalList").unwrap()[0], edited);

    let satisfied = store
        .execute(
            "SatisfyGoal",
            json!({"goal_id":goal,"satisfaction_receipt":current,"receipt_revision":2}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(satisfied["outcome"], "applied", "{satisfied}");
    let paused = self::goal(&mut store, &ws).await;
    for (id, refusal) in [(&goal, "satisfied goal"), (&paused, "paused goal")] {
        let answer = store
            .execute(
                "SatisfyGoal",
                json!({"goal_id":id,"satisfaction_receipt":"acceptance-verified"}),
                Actor::Supervisor,
            )
            .await
            .unwrap_or_else(|error| panic!("{refusal}: {error:#}"));
        assert_eq!(answer["outcome"], "wrong-state", "{refusal}: {answer}");
        assert_eq!(
            answer["error"], "controlplane.host.GoalStateConflict",
            "{refusal}"
        );
    }
}

/// A running goal at revision 1 and the operator's edit of its acceptance criteria.
async fn running_goal_and_edit(store: &mut Store, path: &Path) -> (String, Value) {
    let ws = workspace(store, path).await;
    let goal = goal(store, &ws).await;
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    let mut edit = goal_body(&ws);
    edit.as_object_mut().unwrap().remove("workspace_id");
    edit["goal_id"] = json!(goal);
    edit["acceptance"] = json!("tests, independent review and documentation");
    (goal, edit)
}

/// story:acceptance-edit-ordering, the edit first. Goal acceptance checked the goal at revision
/// 1, then the operator's edit moved it to revision 2. The SatisfyGoal acceptance sends names
/// revision 1 and gets the declared `stale-revision` refusal, recorded as one decision. The goal
/// stays Running at revision 2 with the edited acceptance criteria and no receipt, also after
/// replay.
#[tokio::test]
async fn satisfy_after_edit_names_the_old_revision_and_is_refused() {
    let temp = tempfile::tempdir().unwrap();
    let db = temp.path().join("state.sqlite");
    let mut store = Store::open(&db).await.unwrap();
    let (goal, edit) = running_goal_and_edit(&mut store, temp.path()).await;
    let checked = store.query("GoalList").unwrap()[0].clone();
    assert_eq!(checked["state"], "Running", "{checked}");
    assert_eq!(checked["revision"], 1, "{checked}");
    let receipt = json!({"kind":"goal_acceptance","goal_revision":checked["revision"]}).to_string();

    let edited = store
        .execute("UpdateGoal", edit, Actor::Operator)
        .await
        .unwrap();
    assert_eq!(edited["outcome"], "applied", "{edited}");
    let current = store.query("GoalList").unwrap()[0].clone();
    assert_eq!(current["revision"], 2, "{current}");

    let version = store.version;
    let refused = store
        .execute(
            "SatisfyGoal",
            json!({"goal_id":goal,"satisfaction_receipt":receipt,"receipt_revision":checked["revision"]}),
            Actor::Supervisor,
        )
        .await
        .expect("a receipt for revision 1 is a declared refusal of the goal at revision 2");
    assert_eq!(
        refused,
        json!({"outcome":"stale-revision","error":"controlplane.host.GoalStateConflict","payload":{"state":"Running"},"published":[]})
    );
    assert_eq!(
        store.version,
        version + 1,
        "the refusal is one recorded decision"
    );
    let after = store.query("GoalList").unwrap()[0].clone();
    assert_eq!(after, current);
    assert_eq!(after["state"], "Running");
    assert_eq!(after["revision"], 2);
    assert_eq!(
        after["acceptance"],
        "tests, independent review and documentation"
    );
    assert_eq!(after["satisfaction_receipt"], "");
    drop(store);
    let reopened = Store::open(&db).await.unwrap();
    assert_eq!(reopened.query("GoalList").unwrap()[0], current);
}

/// story:typed-satisfaction-receipt. SatisfyGoal names the revision its acceptance checked in
/// `receipt_revision`, so the generated behaviour compares it with the goal's: a receipt for
/// revision 1 sent after an edit moved the goal to revision 2 gets the declared `stale-revision`
/// outcome, through the host's admission and through the durable contract alike. The refusal is
/// recorded, and the goal stays Running at revision 2 without a receipt, also after replay.
#[tokio::test]
async fn satisfy_with_a_receipt_for_an_old_revision_is_a_declared_refusal() {
    let temp = tempfile::tempdir().unwrap();
    let db = temp.path().join("state.sqlite");
    let mut store = Store::open(&db).await.unwrap();
    let (goal, edit) = running_goal_and_edit(&mut store, temp.path()).await;
    let satisfy = json!({
        "goal_id": goal,
        "satisfaction_receipt": json!({"kind":"goal_acceptance","goal_revision":1}).to_string(),
        "receipt_revision": 1,
    });
    store
        .execute("UpdateGoal", edit, Actor::Operator)
        .await
        .unwrap();
    let current = store.query("GoalList").unwrap()[0].clone();
    assert_eq!(current["revision"], 2, "{current}");
    let refusal = json!({"outcome":"stale-revision","error":"controlplane.host.GoalStateConflict","payload":{"state":"Running"},"published":[]});

    let version = store.version;
    let refused = store
        .execute("SatisfyGoal", satisfy.clone(), Actor::Supervisor)
        .await
        .expect("a stale receipt is a declared refusal, not a host error");
    assert_eq!(refused, refusal);
    assert_eq!(
        store.version,
        version + 1,
        "the declared refusal is recorded"
    );
    let after = store.query("GoalList").unwrap()[0].clone();
    assert_eq!(after, current);
    assert_eq!(after["state"], "Running");
    assert_eq!(after["satisfaction_receipt"], "");
    drop(store);

    let mut contract = contract::ContractStore::open(&db).await.unwrap();
    assert_eq!(contract.query("GoalList").unwrap()[0], current);
    let refused = contract
        .execute("SatisfyGoal", satisfy, Actor::Supervisor)
        .await
        .unwrap();
    assert_eq!(refused, refusal);
    assert_eq!(contract.query("GoalList").unwrap()[0], current);
}

/// story:acceptance-edit-ordering, the acceptance first. SatisfyGoal records the receipt for
/// revision 1, the revision acceptance checked. The operator's edit that arrives afterwards is
/// refused with the declared `satisfied` outcome. `Store::execute` returns that refusal to the
/// operator as the command's result: it names the outcome, the error and the goal's state. The
/// goal keeps its revision, its acceptance criteria and its receipt, also after replay.
#[tokio::test]
async fn edit_after_satisfy_is_refused_and_says_why() {
    let temp = tempfile::tempdir().unwrap();
    let db = temp.path().join("state.sqlite");
    let mut store = Store::open(&db).await.unwrap();
    let (goal, edit) = running_goal_and_edit(&mut store, temp.path()).await;
    let checked = store.query("GoalList").unwrap()[0].clone();
    assert_eq!(checked["revision"], 1, "{checked}");
    let receipt = json!({"kind":"goal_acceptance","goal_revision":checked["revision"]}).to_string();
    let satisfied = store
        .execute(
            "SatisfyGoal",
            json!({"goal_id":goal,"satisfaction_receipt":receipt,"receipt_revision":checked["revision"]}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(satisfied["outcome"], "applied", "{satisfied}");
    let accepted = store.query("GoalList").unwrap()[0].clone();

    let refused = store
        .execute("UpdateGoal", edit, Actor::Operator)
        .await
        .expect("a declared refusal is the command's result, not an error");
    assert_eq!(
        refused,
        json!({"outcome":"satisfied","error":"controlplane.host.GoalStateConflict","payload":{"state":"Satisfied"},"published":[]})
    );
    let after = store.query("GoalList").unwrap()[0].clone();
    assert_eq!(after, accepted);
    assert_eq!(after["state"], "Satisfied");
    assert_eq!(after["revision"], 1);
    assert_eq!(after["acceptance"], "tests and independent review");
    assert_eq!(after["satisfaction_receipt"], receipt);
    let named: Value =
        serde_json::from_str(after["satisfaction_receipt"].as_str().unwrap()).unwrap();
    assert_eq!(named["goal_revision"], checked["revision"]);
    drop(store);
    let reopened = Store::open(&db).await.unwrap();
    assert_eq!(reopened.query("GoalList").unwrap()[0], accepted);
}

#[tokio::test]
async fn cancelled_goal_refuses_edit() {
    let cancel = ("CancelGoal", json!({}), Actor::Operator);
    let goal = finished_goal_refuses_edit(cancel, "cancelled").await;
    assert_eq!(goal["state"], "Cancelled");
}

/// Edits of a goal that can still change keep the host's field checks: a Paused and then a
/// Running goal refuse a non-positive limit and a blank objective, and stay unchanged.
#[tokio::test]
async fn open_goal_edit_keeps_field_checks() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let goal = goal(&mut store, &ws).await;
    for state in ["Paused", "Running"] {
        if state == "Running" {
            store
                .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
                .await
                .unwrap();
        }
        let before = store.query("GoalList").unwrap()[0].clone();
        assert_eq!(before["state"], state);
        for (field, value, message) in [
            ("max_workers", json!(0), "max_workers must be positive"),
            ("objective", json!(" "), "goal objective is empty"),
        ] {
            let mut edit = goal_body(&ws);
            edit.as_object_mut().unwrap().remove("workspace_id");
            edit["goal_id"] = json!(goal);
            edit[field] = value;
            let error = store
                .execute("UpdateGoal", edit, Actor::Operator)
                .await
                .unwrap_err();
            assert_eq!(format!("{error:#}"), message, "{state}");
            assert_eq!(store.query("GoalList").unwrap()[0], before);
        }
    }
}

#[tokio::test]
async fn durable_contract_retains_generated_semantics_below_operational_admission() {
    let temp = scratch();
    let db = temp.path().join("contract.sqlite");
    let mut store = contract::ContractStore::open(&db).await.unwrap();
    let outcome = store
        .execute(
            "RegisterWorkspace",
            json!({"path":"literal-spec-fixture","name":"contract"}),
            Actor::Operator,
        )
        .await
        .unwrap();
    assert_eq!(outcome["outcome"], "created");
    let before = store.query("WorkspaceList").unwrap();
    drop(store);
    let reopened = contract::ContractStore::open(&db).await.unwrap();
    assert_eq!(before, reopened.query("WorkspaceList").unwrap());
}

// Adversarial operational-admission tests: use the real Store, generated commands and SQLite.
fn adversary_scratch() -> tempfile::TempDir {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/adversary-fixtures");
    std::fs::create_dir_all(&root).unwrap();
    adversary_scratch_in(&root)
}
fn adversary_scratch_in(root: &Path) -> tempfile::TempDir {
    let temp = tempfile::tempdir_in(root).unwrap();
    // Stop Git discovery at this non-worktree fixture boundary. In PR CI the
    // surrounding source checkout is detached and has no origin/HEAD ref.
    let output = std::process::Command::new("git")
        .args(["init", "--bare"])
        .arg(temp.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    temp
}

#[test]
fn adversary_fixture_is_independent_of_detached_parent_without_origin_head() {
    let outer = adversary_scratch();
    let parent = outer.path().join("detached-parent");
    repository(&parent);
    for args in [
        vec![
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "--allow-empty",
            "-m",
            "Fixture detached parent",
        ],
        vec!["switch", "--detach"],
        vec!["branch", "-D", "main"],
    ] {
        let output = std::process::Command::new("git")
            .current_dir(&parent)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let unisolated = tempfile::tempdir_in(&parent).unwrap();
    let error = discover(unisolated.path()).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("detached Git repository needs an unambiguous base branch")
    );
    let child = adversary_scratch_in(&parent);
    let discovered =
        discover(child.path()).expect("non-Git fixture inherited its detached parent repository");
    assert!(discovered.repositories.is_empty());
}

#[tokio::test]
async fn archived_workspace_cannot_restart_its_paused_goal() {
    let temp = adversary_scratch();
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let goal = goal(&mut store, &ws).await;
    let archived = store
        .execute(
            "ArchiveWorkspace",
            json!({"workspace_id":ws}),
            Actor::Operator,
        )
        .await
        .unwrap();
    assert_eq!(archived["outcome"], "applied");
    let result = store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await;
    assert_eq!(
        store.query("GoalList").unwrap()[0]["state"],
        "Paused",
        "archived workspace restarted its goal: {result:?}"
    );
}

#[tokio::test]
async fn configured_target_change_invalidates_prepared_publication() {
    let temp = adversary_scratch();
    let repo_path = temp.path().join("repo");
    repository(&repo_path);
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let repo = register_repository(&mut store, &ws, &repo_path).await;
    let goal = goal(&mut store, &ws).await;
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    let assignment = assignment(&mut store, &goal, &repo, "story:target-change").await;
    claim(&mut store, &assignment).await.unwrap();
    store
        .execute(
            "ReviewAssignment",
            json!({"assignment_id":assignment,"candidate":"candidate","test_revision":"candidate"}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    store.execute("ReadyAssignment", json!({"assignment_id":assignment,"reviewer_run":"separate-reviewer","review_revision":"candidate"}), Actor::Supervisor).await.unwrap();
    store.execute("PreparePublication", json!({"assignment_id":assignment,"candidate":"candidate","target":"main","expected_base":"base"}), Actor::Supervisor).await.unwrap();
    let changed = store.execute("ConfigureRepository", json!({"repository_id":repo,"base_branch":"release","test_command":"cargo test","publish_command":"configured-helper"}), Actor::Operator).await.unwrap();
    assert_eq!(changed["outcome"], "applied");
    let result = store
        .execute(
            "MergeAssignment",
            json!({"assignment_id":assignment}),
            Actor::Supervisor,
        )
        .await;
    assert_eq!(
        store.query("AssignmentList").unwrap()[0]["state"],
        "ReadyToMerge",
        "publication admitted against obsolete target: {result:?}"
    );
}

#[tokio::test]
async fn blocked_assignment_does_not_consume_worker_for_another_repository() {
    let temp = adversary_scratch();
    let first_path = temp.path().join("one");
    let second_path = temp.path().join("two");
    repository(&first_path);
    repository(&second_path);
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let first_repo = register_repository(&mut store, &ws, &first_path).await;
    let second_repo = register_repository(&mut store, &ws, &second_path).await;
    let mut body = goal_body(&ws);
    body["max_workers"] = json!(1);
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
    let first = assignment(&mut store, &goal, &first_repo, "story:blocked").await;
    claim(&mut store, &first).await.unwrap();
    store
        .execute(
            "BlockAssignment",
            json!({"assignment_id":first,"reason":"needs external input"}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    let second = assignment(&mut store, &goal, &second_repo, "story:independent").await;
    let result = claim(&mut store, &second).await;
    assert!(
        result.is_ok(),
        "blocked repository retained an idle worker slot: {result:?}"
    );
    let rows = store.query("AssignmentList").unwrap();
    let row = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["assignment_id"] == second)
        .unwrap();
    assert_eq!(row["state"], "Implementing");
}

#[test]
fn detached_head_remains_a_discoverable_repository() {
    let temp = adversary_scratch();
    let repo = temp.path().join("repo");
    repository(&repo);
    for args in [
        vec![
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--allow-empty",
            "-m",
            "fixture",
        ],
        vec!["checkout", "--detach"],
    ] {
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(&repo)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let nested = repo.join("src");
    std::fs::create_dir_all(&nested).unwrap();
    let found = discover(&nested).unwrap();
    assert_eq!(
        found.repositories.len(),
        1,
        "detached Git checkout was silently treated as a nongit directory"
    );
    assert_eq!(found.path, repo);
}

#[tokio::test]
async fn changed_repository_configuration_invalidates_all_admitted_evidence_after_restart() {
    for (field, value) in [
        ("base_branch", "release"),
        ("test_command", "different-tests"),
        ("publish_command", "different-publisher"),
    ] {
        let temp = scratch();
        let db = temp.path().join("state.sqlite");
        let path = temp.path().join("repo");
        repository(&path);
        let mut store = Store::open(&db).await.unwrap();
        let ws = workspace(&mut store, temp.path()).await;
        let repo = register_repository(&mut store, &ws, &path).await;
        let goal = goal(&mut store, &ws).await;
        store
            .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
            .await
            .unwrap();
        let assignment = assignment(&mut store, &goal, &repo, "story:configuration").await;
        claim(&mut store, &assignment).await.unwrap();
        let mut config = json!({"repository_id":repo,"base_branch":"main","test_command":"cargo test","publish_command":"configured-helper"});
        config[field] = json!(value);
        store
            .execute("ConfigureRepository", config, Actor::Operator)
            .await
            .unwrap();
        drop(store);
        let mut reopened = Store::open(&db).await.unwrap();
        let result = reopened.execute("ReviewAssignment", json!({"assignment_id":assignment,"candidate":"candidate","test_revision":"candidate"}), Actor::Supervisor).await;
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("configuration changed"),
            "changed field {field} was not checked"
        );
        assert_eq!(
            reopened.query("AssignmentList").unwrap()[0]["state"],
            "Implementing"
        );
    }
}

#[tokio::test]
async fn blocked_repair_reacquires_worker_capacity() {
    let temp = scratch();
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let mut goal_input = goal_body(&ws);
    goal_input["max_workers"] = json!(1);
    let goal = identity(
        &store
            .execute("CreateGoal", goal_input, Actor::Operator)
            .await
            .unwrap(),
        "goal_id",
    );
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    let mut assignments = Vec::new();
    for name in ["one", "two"] {
        let path = temp.path().join(name);
        repository(&path);
        let repo = register_repository(&mut store, &ws, &path).await;
        assignments.push(assignment(&mut store, &goal, &repo, name).await);
    }
    claim(&mut store, &assignments[0]).await.unwrap();
    store
        .execute(
            "BlockAssignment",
            json!({"assignment_id":assignments[0],"reason":"temporarily blocked"}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    claim(&mut store, &assignments[1]).await.unwrap();
    let result = store
        .execute(
            "RepairAssignment",
            json!({"assignment_id":assignments[0],"reason":"resolved","implementor_run":"new-run"}),
            Actor::Supervisor,
        )
        .await;
    assert!(result.unwrap_err().to_string().contains("worker limit"));
}

/// BlockAssignment on a Blocked assignment replaces its reason and leaves it Blocked; the reason
/// survives a restart. Only the Supervisor may send it, and a merged or cancelled assignment still
/// refuses it.
#[tokio::test]
async fn blocked_reason_is_replaced_by_the_supervisor_only() {
    let temp = adversary_scratch();
    let path = temp.path().join("repo");
    repository(&path);
    let database = temp.path().join("state.sqlite");
    let mut store = Store::open(&database).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let repo = register_repository(&mut store, &ws, &path).await;
    let goal = goal(&mut store, &ws).await;
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    let id = assignment(&mut store, &goal, &repo, "story:blocked").await;
    claim(&mut store, &id).await.unwrap();
    let block = |reason: &str| json!({"assignment_id":id,"reason":reason});
    let row = |store: &Store| {
        store.query("AssignmentList").unwrap()[0]
            .as_object()
            .unwrap()
            .clone()
    };
    let outcome = store
        .execute("BlockAssignment", block("first cause"), Actor::Supervisor)
        .await
        .unwrap();
    assert_eq!(outcome["outcome"], "applied", "{outcome}");

    let refused = store
        .execute("BlockAssignment", block("operator cause"), Actor::Operator)
        .await
        .unwrap_err();
    assert!(refused.to_string().contains("actor"), "{refused}");
    assert_eq!(row(&store)["reason"], "first cause");

    let outcome = store
        .execute("BlockAssignment", block("second cause"), Actor::Supervisor)
        .await
        .unwrap();
    assert_eq!(outcome["outcome"], "applied", "{outcome}");
    let replaced = row(&store);
    assert_eq!(
        (&replaced["state"], &replaced["reason"]),
        (&json!("Blocked"), &json!("second cause")),
        "{replaced:?}"
    );

    drop(store);
    let mut store = Store::open(&database).await.unwrap();
    assert_eq!(row(&store), replaced, "the replaced reason after a restart");
    store
        .execute(
            "CancelAssignment",
            json!({"assignment_id":id}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    let outcome = store
        .execute("BlockAssignment", block("too late"), Actor::Supervisor)
        .await
        .unwrap();
    assert_eq!(outcome["outcome"], "wrong-state", "{outcome}");
    assert_eq!(row(&store)["reason"], "second cause");
}

/// story:repair-on-moved-target. A repair that names a base revision records it as the
/// assignment's base (`rebased`); one that names none keeps the base it has (`applied`), as every
/// repair recorded before the input existed did. A publication must then expect the new base. An
/// empty base is refused, the Operator cannot repair, and the base survives a restart.
#[tokio::test]
async fn repair_takes_a_new_base_only_when_it_names_one() {
    let temp = adversary_scratch();
    let path = temp.path().join("repo");
    repository(&path);
    let database = temp.path().join("state.sqlite");
    let mut store = Store::open(&database).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let repo = register_repository(&mut store, &ws, &path).await;
    let goal = goal(&mut store, &ws).await;
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    let id = assignment(&mut store, &goal, &repo, "story:moved").await;
    claim(&mut store, &id).await.unwrap();
    let row = |store: &Store| store.query("AssignmentList").unwrap()[0].clone();
    let block = json!({"assignment_id":id,"reason":"publication closed as not published"});
    let repair = |base: Option<&str>| {
        let mut body =
            json!({"assignment_id":id,"reason":"retry","implementor_run":"implementor-retry"});
        if let Some(base) = base {
            body["base_revision"] = json!(base);
        }
        body
    };
    store
        .execute("BlockAssignment", block.clone(), Actor::Supervisor)
        .await
        .unwrap();

    let operator = store
        .execute("RepairAssignment", repair(Some("moved")), Actor::Operator)
        .await
        .unwrap_err();
    assert!(operator.to_string().contains("actor"), "{operator}");
    let empty = store
        .execute("RepairAssignment", repair(Some("")), Actor::Supervisor)
        .await;
    assert!(
        empty
            .as_ref()
            .is_ok_and(|answer| answer["outcome"] == "base-missing"),
        "a repair with an empty base: {empty:?}"
    );
    let held = row(&store);
    assert_eq!(
        (&held["state"], &held["base_revision"], &held["attempt"]),
        (&json!("Blocked"), &json!("base"), &json!(1)),
        "{held}"
    );

    let rebased = store
        .execute("RepairAssignment", repair(Some("moved")), Actor::Supervisor)
        .await
        .unwrap();
    assert_eq!(rebased["outcome"], "rebased", "{rebased}");
    let moved = row(&store);
    assert_eq!(
        (&moved["state"], &moved["base_revision"], &moved["attempt"]),
        (&json!("Implementing"), &json!("moved"), &json!(2)),
        "{moved}"
    );

    store
        .execute("BlockAssignment", block, Actor::Supervisor)
        .await
        .unwrap();
    let kept = store
        .execute("RepairAssignment", repair(None), Actor::Supervisor)
        .await
        .unwrap();
    assert_eq!(kept["outcome"], "applied", "{kept}");
    let repaired = row(&store);
    assert_eq!(
        (&repaired["base_revision"], &repaired["attempt"]),
        (&json!("moved"), &json!(3)),
        "{repaired}"
    );

    let candidate = "candidate-moved";
    for (command, body) in [
        (
            "ReviewAssignment",
            json!({"assignment_id":id,"candidate":candidate,"test_revision":candidate}),
        ),
        (
            "ReadyAssignment",
            json!({"assignment_id":id,"reviewer_run":"reviewer","review_revision":candidate}),
        ),
    ] {
        store
            .execute(command, body, Actor::Supervisor)
            .await
            .unwrap();
    }
    let prepare = |base: &str| json!({"assignment_id":id,"candidate":candidate,"target":"main","expected_base":base});
    let stale = store
        .execute("PreparePublication", prepare("base"), Actor::Supervisor)
        .await;
    assert!(stale.is_err(), "a publication on the old base: {stale:?}");
    let prepared = store
        .execute("PreparePublication", prepare("moved"), Actor::Supervisor)
        .await
        .unwrap();
    assert_eq!(prepared["outcome"], "created", "{prepared}");

    let reviewed = row(&store);
    drop(store);
    let store = Store::open(&database).await.unwrap();
    assert_eq!(row(&store), reviewed, "the assignment after a restart");
}

#[tokio::test]
async fn multiple_workspaces_keep_directory_membership_isolated() {
    let temp = scratch();
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    let shared = temp.path().join("shared");
    for path in [&first, &second, &shared] {
        std::fs::create_dir_all(path).unwrap();
    }
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let one = identity(
        &store.register_workspace(&first, "one").await.unwrap(),
        "workspace_id",
    );
    let two = identity(
        &store.register_workspace(&second, "two").await.unwrap(),
        "workspace_id",
    );
    store.add_workspace_directory(&one, &shared).await.unwrap();
    store.add_workspace_directory(&two, &shared).await.unwrap();
    let dirs = store.query("WorkspaceDirectoryList").unwrap();
    assert_eq!(dirs.as_array().unwrap().len(), 4);
    for ws in [&one, &two] {
        assert_eq!(
            dirs.as_array()
                .unwrap()
                .iter()
                .filter(|d| d["workspace_id"] == *ws)
                .count(),
            2
        );
    }
}

#[tokio::test]
async fn non_git_directory_survives_restart_and_canonical_duplicates_are_idempotent() {
    let temp = scratch();
    let db = temp.path().join("state.sqlite");
    let context = temp.path().join("context");
    std::fs::create_dir_all(&context).unwrap();
    let mut store = Store::open(&db).await.unwrap();
    let ws = identity(
        &store
            .register_workspace(temp.path(), "workspace")
            .await
            .unwrap(),
        "workspace_id",
    );
    let first = store.add_workspace_directory(&ws, &context).await.unwrap();
    assert_eq!(
        first,
        store
            .add_workspace_directory(&ws, &context.join("."))
            .await
            .unwrap()
    );
    let before = store.query("WorkspaceDirectoryList").unwrap();
    drop(store);
    let mut reopened = Store::open(&db).await.unwrap();
    assert_eq!(before, reopened.query("WorkspaceDirectoryList").unwrap());
    assert_eq!(
        first,
        reopened
            .add_workspace_directory(&ws, &context)
            .await
            .unwrap()
    );
    assert!(
        reopened
            .query("RepositoryRegistrationList")
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn overlapping_directories_preserve_shared_repositories() {
    let temp = scratch();
    let root = temp.path().join("workspace");
    std::fs::create_dir_all(&root).unwrap();
    let container = temp.path().join("repositories");
    let repo = container.join("repo");
    repository(&repo);
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let ws = identity(
        &store.register_workspace(&root, "workspace").await.unwrap(),
        "workspace_id",
    );
    let parent = identity(
        &store
            .add_workspace_directory(&ws, &container)
            .await
            .unwrap(),
        "directory_id",
    );
    let child = identity(
        &store.add_workspace_directory(&ws, &repo).await.unwrap(),
        "directory_id",
    );
    store.remove_workspace_directory(&parent).await.unwrap();
    assert_eq!(
        store.query("RepositoryRegistrationList").unwrap()[0]["state"],
        "Registered"
    );
    store.remove_workspace_directory(&child).await.unwrap();
    assert_eq!(
        store.query("RepositoryRegistrationList").unwrap()[0]["state"],
        "Disabled"
    );
}

#[tokio::test]
async fn removing_directory_preserves_manual_repository_registration() {
    let temp = scratch();
    let repo = temp.path().join("repo");
    repository(&repo);
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    register_repository(&mut store, &ws, &repo).await;
    let dir = identity(
        &store.add_workspace_directory(&ws, &repo).await.unwrap(),
        "directory_id",
    );
    store.remove_workspace_directory(&dir).await.unwrap();
    assert_eq!(
        store.query("RepositoryRegistrationList").unwrap()[0]["state"],
        "Registered"
    );
}

#[tokio::test]
async fn active_assignment_prevents_directory_removal() {
    let temp = scratch();
    let repo = temp.path().join("repo");
    repository(&repo);
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let dir = identity(
        &store.add_workspace_directory(&ws, &repo).await.unwrap(),
        "directory_id",
    );
    let repo_id = store.query("RepositoryRegistrationList").unwrap()[0]["repository_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let goal = goal(&mut store, &ws).await;
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    let assignment = assignment(&mut store, &goal, &repo_id, "story:retained").await;
    claim(&mut store, &assignment).await.unwrap();
    assert!(store.remove_workspace_directory(&dir).await.is_err());
    assert_eq!(
        store.query("WorkspaceDirectoryList").unwrap()[0]["state"],
        "Registered"
    );
}

#[tokio::test]
async fn directory_and_discovered_repositories_commit_atomically() {
    let temp = scratch();
    let repo = temp.path().join("repo");
    repository(&repo);
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    store.version += 1;
    assert!(store.add_workspace_directory(&ws, &repo).await.is_err());
    assert!(
        store
            .query("WorkspaceDirectoryList")
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .query("RepositoryRegistrationList")
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
    store.version -= 1;
    let directory = identity(
        &store.add_workspace_directory(&ws, &repo).await.unwrap(),
        "directory_id",
    );
    assert_eq!(
        store
            .query("WorkspaceDirectoryList")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        store
            .query("RepositoryRegistrationList")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    store.version += 1;
    assert!(store.remove_workspace_directory(&directory).await.is_err());
    assert_eq!(
        store.query("WorkspaceDirectoryList").unwrap()[0]["state"],
        "Registered"
    );
    assert_eq!(
        store.query("RepositoryRegistrationList").unwrap()[0]["state"],
        "Registered"
    );
    drop(store);
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    assert_eq!(
        store.query("WorkspaceDirectoryList").unwrap()[0]["state"],
        "Registered"
    );
    assert_eq!(
        store.query("RepositoryRegistrationList").unwrap()[0]["state"],
        "Registered"
    );
    store.remove_workspace_directory(&directory).await.unwrap();
    drop(store);
    let store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    assert_eq!(
        store.query("WorkspaceDirectoryList").unwrap()[0]["state"],
        "Removed"
    );
    assert_eq!(
        store.query("RepositoryRegistrationList").unwrap()[0]["state"],
        "Disabled"
    );
}

#[tokio::test]
async fn legacy_workspaces_gain_primary_directory_once_without_losing_goals() {
    let temp = scratch();
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let goal = goal(&mut store, &ws).await;
    store.backfill_workspace_directories().await.unwrap();
    store.backfill_workspace_directories().await.unwrap();
    assert_eq!(
        store
            .query("WorkspaceDirectoryList")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(store.query("GoalList").unwrap()[0]["goal_id"], goal);
}

#[tokio::test]
async fn missing_legacy_workspace_does_not_block_healthy_directory_backfill() {
    let temp = scratch();
    let absent = temp.path().join("disconnected");
    std::fs::create_dir(&absent).unwrap();
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let missing_id = workspace(&mut store, &absent).await;
    let retained_goal = goal(&mut store, &missing_id).await;
    let healthy_id = workspace(&mut store, temp.path()).await;
    std::fs::remove_dir(&absent).unwrap();
    store.backfill_workspace_directories().await.unwrap();
    let directories = store.query("WorkspaceDirectoryList").unwrap();
    assert!(
        directories
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["workspace_id"] == healthy_id)
    );
    assert!(
        !directories
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["workspace_id"] == missing_id)
    );
    assert_eq!(
        store
            .query("WorkspaceList")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        store.query("GoalList").unwrap()[0]["goal_id"],
        retained_goal
    );
    // Reconnecting the directory permits a later startup to finish its migration.
    std::fs::create_dir(&absent).unwrap();
    store.backfill_workspace_directories().await.unwrap();
    assert_eq!(
        store
            .query("WorkspaceDirectoryList")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[tokio::test]
async fn startup_backfill_does_not_restore_explicitly_removed_membership() {
    let temp = scratch();
    let db = temp.path().join("state.sqlite");
    let mut store = Store::open(&db).await.unwrap();
    let registered = store
        .register_workspace(temp.path(), "workspace")
        .await
        .unwrap();
    let ws = identity(&registered, "workspace_id");
    let dir = store.query("WorkspaceDirectoryList").unwrap()[0]["directory_id"]
        .as_str()
        .unwrap()
        .to_owned();
    store.remove_workspace_directory(&dir).await.unwrap();
    drop(store);
    let mut store = Store::open(&db).await.unwrap();
    store.backfill_workspace_directories().await.unwrap();
    store
        .register_workspace(temp.path(), "workspace")
        .await
        .unwrap();
    let directories = store.query("WorkspaceDirectoryList").unwrap();
    assert_eq!(directories.as_array().unwrap().len(), 1);
    assert_eq!(directories[0]["workspace_id"], ws);
    assert_eq!(directories[0]["state"], "Removed");
}

#[tokio::test]
async fn legacy_backfill_still_propagates_storage_failure() {
    let temp = scratch();
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    workspace(&mut store, temp.path()).await;
    store.version += 1;
    assert!(store.backfill_workspace_directories().await.is_err());
    assert!(
        store
            .query("WorkspaceDirectoryList")
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
}

/// Write a recorded host history verbatim, below `Store`, as an older build appended it.
async fn recorded_history(database: &Path, events: &str) {
    let log = SqliteEventStore::open(database.to_str().unwrap(), "control_plane")
        .await
        .unwrap();
    let stream = StreamId::new(TenantId::new("local").unwrap(), "host", "state").unwrap();
    for (index, line) in events.lines().enumerate() {
        let data: Value = serde_json::from_str(line).unwrap();
        let id = format!("recorded-{index}");
        let meta = CommandMeta {
            idempotency_key: id.clone(),
            request_hash: eventlog_core::request_hash(&data).unwrap(),
            subject: "local-operator".into(),
            actor: data["actor"].as_str().unwrap().into(),
            request_id: id.clone(),
            trace_id: id,
            causation_id: None,
            causation_depth: 0,
            occurred_at: time::OffsetDateTime::now_utc(),
            claim: None,
        };
        let expected = if index == 0 {
            Expected::NoStream
        } else {
            Expected::Exact(index as u64)
        };
        log.append(
            &stream,
            expected,
            &[NewEvent::new("HostDecision", 1, data).unwrap()],
            &meta,
        )
        .await
        .unwrap();
    }
}

#[tokio::test]
async fn existing_receipts_still_open() {
    // Written by a09cff6's own progress paths (planner record, fleet progress, acceptance):
    // every receipt carries its whole activity history, fleet map and planner evidence.
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/progress-receipts-a09cff6");
    let temp = scratch();
    let database = temp.path().join("state.sqlite");
    recorded_history(
        &database,
        &std::fs::read_to_string(fixture.join("events.jsonl")).unwrap(),
    )
    .await;
    let expected: Value =
        serde_json::from_str(&std::fs::read_to_string(fixture.join("views.json")).unwrap())
            .unwrap();
    let store = Store::open(&database).await.unwrap();
    for view in [
        "WorkspaceList",
        "GoalList",
        "AssignmentList",
        "PublicationIntentList",
    ] {
        assert_eq!(store.query(view).unwrap(), expected[view], "{view}");
    }
}

#[tokio::test]
async fn first_bounded_progress_continues_an_existing_receipt_history() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/progress-receipts-a09cff6");
    let temp = scratch();
    let database = temp.path().join("state.sqlite");
    recorded_history(
        &database,
        &std::fs::read_to_string(fixture.join("events.jsonl")).unwrap(),
    )
    .await;
    let mut store = Store::open(&database).await.unwrap();
    let goal = store.query("GoalList").unwrap()[0].clone();
    let goal_id = goal["goal_id"].as_str().unwrap().to_owned();
    let legacy: Value = serde_json::from_str(goal["planning_receipt"].as_str().unwrap()).unwrap();
    let event = json!({"id":"next","assignment_id":"new-assignment","goal_revision":1,"action":"tool.run","role":"implementor","at":"2026-10-06T12:00:00Z","worktree":"tree","status":"running","detail":"next"});
    let before = store.appended_event_bytes();
    store
        .record_activity(&goal_id, event.clone())
        .await
        .unwrap();
    assert!(store.appended_event_bytes() - before < 4096);
    let rows = store.query("GoalList").unwrap();
    let row = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|goal| goal["goal_id"] == goal_id.as_str())
        .unwrap();
    let receipt: Value = serde_json::from_str(row["planning_receipt"].as_str().unwrap()).unwrap();
    assert_eq!(receipt["last_activity"], event);
    assert!(receipt.get("activity").is_none() && receipt.get("planner").is_none());
    let history = store.activity_history(&goal_id).unwrap();
    let mut activity = legacy["activity"].as_array().unwrap().clone();
    activity.push(event.clone());
    assert_eq!(history["activity"], json!(activity));
    assert_eq!(history["planner"], legacy["planner"]);
    assert_eq!(history["acceptance"], legacy["acceptance"]);
    let mut fleet = legacy["fleet"].as_object().unwrap().clone();
    fleet.insert("new-assignment".into(), event);
    assert_eq!(history["fleet"], Value::Object(fleet));
    drop(store);
    let reopened = Store::open(&database).await.unwrap();
    assert_eq!(reopened.query("GoalList").unwrap(), rows);
    assert_eq!(reopened.activity_history(&goal_id).unwrap(), history);
}

#[tokio::test]
async fn unchanged_planner_evidence_is_recorded_once() {
    let temp = scratch();
    let database = temp.path().join("state.sqlite");
    let mut store = Store::open(&database).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let goal = goal(&mut store, &ws).await;
    let evidence = json!({"namespace":"plan","transcript":["observed ".repeat(8192)]});
    let mut appended = Vec::new();
    for index in 0..3 {
        let event = json!({"id":index.to_string(),"action":"planning.Planning","role":"planner","status":"running","detail":format!("step {index}")});
        let receipt = json!({"planner":evidence,"last_activity":event,"activity":[event]});
        let before = store.appended_event_bytes();
        store.execute("RecordPlanningProgress",json!({"goal_id":goal,"planning_revision":1,"planning_fingerprint":"f","planning_repository":"","planning_worktree_id":"t","planning_worktree_path":"t","planning_phase":"Planning","planning_reason":"","planning_receipt":receipt.to_string()}),Actor::Supervisor).await.unwrap();
        appended.push(store.appended_event_bytes() - before);
    }
    assert!(appended[0] > 2 * 65536, "{appended:?}");
    assert!(appended[1] < 4096 && appended[2] < 4096, "{appended:?}");
    let history = store.activity_history(&goal).unwrap();
    assert_eq!(history["planner"], evidence);
    assert_eq!(history["activity"].as_array().unwrap().len(), 3);
    drop(store);
    let reopened = Store::open(&database).await.unwrap();
    assert_eq!(reopened.activity_history(&goal).unwrap(), history);
}

#[test]
fn recorded_activity_bounds_are_stable() {
    let large =
        json!({"summary":"界".repeat(900),"event":{"kind":"model-stream","text":"x".repeat(4096)}});
    let unnamed = json!({"program":"cargo","path":"src/lib.rs","output":"y".repeat(4096)});
    let wide = Value::Object(
        (0..40)
            .map(|index| (format!("member{index:02}"), json!("v".repeat(100))))
            .chain([("program".to_owned(), json!("cargo"))])
            .collect(),
    );
    for detail in [
        json!("z".repeat(2048)),
        large,
        unnamed,
        wide,
        json!({"small":true}),
    ] {
        let activity = json!({"id":"a","action":"tool.run","role":"implementor","status":"running","at":"now","worktree":"w".repeat(1024),"unlisted":"dropped","detail":detail});
        let once = memory::bounded_activity(&activity);
        assert_eq!(memory::bounded_activity(&once), once);
        assert!(once.to_string().len() < 4096, "{once}");
        assert!(once.get("unlisted").is_none());
    }
    // Every member of a larger detail is kept, bounded, with the original size.
    let unnamed = memory::bounded_activity(
        &json!({"detail":{"program":"cargo","path":"src/lib.rs","output":"y".repeat(4096)}}),
    );
    assert_eq!(unnamed["detail"]["program"], "cargo");
    assert_eq!(unnamed["detail"]["path"], "src/lib.rs");
    assert_eq!(unnamed["detail"]["output"].as_str().unwrap().len(), 240);
    assert!(unnamed["detail"]["_bounded"]["bytes"].as_u64().unwrap() > 4096);
    // Still over 1 KiB, it keeps what readers show and names what it left out.
    let wide = memory::bounded_activity(&json!({"detail":Value::Object(
        (0..40)
            .map(|index| (format!("member{index:02}"), json!("v".repeat(100))))
            .chain([("program".to_owned(), json!("cargo"))])
            .collect(),
    )}));
    assert_eq!(wide["detail"]["program"], "cargo");
    assert!(wide["detail"].get("member00").is_none());
    assert_eq!(wide["detail"]["_bounded"]["dropped_count"], 40);
    assert_eq!(wide["detail"]["_bounded"]["dropped"][0], "member00");
}

/// A running goal for the adversary cases below.
async fn adversary_goal(store: &mut Store, path: &Path) -> String {
    let ws = workspace(store, path).await;
    let goal = goal(store, &ws).await;
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    goal
}

#[tokio::test]
async fn adversary_progress_after_a_long_planning_reason_stays_under_16_kib() {
    // story:bounded-progress-records outcome: recording runtime progress costs at most 16 KiB
    // of event data per progress decision. The supervisor records a blocked planning attempt
    // with `planning_reason = format!("{error:#}")`; a failed command's ProcessExit displays its
    // whole stdout and stderr. Fleet progress on the same goal (record_activity) copies every
    // planning_* field into each decision, and the core caps none of them.
    let temp = scratch();
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let goal = adversary_goal(&mut store, temp.path()).await;
    let output = "test fleet_fixture::requested_answer ... FAILED\n".repeat(400);
    let reason =
        format!("git [\"commit\", \"-m\", \"plan\"] exited Some(1):\nstdout:\n{output}\nstderr:\n");
    let blocked = store.execute("RecordPlanningProgress",json!({"goal_id":goal,"planning_revision":1,"planning_fingerprint":"f","planning_repository":"","planning_worktree_id":"t","planning_worktree_path":"t","planning_phase":"Blocked","planning_reason":reason,"planning_receipt":"{}"}),Actor::Supervisor).await.unwrap();
    assert_eq!(blocked["outcome"], "applied");
    let mut added = Vec::new();
    for index in 0..3 {
        let before = store.appended_event_bytes();
        store
            .record_activity(
                &goal,
                json!({"id":format!("a{index}"),"assignment_id":"queued-before-planning-blocked","goal_revision":1,"action":"tool.run","role":"implementor","status":"running","detail":format!("step {index}")}),
            )
            .await
            .unwrap();
        added.push(store.appended_event_bytes() - before);
    }
    assert!(
        added.iter().all(|bytes| *bytes <= 16 * 1024),
        "progress decisions after a blocked planning attempt: {added:?} bytes"
    );
}

#[tokio::test]
async fn adversary_goal_list_shows_the_receipt_the_command_recorded() {
    // ess/domains/host.yaml: RecordPlanningProgress `sets planning_receipt:
    // input.planning_receipt` and publishes it in PlanningProgressRecorded; GoalList's
    // planning_receipt is the Goal's field. The view must show what was recorded.
    let temp = scratch();
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let goal = adversary_goal(&mut store, temp.path()).await;
    let mut recorded = Value::Null;
    for index in 0..2 {
        let outcome = store
            .record_activity(
                &goal,
                json!({"id":format!("a{index}"),"action":"tool.run","role":"implementor","status":"running","detail":format!("step {index}")}),
            )
            .await
            .unwrap();
        recorded = outcome["published"][0]["payload"]["planning_receipt"].clone();
    }
    assert!(recorded.is_string(), "no recorded receipt in the outcome");
    let row = store.query("GoalList").unwrap()[0].clone();
    assert_eq!(row["planning_receipt"], recorded);
}

#[test]
fn adversary_structured_detail_bounds_every_member_it_keeps() {
    // The projection of a structured detail over 1 KiB copies `omitted_bytes` verbatim,
    // whatever its type and size.
    let detail = json!({"summary":"governor state","omitted_bytes":"artifact ".repeat(8 * 1024),"artifacts":"x".repeat(2048)});
    let activity = json!({"id":"a","action":"governor-state","role":"runtime","status":"running","at":"now","detail":detail});
    let once = memory::bounded_activity(&activity);
    assert!(
        once.to_string().len() < 4096,
        "bounded activity is {} bytes",
        once.to_string().len()
    );
}

#[tokio::test]
async fn adversary_journal_keeps_the_newest_64_once_each_across_restart() {
    // Pins the history the evidence view showed before bounding (newest 64 activities, newest
    // per assignment) through re-records that repeat the current last activity, as acceptance
    // and unchanged planner phases do. No existing case records more than 24 activities and
    // checks the history length, or re-records an unchanged last activity.
    let temp = scratch();
    let database = temp.path().join("state.sqlite");
    let mut store = Store::open(&database).await.unwrap();
    let goal = adversary_goal(&mut store, temp.path()).await;
    let mut sent = Vec::new();
    for index in 0..100 {
        let event = json!({"id":format!("event-{index:03}"),"assignment_id":format!("assignment-{}",index%3),"goal_revision":1,"action":"tool.run","role":"implementor","status":"running","detail":format!("step {index}")});
        store.record_activity(&goal, event.clone()).await.unwrap();
        sent.push(event);
        if index % 10 == 9 {
            let current = store.query("GoalList").unwrap()[0].clone();
            let mut receipt: Value =
                serde_json::from_str(current["planning_receipt"].as_str().unwrap()).unwrap();
            receipt["acceptance"] = json!({"status":"running","reason":format!("check {index}")});
            let mut body = current.as_object().unwrap().clone();
            body.retain(|key, _| key.starts_with("planning_") || key == "goal_id");
            body.insert("planning_receipt".into(), json!(receipt.to_string()));
            let outcome = store
                .execute(
                    "RecordPlanningProgress",
                    Value::Object(body),
                    Actor::Supervisor,
                )
                .await
                .unwrap();
            assert_eq!(outcome["outcome"], "applied");
        }
    }
    // Correction F2: GoalList shows the recorded receipt; the history it no longer repeats
    // (activity, fleet, acceptance) is read from Store::activity_history.
    let shown = |store: &Store| -> Value {
        let row = store.query("GoalList").unwrap()[0].clone();
        let receipt: Value =
            serde_json::from_str(row["planning_receipt"].as_str().unwrap()).unwrap();
        let mut shown = store
            .activity_history(row["goal_id"].as_str().unwrap())
            .unwrap();
        shown["last_activity"] = receipt["last_activity"].clone();
        shown
    };
    let live = shown(&store);
    assert_eq!(live["activity"], json!(sent[36..].to_vec()));
    assert_eq!(live["last_activity"], sent[99]);
    for assignment in 0..3 {
        let newest = sent
            .iter()
            .rev()
            .find(|event| event["assignment_id"] == format!("assignment-{assignment}"))
            .unwrap();
        assert_eq!(live["fleet"][format!("assignment-{assignment}")], *newest);
    }
    assert_eq!(live["acceptance"]["reason"], "check 99");
    drop(store);
    assert_eq!(shown(&Store::open(&database).await.unwrap()), live);
}

#[tokio::test]
async fn adversary_rejected_acceptance_is_recorded_once_and_survives_restart() {
    // The fleet records a rejected goal review on the receipt it reads from GoalList
    // (fleet.rs:114-123, reason capped with bounded_text at 8 KiB); later progress copies that
    // receipt. The acceptance record must not be repeated by those decisions, and the latch
    // input (Store::activity_history's acceptance) must survive a restart. No earlier case
    // measures a decision recorded after an acceptance record.
    let temp = scratch();
    let database = temp.path().join("state.sqlite");
    let mut store = Store::open(&database).await.unwrap();
    let goal = adversary_goal(&mut store, temp.path()).await;
    store
        .record_activity(
            &goal,
            json!({"id":"before","action":"goal.review","role":"goal_reviewer","status":"running","detail":"reviewing"}),
        )
        .await
        .unwrap();
    let reason = format!(
        "Goal acceptance blocked: final goal review rejected: {}",
        json!({"approved":false,"reason":"Obligation is not demonstrated.\n".repeat(400)})
    );
    let acceptance = json!({"fingerprint":"fp","status":"failed","reason":bounded_text(&reason, 8 * 1024),"at":"2026-10-06T10:00:00Z","goal_revision":1});
    let current = store.query("GoalList").unwrap()[0].clone();
    let mut receipt: Value =
        serde_json::from_str(current["planning_receipt"].as_str().unwrap()).unwrap();
    receipt["acceptance"] = acceptance.clone();
    let mut body = current.as_object().unwrap().clone();
    body.retain(|key, _| key.starts_with("planning_") || key == "goal_id");
    body.insert("planning_receipt".into(), json!(receipt.to_string()));
    let before = store.appended_event_bytes();
    store
        .execute(
            "RecordPlanningProgress",
            Value::Object(body),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    let recorded = store.appended_event_bytes() - before;
    assert!(recorded > 2 * 8 * 1024, "{recorded}");
    let mut later = Vec::new();
    for index in 0..3 {
        let before = store.appended_event_bytes();
        store
            .record_activity(
                &goal,
                json!({"id":format!("after-{index}"),"assignment_id":"a","goal_revision":1,"action":"blocked","role":"goal_reviewer","status":"failed","detail":{"reason":reason}}),
            )
            .await
            .unwrap();
        later.push(store.appended_event_bytes() - before);
    }
    assert!(
        later.iter().all(|bytes| *bytes < 4096),
        "decisions after the acceptance record: {later:?}"
    );
    let row = store.query("GoalList").unwrap()[0].clone();
    let stored: Value = serde_json::from_str(row["planning_receipt"].as_str().unwrap()).unwrap();
    assert!(stored.get("acceptance").is_none(), "{stored}");
    assert_eq!(
        store.activity_history(&goal).unwrap()["acceptance"],
        acceptance
    );
    drop(store);
    let reopened = Store::open(&database).await.unwrap();
    assert_eq!(
        reopened.activity_history(&goal).unwrap()["acceptance"],
        acceptance
    );
}

/// The `(command, outcome)` of every refusal this branch adds to the specification, read from
/// the `outcome-added` acknowledgements in `ess/spec-acknowledgements.json`. The committed
/// history fixture predates them, so this test, not the fixture, records each one.
fn acknowledged_added_outcomes() -> std::collections::BTreeSet<(String, String)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ess/spec-acknowledgements.json");
    let acknowledged: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    acknowledged["acknowledged"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["change"]["changed"]["kind"] == "outcome-added")
        .map(|entry| {
            let command = entry["change"]["subject"].as_str().unwrap();
            (
                command.trim_start_matches("controlplane.host.").to_owned(),
                entry["change"]["changed"]["outcome"]
                    .as_str()
                    .unwrap()
                    .to_owned(),
            )
        })
        .collect()
}

/// Every added admission refusal answers through `Store::execute`, is recorded, and replays:
/// the store reopens over those decisions to the same views.
#[tokio::test]
async fn declared_admission_refusals_are_recorded_and_replay() {
    let temp = scratch();
    let repo_path = temp.path().join("repo");
    repository(&repo_path);
    let db = temp.path().join("state.sqlite");
    let mut store = Store::open(&db).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let repo = register_repository(&mut store, &ws, &repo_path).await;
    let goal = goal(&mut store, &ws).await;
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    let id = assignment(&mut store, &goal, &repo, "story:refusals").await;
    claim(&mut store, &id).await.unwrap();
    store
        .execute(
            "ReviewAssignment",
            json!({"assignment_id":id,"candidate":"candidate","test_revision":"candidate"}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    let unknown = "00000000-0000-4000-8000-000000000000";
    let limits = |workers: i64, attempts: i64, minutes: i64| {
        let mut body = goal_body(&ws);
        body["max_workers"] = json!(workers);
        body["max_attempts"] = json!(attempts);
        body["max_minutes"] = json!(minutes);
        body
    };
    let queue = |goal_id: &str, revision: i64| {
        json!({"goal_id":goal_id,"repository_id":repo,"story_id":"story:refused","case_id":"case",
            "worktree_id":"","candidate":"","attempt":0,"reason":"","implementor_run":"",
            "reviewer_run":"","goal_revision":revision})
    };
    let calls = [
        (
            "CreateGoal",
            limits(0, 1, 1),
            Actor::Operator,
            "workers-invalid",
        ),
        (
            "CreateGoal",
            limits(1, 0, 1),
            Actor::Operator,
            "attempts-invalid",
        ),
        (
            "CreateGoal",
            limits(1, 1, 0),
            Actor::Operator,
            "minutes-invalid",
        ),
        (
            "QueueAssignment",
            queue(unknown, 1),
            Actor::Supervisor,
            "goal-not-found",
        ),
        (
            "QueueAssignment",
            queue(&goal, 2),
            Actor::Supervisor,
            "goal-not-current",
        ),
        (
            "SatisfyGoal",
            json!({"goal_id":goal,"receipt_revision":2,
                "satisfaction_receipt":json!({"kind":"goal_acceptance","goal_revision":2}).to_string()}),
            Actor::Supervisor,
            "stale-revision",
        ),
        (
            "ClaimAssignment",
            json!({"assignment_id":id,"worktree_id":"tree","implementor_run":"","base_revision":"base"}),
            Actor::Supervisor,
            "evidence-missing",
        ),
        (
            "RepairAssignment",
            json!({"assignment_id":id,"reason":"retry","implementor_run":""}),
            Actor::Supervisor,
            "evidence-missing",
        ),
        (
            "RepairAssignment",
            json!({"assignment_id":id,"reason":"retry","implementor_run":"retry","base_revision":""}),
            Actor::Supervisor,
            "base-missing",
        ),
        (
            "ReviewAssignment",
            json!({"assignment_id":id,"candidate":"candidate","test_revision":"other"}),
            Actor::Supervisor,
            "tests-not-current",
        ),
        (
            "ReadyAssignment",
            json!({"assignment_id":id,"reviewer_run":"","review_revision":"candidate"}),
            Actor::Supervisor,
            "reviewer-missing",
        ),
        (
            "ReadyAssignment",
            json!({"assignment_id":id,"reviewer_run":format!("impl-{id}"),"review_revision":"candidate"}),
            Actor::Supervisor,
            "review-not-independent",
        ),
        (
            "ReadyAssignment",
            json!({"assignment_id":id,"reviewer_run":"reviewer","review_revision":"stale"}),
            Actor::Supervisor,
            "evidence-not-current",
        ),
        (
            "CompleteAssignment",
            json!({"assignment_id":id,"merge_receipt":""}),
            Actor::Supervisor,
            "receipt-missing",
        ),
        (
            "ReconcileAssignment",
            json!({"assignment_id":id,"merge_receipt":""}),
            Actor::Supervisor,
            "receipt-missing",
        ),
    ];
    let mut answered = std::collections::BTreeSet::new();
    for (command, body, actor, outcome) in calls {
        let at = *store.subscribe().borrow();
        let answer = store.execute(command, body, actor).await.unwrap();
        assert_eq!(answer["outcome"], outcome, "{command}: {answer}");
        assert!(answer.get("error").is_some(), "{command}: {answer}");
        assert_eq!(
            *store.subscribe().borrow(),
            at + 1,
            "{command}: one decision"
        );
        answered.insert((command.to_owned(), outcome.to_owned()));
    }
    assert_eq!(answered, acknowledged_added_outcomes());
    let views = ["AssignmentList", "GoalList", "RepositoryRegistrationList"]
        .map(|view| store.query(view).unwrap());
    drop(store);
    let reopened = Store::open(&db).await.unwrap();
    for (view, before) in ["AssignmentList", "GoalList", "RepositoryRegistrationList"]
        .iter()
        .zip(views)
    {
        assert_eq!(reopened.query(view).unwrap(), before, "{view} after replay");
    }
}

/// A queued assignment, with its store, for the evidence cases below.
async fn queued_assignment(temp: &tempfile::TempDir) -> (Store, String) {
    let repo_path = temp.path().join("repo");
    repository(&repo_path);
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let ws = workspace(&mut store, temp.path()).await;
    let repo = register_repository(&mut store, &ws, &repo_path).await;
    let goal = goal(&mut store, &ws).await;
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    let id = assignment(&mut store, &goal, &repo, "story:evidence-part").await;
    (store, id)
}

/// ClaimAssignment's `evidence-missing` part `input.worktree_id == ""` alone: a claim of a
/// queued assignment that names an implementor run and a base but no worktree is refused.
#[tokio::test]
async fn claim_without_worktree_is_evidence_missing() {
    let temp = scratch();
    let (mut store, id) = queued_assignment(&temp).await;
    let answer = store
        .execute(
            "ClaimAssignment",
            json!({"assignment_id":id,"worktree_id":"","implementor_run":"impl","base_revision":"base"}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(answer["outcome"], "evidence-missing", "{answer}");
}

/// ReviewAssignment's `tests-not-current` part `input.candidate == ""` alone: an empty candidate
/// with an equally empty test revision, so `input.candidate != input.test_revision` does not hold.
#[tokio::test]
async fn review_of_empty_candidate_is_tests_not_current() {
    let temp = scratch();
    let (mut store, id) = queued_assignment(&temp).await;
    claim(&mut store, &id).await.unwrap();
    let answer = store
        .execute(
            "ReviewAssignment",
            json!({"assignment_id":id,"candidate":"","test_revision":""}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(answer["outcome"], "tests-not-current", "{answer}");
}
