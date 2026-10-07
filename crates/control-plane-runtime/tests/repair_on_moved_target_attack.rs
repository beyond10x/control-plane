//! Adversarial cases for story:repair-on-moved-target, pass 1, driven through the real fleet.
//!
//! The fixture follows `tests/publication_exit_pass2_attack.rs`: one repository with a bare
//! remote, a managed worktree workspace, an active AEP story and a publish command that exits
//! without pushing (`git --version`), under its own scratch directory. The repository's checks
//! run `true`, so no case compiles the fixture crate; nothing here depends on their output.
//!
//! The unit's own cases move the target by a file no candidate touches (`NOTES.md`) or by a
//! commit with the base's own tree, on a worktree the previous attempt left clean. These cases
//! move it in the ways those leave out: a change to the candidate's own line, a change to a file
//! a failed attempt left uncommitted, a rewind, a squash of the candidate itself, and a commit
//! command that fails once.
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
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::sync::Notify;

/// The fixture library before any implementation. Its last line is far enough from the first
/// that a change to each merges cleanly once both are committed.
const LIBRARY: &str = "pub fn answer() -> u32 { 0 }\n\n// one\n// two\n// three\n// end\n";
/// What every implementation run writes.
const IMPLEMENTED: &str = "pub fn answer() -> u32 { 42 }\n\n// one\n// two\n// three\n// end\n";
/// The message `integrate` commits the target's current head with (crates/control-plane-runtime/src/fleet.rs).
const INTEGRATION: &str = "Integrate the target's current head";

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

/// Whether `git args` succeeds in `cwd`.
fn git_ok(cwd: &Path, args: &[&str]) -> bool {
    Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap()
        .status
        .success()
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
}

impl Fixture {
    fn primary(&self) -> PathBuf {
        self.root.join("repos/repo0")
    }
}

/// How a case shapes its fixture.
struct Setup {
    max_attempts: i64,
    /// The commit command fails once, for the first commit `integrate` makes, and then commits
    /// as `git commit -m` does: a hook or a credential that is unavailable for one commit.
    refuse_first_integration_commit: bool,
}

