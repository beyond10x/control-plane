//! Adversarial cases for story:precise-outcome-acknowledgements, wave 3 pass 1: does an
//! acknowledgement of an added outcome admit only the outcome that was reviewed?
//!
//! The gate module is compiled in by path so the cases drive `check` exactly as the binary does;
//! that also re-runs the module's own unit tests in this binary.
#[allow(dead_code)]
#[path = "../src/spec_history.rs"]
mod spec_history;

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

const HOST: &str = "ess/domains/host.yaml";
const COMMAND: &str = "controlplane.host.UpdateGoal";
const ATTEMPTS_ADDED: &str = "command/controlplane.host.UpdateGoal/outcome-added/too-many-attempts";
const ORDER_CHANGED: &str = "command/controlplane.host.UpdateGoal/outcome-order-changed";
const SATISFIED_ADDED: &str = "command/controlplane.host.UpdateGoal/outcome-added/satisfied";
const CANCELLED_ADDED: &str = "command/controlplane.host.UpdateGoal/outcome-added/cancelled";
const APPLIED_ADDED: &str = "command/controlplane.host.UpdateGoal/outcome-added/applied";
const EDITED_REMOVED: &str = "command/controlplane.host.UpdateGoal/outcome-removed/edited";

/// `UpdateGoal`'s `applied` outcome as this tree declares it: the head of its outcome list.
const APPLIED: &str = "  - name: applied\n    updates: controlplane.host.Goal\n    instance: goal_id\n    when_subject_state:\n    - Paused\n    - Running\n";
/// The same outcome before story:terminal-goal-edits: it applied in every state.
const APPLIED_OTHERWISE: &str =
    "  - name: applied\n    updates: controlplane.host.Goal\n    instance: goal_id\n";
/// The same outcome under another name, so that `applied` is an added outcome.
const EDITED: &str = "  - name: edited\n    updates: controlplane.host.Goal\n    instance: goal_id\n    when_subject_state:\n    - Paused\n    - Running\n";
/// The two refusals story:terminal-goal-edits added and reviewed.
const TERMINAL_REFUSALS: &str = "  - name: satisfied\n    when_subject_state: Satisfied\n    error: controlplane.host.GoalStateConflict\n  - name: cancelled\n    when_subject_state: Cancelled\n    error: controlplane.host.GoalStateConflict\n";
/// The same names, swapped between the states and answering `GoalNotFound`.
const SWAPPED_REFUSALS: &str = "  - name: satisfied\n    when_subject_state: Cancelled\n    error: controlplane.host.GoalNotFound\n  - name: cancelled\n    when_subject_state: Satisfied\n    error: controlplane.host.GoalNotFound\n";
/// The last `Goal` error the specification declares: where the baseline declares two more.
const GOAL_NOT_FOUND: &str = "- name: controlplane.host.GoalNotFound\n  summary: The requested identity is not held.\n  fields: []\n";
/// Two errors without fields, so that every refusal below is generated in full, as `generate`
/// requires of this product ("generation left unmet capabilities" otherwise).
const LIMIT_ERRORS: &str = "- name: controlplane.host.GoalWorkerLimit\n  summary: The update asks for too many workers.\n  fields: []\n- name: controlplane.host.GoalAttemptLimit\n  summary: The update asks for too many attempts.\n  fields: []\n";
/// An input-guarded refusal the baseline already declares.
const WORKERS: &str = "  - name: too-many-workers\n    when: max_workers > 100\n    error: controlplane.host.GoalWorkerLimit\n";
/// The input-guarded refusal the branch adds. Its guard overlaps `WORKERS`: an update asking for
/// more than 100 workers and more than 100 attempts satisfies both.
const ATTEMPTS: &str = "  - name: too-many-attempts\n    when: max_attempts > 100\n    error: controlplane.host.GoalAttemptLimit\n";
/// One field update of `applied`, and the same update by another amount.
const REVISION_BY_ONE: &str =
    "      merge_authority: input.merge_authority\n      revision:\n        increment: 1\n";
const REVISION_BY_TWO: &str =
    "      merge_authority: input.merge_authority\n      revision:\n        increment: 2\n";

