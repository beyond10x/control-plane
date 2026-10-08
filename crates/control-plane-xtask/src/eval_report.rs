//! `eval report`: whether one recorded goal ran unattended, read from a state store a run left
//! behind. It needs no model and never writes to the store or the repository.
//!
//! The store is copied under a shared lock on the service's lock file, so a running service
//! (which holds that lock exclusively) is never interrupted: the report refuses instead. The copy
//! is opened through `Store::open`, which replays every decision through the generated behaviour,
//! and the views answer the goal's state, its merged assignments and their repository. The
//! decision log of the same copy answers what the views do not keep: which actor recorded each
//! decision, in order, and when.
use anyhow::{Context, Result, bail, ensure};
use control_plane_core::{Actor, Store};
use eventlog_core::{EventStore, RecordedEvent, StreamId, TenantId};
use eventlog_sqlite::SqliteEventStore;
use fs2::FileExt;
use serde_json::Value;
use std::{
    fs::File,
    path::{Path, PathBuf},
    process::Command,
};

pub fn run(state: &Path, goal: &str, repo: &Path) -> Result<()> {
    let snapshot = tempfile::tempdir()?;
    let copy = snapshot_store(state, snapshot.path())?;
    let recorded = tokio::runtime::Runtime::new()?.block_on(read(&copy, goal))?;
    let repo = repo
        .canonicalize()
        .with_context(|| format!("repository {} does not exist", repo.display()))?;

    let mut reasons = Vec::new();
    println!("goal: {goal}");
    println!("state: {}", recorded.state);
    if recorded.state != "Satisfied" {
        reasons.push(format!("goal is {}, not Satisfied", recorded.state));
    }
    match recorded.receipt_revision {
        Some(revision) => println!("receipt revision: {revision}"),
        None => println!("receipt revision: none"),
    }
    let merged = recorded
        .merged
        .iter()
        .filter(|commit| commit.path == repo)
        .collect::<Vec<_>>();
    if merged.is_empty() {
        println!("merged commit: none");
        reasons.push(format!(
            "no merged commit of this goal is recorded for {}",
            repo.display()
        ));
    }
    for commit in merged {
        let head = origin_head(&repo, &commit.target)?;
        let on_target = is_ancestor(&repo, &commit.candidate, &head)?;
        println!(
            "merged commit: {} on origin/{}: {}",
            commit.candidate,
            commit.target,
            if on_target { "yes" } else { "no" }
        );
        if !on_target {
            reasons.push(format!(
                "merged commit {} is not on the target origin/{} (at {head})",
                commit.candidate, commit.target
            ));
        }
    }
    match recorded.elapsed_seconds {
        Some(seconds) => println!("elapsed: {seconds:.3}s"),
        None => println!("elapsed: none (no SatisfyGoal recorded)"),
    }
    println!("operator commands after start: {}", recorded.operator);
    if recorded.operator > 0 {
        reasons.push(format!(
            "{} Operator command(s) recorded after StartGoal",
            recorded.operator
        ));
    }
    if reasons.is_empty() {
        println!("unattended: yes");
        return Ok(());
    }
    println!("unattended: no");
    bail!("not an unattended run: {}", reasons.join("; "))
}

/// Copy the database and its write-ahead log while holding a shared lock on the lock file
/// `Store::open` holds exclusively, so no service is writing it. Nothing is created beside
/// the store: a missing store is refused, and a missing lock file means no store owner exists.
fn snapshot_store(state: &Path, into: &Path) -> Result<PathBuf> {
    ensure!(
        state.is_file(),
        "no recorded state store at {}",
        state.display()
    );
    let state = state.canonicalize()?;
    let lock = state.with_extension("lock");
    let held = if lock.exists() {
        let file = File::open(&lock)?;
        FileExt::try_lock_shared(&file).with_context(|| {
            format!(
                "a running control-plane service owns this store ({}); stop it, or report on a copy",
                state.display()
            )
        })?;
        Some(file)
    } else {
        None
    };
    let copy = into.join("state.sqlite");
    std::fs::copy(&state, &copy)?;
    let wal = PathBuf::from(format!("{}-wal", state.display()));
    if wal.exists() {
        std::fs::copy(&wal, into.join("state.sqlite-wal"))?;
    }
    if let Some(file) = held {
        FileExt::unlock(&file)?;
    }
    Ok(copy)
}

struct Merged {
    candidate: String,
    path: PathBuf,
    target: String,
}

struct Recorded {
    state: String,
    receipt_revision: Option<i64>,
    merged: Vec<Merged>,
    elapsed_seconds: Option<f64>,
    operator: usize,
}