/// One repository with a bare remote and one started goal, whose publish command exits without
/// pushing (`git --version`) and whose checks run `true`.
async fn fixture(setup: Setup) -> Fixture {
    let scratch =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.scratch/repair-on-moved-target-attack");
    std::fs::create_dir_all(&scratch).unwrap();
    let root = tempfile::tempdir_in(scratch).unwrap().keep();
    std::fs::create_dir(root.join("repos")).unwrap();
    std::fs::create_dir(root.join("remotes")).unwrap();
    let commit_command = if setup.refuse_first_integration_commit {
        let marker = root.join("integration-commit-refused");
        vec![
            "sh".into(),
            "-c".into(),
            format!(
                "if [ \"$0\" = \"{INTEGRATION}\" ] && [ ! -e '{}' ]; then : > '{}'; \
                 echo 'commit refused once' >&2; exit 1; fi; exec git commit -m \"$0\"",
                marker.display(),
                marker.display()
            ),
        ]
    } else {
        vec!["git".into(), "commit".into(), "-m".into()]
    };
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
        commit_command,
        ..RuntimeConfig::default()
    };
    let profile = root.join("profile.toml");
    std::fs::write(&profile,"version = 1\nname = 'repair-on-moved-target-attack'\nexpire_after_seconds = 604800\nprotect_workspace_root = false\n").unwrap();
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
    std::fs::create_dir_all(repo.join("ess/domains")).unwrap();
    std::fs::write(repo.join("src/lib.rs"), LIBRARY).unwrap();
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
        .register_workspace(&repo, "repair on moved target attack")
        .await
        .unwrap()["published"][0]["payload"]["workspace_id"]
        .clone();
    let repository = host.query("RepositoryRegistrationList").unwrap()[0].clone();
    host.execute("ConfigureRepository",json!({"repository_id":repository["repository_id"],"base_branch":"main","test_command":"true","publish_command":"git --version"}),Actor::Operator).await.unwrap();
    let goal=host.execute("CreateGoal",json!({"workspace_id":workspace,"objective":"Return 42 in every registered repository","acceptance":"requested_answer passes on each reviewed merged target","max_workers":3,"max_attempts":setup.max_attempts,"max_minutes":1,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":true}),Actor::Operator).await.unwrap()["published"][0]["payload"]["goal_id"].clone();
    host.execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    Fixture {
        root,
        store: Arc::new(tokio::sync::Mutex::new(host)),
        config,
    }
}

/// The fleet tests' scripted model. Every implementation run writes [`IMPLEMENTED`] and then
/// finishes; with `fail_first_run`, the first run fails after its write instead, as a model
/// provider outage does, so its write stays uncommitted in the assignment's worktree.
struct Scripted {
    steps: Mutex<BTreeMap<String, usize>>,
    first: Mutex<Option<String>>,
    fail_first_run: bool,
    implementor_calls: AtomicUsize,
}
impl Scripted {
    fn new(fail_first_run: bool) -> Self {
        Self {
            steps: Mutex::new(BTreeMap::new()),
            first: Mutex::new(None),
            fail_first_run,
            implementor_calls: AtomicUsize::new(0),
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
                self.implementor_calls.fetch_add(1, Ordering::SeqCst);
                let context = request.execution_context.clone();
                let first = self
                    .first
                    .lock()
                    .unwrap()
                    .get_or_insert_with(|| context.clone())
                    .clone();
                let mut steps = self.steps.lock().unwrap();
                let count = steps.entry(context.clone()).or_default();
                *count += 1;
                if *count == 1 {
                    Ok(json!({"action":"write","path":"src/lib.rs","contents":IMPLEMENTED}))
                } else if self.fail_first_run && context == first {
                    anyhow::bail!("model provider unavailable")
                } else {
                    Ok(json!({"action":"finish","summary":"Requested behavior implemented"}))
                }
            }
            other => anyhow::bail!("unexpected model role {other}"),
        }
    }
}

fn supervisor(fixture: &Fixture, grace: Duration, model: Arc<Scripted>) -> Supervisor {
    Supervisor::new(
        fixture.store.clone(),
        Arc::new(Notify::new()),
        RuntimeConfig {
            publication_grace: grace,
            ..fixture.config.clone()
        },
        model,
    )
}

/// Plan and deliver once: the publisher exits without a merge, so the intent is Uncertain and
/// the assignment Blocked.
async fn unresolved(fixture: &Fixture, supervisor: &Supervisor) -> Value {
    supervisor.tick().await.unwrap();
    supervisor.fleet_tick().await.unwrap();
    let intents = rows(&fixture.store, "PublicationIntentList").await;
    assert_eq!(intents.len(), 1, "precondition: {intents:?}");
    assert_eq!(
        intents[0]["state"], "Uncertain",
        "precondition: {intents:?}"
    );
    let assignment = rows(&fixture.store, "AssignmentList").await[0].clone();
    assert_eq!(assignment["state"], "Blocked", "precondition: {assignment}");
    intents[0].clone()
}

/// Another change lands on the target: `contents` written to `file` on top of `base`, committed
/// in a separate clone of the remote (`clone`) and pushed. Returns the target's new head.
fn land(fixture: &Fixture, clone: &str, base: &str, file: &str, contents: &str) -> String {
    let remote = fixture.root.join("remotes/repo0.git");
    let other = fixture.root.join(clone);
    cmd(
        &fixture.root,
        "git",
        &[
            "clone",
            "--quiet",
            remote.to_str().unwrap(),
            other.to_str().unwrap(),
        ],
        &[],
    );
    cmd(&other, "git", &["switch", "--quiet", "--detach", base], &[]);
    std::fs::write(other.join(file), contents).unwrap();
    cmd(&other, "git", &["add", file], &[]);
    cmd(
        &other,
        "git",
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "a change that landed on the target",
        ],
        &[],
    );
    cmd(
        &other,
        "git",
        &["push", "--quiet", "origin", "HEAD:refs/heads/main"],
        &[],
    );
    cmd(&other, "git", &["rev-parse", "HEAD"], &[])
        .trim()
        .to_owned()
}

