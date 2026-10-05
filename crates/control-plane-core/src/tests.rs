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

// Adversarial operational-admission tests: use the real Store, generated commands and SQLite.
fn adversary_scratch() -> tempfile::TempDir {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/adversary-fixtures");
    std::fs::create_dir_all(&root).unwrap();
    tempfile::tempdir_in(root).unwrap()
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