fn git(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .current_dir(root)
        .args([
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
        ])
        .args(args)
        .output()?;
    ensure!(
        output.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn replace_once(text: &str, from: &str, to: &str) -> String {
    assert_eq!(text.matches(from).count(), 1, "anchor not unique: {from}");
    text.replacen(from, to, 1)
}

fn edit(root: &Path, from: &str, to: &str) -> Result<()> {
    let path = root.join(HOST);
    let before = fs::read_to_string(&path)?;
    fs::write(&path, replace_once(&before, from, to))?;
    Ok(())
}

/// `main` whose only commit is this tree's specification with `UpdateGoal`'s outcome list
/// rewritten by `baseline`; the working tree then holds the specification `current` makes of it.
fn repository(
    baseline: impl Fn(&str) -> String,
    current: impl Fn(&str) -> String,
) -> Result<(tempfile::TempDir, String)> {
    let scratch = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.scratch/precise-outcome-acknowledgements-attack-tests");
    fs::create_dir_all(&scratch)?;
    let dir = tempfile::tempdir_in(scratch)?;
    let root = dir.path();
    let spec = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ess");
    for name in [
        "ess-inputs.yaml",
        "system.yaml",
        "components.yaml",
        "domains/host.yaml",
    ] {
        let to = root.join("ess").join(name);
        fs::create_dir_all(to.parent().unwrap())?;
        fs::copy(spec.join(name), to)?;
    }
    let tree = fs::read_to_string(root.join(HOST))?;
    fs::write(root.join(HOST), baseline(&tree))?;
    git(root, &["init", "--quiet", "--initial-branch=main"])?;
    git(root, &["add", "ess"])?;
    git(root, &["commit", "--quiet", "--message", "Baseline"])?;
    let commit = git(root, &["rev-parse", "HEAD"])?;
    fs::write(root.join(HOST), current(&tree))?;
    Ok((dir, commit))
}

/// The acknowledgement file, format 2, every entry reviewed against `baseline`.
fn acknowledge(root: &Path, baseline: &str, acknowledged: Vec<Value>) -> Result<()> {
    let document = json!({
        "format": "control-plane-spec-acknowledgements/2",
        "baseline": baseline,
        "acknowledged": acknowledged,
    });
    fs::write(
        root.join("ess/spec-acknowledgements.json"),
        serde_json::to_vec_pretty(&document)?,
    )?;
    Ok(())
}

/// An entry for `UpdateGoal`'s `kind` change of `outcome`, reviewed against `baseline`.
fn entry(id: &str, kind: &str, outcome: &str, baseline: &str) -> Value {
    json!({"id": id, "baseline": baseline, "reason": "reviewed in the pass-1 attack test",
        "change": {"category": "command", "subject": COMMAND,
            "changed": {"kind": kind, "outcome": outcome}}})
}

/// The outcome `name` of `UpdateGoal` exactly as `ess specify compile --path ess --format json`
/// prints it for the specification under `root`: what the README tells a reviewer to record.
fn compiled(root: &Path, name: &str) -> Result<Value> {
    let output = Command::new("ess")
        .current_dir(root)
        .args(["specify", "compile", "--path", "ess", "--format", "json"])
        .output()?;
    ensure!(
        output.status.success(),
        "ess specify compile failed: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let model: Value = serde_json::from_slice(&output.stdout)?;
    model["commands"][COMMAND]["outcomes"]
        .as_array()
        .and_then(|outcomes| outcomes.iter().find(|outcome| outcome["name"] == name))
        .cloned()
        .with_context(|| format!("no compiled outcome {name}"))
}

/// The names of the outcomes `UpdateGoal` declares before `name`, as `ess specify compile` lists
/// them for the specification under `root`: the position the README tells a reviewer to record.
fn preceded_by(root: &Path, name: &str) -> Result<Value> {
    let output = Command::new("ess")
        .current_dir(root)
        .args(["specify", "compile", "--path", "ess", "--format", "json"])
        .output()?;
    ensure!(
        output.status.success(),
        "ess specify compile failed: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let model: Value = serde_json::from_slice(&output.stdout)?;
    let outcomes = model["commands"][COMMAND]["outcomes"]
        .as_array()
        .context("no compiled outcomes")?;
    let position = outcomes
        .iter()
        .position(|outcome| outcome["name"] == name)
        .with_context(|| format!("no compiled outcome {name}"))?;
    Ok(json!(
        outcomes[..position]
            .iter()
            .map(|outcome| outcome["name"].clone())
            .collect::<Vec<_>>()
    ))
}

/// The specification under `root` is one `ess specify validate --strict-requires` accepts, as
/// `task check` requires before it runs the gate.
fn assert_valid(root: &Path) -> Result<()> {
    let validate = Command::new("ess")
        .current_dir(root)
        .args(["specify", "validate", "--path", "ess", "--strict-requires"])
        .output()?;
    ensure!(
        validate.status.success(),
        "not a valid specification: {}{}",
        String::from_utf8_lossy(&validate.stdout),
        String::from_utf8_lossy(&validate.stderr)
    );
    Ok(())
}

/// `UpdateGoal`'s generated Rust behaviour for the specification under `root`, with every
/// capability generated, as `cargo run -p control-plane-xtask -- generate` requires.
fn generated_update_goal(root: &Path) -> Result<String> {
    let out = root.join(".scratch/generated");
    let synthesize = Command::new("ess")
        .current_dir(root)
        .args([
            "generate",
            "synthesize",
            "--path",
            "ess",
            "--target",
            "rust",
        ])
        .args(["--layout", "crate", "--out"])
        .arg(&out)
        .output()?;
    ensure!(
        synthesize.status.success(),
        "ess generate synthesize failed: {}",
        String::from_utf8_lossy(&synthesize.stderr)
    );
    let plan: Value = serde_json::from_slice(&fs::read(out.join("plan.json"))?)?;
    ensure!(
        plan["capabilities"].as_array().is_some_and(|all| all
            .iter()
            .all(|c| c["disposition"]["disposition"] == "generated")),
        "generation left unmet capabilities"
    );
    let behaviour = fs::read_to_string(out.join("src/behaviour.rs"))?;
    fs::remove_dir_all(&out)?;
    let start = behaviour
        .find("    fn update_goal(")
        .context("no generated update_goal")?;
    let end = behaviour[start..]
        .find("\n    }\n")
        .context("unterminated update_goal")?;
    Ok(behaviour[start..start + end].to_owned())
}

/// Which of the two limit refusals the generated `update_goal` answers first for an update that
/// asks for more than 100 workers and more than 100 attempts.
fn first_limit(update_goal: &str) -> &'static str {
    let workers = update_goal
        .find("TooManyWorkers")
        .expect("no too-many-workers branch");
    let attempts = update_goal
        .find("TooManyAttempts")
        .expect("no too-many-attempts branch");
    if workers < attempts {
        "too-many-workers"
    } else {
        "too-many-attempts"
    }
}

/// The specification with the two limit errors declared, `UpdateGoal`'s outcome list opened by
/// `refusals`.
fn with_limits(tree: &str, refusals: &str) -> String {
    let tree = replace_once(
        tree,
        GOAL_NOT_FOUND,
        &format!("{GOAL_NOT_FOUND}{LIMIT_ERRORS}"),
    );
    replace_once(&tree, APPLIED, &format!("{refusals}{APPLIED}"))
}

/// The acceptance: "an acknowledgement reviewed for an added outcome refuses a later change that
/// keeps the outcome name and changes its state condition or its error".
///
/// Input-guarded refusals are decided in declaration order, the first whose guard holds (ESS
/// 0.53.0, `ess-synth/src/rust/behaviour.rs`, "The order of evaluation"). The branch adds
/// `too-many-attempts` after the baseline's `too-many-workers`, and the acknowledgement records
/// it as compiled. A later edit moves it before `too-many-workers`. Its compiled object is the
/// same, `ess verify diff` reports the same single `outcome-added` change (ESS orders only the
/// outcomes both sides declare), and the gate admits it. Yet the generated `update_goal` now
/// answers `too-many-attempts` first: an `UpdateGoal` asking for more than 100 workers and more
/// than 100 attempts, recorded as `too-many-workers` (`GoalWorkerLimit`), replays as
/// `too-many-attempts` (`GoalAttemptLimit`). The condition under which the reviewed outcome
/// answers grew, and the existing outcome's shrank, with no change reported for either.
#[test]
fn moving_an_added_outcome_ahead_of_an_overlapping_one_is_not_admitted() -> Result<()> {
    let (dir, baseline) = repository(
        |tree| with_limits(tree, WORKERS),
        |tree| with_limits(tree, &format!("{WORKERS}{ATTEMPTS}")),
    )?;
    let root = dir.path();
    let reviewed = compiled(root, "too-many-attempts")?;
    let mut acknowledgement = entry(
        ATTEMPTS_ADDED,
        "outcome-added",
        "too-many-attempts",
        &baseline,
    );
    acknowledgement["outcome"] = reviewed.clone();
    acknowledgement["preceded_by"] = preceded_by(root, "too-many-attempts")?;
    acknowledge(root, &baseline, vec![acknowledgement])?;
    assert_valid(root)?;
    assert_eq!(
        first_limit(&generated_update_goal(root)?),
        "too-many-workers"
    );
    // The outcome as reviewed passes: the acknowledgement is precise for what was reviewed.
    let summary = spec_history::check(root)?;
    assert!(summary.contains("1 acknowledged"), "{summary}");

    edit(
        root,
        &format!("{WORKERS}{ATTEMPTS}"),
        &format!("{ATTEMPTS}{WORKERS}"),
    )?;
    assert_valid(root)?;
    assert_eq!(compiled(root, "too-many-attempts")?, reviewed);
    assert_eq!(
        first_limit(&generated_update_goal(root)?),
        "too-many-attempts"
    );
    match spec_history::check(root) {
        Ok(summary) => panic!(
            "the acknowledgement reviewed for too-many-attempts declared after too-many-workers \
             admitted it moved ahead of too-many-workers; an UpdateGoal recorded as \
             too-many-workers (GoalWorkerLimit) now replays as too-many-attempts \
             (GoalAttemptLimit): {summary}"
        ),
        Err(error) => {
            let error = format!("{error:#}");
            assert!(
                error.contains(&format!("  {ATTEMPTS_ADDED} (callers")),
                "{error}"
            );
        }
    }
    Ok(())
}

/// The control for the case above: the same swap between two outcomes the baseline already
/// declares is a change ESS reports (`outcome-order-changed`, callers and readers `unknown`) and
/// the gate refuses unacknowledged. ESS's own diff treats declaration order as behaviour.
#[test]
fn the_same_swap_of_two_declared_outcomes_is_refused() -> Result<()> {
    let (dir, baseline) = repository(
        |tree| with_limits(tree, &format!("{WORKERS}{ATTEMPTS}")),
        |tree| with_limits(tree, &format!("{ATTEMPTS}{WORKERS}")),
    )?;
    let root = dir.path();
    acknowledge(root, &baseline, vec![])?;
    let error = format!(
        "{:#}",
        spec_history::check(root).expect_err("the swap must be refused")
    );
    assert!(
        error.contains(&format!("  {ORDER_CHANGED} (callers")),
        "{error}"
    );
    Ok(())
}

/// The unit's own document: the three entries it migrated into `ess/spec-acknowledgements.json`,
/// rebased onto a baseline without story:terminal-goal-edits' `UpdateGoal` change, admit that
/// change as this tree declares it and refuse the refusals swapped under the same names.
#[test]
fn migrated_acknowledgements_admit_only_the_reviewed_refusals() -> Result<()> {
    let (dir, baseline) = repository(
        |tree| {
            replace_once(
                &replace_once(tree, APPLIED, APPLIED_OTHERWISE),
                TERMINAL_REFUSALS,
                "",
            )
        },
        str::to_owned,
    )?;
    let root = dir.path();
    let live: Value = serde_json::from_slice(&fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ess/spec-acknowledgements.json"),
    )?)?;
    let mut acknowledged = live["acknowledged"].as_array().unwrap().clone();
    assert_eq!(acknowledged.len(), 3, "{live}");
    for entry in &mut acknowledged {
        entry["baseline"] = json!(baseline);
    }
    acknowledge(root, &baseline, acknowledged)?;
    let summary = spec_history::check(root)?;
    assert!(summary.contains("3 acknowledged"), "{summary}");

    edit(root, TERMINAL_REFUSALS, SWAPPED_REFUSALS)?;
    assert_valid(root)?;
    let error = format!(
        "{:#}",
        spec_history::check(root).expect_err("the swapped refusals must be refused")
    );
    for id in [SATISFIED_ADDED, CANCELLED_ADDED] {
        assert!(error.contains(&format!("  {id} (callers")), "{error}");
    }
    Ok(())
}

/// The README binds an added outcome's "field updates" as well as its condition and error. The
/// module's own cases change only the condition and the error, so a comparison of those two
/// fields alone would pass them. Here `applied` is the added outcome (the baseline declares it
/// as `edited`) and a later edit increments `revision` by two instead of one: `ess verify diff`
/// reports exactly the same two changes, only the compiled `sets` differs, and the gate must
/// refuse it.
#[test]
fn added_outcome_acknowledgement_binds_its_field_updates() -> Result<()> {
    let (dir, baseline) = repository(|tree| replace_once(tree, APPLIED, EDITED), str::to_owned)?;
    let root = dir.path();
    let mut added = entry(APPLIED_ADDED, "outcome-added", "applied", &baseline);
    added["outcome"] = compiled(root, "applied")?;
    added["preceded_by"] = preceded_by(root, "applied")?;
    let removed = entry(EDITED_REMOVED, "outcome-removed", "edited", &baseline);
    acknowledge(root, &baseline, vec![added, removed])?;
    let summary = spec_history::check(root)?;
    assert!(summary.contains("2 acknowledged"), "{summary}");

    edit(root, REVISION_BY_ONE, REVISION_BY_TWO)?;
    assert_valid(root)?;
    let error = format!(
        "{:#}",
        spec_history::check(root)
            .expect_err("a different field update under the same outcome name must be refused")
    );
    assert!(
        error.contains(&format!("  {APPLIED_ADDED} (callers")),
        "{error}"
    );
    assert!(error.contains("      sets: reviewed "), "{error}");
    assert!(
        !error.contains(&format!("  {EDITED_REMOVED} (callers")),
        "{error}"
    );
    Ok(())
}