/// The path of the managed worktree that holds `assignment`'s attempts.
fn worktree_of(fixture: &Fixture, assignment: &Value) -> PathBuf {
    let primary = fixture.primary();
    let inspection: Value = serde_json::from_str(&cmd(
        &primary,
        "worktree",
        &["inspect", "--json", "--repo", primary.to_str().unwrap()],
        &fixture.config.environment,
    ))
    .unwrap();
    let id = &assignment["worktree_id"];
    let found = inspection["inspections"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| &item["record"]["id"] == id)
        .unwrap_or_else(|| panic!("no managed worktree {id}: {inspection}"));
    PathBuf::from(found["record"]["path"].as_str().unwrap())
}

/// Plan and run one attempt whose model run fails after writing `src/lib.rs`: the assignment is
/// Blocked at attempt 1 and its worktree holds the write uncommitted.
async fn failed_with_leftovers(fixture: &Fixture, supervisor: &Supervisor) -> Value {
    supervisor.tick().await.unwrap();
    supervisor.fleet_tick().await.unwrap();
    let assignment = rows(&fixture.store, "AssignmentList").await[0].clone();
    assert_eq!(
        (assignment["state"].as_str(), assignment["attempt"].as_i64()),
        (Some("Blocked"), Some(1)),
        "precondition: {assignment}"
    );
    assert!(
        rows(&fixture.store, "PublicationIntentList")
            .await
            .is_empty(),
        "precondition: the failed attempt published nothing"
    );
    let tree = worktree_of(fixture, &assignment);
    assert_eq!(
        cmd(&tree, "git", &["status", "--porcelain"], &[]).trim(),
        "M src/lib.rs",
        "precondition: the failed attempt left its write uncommitted"
    );
    assignment
}

/// Implementor edge 1. The target moves by a change to the line the candidate changed, so the
/// candidate's work cannot be merged onto the target's current head. The first tick closes the
/// intent, repairs the assignment onto that head (an attempt) and fails to merge. No later
/// attempt can merge it either: nothing in the worktree or on the target has changed, and the
/// implementor is never asked. Wave 4 blocked a moved target once without spending an attempt,
/// and `deliver` blocks a changed repository configuration once "instead of attempted on every
/// tick"; a merge that cannot succeed is the same kind of cause, so later ticks must not spend
/// the goal's remaining attempts on it.
///
/// Correction 1 (coordinator decision 1): the conflict is now decided before the repair, so the
/// first tick spends no attempt either. The precondition reads that state instead of a repair
/// onto the moved head whose merge failed; the property, no attempt over two more ticks, is
/// unchanged.
#[tokio::test]
async fn conflicting_moved_target_is_not_attempted_on_every_tick() {
    let fixture = fixture(Setup {
        max_attempts: 4,
        refuse_first_integration_commit: false,
    })
    .await;
    let model = Arc::new(Scripted::new(false));
    let supervisor = supervisor(&fixture, PUBLICATION_GRACE, model.clone());
    let first = unresolved(&fixture, &supervisor).await;
    let moved = land(
        &fixture,
        "conflicting-clone",
        first["expected_base"].as_str().unwrap(),
        "src/lib.rs",
        &LIBRARY.replace("{ 0 }", "{ 7 }"),
    );
    supervisor.fleet_tick().await.unwrap();
    let conflicted = rows(&fixture.store, "AssignmentList").await[0].clone();
    assert_eq!(
        (
            conflicted["state"].as_str(),
            conflicted["attempt"].as_i64(),
            conflicted["base_revision"].as_str()
        ),
        (Some("Blocked"), Some(1), first["expected_base"].as_str()),
        "precondition: the conflict with {moved} was decided before the repair: {conflicted}"
    );
    assert!(
        conflicted["reason"]
            .as_str()
            .is_some_and(|reason| reason.contains(&moved) && reason.contains("src/lib.rs")),
        "precondition: {conflicted}"
    );
    let calls = model.implementor_calls.load(Ordering::SeqCst);

    for _ in 0..2 {
        supervisor.fleet_tick().await.unwrap();
    }
    let now = rows(&fixture.store, "AssignmentList").await[0].clone();
    assert_eq!(
        now["attempt"].as_i64(),
        conflicted["attempt"].as_i64(),
        "two more ticks over the same unmergeable target spent attempts: the assignment is {} at \
         attempt {} of 4, the implementor was asked {} more times, reason: {}",
        now["state"],
        now["attempt"],
        model.implementor_calls.load(Ordering::SeqCst) - calls,
        now["reason"]
    );
}