async fn read(copy: &Path, goal: &str) -> Result<Recorded> {
    let (state, merged) = {
        let store = Store::open(copy).await?;
        let goals = store.query("GoalList")?;
        let row = goals
            .as_array()
            .context("goals view is not an array")?
            .iter()
            .find(|row| row["goal_id"] == goal)
            .with_context(|| format!("goal {goal} is not in the store"))?;
        let state = text(row, "state")?.to_owned();
        let repositories = store.query("RepositoryRegistrationList")?;
        let mut merged = Vec::new();
        for assignment in store
            .query("AssignmentList")?
            .as_array()
            .context("assignments view is not an array")?
            .iter()
            .filter(|a| a["goal_id"] == goal && a["state"] == "Merged")
        {
            let repository = repositories
                .as_array()
                .context("repositories view is not an array")?
                .iter()
                .find(|r| r["repository_id"] == assignment["repository_id"])
                .context("merged assignment names no registered repository")?;
            merged.push(Merged {
                candidate: text(assignment, "candidate")?.to_owned(),
                path: PathBuf::from(text(repository, "path")?),
                target: text(repository, "base_branch")?.to_owned(),
            });
        }
        (state, merged)
    };

    let log = SqliteEventStore::open_existing(
        copy.to_str().context("store path is not UTF-8")?,
        "control_plane",
    )
    .await?;
    let stream = StreamId::new(TenantId::new("local")?, "host", "state")?;
    let mut events = Vec::new();
    let mut after = 0;
    loop {
        let slice = log.read_stream(&stream, after, 1000).await?;
        events.extend(slice.events);
        after = slice.next_version;
        if slice.end_of_stream {
            break;
        }
    }
    let decisions = events
        .iter()
        .map(Decision::of)
        .collect::<Result<Vec<_>>>()?;

    let start = decisions
        .iter()
        .position(|d| d.applied("StartGoal", goal))
        .with_context(|| format!("goal {goal} has no applied StartGoal recorded"))?;
    let satisfy = decisions[start..]
        .iter()
        .position(|d| d.applied("SatisfyGoal", goal))
        .map(|offset| start + offset);
    let end = satisfy.unwrap_or(decisions.len());
    let operator = decisions[start + 1..end]
        .iter()
        .filter(|d| d.actor == Actor::Operator)
        .count();
    Ok(Recorded {
        state,
        receipt_revision: satisfy
            .and_then(|i| decisions[i].event.data["body"]["receipt_revision"].as_i64()),
        merged,
        elapsed_seconds: satisfy.map(|i| {
            (decisions[i].event.occurred_at - decisions[start].event.occurred_at).as_seconds_f64()
        }),
        operator,
    })
}

/// One recorded host decision and the generated actor that recorded it.
struct Decision<'a> {
    event: &'a RecordedEvent,
    actor: Actor,
}

impl<'a> Decision<'a> {
    fn of(event: &'a RecordedEvent) -> Result<Self> {
        ensure!(
            event.name == "HostDecision" && event.schema_version == 1 && !event.is_redacted(),
            "unsupported or redacted host history"
        );
        let actor = *Actor::ALL
            .iter()
            .find(|actor| actor.name() == event.actor)
            .with_context(|| format!("decision {} names an undeclared actor", event.version))?;
        Ok(Self { event, actor })
    }

    fn applied(&self, command: &str, goal: &str) -> bool {
        let data = &self.event.data;
        data["command"] == command
            && data["body"]["goal_id"] == goal
            && data["outcome"]["outcome"] == "applied"
    }
}

fn text<'a>(row: &'a Value, field: &str) -> Result<&'a str> {
    row[field]
        .as_str()
        .with_context(|| format!("{field} is not text"))
}

fn git(repo: &Path, args: &[&str]) -> Result<std::process::Output> {
    Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .context("git did not run")
}

/// The head of `target` on the repository's origin, as the fleet observes a publication.
fn origin_head(repo: &Path, target: &str) -> Result<String> {
    let output = git(
        repo,
        &[
            "ls-remote",
            "--refs",
            "origin",
            &format!("refs/heads/{target}"),
        ],
    )?;
    ensure!(
        output.status.success(),
        "git ls-remote origin: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let listing = String::from_utf8(output.stdout)?;
    let mut lines = listing.lines();
    let line = lines
        .next()
        .with_context(|| format!("origin has no branch {target}"))?;
    ensure!(lines.next().is_none(), "ambiguous origin target {target}");
    Ok(line
        .split_once('\t')
        .context("invalid ls-remote line")?
        .0
        .to_owned())
}

fn is_ancestor(repo: &Path, commit: &str, head: &str) -> Result<bool> {
    let output = git(repo, &["merge-base", "--is-ancestor", commit, head])?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => bail!(
            "cannot compare {commit} with target head {head} in {}: {}",
            repo.display(),
            String::from_utf8_lossy(&output.stderr)
        ),
    }
}
