//! Adversarial cases for story:publication-exit, wave 4 pass 2, driven through the real fleet.
//!
//! The fixture is the one `tests/publication_exit_attack.rs` builds (after `tests/fleet.rs`): a
//! bare remote, a managed worktree workspace, an active AEP story and a publish command that
//! exits without pushing (`git --version`), kept under its own scratch directory.
use anyhow::Result;
use control_plane_core::{Actor, Store};
use control_plane_runtime::{
    AgentModel, ModelRequest, PUBLICATION_GRACE, RuntimeConfig, SharedStore, Supervisor,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::Notify;

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

async fn rows(store: &SharedStore, view: &str) -> Vec<Value> {
    store
        .lock()
        .await
        .query(view)
        .unwrap()
        .as_array()
        .unwrap()
        .clone()
}

struct Fixture {
    root: PathBuf,
    store: SharedStore,
    config: RuntimeConfig,
    goal: Value,
}

/// One repository with a bare remote and one started goal, whose publish command exits without
/// pushing (`git --version`).
async fn fixture() -> Fixture {
    let scratch =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.scratch/publication-exit-pass2-attack");
    std::fs::create_dir_all(&scratch).unwrap();
    let root = tempfile::tempdir_in(scratch).unwrap().keep();
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
        ..RuntimeConfig::default()
    };
    let profile = root.join("profile.toml");
    std::fs::write(&profile,"version = 1\nname = 'publication-exit-pass2-attack'\nexpire_after_seconds = 604800\nprotect_workspace_root = false\n").unwrap();
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
    let repo = root.join("repos/repo0");
    std::fs::create_dir_all(repo.join("src")).unwrap();
    std::fs::create_dir(repo.join("tests")).unwrap();
    std::fs::create_dir_all(repo.join("ess/domains")).unwrap();
    std::fs::write(
        repo.join("Cargo.toml"),
        "[package]\nname = 'fleet_fixture_0'\nversion = '0.1.0'\nedition = '2024'\n[workspace]\n",
    )
    .unwrap();
    std::fs::write(repo.join("src/lib.rs"), "pub fn answer() -> u32 { 0 }\n").unwrap();
    std::fs::write(
        repo.join("tests/acceptance.rs"),
        "#[test]\nfn requested_answer() { assert_eq!(fleet_fixture_0::answer(), 42); }\n",
    )
    .unwrap();
    std::fs::write(repo.join(".gitignore"), "target\nCargo.lock\n").unwrap();
    std::fs::write(
        repo.join("ess/system.yaml"),
        "format: ess/22\nsystem: demo\nversion: v1\ndomains: [demo.item]\n",
    )
    .unwrap();
    std::fs::write(repo.join("ess/domains/item.yaml"),"domain: demo.item\nentities:\n  - name: demo.item.Item\n    identity: {name: item_id, type: Uuid}\n    fields: []\n    lifecycle:\n      initial: Present\n      states: [Present]\n      terminal: [Present]\n").unwrap();
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
    let remote = root.join("remotes/repo0.git");
    cmd(
        &root,
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
    let mut host = Store::open(root.join("host.sqlite3")).await.unwrap();
    let workspace = host
        .register_workspace(&repo, "publication exit pass 2")
        .await
        .unwrap()["published"][0]["payload"]["workspace_id"]
        .clone();
    let repository = host.query("RepositoryRegistrationList").unwrap()[0].clone();
    host.execute("ConfigureRepository",json!({"repository_id":repository["repository_id"],"base_branch":"main","test_command":"cargo test --quiet","publish_command":"git --version"}),Actor::Operator).await.unwrap();
    let goal=host.execute("CreateGoal",json!({"workspace_id":workspace,"objective":"Return 42 in every registered repository","acceptance":"requested_answer passes on each reviewed merged target","max_workers":3,"max_attempts":2,"max_minutes":1,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":true}),Actor::Operator).await.unwrap()["published"][0]["payload"]["goal_id"].clone();
    host.execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    Fixture {
        root,
        store: Arc::new(tokio::sync::Mutex::new(host)),
        config,
        goal,
    }
}

/// The fleet tests' scripted model; a second implementation run writes the same behaviour in
/// other words, as a fresh model run does, so its candidate is a new commit.
struct Scripted {
    steps: Mutex<BTreeMap<String, usize>>,
}
impl Scripted {
    fn new() -> Self {
        Self {
            steps: Mutex::new(BTreeMap::new()),
        }
    }
}
impl AgentModel for Scripted {
    fn respond(&self, request: &ModelRequest) -> Result<Value> {
        match request.role.as_str() {
            "planner" => Ok(
                json!({"action":"finish","stories":["story:deliver"],"summary":"Existing story describes the goal"}),
            ),
            "critic" | "reviewer" | "goal_reviewer" => Ok(
                json!({"approved":true,"reason":"Observed diff and actual checks satisfy the requested behavior"}),
            ),
            "implementor" => {
                let mut steps = self.steps.lock().unwrap();
                let runs = steps.len();
                let count = steps.entry(request.execution_context.clone()).or_default();
                *count += 1;
                let contents = if runs == 0 || (runs == 1 && *count > 1) {
                    "pub fn answer() -> u32 { 42 }\n"
                } else {
                    "pub fn answer() -> u32 { 40 + 2 }\n"
                };
                Ok(if *count == 1 {
                    json!({"action":"write","path":"src/lib.rs","contents":contents})
                } else {
                    json!({"action":"finish","summary":"Requested behavior implemented"})
                })
            }
            other => anyhow::bail!("unexpected model role {other}"),
        }
    }
}

fn supervisor(fixture: &Fixture, grace: Duration) -> Supervisor {
    supervisor_on(fixture.store.clone(), &fixture.config, grace)
}

fn supervisor_on(store: SharedStore, config: &RuntimeConfig, grace: Duration) -> Supervisor {
    Supervisor::new(
        store,
        Arc::new(Notify::new()),
        RuntimeConfig {
            publication_grace: grace,
            ..config.clone()
        },
        Arc::new(Scripted::new()),
    )
}

/// Plan and deliver once: the publisher exits without a merge, so the intent is Uncertain and
/// the assignment Blocked.
async fn unresolved(fixture: &Fixture, supervisor: &Supervisor) -> Value {
    supervisor.tick().await.unwrap();
    supervisor.fleet_tick().await.unwrap();
    let intents = rows(&fixture.store, "PublicationIntentList").await;
    assert_eq!(intents.len(), 1, "{intents:?}");
    assert_eq!(intents[0]["state"], "Uncertain", "{intents:?}");
    let assignment = rows(&fixture.store, "AssignmentList").await[0].clone();
    assert_eq!(assignment["state"], "Blocked", "{assignment}");
    intents[0].clone()
}

/// Another change lands on main, built on `base`; the candidate is not part of it.
fn land_unrelated_change(fixture: &Fixture, base: &str) -> String {
    let repo = fixture.root.join("repos/repo0");
    let other = cmd(
        &repo,
        "git",
        &[
            "commit-tree",
            &format!("{base}^{{tree}}"),
            "-p",
            base,
            "-m",
            "an unrelated change",
        ],
        &[],
    )
    .trim()
    .to_owned();
    cmd(
        &repo,
        "git",
        &["push", "origin", &format!("{other}:refs/heads/main")],
        &[],
    );
    other
}

/// The story is "a publication the remote never received can be closed and retried", and
/// `closed_publication_is_retried_with_a_new_intent` shows the retry for a close on an unmoved
/// target. `target_moved_past_uncertain_head_closes_intent` pauses the goal before its close, so
/// nothing shows what follows the other close rule. Here the goal keeps running: the target
/// moves past the head observed at Uncertain and the intent closes. The assignment's attempt
/// started on the old base, and no command gives it a new one, so a retry could never publish.
/// The coordinator chose option (a) of the pass-2 finding: the assignment is blocked once, with
/// a reason that says the target moved since its attempt started and that it must be cancelled
/// or re-planned, spends no attempt and opens no new intent. A retry on a fresh base is a later
/// story.
#[tokio::test]
async fn publication_closed_after_its_target_moved_blocks_once_without_spending_an_attempt() {
    let fixture = fixture().await;
    let supervisor = supervisor(&fixture, PUBLICATION_GRACE);
    let first = unresolved(&fixture, &supervisor).await;
    supervisor.fleet_tick().await.unwrap();
    assert_eq!(
        rows(&fixture.store, "PublicationIntentList").await[0]["state"],
        "Uncertain"
    );
    let attempt = rows(&fixture.store, "AssignmentList").await[0]["attempt"].clone();
    let moved = land_unrelated_change(&fixture, first["expected_base"].as_str().unwrap());
    supervisor.fleet_tick().await.unwrap();
    let intents = rows(&fixture.store, "PublicationIntentList").await;
    let assignment = rows(&fixture.store, "AssignmentList").await[0].clone();
    let closed = intents
        .iter()
        .find(|i| i["publication_id"] == first["publication_id"])
        .unwrap();
    assert_eq!(closed["state"], "NotPublished", "{intents:?}");
    assert!(
        closed["reason"]
            .as_str()
            .is_some_and(|reason| reason.contains(&moved)),
        "{intents:?}"
    );
    let newest = |store: &Store| {
        let goal = store.query("GoalList").unwrap()[0]["goal_id"].clone();
        store.activity_history(goal.as_str().unwrap()).unwrap()["fleet"]
            [assignment["assignment_id"].as_str().unwrap()]
        .clone()
    };
    let blocker = newest(&*fixture.store.lock().await);
    let explained = |blocker: &Value| {
        blocker["action"] == "blocked"
            && blocker["detail"]["reason"].as_str().is_some_and(|reason| {
                reason.contains(&moved)
                    && reason.contains("since this attempt started")
                    && reason.contains("cancel or re-plan")
            })
    };
    assert_eq!(
        (
            intents.len(),
            assignment["state"].as_str(),
            &assignment["attempt"]
        ),
        (1, Some("Blocked"), &attempt),
        "the intent closed because main moved to {moved}; the assignment must be blocked at \
         attempt {attempt} with no second intent: it is {} at attempt {} ({}), intents {intents:?}",
        assignment["state"],
        assignment["attempt"],
        blocker["detail"]["reason"]
    );
    assert!(explained(&blocker), "{blocker}");

    // Ten more ticks over the unchanged assignment append at most one progress decision.
    let database = fixture.root.join("host.sqlite3");
    drop(supervisor);
    let Fixture { store, config, .. } = fixture;
    drop(store);
    let reopened = Store::open(&database).await.unwrap();
    let before = reopened.replayed_progress();
    let store: SharedStore = Arc::new(tokio::sync::Mutex::new(reopened));
    let supervisor = supervisor_on(store.clone(), &config, PUBLICATION_GRACE);
    for _ in 0..10 {
        supervisor.fleet_tick().await.unwrap();
    }
    let intents = rows(&store, "PublicationIntentList").await;
    let now = rows(&store, "AssignmentList").await[0].clone();
    let blocker = newest(&*store.lock().await);
    assert_eq!(
        (intents.len(), now["state"].as_str(), &now["attempt"]),
        (1, Some("Blocked"), &attempt),
        "{now} {intents:?}"
    );
    assert!(explained(&blocker), "{blocker}");
    drop(supervisor);
    drop(store);
    let mut reopened = Store::open(&database).await.unwrap();
    let after = reopened.replayed_progress();
    assert!(
        after - before <= 1,
        "ten fleet ticks over the blocked assignment appended {} progress decisions; newest: {}",
        after - before,
        blocker["detail"]["reason"]
    );

    // The closed intent no longer holds the assignment, so it can be cancelled.
    let cancelled = reopened
        .execute(
            "CancelAssignment",
            json!({"assignment_id":assignment["assignment_id"]}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(cancelled["outcome"], "applied", "{cancelled}");
}

/// The grace period starts when the intent became Uncertain, and the commit says that start is
/// carried in the intent's blocker records "so they survive a restart". A process restarted
/// inside the grace window closes the intent at the original deadline; it does not start the
/// grace period again at its first observation.
#[tokio::test]
async fn grace_period_counts_from_before_a_restart() {
    let grace = Duration::from_secs(4);
    let fixture = fixture().await;
    let supervisor = supervisor(&fixture, grace);
    let first = unresolved(&fixture, &supervisor).await;
    let database = fixture.root.join("host.sqlite3");
    drop(supervisor);
    let Fixture {
        store,
        config,
        goal,
        ..
    } = fixture;
    drop(store);
    let store: SharedStore = Arc::new(tokio::sync::Mutex::new(
        Store::open(&database).await.unwrap(),
    ));
    store
        .lock()
        .await
        .execute("PauseGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    let supervisor = supervisor_on(store.clone(), &config, grace);
    tokio::time::sleep(Duration::from_secs(2)).await;
    // The first observation after the restart.
    let observed = Instant::now();
    supervisor.fleet_tick().await.unwrap();
    // Had the grace period started again at that observation, it would not end before
    // `observed + grace`.
    tokio::time::sleep(Duration::from_millis(2500).saturating_sub(observed.elapsed())).await;
    assert!(
        observed.elapsed() < Duration::from_secs(3),
        "the first observation after the restart took too long for this case to tell"
    );
    supervisor.fleet_tick().await.unwrap();
    let intent = rows(&store, "PublicationIntentList")
        .await
        .into_iter()
        .find(|i| i["publication_id"] == first["publication_id"])
        .unwrap();
    assert_eq!(intent["state"], "NotPublished", "{intent}");
    assert!(
        intent["reason"]
            .as_str()
            .is_some_and(|reason| reason.contains("4 s passed")),
        "{intent}"
    );
}

/// A remote that cannot be reached is the story's own example of a failed observation. Its
/// diagnostics need not repeat byte for byte: curl reports how long the attempt took ("Failed to
/// connect to <host> port 443 after <n> ms"). The fixture's remote fails that way through a
/// `remote.origin.uploadpack` that prints such a line with a changing number and exits 128. The
/// intent and its assignment are unchanged, so ten ticks still append at most one progress
/// decision (`unchanged_unresolved_publication_appends_once`).
#[tokio::test]
async fn unreachable_remote_with_changing_diagnostics_appends_once() {
    let fixture = fixture().await;
    let supervisor = supervisor(&fixture, PUBLICATION_GRACE);
    unresolved(&fixture, &supervisor).await;
    let failing = fixture.root.join("unreachable-upload-pack");
    std::fs::write(
        &failing,
        "#!/bin/sh\necho \"fatal: unable to access 'https://git.example.invalid/repo0.git/': Failed \
         to connect to git.example.invalid port 443 after $(date +%N) ms: Could not connect to \
         server\" >&2\nexit 128\n",
    )
    .unwrap();
    cmd(
        &fixture.root,
        "chmod",
        &["755", failing.to_str().unwrap()],
        &[],
    );
    cmd(
        &fixture.root.join("repos/repo0"),
        "git",
        &[
            "config",
            "remote.origin.uploadpack",
            failing.to_str().unwrap(),
        ],
        &[],
    );
    let database = fixture.root.join("host.sqlite3");
    drop(supervisor);
    let Fixture { store, config, .. } = fixture;
    drop(store);
    let reopened = Store::open(&database).await.unwrap();
    let before = reopened.replayed_progress();
    let store: SharedStore = Arc::new(tokio::sync::Mutex::new(reopened));
    let supervisor = supervisor_on(store.clone(), &config, PUBLICATION_GRACE);
    for _ in 0..10 {
        supervisor.fleet_tick().await.unwrap();
    }
    assert_eq!(
        rows(&store, "PublicationIntentList").await[0]["state"],
        "Uncertain"
    );
    let assignment = rows(&store, "AssignmentList").await[0].clone();
    assert_eq!(assignment["state"], "Blocked", "{assignment}");
    let newest = {
        let store = store.lock().await;
        let goal = store.query("GoalList").unwrap()[0]["goal_id"].clone();
        store.activity_history(goal.as_str().unwrap()).unwrap()["fleet"]
            [assignment["assignment_id"].as_str().unwrap()]
        .clone()
    };
    assert!(
        newest["detail"]["reason"]
            .as_str()
            .is_some_and(|reason| reason.starts_with("Publication observation unavailable")),
        "{newest}"
    );
    drop(supervisor);
    drop(store);
    let after = Store::open(&database).await.unwrap().replayed_progress();
    assert!(
        after - before <= 1,
        "ten fleet ticks over one unresolved publication whose remote cannot be reached appended \
         {} progress decisions; newest: {}",
        after - before,
        newest["detail"]["reason"]
    );
}

/// README: once a publication is recorded as not published, "the assignment may be attempted
/// again". An operator whose publish command never merged corrects it (`ConfigureRepository`)
/// and resumes the goal. The closed assignment's evidence was admitted under the old
/// configuration, so the store refuses its repair ("repository configuration changed; assignment
/// evidence is stale"), and the fleet's slot rule does not know that refusal. The assignment is
/// unchanged from tick to tick, so ten ticks must not append progress decisions on every tick.
#[tokio::test]
async fn reconfigured_repository_does_not_append_on_every_tick() {
    let fixture = fixture().await;
    let supervisor = supervisor(&fixture, Duration::ZERO);
    let first = unresolved(&fixture, &supervisor).await;
    let goal = fixture.goal.clone();
    let operator = |command: &'static str, body: Value| {
        let store = fixture.store.clone();
        async move {
            store
                .lock()
                .await
                .execute(command, body, Actor::Operator)
                .await
                .unwrap()
        }
    };
    operator("PauseGoal", json!({"goal_id":goal})).await;
    supervisor.fleet_tick().await.unwrap();
    let intent = rows(&fixture.store, "PublicationIntentList").await[0].clone();
    assert_eq!(intent["publication_id"], first["publication_id"]);
    assert_eq!(intent["state"], "NotPublished", "{intent}");
    let repository = rows(&fixture.store, "RepositoryRegistrationList").await[0].clone();
    operator("ConfigureRepository",json!({"repository_id":repository["repository_id"],"base_branch":"main","test_command":"cargo test --quiet","publish_command":"git push origin {candidate}:refs/heads/{target}"})).await;
    operator("StartGoal", json!({"goal_id":goal})).await;

    let database = fixture.root.join("host.sqlite3");
    drop(supervisor);
    let Fixture { store, config, .. } = fixture;
    drop(store);
    let reopened = Store::open(&database).await.unwrap();
    let before = reopened.replayed_progress();
    let store: SharedStore = Arc::new(tokio::sync::Mutex::new(reopened));
    let supervisor = supervisor_on(store.clone(), &config, Duration::ZERO);
    for _ in 0..10 {
        supervisor.fleet_tick().await.unwrap();
    }
    let assignment = rows(&store, "AssignmentList").await[0].clone();
    drop(supervisor);
    drop(store);
    let after = Store::open(&database).await.unwrap().replayed_progress();
    assert!(
        after - before <= 2,
        "ten fleet ticks over an unchanged {} assignment (attempt {}) appended {} progress \
         decisions; its reason: {}",
        assignment["state"],
        assignment["attempt"],
        after - before,
        assignment["reason"]
    );
}