/// Implementor edge 2, the overlapping half. An attempt fails after the model wrote
/// `src/lib.rs` (a provider outage; a timeout, `max_steps` or a paused goal leave the same), and
/// the target then moves by a change to the end of that file, which merges cleanly with the
/// write once it is committed. The repair takes the target's head, and the next attempt must
/// reach publication on it: the story's "the next attempt implements on it and publishes a new
/// intent". Git refuses to merge into a file with local changes, and `integrate` then aborts a
/// merge that never started.
#[tokio::test]
async fn failed_attempt_leftovers_do_not_wedge_the_repair_on_a_moved_target() {
    let fixture = fixture(Setup {
        max_attempts: 3,
        refuse_first_integration_commit: false,
    })
    .await;
    let model = Arc::new(Scripted::new(true));
    let supervisor = supervisor(&fixture, PUBLICATION_GRACE, model);
    let failed = failed_with_leftovers(&fixture, &supervisor).await;
    let moved = land(
        &fixture,
        "same-file-clone",
        failed["base_revision"].as_str().unwrap(),
        "src/lib.rs",
        &LIBRARY.replace("// end", "// end, edited on the target"),
    );
    for _ in 0..2 {
        supervisor.fleet_tick().await.unwrap();
    }
    let intents = rows(&fixture.store, "PublicationIntentList").await;
    let now = rows(&fixture.store, "AssignmentList").await[0].clone();
    assert!(
        intents.iter().any(|i| i["expected_base"] == moved.as_str()),
        "two attempts after the target moved to {moved} published nothing: the assignment is {} \
         at attempt {} of 3 on {}, reason: {}",
        now["state"],
        now["attempt"],
        now["base_revision"],
        now["reason"]
    );
}

/// Implementor edge 2, the disjoint half. The same failed attempt, but the target moves by a
/// file the write does not touch, so the merge proceeds, and `integrate` stages everything
/// (`git add --all`) into the commit it names "Integrate the target's current head". The
/// failed run's write then lives in that merge commit and in no parent of it: the default
/// `git log -p` shows no diff for a merge, and the "Implement accepted engineering story"
/// commit carries nothing. A merge that only integrates the target has an empty combined diff.
#[tokio::test]
async fn integration_merge_carries_no_change_of_its_own() {
    let fixture = fixture(Setup {
        max_attempts: 3,
        refuse_first_integration_commit: false,
    })
    .await;
    let model = Arc::new(Scripted::new(true));
    let supervisor = supervisor(&fixture, PUBLICATION_GRACE, model);
    let failed = failed_with_leftovers(&fixture, &supervisor).await;
    let moved = land(
        &fixture,
        "notes-clone",
        failed["base_revision"].as_str().unwrap(),
        "NOTES.md",
        "Landed while an attempt was blocked.\n",
    );
    supervisor.fleet_tick().await.unwrap();
    let intents = rows(&fixture.store, "PublicationIntentList").await;
    let intent = intents
        .iter()
        .find(|i| i["expected_base"] == moved.as_str())
        .unwrap_or_else(|| panic!("precondition: the attempt on {moved} published: {intents:?}"));
    let primary = fixture.primary();
    let candidate = intent["candidate"].as_str().unwrap();
    let merge = cmd(
        &primary,
        "git",
        &[
            "log",
            "--merges",
            "--format=%H",
            &format!("--grep=^{INTEGRATION}"),
            candidate,
        ],
        &[],
    );
    let merge = merge
        .lines()
        .next()
        .unwrap_or_else(|| panic!("precondition: {candidate} has an integration merge"));
    let own = cmd(&primary, "git", &["show", "--cc", "--format=", merge], &[]);
    assert_eq!(
        own.trim(),
        "",
        "the integration merge {merge} of {candidate} carries changes from neither parent"
    );
}

