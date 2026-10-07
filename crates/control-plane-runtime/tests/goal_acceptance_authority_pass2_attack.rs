//! Adversarial cases for story:goal-acceptance-authority, wave 3 pass 2 (attacks 65cba8c).
//!
//! The correction says an acceptance that ends because the goal's revision changed is an
//! interruption: it does not latch, and the edited goal is planned again. `satisfy_goals`
//! (fleet.rs) reads every Running goal once, before it runs any acceptance, and then accepts
//! the goals one after another; one acceptance runs checks and a model review and can take
//! minutes. These cases have two workspaces, each with one Running goal whose work is merged,
//! and an operator edits the second goal while the first goal's acceptance runs.
use anyhow::Result;
use control_plane_core::{Actor, Store};
use control_plane_runtime::{AgentModel, ModelRequest, RuntimeConfig, SharedStore, Supervisor};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::Notify;

const PUBLISH: &str = "git push --force-with-lease=refs/heads/{target}:{expected_base} origin {candidate}:refs/heads/{target}";

fn cmd(cwd: &Path, program: &str, args: &[&str], env: &[(String, String)]) -> String {
    let output = Command::new(program)
        .current_dir(cwd)
        .args(args)
        .envs(env.iter().cloned())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{program} {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

struct Fixture {
    store: SharedStore,
    config: RuntimeConfig,
    goals: Vec<Value>,
}

/// One repository with a story and a bare `origin`, the shape of the repositories of
/// `fixture` in tests/fleet.rs.
fn repository(root: &Path, index: usize, config: &RuntimeConfig) -> PathBuf {
    let repo = root.join(format!("repos/repo{index}"));
    std::fs::create_dir_all(repo.join("src")).unwrap();
    std::fs::create_dir(repo.join("tests")).unwrap();
    std::fs::create_dir_all(repo.join("ess/domains")).unwrap();
    std::fs::write(
        repo.join("Cargo.toml"),
        format!(
            "[package]\nname = 'fleet_fixture_{index}'\nversion = '0.1.0'\nedition = '2024'\n[workspace]\n"
        ),
    )
    .unwrap();
    std::fs::write(repo.join("src/lib.rs"), "pub fn answer() -> u32 { 0 }\n").unwrap();
    std::fs::write(
        repo.join("tests/acceptance.rs"),
        format!(
            "#[test]\nfn requested_answer() {{ assert_eq!(fleet_fixture_{index}::answer(), 42); }}\n"
        ),
    )
    .unwrap();
    std::fs::write(repo.join(".gitignore"), "target\nCargo.lock\n").unwrap();
    std::fs::write(
        repo.join("ess/system.yaml"),
        "format: ess/22\nsystem: demo\nversion: v1\ndomains: [demo.item]\n",
    )
    .unwrap();
    std::fs::write(repo.join("ess/domains/item.yaml"), "domain: demo.item\nentities:\n  - name: demo.item.Item\n    identity: {name: item_id, type: Uuid}\n    fields: []\n    lifecycle:\n      initial: Present\n      states: [Present]\n      terminal: [Present]\n").unwrap();
    cmd(&repo, "git", &["init", "--initial-branch=main"], &[]);
    cmd(&repo, "git", &["config", "user.name", "Fixture"], &[]);
    cmd(
        &repo,
        "git",
        &["config", "user.email", "fixture@example.invalid"],
        &[],
    );
    let aep = |args: &[&str]| cmd(&repo, "aep", args, &config.environment);
    aep(&[
        "plan",
        "reverse",
        "init",
        "--protocols",
        &config.aep_protocols,
        "--profile",
        "development.standard",
    ]);
    aep(&[
        "plan",
        "artifact",
        "new",
        "story",
        "deliver",
        "--title",
        "Return the requested answer",
    ]);
    aep(&[
        "plan",
        "artifact",
        "scope",
        "story:deliver",
        "--add",
        "src/",
        "--inferred",
    ]);
    aep(&[
        "plan",
        "artifact",
        "move",
        "story:deliver",
        "--to",
        "proposed",
    ]);
    aep(&[
        "plan",
        "artifact",
        "move",
        "story:deliver",
        "--to",
        "active",
    ]);
    cmd(&repo, "git", &["add", "."], &[]);
    cmd(
        &repo,
        "git",
        &["commit", "-m", "fixture before requested behavior"],
        &[],
    );
    let remote = root.join(format!("remotes/repo{index}.git"));
    cmd(
        root,
        "git",
        &["init", "--bare", remote.to_str().unwrap()],
        &[],
    );
    cmd(
        &repo,
        "git",
        &["remote", "add", "origin", remote.to_str().unwrap()],
        &[],
    );
    cmd(&repo, "git", &["push", "origin", "HEAD:main"], &[]);
    repo
}

/// Two workspaces, each with one registered repository and one Running goal.
async fn fixture() -> Fixture {
    fixture_with(2).await
}

/// `workspaces` workspaces, each with one registered repository and one Running goal.
async fn fixture_with(workspaces: usize) -> Fixture {
    let scratch =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.scratch/goal-acceptance-pass2");
    std::fs::create_dir_all(&scratch).unwrap();
    let root: PathBuf = tempfile::tempdir_in(scratch).unwrap().keep();
    std::fs::create_dir(root.join("repos")).unwrap();
    std::fs::create_dir(root.join("remotes")).unwrap();
    let config = RuntimeConfig {
        environment: vec![
            (
                "XDG_STATE_HOME".into(),
                root.join("state").display().to_string(),
            ),
            (
                "XDG_CONFIG_HOME".into(),
                root.join("config").display().to_string(),
            ),
            (
                "CARGO_TARGET_DIR".into(),
                root.join("cargo-target").display().to_string(),
            ),
        ],
        commit_command: vec!["git".into(), "commit".into(), "-m".into()],
        poll_interval: Duration::from_millis(200),
        ..RuntimeConfig::default()
    };
    let profile = root.join("profile.toml");
    std::fs::write(&profile, "version = 1\nname = 'fleet-fixture'\nexpire_after_seconds = 604800\nprotect_workspace_root = false\n").unwrap();
    cmd(
        &root,
        "worktree",
        &[
            "activate",
            "--profile",
            profile.to_str().unwrap(),
            "--workspace",
            root.join("repos").to_str().unwrap(),
        ],
        &config.environment,
    );
    let mut host = Store::open(root.join("host.sqlite3")).await.unwrap();
    let mut goals = Vec::new();
    for index in 0..workspaces {
        let repo = repository(&root, index, &config);
        let workspace = host
            .register_workspace(&repo, &format!("workspace {index}"))
            .await
            .unwrap()["published"][0]["payload"]["workspace_id"]
            .clone();
        let registered = host
            .query("RepositoryRegistrationList")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["workspace_id"] == workspace)
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(registered.len(), 1, "{registered:?}");
        host.execute(
            "ConfigureRepository",
            json!({"repository_id":registered[0]["repository_id"],"base_branch":"main","test_command":"cargo test --quiet","publish_command":PUBLISH}),
            Actor::Operator,
        )
        .await
        .unwrap();
        let goal = host
            .execute(
                "CreateGoal",
                json!({"workspace_id":workspace,"objective":format!("Return 42 in workspace {index}"),"acceptance":"requested_answer passes on each reviewed merged target","max_workers":3,"max_attempts":2,"max_minutes":1,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":true}),
                Actor::Operator,
            )
            .await
            .unwrap()["published"][0]["payload"]["goal_id"]
            .clone();
        let started = host
            .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
            .await
            .unwrap();
        assert_eq!(started["outcome"], "applied", "{started}");
        goals.push(goal);
    }
    Fixture {
        store: Arc::new(tokio::sync::Mutex::new(host)),
        config,
        goals,
    }
}

/// The operator's edit and the goal it edited, recorded by the model double.
#[derive(Clone)]
struct Edit {
    reviewed: Value,
    edited: Value,
    before: Value,
    outcome: Value,
}

/// The scripted roles of tests/fleet.rs: plan the existing story, write the answer, approve.
/// On the first goal review, before it answers, an operator edits the *other* goal, the way
/// an operator edits one goal while the console shows another goal under acceptance.
struct EditOtherGoalDuringReview {
    store: SharedStore,
    goals: Vec<Value>,
    steps: Mutex<BTreeMap<String, usize>>,
    edit: Mutex<Option<Edit>>,
}
impl AgentModel for EditOtherGoalDuringReview {
    fn respond(&self, request: &ModelRequest) -> Result<Value> {
        match request.role.as_str() {
            "planner" => Ok(
                json!({"action":"finish","stories":["story:deliver"],"summary":"Existing story describes the goal"}),
            ),
            "critic" | "reviewer" | "goal_reviewer" => {
                let mut edit = self.edit.lock().unwrap();
                if request.role == "goal_reviewer" && edit.is_none() {
                    let prompt: Value = serde_json::from_str(&request.prompt).unwrap();
                    let reviewed = prompt["goal"]["goal_id"].clone();
                    let edited = self
                        .goals
                        .iter()
                        .find(|goal| **goal != reviewed)
                        .unwrap()
                        .clone();
                    let mut store = self.store.blocking_lock();
                    let before = store
                        .query("GoalList")
                        .unwrap()
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|goal| goal["goal_id"] == edited)
                        .unwrap()
                        .clone();
                    let body = json!({"goal_id":edited,"objective":"Return 42 and document it","acceptance":"requested_answer passes and the README names the answer","max_workers":3,"max_attempts":2,"max_minutes":1,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":true});
                    let outcome = tokio::runtime::Handle::current()
                        .block_on(store.execute("UpdateGoal", body, Actor::Operator))
                        .unwrap();
                    *edit = Some(Edit {
                        reviewed,
                        edited,
                        before,
                        outcome,
                    });
                }
                Ok(
                    json!({"approved":true,"reason":"Observed diff and actual checks satisfy the requested behavior"}),
                )
            }
            "implementor" => {
                let mut steps = self.steps.lock().unwrap();
                let count = steps.entry(request.execution_context.clone()).or_default();
                *count += 1;
                Ok(if *count == 1 {
                    json!({"action":"write","path":"src/lib.rs","contents":"pub fn answer() -> u32 { 42 }\n"})
                } else {
                    json!({"action":"finish","summary":"Requested behavior implemented"})
                })
            }
            other => anyhow::bail!("unexpected model role {other}"),
        }
    }
}

fn model(fixture: &Fixture) -> Arc<EditOtherGoalDuringReview> {
    Arc::new(EditOtherGoalDuringReview {
        store: fixture.store.clone(),
        goals: fixture.goals.clone(),
        steps: Mutex::new(BTreeMap::new()),
        edit: Mutex::new(None),
    })
}

fn row(rows: &Value, key: &str, id: &Value) -> Value {
    rows.as_array()
        .unwrap()
        .iter()
        .find(|row| &row[key] == id)
        .unwrap()
        .clone()
}

/// The edit landed while the edited goal was Running at revision 1 with its only assignment
/// Merged at revision 1, and the reviewed goal comes first in `GoalList`: the edited goal's
/// acceptance runs later in the same `satisfy_goals` pass, on the goal row read before the edit.
async fn assert_edit_landed_before_the_edited_goal_was_accepted(store: &SharedStore, edit: &Edit) {
    assert_eq!(edit.outcome["outcome"], "applied", "{}", edit.outcome);
    assert_eq!(edit.before["state"], "Running", "{}", edit.before);
    assert_eq!(edit.before["revision"], 1, "{}", edit.before);
    let store = store.lock().await;
    let goals = store.query("GoalList").unwrap();
    let order = goals
        .as_array()
        .unwrap()
        .iter()
        .map(|goal| goal["goal_id"].clone())
        .collect::<Vec<_>>();
    let position = |id: &Value| order.iter().position(|goal| goal == id).unwrap();
    assert!(
        position(&edit.reviewed) < position(&edit.edited),
        "precondition: the reviewed goal is accepted first: {goals}"
    );
    let assignments = store.query("AssignmentList").unwrap();
    let edited = assignments
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| a["goal_id"] == edit.edited)
        .collect::<Vec<_>>();
    assert_eq!(edited.len(), 1, "{assignments}");
    assert_eq!(edited[0]["state"], "Merged", "{assignments}");
    assert_eq!(edited[0]["goal_revision"], 1, "{assignments}");
    let reviewed = row(&goals, "goal_id", &edit.reviewed);
    assert_eq!(reviewed["state"], "Satisfied", "{reviewed}");
}

/// An operator edits goal B while goal A's acceptance runs. A is accepted. B's acceptance then
/// starts from the goal row `satisfy_goals` read before A's acceptance (revision 1), and its
/// first store write, the `running` acceptance record (`acceptance_progress`, fleet.rs:2013),
/// refuses because the goal is at revision 2. The `?` there fails `satisfy_goals`, so the whole
/// fleet tick fails, where the correction says an acceptance ended by a revision change is an
/// interruption and the edited goal is planned again.
#[tokio::test]
async fn editing_another_goal_during_acceptance_does_not_fail_the_fleet_tick() {
    let fixture = fixture().await;
    let model = model(&fixture);
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config.clone(),
        model.clone(),
    );
    let planning = supervisor.tick().await.unwrap();
    assert_eq!(planning.queued, 2, "{planning:?}");
    let report = supervisor.fleet_tick().await;
    let edit = model
        .edit
        .lock()
        .unwrap()
        .clone()
        .expect("a goal reached its final review");
    assert_edit_landed_before_the_edited_goal_was_accepted(&fixture.store, &edit).await;
    let report = report.unwrap_or_else(|error| {
        panic!(
            "an operator edit of one goal during another goal's acceptance failed the whole \
             fleet tick: {error:#}"
        )
    });
    assert_eq!(report.satisfied, 1, "{report:?}");
    let goals = fixture.store.lock().await.query("GoalList").unwrap();
    let edited = row(&goals, "goal_id", &edit.edited);
    assert_eq!(edited["state"], "Running", "{edited}");
    assert_eq!(edited["revision"], 2, "{edited}");
    assert_eq!(edited["satisfaction_receipt"], "", "{edited}");
    supervisor.tick().await.unwrap();
    let goals = fixture.store.lock().await.query("GoalList").unwrap();
    let edited = row(&goals, "goal_id", &edit.edited);
    assert_eq!(edited["planning_revision"], 2, "{edited}");
}

/// The same edit, under the production loop (`Supervisor::run`, which `serve_with_runtime`
/// drives). The service must keep running and plan the edited goal's new revision; a fleet tick
/// error ends `run`, and the console then reports that autonomous processing stopped until the
/// service is restarted.
#[tokio::test]
async fn editing_another_goal_during_acceptance_keeps_the_service_running() {
    let fixture = fixture().await;
    let model = model(&fixture);
    let service = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config.clone(),
        model.clone(),
    );
    let shutdown = tokio_util::sync::CancellationToken::new();
    let run = service.run(shutdown.clone());
    tokio::pin!(run);
    let deadline = tokio::time::sleep(Duration::from_secs(600));
    tokio::pin!(deadline);
    loop {
        tokio::select! {
            result = &mut run => {
                let edit = model.edit.lock().unwrap().clone();
                if let Some(edit) = &edit {
                    assert_edit_landed_before_the_edited_goal_was_accepted(&fixture.store, edit)
                        .await;
                }
                panic!(
                    "the service stopped before shutdown, after an operator edit of one goal \
                     during another goal's acceptance (edit {}): {result:?}",
                    edit.map(|edit| edit.outcome).unwrap_or_default()
                );
            }
            _ = tokio::time::sleep(Duration::from_millis(200)) => {
                // `run` is not polled while this arm runs, so never wait for the store here.
                let edited = model.edit.lock().unwrap().clone().map(|edit| edit.edited);
                if let (Some(edited), Ok(store)) = (edited, fixture.store.try_lock()) {
                    let goals = store.query("GoalList").unwrap();
                    drop(store);
                    let goal = row(&goals, "goal_id", &edited);
                    if goal["revision"] == 2 && goal["planning_revision"] == 2 {
                        break;
                    }
                }
            }
            _ = &mut deadline => panic!("the edited goal was not planned again within 600 s"),
        }
    }
    shutdown.cancel();
    let _ = tokio::time::timeout(Duration::from_secs(300), run).await;
}

/// The scripted roles of tests/fleet.rs. During the first goal review the reviewed
/// repository's `origin` becomes unreachable, the way a remote does during a short network
/// outage; the test makes it reachable again afterwards.
struct OriginUnreachableDuringReview {
    steps: Mutex<BTreeMap<String, usize>>,
    reviews: Mutex<usize>,
    broken: Mutex<Option<(PathBuf, String)>>,
}
impl AgentModel for OriginUnreachableDuringReview {
    fn respond(&self, request: &ModelRequest) -> Result<Value> {
        match request.role.as_str() {
            "planner" => Ok(
                json!({"action":"finish","stories":["story:deliver"],"summary":"Existing story describes the goal"}),
            ),
            "critic" | "reviewer" | "goal_reviewer" => {
                if request.role == "goal_reviewer" {
                    *self.reviews.lock().unwrap() += 1;
                }
                Ok(
                    json!({"approved":true,"reason":"Observed diff and actual checks satisfy the requested behavior"}),
                )
            }
            "implementor" => {
                let mut steps = self.steps.lock().unwrap();
                let count = steps.entry(request.execution_context.clone()).or_default();
                *count += 1;
                Ok(if *count == 1 {
                    json!({"action":"write","path":"src/lib.rs","contents":"pub fn answer() -> u32 { 42 }\n"})
                } else {
                    json!({"action":"finish","summary":"Requested behavior implemented"})
                })
            }
            other => anyhow::bail!("unexpected model role {other}"),
        }
    }
    fn respond_in(
        &self,
        request: &ModelRequest,
        environment: &control_plane_runtime::ModelEnvironment,
    ) -> Result<Value> {
        if request.role == "goal_reviewer" {
            let mut broken = self.broken.lock().unwrap();
            if broken.is_none() {
                let path = environment.workspace.clone();
                let url = cmd(&path, "git", &["remote", "get-url", "origin"], &[])
                    .trim()
                    .to_owned();
                cmd(
                    &path,
                    "git",
                    &["remote", "set-url", "origin", &format!("{url}-unreachable")],
                    &[],
                );
                *broken = Some((path, url));
            }
        }
        self.respond(request)
    }
}

/// The reviewer approves, and the post-review observation of the target (`git ls-remote`,
/// fleet.rs:2128) fails because `origin` is briefly unreachable. Nothing acceptance depends on
/// changes: not the goal, its repository, its directories or the target. The correction records
/// interruptions without the latching `failed` status and names three causes that still latch
/// (a review rejection, a deadline, a moved target); a failed observation is none of them. Once
/// `origin` answers again, acceptance must run again and accept the unchanged goal.
#[tokio::test]
async fn transient_origin_failure_after_goal_review_does_not_latch_acceptance() {
    let fixture = fixture_with(1).await;
    let model = Arc::new(OriginUnreachableDuringReview {
        steps: Mutex::new(BTreeMap::new()),
        reviews: Mutex::new(0),
        broken: Mutex::new(None),
    });
    let supervisor = Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        fixture.config.clone(),
        model.clone(),
    );
    supervisor.tick().await.unwrap();
    supervisor.fleet_tick().await.unwrap();
    let (path, url) = model
        .broken
        .lock()
        .unwrap()
        .clone()
        .expect("goal acceptance reached its review");
    let goal = fixture.store.lock().await.query("GoalList").unwrap()[0].clone();
    assert_eq!(
        goal["state"], "Running",
        "precondition: not satisfied while origin is unreachable: {goal}"
    );
    assert_eq!(*model.reviews.lock().unwrap(), 1);
    cmd(&path, "git", &["remote", "set-url", "origin", &url], &[]);
    for _ in 0..2 {
        supervisor.tick().await.unwrap();
        supervisor.fleet_tick().await.unwrap();
    }
    let goal = fixture.store.lock().await.query("GoalList").unwrap()[0].clone();
    let acceptance = {
        let store = fixture.store.lock().await;
        store
            .activity_history(goal["goal_id"].as_str().unwrap())
            .unwrap()["acceptance"]
            .clone()
    };
    assert_eq!(
        goal["state"],
        "Satisfied",
        "origin answers again and nothing changed, yet acceptance never ran again ({} goal \
         reviews); acceptance {}",
        model.reviews.lock().unwrap(),
        json!({"status":acceptance["status"],"goal_revision":acceptance["goal_revision"]})
    );
}
