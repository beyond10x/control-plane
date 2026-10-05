use control_plane_core::{Actor, Store};
use control_plane_runtime::{AgentModel, ModelRequest, RuntimeConfig, Supervisor};
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    path::Path,
    process::Command,
    sync::{Arc, Mutex},
};
use tokio::sync::Notify;

struct Scripted(Mutex<VecDeque<Value>>);
impl AgentModel for Scripted {
    fn respond(&self, request: &ModelRequest) -> anyhow::Result<Value> {
        let next = self
            .0
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected idle model call");
        if request.role == "critic" {
            assert!(request.execution_context.starts_with("critic-"));
        }
        Ok(next)
    }
}

fn run(root: &Path, program: &str, args: &[&str], env: &[(String, String)]) -> String {
    let out = Command::new(program)
        .args(args)
        .envs(env.iter().cloned())
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{program} {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

struct Fixture(std::path::PathBuf);
impl Fixture {
    fn path(&self) -> &Path {
        &self.0
    }
}

// Retain managed fixture records and archives for inspection; never let TempDir
// recursively delete linked Git worktrees behind the registry's back.
async fn setup(
    existing: bool,
) -> (
    Fixture,
    Arc<tokio::sync::Mutex<Store>>,
    RuntimeConfig,
    String,
    String,
) {
    let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.scratch/planner");
    std::fs::create_dir_all(&scratch).unwrap();
    let temp = Fixture(tempfile::tempdir_in(scratch).unwrap().keep());
    let repo = temp.path().join("repos/demo");
    std::fs::create_dir_all(&repo).unwrap();
    run(&repo, "git", &["init", "--initial-branch=main"], &[]);
    run(&repo, "git", &["config", "user.name", "Fixture"], &[]);
    run(
        &repo,
        "git",
        &["config", "user.email", "fixture@example.invalid"],
        &[],
    );
    std::fs::write(repo.join("README.md"), "# Demo\n").unwrap();
    std::fs::create_dir_all(repo.join("ess/domains")).unwrap();
    std::fs::write(
        repo.join("ess/system.yaml"),
        "format: ess/22\nsystem: demo\nversion: v1\ndomains: [demo.item]\n",
    )
    .unwrap();
    std::fs::write(repo.join("ess/domains/item.yaml"),"domain: demo.item\nentities:\n  - name: demo.item.Item\n    identity: {name: item_id, type: Uuid}\n    fields: []\n    lifecycle:\n      initial: Present\n      states: [Present]\n      terminal: [Present]\n").unwrap();
    let config = RuntimeConfig {
        environment: vec![
            (
                "XDG_STATE_HOME".into(),
                temp.path().join("state").display().to_string(),
            ),
            (
                "XDG_CONFIG_HOME".into(),
                temp.path().join("config").display().to_string(),
            ),
        ],
        commit_command: vec!["git".into(), "commit".into(), "-m".into()],
        ..RuntimeConfig::default()
    };
    let profile = temp.path().join("profile.toml");
    std::fs::write(&profile,"version = 1\nname = 'fixture'\nexpire_after_seconds = 604800\nprotect_workspace_root = false\n").unwrap();
    run(
        &repo,
        "worktree",
        &[
            "activate",
            "--profile",
            profile.to_str().unwrap(),
            "--workspace",
            temp.path().join("repos").to_str().unwrap(),
        ],
        &config.environment,
    );
    if existing {
        run(
            &repo,
            "aep",
            &[
                "plan",
                "reverse",
                "init",
                "--protocols",
                &config.aep_protocols,
                "--profile",
                "development.standard",
            ],
            &config.environment,
        );
        run(
            &repo,
            "aep",
            &[
                "plan",
                "artifact",
                "new",
                "story",
                "deliver",
                "--title",
                "Deliver requested change",
            ],
            &config.environment,
        );
        run(
            &repo,
            "aep",
            &[
                "plan",
                "artifact",
                "scope",
                "story:deliver",
                "--add",
                "src/",
                "--inferred",
            ],
            &config.environment,
        );
        run(
            &repo,
            "aep",
            &[
                "plan",
                "artifact",
                "move",
                "story:deliver",
                "--to",
                "proposed",
            ],
            &config.environment,
        );
        run(
            &repo,
            "aep",
            &[
                "plan",
                "artifact",
                "move",
                "story:deliver",
                "--to",
                "active",
            ],
            &config.environment,
        );
    }
    run(&repo, "git", &["add", "."], &[]);
    run(&repo, "git", &["commit", "-m", "fixture"], &[]);
    let mut store = Store::open(temp.path().join("host.sqlite3")).await.unwrap();
    let workspace=store.register_workspace(&repo,"demo").await.unwrap()["published"][0]["payload"]["workspace_id"].as_str().unwrap().to_owned();
    let goal=store.execute("CreateGoal",json!({"workspace_id":workspace,"objective":"Deliver requested change","acceptance":"Required checks pass after reviewed merge","max_workers":3,"max_attempts":3,"max_minutes":1,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":true}),Actor::Operator).await.unwrap()["published"][0]["payload"]["goal_id"].as_str().unwrap().to_owned();
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    let repository = store.query("RepositoryRegistrationList").unwrap()[0]["repository_id"]
        .as_str()
        .unwrap()
        .to_owned();
    (
        temp,
        Arc::new(tokio::sync::Mutex::new(store)),
        config,
        goal,
        repository,
    )
}

#[tokio::test]
async fn existing_backlog_is_not_duplicated() {
    let (_temp, store, config, _goal, repo) = setup(true).await;
    let model = Arc::new(Scripted(Mutex::new(VecDeque::from([
        json!({"action":"finish","stories":["story:deliver"],"summary":"Existing story covers goal."}),
        json!({"approved":true,"reason":"Current acceptance and scope cover the goal."}),
    ]))));
    let supervisor = Supervisor::new(
        store.clone(),
        Arc::new(Notify::new()),
        config,
        model.clone(),
    );
    let first = supervisor.tick().await.unwrap();
    assert_eq!(first.queued, 1, "{first:?}");
    let rows = store.lock().await.query("AssignmentList").unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 1);
    assert_eq!(rows[0]["story_id"], format!("{repo}::story:deliver"));
    assert!(!rows[0]["candidate"].as_str().unwrap().is_empty());
    assert_eq!(supervisor.tick().await.unwrap().queued, 0);
    assert!(model.0.lock().unwrap().is_empty());
}

#[tokio::test]
async fn planner_can_read_aep_help_then_finish_existing_work() {
    let (_fixture, store, config, _goal, _repo) = setup(true).await;
    let model = Arc::new(Scripted(Mutex::new(VecDeque::from([
        json!({"action":"aep","args":["--help"],"body":null}),
        json!({"action":"aep","args":["artifact","--help"],"body":null}),
        json!({"action":"aep","args":["plan","artifact","new","--help"],"body":null}),
        json!({"action":"aep","args":["aep","plan","artifact","scope","-h"],"body":null}),
        json!({"action":"aep","args":["new","--help"],"body":null}),
        json!({"action":"aep","args":["scope","-h"],"body":null}),
        json!({"action":"finish","stories":["story:deliver"],"summary":"Read CLI help and reused existing work."}),
        json!({"approved":true,"reason":"Existing story remains authoritative."}),
    ]))));
    let supervisor = Supervisor::new(
        store.clone(),
        Arc::new(Notify::new()),
        config,
        model.clone(),
    );
    let report = supervisor.tick().await.unwrap();
    assert_eq!(report.queued, 1, "{report:?}");
    assert!(model.0.lock().unwrap().is_empty());
    let goals = store.lock().await.query("GoalList").unwrap();
    assert!(
        goals[0]["planning_receipt"]
            .as_str()
            .unwrap()
            .contains("Usage: aep plan artifact")
    );
}

#[tokio::test]
async fn aep_help_does_not_require_valid_ess_but_finish_still_does() {
    let (fixture, store, config, _goal, _repo) = setup(true).await;
    let repo = fixture.path().join("repos/demo");
    std::fs::write(repo.join("ess/system.yaml"), "invalid: specification\n").unwrap();
    run(&repo, "git", &["add", "ess/system.yaml"], &[]);
    run(
        &repo,
        "git",
        &["commit", "-m", "specification needs repair"],
        &[],
    );
    let model = Arc::new(Scripted(Mutex::new(VecDeque::from([
        json!({"action":"aep","args":["new","--help"],"body":null}),
        json!({"action":"finish","stories":["story:deliver"],"summary":"Validation must still refuse."}),
    ]))));
    let supervisor = Supervisor::new(
        store.clone(),
        Arc::new(Notify::new()),
        config,
        model.clone(),
    );
    let report = supervisor.tick().await.unwrap();
    assert!(
        model.0.lock().unwrap().is_empty(),
        "Help should reach Finish: {report:?}"
    );
    assert_eq!(report.queued, 0);
    assert!(!report.blockers.is_empty());
    assert_eq!(
        store.lock().await.query("AssignmentList").unwrap(),
        json!([])
    );
}

#[tokio::test]
async fn noun_first_aep_syntax_is_feedback_then_corrected_without_authority() {
    let (_fixture, store, config, _goal, _repo) = setup(true).await;
    let model = Arc::new(Scripted(Mutex::new(VecDeque::from([
        json!({"action":"aep","args":["story","show","operator-console"],"body":null}),
        json!({"action":"aep","args":["show","story:deliver"],"body":null}),
        json!({"action":"finish","stories":["story:deliver"],"summary":"Corrected noun-first CLI syntax."}),
        json!({"approved":true,"reason":"Existing story remains authoritative."}),
    ]))));
    let supervisor = Supervisor::new(
        store.clone(),
        Arc::new(Notify::new()),
        config,
        model.clone(),
    );
    let report = supervisor.tick().await.unwrap();
    assert_eq!(report.queued, 1, "{report:?}");
    assert!(model.0.lock().unwrap().is_empty());
    let goals = store.lock().await.query("GoalList").unwrap();
    let receipt = goals[0]["planning_receipt"].as_str().unwrap();
    assert!(receipt.contains("AEP syntax feedback"));
    assert!(receipt.contains("operator-console"));
}

#[tokio::test]
async fn aep_syntax_feedback_reaches_model_and_recovers_in_same_attempt() {
    struct Recovering(Mutex<usize>);
    impl AgentModel for Recovering {
        fn respond(&self, request: &ModelRequest) -> anyhow::Result<Value> {
            let mut calls = self.0.lock().unwrap();
            let answer = match *calls {
                0 => json!({"action":"aep","args":["list","--not-a-real-option"],"body":null}),
                1 => {
                    assert!(request.prompt.contains("--not-a-real-option"));
                    assert!(request.prompt.contains("AEP syntax feedback"));
                    assert!(request.prompt.contains("Admitted grammar"));
                    assert!(request.prompt.contains("Usage:"));
                    json!({"action":"aep","args":["new","story"],"body":null})
                }
                2 => {
                    assert!(request.prompt.contains("required arguments"));
                    json!({"action":"finish","stories":["story:deliver"],"summary":"Corrected syntax and retained existing story."})
                }
                3 => json!({"approved":true,"reason":"Existing authoritative story is unchanged."}),
                _ => panic!("unexpected model retry"),
            };
            *calls += 1;
            Ok(answer)
        }
    }
    let (_fixture, store, config, _goal, _repo) = setup(true).await;
    let model = Arc::new(Recovering(Mutex::new(0)));
    let supervisor = Supervisor::new(
        store.clone(),
        Arc::new(Notify::new()),
        config,
        model.clone(),
    );
    let report = supervisor.tick().await.unwrap();
    assert_eq!(report.queued, 1, "{report:?}");
    assert_eq!(*model.0.lock().unwrap(), 4);
    let goals = store.lock().await.query("GoalList").unwrap();
    let receipt: Value =
        serde_json::from_str(goals[0]["planning_receipt"].as_str().unwrap()).unwrap();
    let failed = receipt["activity"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["action"] == "aep.syntax_rejected")
        .collect::<Vec<_>>();
    assert_eq!(failed.len(), 2);
    assert!(failed.iter().all(|event| event["status"] == "failed"));
}

#[tokio::test]
async fn prior_actions_and_unchanged_feedback_break_a_stationary_model_loop() {
    struct NeedsMemory(Mutex<Vec<String>>);
    impl AgentModel for NeedsMemory {
        fn respond(&self, request: &ModelRequest) -> anyhow::Result<Value> {
            assert!(request.prompt.contains("Deliver requested change"));
            if request.role == "critic" {
                return Ok(json!({"approved":true,"reason":"Existing story is sufficient."}));
            }
            assert!(
                request
                    .prompt
                    .contains("Required checks pass after reviewed merge")
            );
            let mut prompts = self.0.lock().unwrap();
            prompts.push(request.prompt.clone());
            if request
                .prompt
                .contains("Repeated unchanged observation (2)")
            {
                assert!(request.prompt.contains("Recent attempted actions"));
                assert!(request.prompt.contains("README.md"));
                assert!(request.prompt.contains("Choose new evidence"));
                assert_ne!(prompts[prompts.len() - 1], prompts[prompts.len() - 2]);
                Ok(
                    json!({"action":"finish","stories":["story:deliver"],"summary":"Repeated read adds no evidence; existing plan is ready."}),
                )
            } else {
                Ok(json!({"action":"read","paths":["README.md"]}))
            }
        }
    }
    let (_fixture, store, mut config, _goal, _repo) = setup(true).await;
    config.max_steps = 6;
    let model = Arc::new(NeedsMemory(Mutex::new(Vec::new())));
    let supervisor = Supervisor::new(store, Arc::new(Notify::new()), config, model.clone());
    let report = supervisor.tick().await.unwrap();
    assert_eq!(report.queued, 1, "{report:?}");
    assert_eq!(model.0.lock().unwrap().len(), 3);
}

#[tokio::test]
async fn unchanged_aep_reads_stop_with_explicit_stall_reason() {
    struct Repeats(Mutex<usize>);
    impl AgentModel for Repeats {
        fn respond(&self, _: &ModelRequest) -> anyhow::Result<Value> {
            *self.0.lock().unwrap() += 1;
            Ok(json!({"action":"aep","args":["show","story:deliver"],"body":null}))
        }
    }
    let (_fixture, store, mut config, _goal, _repo) = setup(true).await;
    config.max_steps = 6;
    let model = Arc::new(Repeats(Mutex::new(0)));
    let supervisor = Supervisor::new(store, Arc::new(Notify::new()), config, model.clone());
    let report = supervisor.tick().await.unwrap();
    assert!(
        report
            .blockers
            .iter()
            .any(|reason| reason.contains("planner stalled after 4 unchanged observations")),
        "{report:?}"
    );
    assert_eq!(*model.0.lock().unwrap(), 4);
}

#[tokio::test]
async fn alternating_reads_retain_unchanged_feedback_across_actions() {
    struct Alternates(Mutex<usize>);
    impl AgentModel for Alternates {
        fn respond(&self, request: &ModelRequest) -> anyhow::Result<Value> {
            if request.role == "critic" {
                return Ok(
                    json!({"approved":true,"reason":"Existing scoped story satisfies the goal."}),
                );
            }
            let mut calls = self.0.lock().unwrap();
            *calls += 1;
            if request
                .prompt
                .contains("Repeated unchanged observation (2)")
            {
                assert!(request.prompt.contains("Recent attempted actions"));
                assert!(request.prompt.contains("README.md"));
                assert!(request.prompt.contains("ess/system.yaml"));
                Ok(
                    json!({"action":"finish","stories":["story:deliver"],"summary":"Alternating unchanged reads add no evidence."}),
                )
            } else {
                Ok(
                    json!({"action":"read","paths":[if *calls % 2 == 1 {"README.md"} else {"ess/system.yaml"}]}),
                )
            }
        }
    }
    let (_fixture, store, mut config, _goal, _repo) = setup(true).await;
    config.max_steps = 7;
    let model = Arc::new(Alternates(Mutex::new(0)));
    let supervisor = Supervisor::new(store, Arc::new(Notify::new()), config, model.clone());
    let report = supervisor.tick().await.unwrap();
    assert_eq!(report.queued, 1, "{report:?}");
    assert_eq!(*model.0.lock().unwrap(), 4);
}

#[tokio::test]
async fn repeated_aep_syntax_errors_stop_at_a_bounded_failure_budget() {
    let (_fixture, store, config, _goal, _repo) = setup(true).await;
    let bad = json!({"action":"aep","args":["list","--not-a-real-option"],"body":null});
    let model = Arc::new(Scripted(Mutex::new(VecDeque::from([
        bad.clone(),
        bad.clone(),
        bad,
        json!({"action":"finish","stories":["story:deliver"],"summary":"Must not be reached."}),
    ]))));
    let supervisor = Supervisor::new(
        store.clone(),
        Arc::new(Notify::new()),
        config,
        model.clone(),
    );
    let report = supervisor.tick().await.unwrap();
    assert!(
        report
            .blockers
            .iter()
            .any(|reason| reason.contains("AEP syntax error budget exhausted")),
        "{report:?}"
    );
    assert_eq!(model.0.lock().unwrap().len(), 1);
    assert_eq!(
        store.lock().await.query("AssignmentList").unwrap(),
        json!([])
    );
}

#[tokio::test]
async fn prefixed_aep_authority_and_path_violations_are_not_syntax_retries() {
    for args in [
        json!(["artifact", "list", "--store", "elsewhere"]),
        json!(["aep", "plan", "artifact", "new", "evidence", "forged"]),
        json!([
            "plan",
            "artifact",
            "move",
            "story:deliver",
            "--to",
            "implemented"
        ]),
    ] {
        let (_fixture, store, config, _goal, _repo) = setup(true).await;
        let model = Arc::new(Scripted(Mutex::new(VecDeque::from([
            json!({"action":"aep","args":args,"body":null}),
            json!({"action":"finish","stories":["story:deliver"],"summary":"Must not be reached."}),
        ]))));
        let supervisor = Supervisor::new(
            store.clone(),
            Arc::new(Notify::new()),
            config,
            model.clone(),
        );
        let report = supervisor.tick().await.unwrap();
        assert!(!report.blockers.is_empty());
        assert_eq!(model.0.lock().unwrap().len(), 1);
        assert_eq!(
            store.lock().await.query("AssignmentList").unwrap(),
            json!([])
        );
    }
}

#[tokio::test]
async fn goal_drives_plan() {
    let (_temp, store, config, _goal, _repo) = setup(false).await;
    let model = Arc::new(Scripted(Mutex::new(VecDeque::from([
        json!({"action":"aep","args":["new","story","deliver","--title","Deliver requested change","--from","-"],"body":"## Acceptance\nNamed scenario: delivery_works.\n## Scope\nsrc/\n"}),
        json!({"action":"aep","args":["scope","story:deliver","--add","src/","--inferred"],"body":null}),
        json!({"action":"finish","stories":["story:deliver"],"summary":"New ESS-backed work."}),
        json!({"approved":true,"reason":"Validated specification and named scenario."}),
    ]))));
    let supervisor = Supervisor::new(store.clone(), Arc::new(Notify::new()), config, model);
    let report = supervisor.tick().await.unwrap();
    assert_eq!(report.queued, 1, "{report:?}");
    assert_eq!(
        store.lock().await.query("AssignmentList").unwrap()[0]["goal_revision"],
        1
    );
}

#[tokio::test]
async fn goal_completion_requires_evidence() {
    let (_temp, store, config, _goal, _repo) = setup(true).await;
    let model = Arc::new(Scripted(Mutex::new(VecDeque::from([
        json!({"action":"finish","stories":[],"summary":"Nothing else to plan."}),
        json!({"approved":true,"reason":"No further proposal."}),
    ]))));
    let supervisor = Supervisor::new(store.clone(), Arc::new(Notify::new()), config, model);
    let report = supervisor.tick().await.unwrap();
    assert_eq!(report.queued, 0);
    assert_eq!(
        store.lock().await.query("GoalList").unwrap()[0]["state"],
        "Running"
    );
}

#[tokio::test]
async fn rejected_critique_is_durable_and_idle_after_restart() {
    let (fixture, store, config, _goal, _repo) = setup(true).await;
    let model = Arc::new(Scripted(Mutex::new(VecDeque::from([
        json!({"action":"finish","stories":["story:deliver"],"summary":"Review me"}),
        json!({"approved":false,"reason":"Named acceptance scenarios are missing"}),
    ]))));
    let supervisor = Supervisor::new(
        store.clone(),
        Arc::new(Notify::new()),
        config.clone(),
        model,
    );
    let report = supervisor.tick().await.unwrap();
    assert!(
        report.blockers[0].contains("critique rejected"),
        "{report:?}"
    );
    assert!(
        store
            .lock()
            .await
            .query("AssignmentList")
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        store.lock().await.query("GoalList").unwrap()[0]["planning_phase"],
        "Blocked"
    );
    drop(supervisor);
    drop(store);
    let reopened = Arc::new(tokio::sync::Mutex::new(
        Store::open(fixture.path().join("host.sqlite3"))
            .await
            .unwrap(),
    ));
    let idle = Arc::new(Scripted(Mutex::new(VecDeque::new())));
    let supervisor = Supervisor::new(reopened, Arc::new(Notify::new()), config, idle);
    assert_eq!(supervisor.tick().await.unwrap().planned, 0);
}

#[tokio::test]
async fn model_cannot_write_workflows_when_ess_is_at_root() {
    let (fixture, store, config, _goal, _repo) = setup(true).await;
    let repo = fixture.path().join("repos/demo");
    std::fs::rename(repo.join("ess/system.yaml"), repo.join("system.yaml")).unwrap();
    std::fs::rename(repo.join("ess/domains"), repo.join("domains")).unwrap();
    run(&repo, "git", &["add", "."], &[]);
    run(&repo, "git", &["commit", "-m", "root specification"], &[]);
    let model = Arc::new(Scripted(Mutex::new(VecDeque::from([
        json!({"action":"write_specification","path":".github/workflows/publish.yaml","contents":"name: unauthorized\n"}),
    ]))));
    let supervisor = Supervisor::new(store.clone(), Arc::new(Notify::new()), config, model);
    let report = supervisor.tick().await.unwrap();
    assert!(
        report.blockers[0].contains("not an admitted ESS source"),
        "{report:?}"
    );
    let goals = store.lock().await.query("GoalList").unwrap();
    let path = Path::new(goals[0]["planning_worktree_path"].as_str().unwrap());
    assert!(!path.join(".github/workflows/publish.yaml").exists());
}

#[tokio::test]
async fn model_cannot_override_aep_store() {
    let (_fixture, store, config, _goal, _repo) = setup(true).await;
    let model = Arc::new(Scripted(Mutex::new(VecDeque::from([
        json!({"action":"aep","args":["new","story","evil","--store","elsewhere","--title","bad"],"body":null}),
    ]))));
    let supervisor = Supervisor::new(store, Arc::new(Notify::new()), config, model);
    let report = supervisor.tick().await.unwrap();
    assert!(
        report.blockers[0].contains("path override is forbidden"),
        "{report:?}"
    );
}

#[tokio::test]
async fn commit_refusal_is_a_blocker_and_never_queues() {
    let (_fixture, store, mut config, _goal, _repo) = setup(false).await;
    config.commit_command = vec!["git".into(), "definitely-not-a-command".into()];
    let model = Arc::new(Scripted(Mutex::new(VecDeque::from([
        json!({"action":"finish","stories":[],"summary":"Valid adopted store"}),
        json!({"approved":true,"reason":"No implementation work selected"}),
    ]))));
    let supervisor = Supervisor::new(store.clone(), Arc::new(Notify::new()), config, model);
    let report = supervisor.tick().await.unwrap();
    assert!(
        report.blockers[0].contains("definitely-not-a-command"),
        "{report:?}"
    );
    assert_eq!(
        store.lock().await.query("AssignmentList").unwrap(),
        json!([])
    );
}

struct PausingModel {
    store: Arc<tokio::sync::Mutex<Store>>,
    goal: String,
}

struct ObservingModel {
    store: Arc<tokio::sync::Mutex<Store>>,
    entered: Mutex<Vec<Value>>,
}

struct RepeatedReadModel {
    calls: Mutex<usize>,
    prompts: Mutex<Vec<usize>>,
}

struct LongLineModel {
    calls: Mutex<usize>,
}
impl AgentModel for LongLineModel {
    fn respond(&self, request: &ModelRequest) -> anyhow::Result<Value> {
        if request.role == "critic" {
            return Ok(json!({"approved":true,"reason":"Complete long-line context inspected"}));
        }
        let mut calls = self.calls.lock().unwrap();
        *calls += 1;
        if *calls == 1 {
            return Ok(
                json!({"action":"read_range","path":"single-line.json","start_line":1,"line_count":1}),
            );
        }
        if *calls == 2 {
            anyhow::ensure!(
                request.prompt.contains("read_bytes"),
                "long line omitted its byte continuation instruction"
            );
            return Ok(
                json!({"action":"read_bytes","path":"single-line.json","start_byte":15000,"byte_count":4096}),
            );
        }
        anyhow::ensure!(
            request.prompt.contains("Final long-line acceptance marker"),
            "byte continuation lost the tail of a long UTF-8 line"
        );
        Ok(json!({"action":"finish","stories":["story:deliver"],"summary":"Read complete context"}))
    }
}

#[tokio::test]
async fn single_long_utf8_line_remains_retrievable_through_byte_pages() {
    let (fixture, store, config, _goal, _repo) = setup(true).await;
    let repo = fixture.path().join("repos/demo");
    std::fs::write(
        repo.join("single-line.json"),
        format!("{}Final long-line acceptance marker", "界".repeat(5000)),
    )
    .unwrap();
    run(
        &repo,
        "git",
        &["add", "single-line.json"],
        &config.environment,
    );
    run(
        &repo,
        "git",
        &["commit", "-m", "Fixture long UTF-8 line"],
        &config.environment,
    );
    let supervisor = Supervisor::new(
        store,
        Arc::new(Notify::new()),
        config,
        Arc::new(LongLineModel {
            calls: Mutex::new(0),
        }),
    );
    let report = supervisor.tick().await.unwrap();
    assert_eq!(
        report.queued, 1,
        "long line remained inaccessible: {report:?}"
    );
}
impl AgentModel for RepeatedReadModel {
    fn respond(&self, request: &ModelRequest) -> anyhow::Result<Value> {
        self.prompts.lock().unwrap().push(request.prompt.len());
        if request.role == "critic" {
            return Ok(
                json!({"approved":true,"reason":"The existing scoped story matches the goal"}),
            );
        }
        let mut calls = self.calls.lock().unwrap();
        *calls += 1;
        Ok(if *calls <= 8 {
            json!({"action":"read","paths":[format!("context{}.txt",*calls)]})
        } else if *calls == 9 {
            json!({"action":"read_range","path":"context1.txt","start_line":7001,"line_count":1})
        } else {
            assert!(request.prompt.contains("Final page acceptance marker"));
            json!({"action":"finish","stories":["story:deliver"],"summary":"Context inspected; deliver the existing story"})
        })
    }
}

#[tokio::test]
async fn distinct_large_reads_do_not_exhaust_planner_context() {
    let (fixture, store, config, _goal, _repo) = setup(true).await;
    let repo = fixture.path().join("repos/demo");
    for index in 1..=8 {
        let name = format!("context{index}.txt");
        std::fs::write(
            repo.join(&name),
            format!(
                "{}Final page acceptance marker\n",
                "Observed repository context.\n".repeat(7_000)
            ),
        )
        .unwrap();
        run(&repo, "git", &["add", &name], &config.environment);
    }
    run(
        &repo,
        "git",
        &["commit", "-m", "Add large context fixture"],
        &config.environment,
    );
    let model = Arc::new(RepeatedReadModel {
        calls: Mutex::new(0),
        prompts: Mutex::new(Vec::new()),
    });
    let supervisor = Supervisor::new(
        store.clone(),
        Arc::new(Notify::new()),
        config,
        model.clone(),
    );
    let report = supervisor.tick().await.unwrap();
    assert_eq!(
        report.queued, 1,
        "Bounded reads prevented planning: {report:?}"
    );
    assert!(
        model
            .prompts
            .lock()
            .unwrap()
            .iter()
            .all(|size| *size <= 160 * 1024),
        "planner sent oversized context"
    );
}
impl AgentModel for ObservingModel {
    fn respond(&self, request: &ModelRequest) -> anyhow::Result<Value> {
        let receipt = tokio::runtime::Handle::current().block_on(async {
            self.store.lock().await.query("GoalList").unwrap()[0]["planning_receipt"].clone()
        });
        self.entered
            .lock()
            .unwrap()
            .push(serde_json::from_str(receipt.as_str().unwrap()).unwrap_or(json!({})));
        Ok(if request.role == "critic" {
            json!({"approved":true,"reason":"Requested story matches the goal"})
        } else {
            json!({"action":"finish","stories":["story:deliver"],"summary":"Deliver existing story"})
        })
    }
}

#[tokio::test]
async fn model_wait_is_visible_before_response_and_activity_survives_restart() {
    let (_fixture, store, config, _goal, _repo) = setup(true).await;
    let model = Arc::new(ObservingModel {
        store: store.clone(),
        entered: Mutex::new(Vec::new()),
    });
    let supervisor = Supervisor::new(
        store.clone(),
        Arc::new(Notify::new()),
        config,
        model.clone(),
    );
    let report = supervisor.tick().await.unwrap();
    assert_eq!(report.queued, 1);
    {
        let observed = model.entered.lock().unwrap();
        assert_eq!(observed.len(), 2);
        for (entry, role) in observed.iter().zip(["planner", "critic"]) {
            assert_eq!(entry["last_activity"]["action"], "model.requested");
            assert_eq!(entry["last_activity"]["role"], role);
            assert_eq!(entry["last_activity"]["status"], "running");
            time::OffsetDateTime::parse(
                entry["last_activity"]["at"].as_str().unwrap(),
                &time::format_description::well_known::Rfc3339,
            )
            .unwrap();
        }
    }
    let before = store.lock().await.query("GoalList").unwrap();
    let receipt: Value =
        serde_json::from_str(before[0]["planning_receipt"].as_str().unwrap()).unwrap();
    assert!(
        receipt["activity"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["action"] == "model.completed" && a["role"] == "critic")
    );
    assert_eq!(before[0]["planning_phase"], "Queued");
    drop(supervisor);
    drop(model);
    drop(store);
    let reopened = Store::open(_fixture.path().join("host.sqlite3"))
        .await
        .unwrap();
    assert_eq!(reopened.query("GoalList").unwrap(), before);
}
impl AgentModel for PausingModel {
    fn respond(&self, _: &ModelRequest) -> anyhow::Result<Value> {
        tokio::runtime::Handle::current().block_on(async {
            self.store
                .lock()
                .await
                .execute("PauseGoal", json!({"goal_id":self.goal}), Actor::Operator)
                .await
        })?;
        Ok(
            json!({"action":"finish","stories":["story:deliver"],"summary":"Stale response after operator pause"}),
        )
    }
}
#[tokio::test]
async fn pause_during_model_call_prevents_further_effects() {
    let (_fixture, store, config, goal, _repo) = setup(true).await;
    let model = Arc::new(PausingModel {
        store: store.clone(),
        goal: goal.clone(),
    });
    let supervisor = Supervisor::new(
        store.clone(),
        Arc::new(Notify::new()),
        config.clone(),
        model,
    );
    let report = supervisor.tick().await.unwrap();
    assert_eq!(report.queued, 0);
    assert!(!report.blockers.is_empty());
    assert_eq!(
        store.lock().await.query("GoalList").unwrap()[0]["state"],
        "Paused"
    );
    assert_eq!(
        store.lock().await.query("AssignmentList").unwrap(),
        json!([])
    );
    let retained = store.lock().await.query("GoalList").unwrap()[0]["planning_worktree_id"].clone();
    store
        .lock()
        .await
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    drop(supervisor);
    let model = Arc::new(Scripted(Mutex::new(VecDeque::from([
        json!({"action":"finish","stories":["story:deliver"],"summary":"Resume retained work"}),
        json!({"approved":true,"reason":"The retained AEP story covers the goal"}),
    ]))));
    let resumed = Supervisor::new(store.clone(), Arc::new(Notify::new()), config, model);
    let report = resumed.tick().await.unwrap();
    assert_eq!(report.queued, 1, "{report:?}");
    assert_eq!(
        store.lock().await.query("AssignmentList").unwrap()[0]["worktree_id"],
        retained
    );
}

#[test]
fn process_exit_timeout_and_cancellation_are_not_successful_observations() {
    use control_plane_runtime::process::ProcessRunner;
    use std::time::Duration;
    use tokio_util::sync::CancellationToken;
    let cancel = CancellationToken::new();
    let runner = ProcessRunner {
        environment: vec![],
        timeout: Duration::from_millis(30),
        cancel: cancel.clone(),
    };
    let cwd = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(runner.command(cwd, "git", &["not-a-real-command"]).is_err());
    assert!(
        runner
            .command(cwd, "sleep", &["5"])
            .unwrap_err()
            .to_string()
            .contains("timeout")
    );
    cancel.cancel();
    assert!(
        runner
            .command(cwd, "git", &["--version"])
            .unwrap_err()
            .to_string()
            .contains("cancelled before")
    );
}

#[tokio::test]
async fn updated_goal_replans_past_stale_queued_assignments() {
    let (_fixture, store, config, goal, _repo) = setup(true).await;
    let model = Arc::new(Scripted(Mutex::new(VecDeque::from([
        json!({"action":"finish","stories":["story:deliver"],"summary":"Initial goal"}),
        json!({"approved":true,"reason":"Existing story covers initial acceptance"}),
        json!({"action":"finish","stories":["story:deliver"],"summary":"Revalidated for revised acceptance"}),
        json!({"approved":true,"reason":"Existing story also covers revised acceptance"}),
    ]))));
    let supervisor = Supervisor::new(store.clone(), Arc::new(Notify::new()), config, model);
    assert_eq!(supervisor.tick().await.unwrap().queued, 1);
    let update = json!({"goal_id":goal,"objective":"Deliver revised requested change","acceptance":"Revised checks pass after reviewed merge","max_workers":3,"max_attempts":3,"max_minutes":1,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":true});
    assert_eq!(
        store
            .lock()
            .await
            .execute("UpdateGoal", update, Actor::Operator)
            .await
            .unwrap()["outcome"],
        "applied"
    );
    let next = supervisor.tick().await.unwrap();
    let assignments = store.lock().await.query("AssignmentList").unwrap();
    assert!(
        assignments
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["goal_revision"] == 2 && a["state"] == "Queued"),
        "changed goal must produce current-revision work, not idle behind a stale queued assignment: {next:?}; {assignments}"
    );
}

#[tokio::test]
async fn unavailable_registered_repository_is_a_durable_blocker_not_a_dead_supervisor() {
    let (fixture, store, config, _goal, _repo) = setup(true).await;
    std::fs::rename(
        fixture.path().join("repos/demo"),
        fixture.path().join("repos/offline-demo"),
    )
    .unwrap();
    let model = Arc::new(Scripted(Mutex::new(VecDeque::new())));
    let supervisor = Supervisor::new(store.clone(), Arc::new(Notify::new()), config, model);
    let result = supervisor.tick().await;
    assert!(
        result.is_ok(),
        "one unavailable repository must be recorded as a blocker instead of ending Supervisor::run: {result:?}"
    );
    assert!(!result.unwrap().blockers.is_empty());
    assert_eq!(
        store.lock().await.query("GoalList").unwrap()[0]["planning_phase"],
        "Blocked"
    );
}

#[tokio::test]
async fn linked_aep_store_cannot_be_mutated_outside_planning_checkout() {
    let (fixture, store, config, _goal, _repo) = setup(true).await;
    let primary = fixture.path().join("repos/demo");
    let external = fixture.path().join("outside-engineering");
    std::fs::rename(primary.join(".engineering"), &external).unwrap();
    std::os::unix::fs::symlink(&external, primary.join(".engineering")).unwrap();
    run(&primary, "git", &["add", "-A"], &[]);
    run(
        &primary,
        "git",
        &["commit", "-m", "linked planning store fixture"],
        &[],
    );
    let model = Arc::new(Scripted(Mutex::new(VecDeque::from([
        json!({"action":"aep","args":["new","story","escape","--title","This must stay inside the isolated checkout"],"body":null}),
        json!({"action":"finish","stories":[],"summary":"Do not approve"}),
        json!({"approved":false,"reason":"Linked planning store must be rejected"}),
    ]))));
    let supervisor = Supervisor::new(store, Arc::new(Notify::new()), config, model);
    let result = supervisor.tick().await;
    assert!(
        !external.join("planning/story/escape.md").exists(),
        "model-directed AEP mutation escaped the managed checkout through .engineering symlink: {result:?}"
    );
}

struct CountedModel {
    responses: Mutex<VecDeque<Value>>,
    calls: std::sync::atomic::AtomicUsize,
}
impl AgentModel for CountedModel {
    fn respond(&self, _: &ModelRequest) -> anyhow::Result<Value> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(self
            .responses
            .lock()
            .unwrap()
            .pop_front()
            .expect("bounded scripted response"))
    }
}

#[tokio::test]
async fn another_workspaces_assignment_does_not_rewake_unchanged_idle_goal() {
    let (fixture, store, config, _goal, _repo) = setup(true).await;
    let model = Arc::new(CountedModel {
        responses: Mutex::new(VecDeque::from([
            json!({"action":"finish","stories":[],"summary":"No work selected"}),
            json!({"approved":true,"reason":"Acceptance evidence still outstanding"}),
            json!({"action":"finish","stories":[],"summary":"No work selected"}),
            json!({"approved":true,"reason":"Acceptance evidence still outstanding"}),
        ])),
        calls: std::sync::atomic::AtomicUsize::new(0),
    });
    let supervisor = Supervisor::new(
        store.clone(),
        Arc::new(Notify::new()),
        config,
        model.clone(),
    );
    supervisor.tick().await.unwrap();
    assert_eq!(model.calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    let unrelated = fixture.path().join("repos/unrelated");
    std::fs::create_dir_all(&unrelated).unwrap();
    run(&unrelated, "git", &["init", "--initial-branch=main"], &[]);
    let mut host = store.lock().await;
    let ws = host
        .register_workspace(&unrelated, "unrelated")
        .await
        .unwrap()["published"][0]["payload"]["workspace_id"]
        .clone();
    let repository = host
        .query("RepositoryRegistrationList")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["workspace_id"] == ws)
        .unwrap()["repository_id"]
        .clone();
    let other_goal = host.execute("CreateGoal", json!({"workspace_id":ws,"objective":"Unrelated objective","acceptance":"Unrelated acceptance","max_workers":1,"max_attempts":1,"max_minutes":1,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":false}), Actor::Operator).await.unwrap()["published"][0]["payload"]["goal_id"].clone();
    host.execute("StartGoal", json!({"goal_id":other_goal}), Actor::Operator)
        .await
        .unwrap();
    host.execute("QueueAssignment", json!({"goal_id":other_goal,"repository_id":repository,"story_id":"story:unrelated","case_id":"unrelated-case","worktree_id":"","candidate":"","attempt":0,"reason":"","implementor_run":"","reviewer_run":"","goal_revision":1}), Actor::Supervisor).await.unwrap();
    host.execute("PauseGoal", json!({"goal_id":other_goal}), Actor::Operator)
        .await
        .unwrap();
    drop(host);
    supervisor.tick().await.unwrap();
    assert_eq!(
        model.calls.load(std::sync::atomic::Ordering::SeqCst),
        2,
        "unrelated assignment changed no input of the first workspace, which must remain idle"
    );
}

#[tokio::test]
async fn planning_commit_is_retained_on_a_named_branch() {
    let (_fixture, store, config, _goal, _repo) = setup(false).await;
    let model = Arc::new(Scripted(Mutex::new(VecDeque::from([
        json!({"action":"aep","args":["new","story","deliver","--title","Requested change","--from","-"],"body":"## Acceptance\nNamed scenario: delivery_works.\n"}),
        json!({"action":"aep","args":["scope","story:deliver","--add","src/","--inferred"],"body":null}),
        json!({"action":"finish","stories":["story:deliver"],"summary":"Ready for implementation"}),
        json!({"approved":true,"reason":"Validated specification and named acceptance"}),
    ]))));
    let supervisor = Supervisor::new(store.clone(), Arc::new(Notify::new()), config, model);
    assert_eq!(supervisor.tick().await.unwrap().queued, 1);
    let goals = store.lock().await.query("GoalList").unwrap();
    let path = Path::new(goals[0]["planning_worktree_path"].as_str().unwrap());
    let branch = Command::new("git")
        .args(["symbolic-ref", "--short", "HEAD"])
        .current_dir(path)
        .output()
        .unwrap();
    assert!(
        branch.status.success(),
        "the managed planning checkout must create a branch before its first commit, preserving a named handoff: {}",
        String::from_utf8_lossy(&branch.stderr)
    );
}

#[tokio::test]
async fn nested_aep_symlinks_are_refused_before_any_model_or_cli_store_access() {
    for relative in [
        "planning/story/deliver.md",
        "planning/journal.jsonl",
        "planning/evidence",
    ] {
        let (fixture, store, config, _goal, _repo) = setup(true).await;
        let primary = fixture.path().join("repos/demo");
        let linked = primary.join(".engineering").join(relative);
        let external = fixture.path().join("external-planning-input");
        if linked.exists() {
            std::fs::rename(&linked, &external).unwrap();
        } else {
            std::fs::create_dir_all(linked.parent().unwrap()).unwrap();
            std::fs::create_dir(&external).unwrap();
        }
        std::os::unix::fs::symlink(&external, &linked).unwrap();
        run(&primary, "git", &["add", "-A"], &[]);
        run(
            &primary,
            "git",
            &["commit", "-m", "nested linked store fixture"],
            &[],
        );
        let model = Arc::new(Scripted(Mutex::new(VecDeque::new())));
        let supervisor = Supervisor::new(store, Arc::new(Notify::new()), config, model);
        let report = supervisor.tick().await.unwrap();
        assert!(
            report
                .blockers
                .iter()
                .any(|reason| reason.contains("AEP store symlink refused")),
            "{relative}: {report:?}"
        );
    }
}

#[tokio::test]
async fn unavailable_workspace_does_not_prevent_healthy_workspace_planning() {
    let (fixture, store, config, _goal, _repo) = setup(true).await;
    let offline = fixture.path().join("repos/offline-demo");
    std::fs::rename(fixture.path().join("repos/demo"), &offline).unwrap();
    let healthy = fixture.path().join("repos/healthy");
    run(
        fixture.path(),
        "git",
        &[
            "clone",
            offline.to_str().unwrap(),
            healthy.to_str().unwrap(),
        ],
        &[],
    );
    let mut host = store.lock().await;
    let workspace=host.register_workspace(&healthy,"healthy").await.unwrap()["published"][0]["payload"]["workspace_id"].clone();
    let goal=host.execute("CreateGoal",json!({"workspace_id":workspace,"objective":"Healthy workspace","acceptance":"Verified delivery","max_workers":1,"max_attempts":1,"max_minutes":1,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":false}),Actor::Operator).await.unwrap()["published"][0]["payload"]["goal_id"].clone();
    host.execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    drop(host);
    let model = Arc::new(Scripted(Mutex::new(VecDeque::from([
        json!({"action":"finish","stories":["story:deliver"],"summary":"Healthy workspace remains schedulable"}),
        json!({"approved":true,"reason":"Existing story covers the healthy goal"}),
    ]))));
    let supervisor = Supervisor::new(store.clone(), Arc::new(Notify::new()), config, model);
    let report = supervisor.tick().await.unwrap();
    assert_eq!(report.queued, 1, "{report:?}");
    assert!(!report.blockers.is_empty());
    assert!(
        store
            .lock()
            .await
            .query("AssignmentList")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["goal_id"] == goal && a["state"] == "Queued")
    );
}