/// A rewound target. The target is reset to an ancestor of the assignment's base, dropping the
/// commit the base was (here a commit adding `NOTES.md`). With no grace the intent closes at the
/// next observation, and the repair takes the rewound head as the new base. That head is an
/// ancestor of the worktree, so `integrate` merges nothing and the next candidate still descends
/// from the dropped commit: its publication would put back on the target what the target
/// dropped. Wave 4 blocked here; `settlement` (fleet.rs) says a rewritten target is not "moved
/// past". The next candidate must not carry the dropped commit.
///
/// Correction 1 (coordinator decision 1): a rewound target now blocks the retry before the
/// repair, so there is no retry to take the rewound head. The precondition reads that state
/// instead of requiring such a retry, and the property is checked on every publication after the
/// first, as `squashed_candidate_is_not_published_again_as_an_empty_change` checks its own.
#[tokio::test]
async fn rewound_target_is_not_republished_with_the_commit_it_dropped() {
    let fixture = fixture(Setup {
        max_attempts: 2,
        refuse_first_integration_commit: false,
    })
    .await;
    let primary = fixture.primary();
    let root = cmd(&primary, "git", &["rev-parse", "HEAD"], &[])
        .trim()
        .to_owned();
    let dropped = land(
        &fixture,
        "dropped-clone",
        &root,
        "NOTES.md",
        "Landed before the claim; the target drops it later.\n",
    );
    let model = Arc::new(Scripted::new(false));
    let supervisor = supervisor(&fixture, Duration::ZERO, model);
    let first = unresolved(&fixture, &supervisor).await;
    assert_eq!(
        first["expected_base"], dropped,
        "precondition: the claim took the landed commit as its base"
    );
    cmd(
        &primary,
        "git",
        &[
            "push",
            "--quiet",
            "--force",
            "origin",
            &format!("{root}:refs/heads/main"),
        ],
        &[],
    );
    supervisor.fleet_tick().await.unwrap();
    let intents = rows(&fixture.store, "PublicationIntentList").await;
    let now = rows(&fixture.store, "AssignmentList").await[0].clone();
    assert_eq!(
        (
            now["state"].as_str(),
            now["attempt"].as_i64(),
            now["base_revision"].as_str()
        ),
        (Some("Blocked"), Some(1), Some(dropped.as_str())),
        "precondition: the rewound target was decided before the repair: {now}"
    );
    for second in intents
        .iter()
        .filter(|i| i["publication_id"] != first["publication_id"])
    {
        let candidate = second["candidate"].as_str().unwrap();
        assert!(
            !git_ok(
                &primary,
                &["merge-base", "--is-ancestor", &dropped, candidate]
            ),
            "the publication {} expects {} (the rewound target is {root}) and its candidate \
             {candidate} still carries {dropped}, the commit the target dropped",
            second["publication_id"],
            second["expected_base"]
        );
    }
}

