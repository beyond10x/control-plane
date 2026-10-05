use super::*;
use serde_json::json;

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
    assert!(store.execute("ReadyAssignment",json!({"assignment_id":assignment,"reviewer_run":format!("impl-{assignment}"),"review_revision":"candidate"}),Actor::Supervisor).await.is_err());
    assert!(store.execute("ReadyAssignment",json!({"assignment_id":assignment,"reviewer_run":"reviewer","review_revision":"stale"}),Actor::Supervisor).await.is_err());
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