/// A publisher that squashes. The README records a squashed publication as not published. Here
/// the squash lands as one commit on the base holding the candidate's exact tree, which moves the
/// target past the head observed at Uncertain, and the intent closes. The repair takes the squash
/// as the new base and `integrate` merges it: both sides hold the same tree, so the merge and the
/// implementor change nothing, and the next candidate is the integration merge itself, with no
/// change against the base its publication expects. Its review reads an empty diff, and a
/// publisher that merges it records the story's work as merged by a commit that carries none of
/// it. Wave 4 blocked here. A candidate that changes nothing on its target must not be published.
#[tokio::test]
async fn squashed_candidate_is_not_published_again_as_an_empty_change() {
    let fixture = fixture(Setup {
        max_attempts: 3,
        refuse_first_integration_commit: false,
    })
    .await;
    let model = Arc::new(Scripted::new(false));
    let supervisor = supervisor(&fixture, PUBLICATION_GRACE, model);
    let first = unresolved(&fixture, &supervisor).await;
    let primary = fixture.primary();
    let base = first["expected_base"].as_str().unwrap();
    let squashed = first["candidate"].as_str().unwrap();
    let squash = cmd(
        &primary,
        "git",
        &[
            "commit-tree",
            &format!("{squashed}^{{tree}}"),
            "-p",
            base,
            "-m",
            "Return the requested answer (squashed)",
        ],
        &[],
    )
    .trim()
    .to_owned();
    cmd(
        &primary,
        "git",
        &[
            "push",
            "--quiet",
            "origin",
            &format!("{squash}:refs/heads/main"),
        ],
        &[],
    );
    supervisor.fleet_tick().await.unwrap();
    let intents = rows(&fixture.store, "PublicationIntentList").await;
    let now = rows(&fixture.store, "AssignmentList").await[0].clone();
    let closed = intents
        .iter()
        .find(|i| i["publication_id"] == first["publication_id"])
        .unwrap();
    assert_eq!(
        closed["state"], "NotPublished",
        "precondition: the squash closed the first intent: {intents:?}"
    );
    for intent in intents
        .iter()
        .filter(|i| i["publication_id"] != first["publication_id"])
    {
        let expected = intent["expected_base"].as_str().unwrap();
        let candidate = intent["candidate"].as_str().unwrap();
        assert!(
            !git_ok(&primary, &["diff", "--quiet", expected, candidate, "--"]),
            "the publication {} of {candidate} expects {expected}, the squash of {squashed}, and \
             changes nothing on it: the assignment is {} at attempt {} on {}",
            intent["publication_id"],
            now["state"],
            now["attempt"],
            now["base_revision"]
        );
    }
}

/// `integrate` aborts a merge that fails, so that, as its comment says, the worktree is left as
/// it was. A commit command that fails once (a hook, or the bot credential unavailable for one
/// commit) fails `integrate` after the merge started, and the merge is not aborted. The next
/// attempt then meets a merge in progress, fails on it and aborts it, so one failed commit costs
/// two attempts. With three attempts, the one after the failed commit must publish.
#[tokio::test]
async fn failed_integration_commit_leaves_no_merge_in_progress() {
    let fixture = fixture(Setup {
        max_attempts: 3,
        refuse_first_integration_commit: true,
    })
    .await;
    let model = Arc::new(Scripted::new(false));
    let supervisor = supervisor(&fixture, PUBLICATION_GRACE, model);
    let first = unresolved(&fixture, &supervisor).await;
    let moved = land(
        &fixture,
        "notes-clone",
        first["expected_base"].as_str().unwrap(),
        "NOTES.md",
        "Landed while a publication was open.\n",
    );
    supervisor.fleet_tick().await.unwrap();
    let refused = rows(&fixture.store, "AssignmentList").await[0].clone();
    assert_eq!(
        (refused["state"].as_str(), refused["attempt"].as_i64()),
        (Some("Blocked"), Some(2)),
        "precondition: {refused}"
    );
    assert!(
        refused["reason"]
            .as_str()
            .is_some_and(|reason| reason.contains("commit refused once")),
        "precondition: the integration commit was refused: {refused}"
    );

    supervisor.fleet_tick().await.unwrap();
    let intents = rows(&fixture.store, "PublicationIntentList").await;
    let now = rows(&fixture.store, "AssignmentList").await[0].clone();
    assert!(
        intents.iter().any(|i| i["expected_base"] == moved.as_str()),
        "the attempt after one refused commit published nothing on {moved}: the assignment is {} \
         at attempt {} of 3, reason: {}",
        now["state"],
        now["attempt"],
        now["reason"]
    );
}
