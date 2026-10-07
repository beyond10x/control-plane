//! Adversarial cases for story:precise-outcome-acknowledgements, wave 3 pass 2: does the position
//! an `outcome-added` acknowledgement records (`preceded_by`) bind what decides which calls the
//! reviewed outcome answers?
//!
//! The README and `AddedOutcome` say "a command answers with the first outcome whose condition
//! holds, so the outcomes before an added outcome decide which calls it answers". ESS 0.53.0 does
//! not decide in declaration order: input-guarded refusals answer first, before the addressed row
//! is read, wherever they are declared (`ess-synth/src/rust/behaviour.rs`, "The order of
//! evaluation"). The cases below build two reviewed specifications whose generated `update_goal`
//! is byte-identical and show the gate deciding the same later edit two ways.
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
const APPLIED_ID: &str = "command/controlplane.host.UpdateGoal/outcome-condition-changed/applied";
const SATISFIED_ID: &str = "command/controlplane.host.UpdateGoal/outcome-added/satisfied";
const CANCELLED_ID: &str = "command/controlplane.host.UpdateGoal/outcome-added/cancelled";
const WORKERS_ID: &str = "command/controlplane.host.UpdateGoal/outcome-added/too-many-workers";

/// `UpdateGoal`'s `applied` outcome as this tree declares it.
const APPLIED: &str = "  - name: applied\n    updates: controlplane.host.Goal\n    instance: goal_id\n    when_subject_state:\n    - Paused\n    - Running\n";
/// The same outcome before story:terminal-goal-edits: it applied in every state.
const APPLIED_OTHERWISE: &str =
    "  - name: applied\n    updates: controlplane.host.Goal\n    instance: goal_id\n";
/// The two refusals story:terminal-goal-edits added, as this tree declares them.
const TERMINAL_REFUSALS: &str = "  - name: satisfied\n    when_subject_state: Satisfied\n    error: controlplane.host.GoalStateConflict\n  - name: cancelled\n    when_subject_state: Cancelled\n    error: controlplane.host.GoalStateConflict\n";
/// The last `Goal` error the specification declares: where the worker-limit error goes.
const GOAL_NOT_FOUND: &str = "- name: controlplane.host.GoalNotFound\n  summary: The requested identity is not held.\n  fields: []\n";
/// A field-less error, so that every refusal below is generated in full.
const WORKER_LIMIT: &str = "- name: controlplane.host.GoalWorkerLimit\n  summary: The update asks for too many workers.\n  fields: []\n";
/// An input-guarded refusal. Its guard overlaps `satisfied` and `cancelled`: an update of a
/// Satisfied goal asking for more than 100 workers satisfies both.
const WORKERS: &str = "  - name: too-many-workers\n    when: max_workers > 100\n    error: controlplane.host.GoalWorkerLimit\n";

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

/// The specification before story:terminal-goal-edits: `applied` in every state, no refusals.
fn before_terminal_refusals(tree: &str) -> String {
    replace_once(
        &replace_once(tree, APPLIED, APPLIED_OTHERWISE),
        TERMINAL_REFUSALS,
        "",
    )
}

/// This tree's specification with the worker-limit error declared and `UpdateGoal`'s two
/// terminal refusals replaced by `refusals`.
fn with_worker_limit(tree: &str, refusals: &str) -> String {
    replace_once(
        &replace_once(
            tree,
            GOAL_NOT_FOUND,
            &format!("{GOAL_NOT_FOUND}{WORKER_LIMIT}"),
        ),
        TERMINAL_REFUSALS,
        refusals,
    )
}

/// `main` whose only commit is the specification before story:terminal-goal-edits; the working
/// tree holds this tree's specification as `current` rewrites it.
fn repository(current: impl Fn(&str) -> String) -> Result<(tempfile::TempDir, String)> {
    let scratch = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.scratch/precise-outcome-acknowledgements-pass2-attack-tests");
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
    fs::write(root.join(HOST), before_terminal_refusals(&tree))?;
    git(root, &["init", "--quiet", "--initial-branch=main"])?;
    git(root, &["add", "ess"])?;
    git(root, &["commit", "--quiet", "--message", "Baseline"])?;
    let commit = git(root, &["rev-parse", "HEAD"])?;
    fs::write(root.join(HOST), current(&tree))?;
    Ok((dir, commit))
}

/// Rewrite the working tree's `host.yaml` from this tree's specification.
fn rewrite(root: &Path, current: impl Fn(&str) -> String) -> Result<()> {
    let tree = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../")
            .join(HOST),
    )?;
    fs::write(root.join(HOST), current(&tree))?;
    Ok(())
}

/// The acknowledgement file, format 2, every entry reviewed against `baseline`.
fn acknowledge(root: &Path, baseline: &str, acknowledged: &[Value]) -> Result<()> {
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

/// `UpdateGoal` as `ess specify compile --path ess --format json` prints it under `root`.
fn compiled_command(root: &Path) -> Result<Value> {
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
    Ok(model["commands"][COMMAND].clone())
}

/// The README's two fields for the added outcome `name`, read from the compiler under `root`:
/// the outcome as printed, and the names of all the command's outcomes, in order.
fn reviewed(root: &Path, name: &str) -> Result<(Value, Value)> {
    let command = compiled_command(root)?;
    let outcomes = command["outcomes"].as_array().context("no outcomes")?;
    let position = outcomes
        .iter()
        .position(|outcome| outcome["name"] == name)
        .with_context(|| format!("no compiled outcome {name}"))?;
    let names: Vec<Value> = outcomes
        .iter()
        .map(|outcome| outcome["name"].clone())
        .collect();
    Ok((outcomes[position].clone(), json!(names)))
}

/// An `outcome-added` entry for `name`, recording what the compiler prints for it now.
fn added_entry(root: &Path, id: &str, name: &str, baseline: &str) -> Result<Value> {
    let (outcome, command_outcomes) = reviewed(root, name)?;
    Ok(
        json!({"id": id, "baseline": baseline, "reason": "reviewed in the pass-2 attack test",
        "change": {"category": "command", "subject": COMMAND,
            "changed": {"kind": "outcome-added", "outcome": name}},
        "outcome": outcome, "command_outcomes": command_outcomes}),
    )
}

/// The `applied` entry exactly as the live acknowledgement file records it, rebased.
fn applied_entry(baseline: &str) -> Value {
    json!({"id": APPLIED_ID, "baseline": baseline, "reason": "reviewed in the pass-2 attack test",
        "change": {"category": "command", "subject": COMMAND, "changed": {
            "kind": "outcome-condition-changed", "outcome": "applied",
            "before": "otherwise", "after": "when subject state is Paused or Running"}}})
}

/// The specification under `root` is one `ess specify validate --strict-requires` accepts.
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
/// capability generated.
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

/// The reviewed state: `too-many-workers` declared where the working tree puts it, all four changes
/// acknowledged as the compiler prints them. Returns the generated `update_goal` and the entries.
fn reviewed_with_workers(root: &Path, baseline: &str) -> Result<(String, Vec<Value>)> {
    assert_valid(root)?;
    let entries = vec![
        applied_entry(baseline),
        added_entry(root, SATISFIED_ID, "satisfied", baseline)?,
        added_entry(root, CANCELLED_ID, "cancelled", baseline)?,
        added_entry(root, WORKERS_ID, "too-many-workers", baseline)?,
    ];
    acknowledge(root, baseline, &entries)?;
    let summary = spec_history::check(root)?;
    assert!(summary.contains("4 acknowledged"), "{summary}");
    let update_goal = generated_update_goal(root)?;
    // An update of a Satisfied goal asking for more than 100 workers is answered by
    // `too-many-workers`, before the row is even read, whichever outcome is declared first.
    let workers = update_goal.find("UpdateGoalOutcome::TooManyWorkers");
    let satisfied = update_goal.find("UpdateGoalOutcome::Satisfied");
    assert!(
        workers.is_some() && satisfied.is_some() && workers < satisfied,
        "{update_goal}"
    );
    Ok((update_goal, entries))
}

/// Drop `too-many-workers` and its error from the specification, then follow the gate: it names
/// the entry stale and says "remove it". The entries for `satisfied` and `cancelled` stay as
/// reviewed. Returns what the gate says once the stale entry is gone.
fn drop_workers(
    root: &Path,
    baseline: &str,
    entries: &[Value],
) -> Result<std::result::Result<String, String>> {
    rewrite(root, str::to_owned)?;
    assert_valid(root)?;
    let error = format!(
        "{:#}",
        spec_history::check(root).expect_err("the entry of the dropped outcome must be stale")
    );
    assert!(
        error.contains(&format!(
            "stale acknowledgement in ess/spec-acknowledgements.json: {WORKERS_ID}"
        )),
        "{error}"
    );
    let kept: Vec<Value> = entries
        .iter()
        .filter(|entry| entry["id"] != WORKERS_ID)
        .cloned()
        .collect();
    acknowledge(root, baseline, &kept)?;
    // Now every update of a Satisfied goal is answered by `satisfied`, including the ones
    // `too-many-workers` answered when `satisfied` was reviewed.
    let update_goal = generated_update_goal(root)?;
    assert!(
        !update_goal.contains("TooManyWorkers")
            && update_goal.contains("UpdateGoalOutcome::Satisfied"),
        "{update_goal}"
    );
    Ok(spec_history::check(root).map_err(|error| format!("{error:#}")))
}

/// Control: `too-many-workers` declared between `applied` and `satisfied`. Dropping it changes
/// `UpdateGoal`'s outcome list (`command_outcomes` loses `too-many-workers`), and the gate refuses
/// `satisfied` and `cancelled` until they are reviewed again. The ready entries it prints are accepted as they
/// stand, as the README promises.
#[test]
fn dropping_an_earlier_declared_refusal_that_answers_first_is_refused() -> Result<()> {
    let (dir, baseline) =
        repository(|tree| with_worker_limit(tree, &format!("{WORKERS}{TERMINAL_REFUSALS}")))?;
    let root = dir.path();
    let (_, entries) = reviewed_with_workers(root, &baseline)?;
    let error = drop_workers(root, &baseline, &entries)?
        .expect_err("dropping an outcome declared before satisfied must be refused");
    for id in [SATISFIED_ID, CANCELLED_ID] {
        assert!(error.contains(&format!("  {id} (callers")), "{error}");
    }
    assert!(
        error.contains("      command_outcomes: reviewed [\"applied\",\"too-many-workers\",\"satisfied\",\"cancelled\",\"not-found\"], the model now says [\"applied\",\"satisfied\",\"cancelled\",\"not-found\"]\n"),
        "{error}"
    );

    // The README: "The ready entry carries both as they compile now". Take the gate's ready
    // entries in place of the reviewed ones; the gate accepts them.
    let mut ready: Vec<Value> = error
        .lines()
        .filter_map(|line| line.strip_prefix("    acknowledge after review: "))
        .map(serde_json::from_str::<Value>)
        .collect::<std::result::Result<_, _>>()?;
    assert_eq!(ready.len(), 2, "{error}");
    ready.push(applied_entry(&baseline));
    acknowledge(root, &baseline, &ready)?;
    let summary = spec_history::check(root)?;
    assert!(summary.contains("3 acknowledged"), "{summary}");
    Ok(())
}

/// The same reviewed behaviour, declared the other way: `too-many-workers` after `cancelled`.
/// The generated `update_goal` is byte-identical to the control's, so `satisfied` answered exactly
/// the same calls when it was reviewed. Dropping `too-many-workers` widens `satisfied` exactly as
/// in the control: an update of a Satisfied goal asking for more than 100 workers was answered
/// `too-many-workers` (`GoalWorkerLimit`) when `satisfied` was reviewed and is now answered
/// `satisfied` (`GoalStateConflict`). `preceded_by` names only the outcomes declared before
/// `satisfied`, so nothing it records changed, and the gate admits the wider `satisfied` on the
/// acknowledgement reviewed for the narrower one.
#[test]
fn dropping_a_later_declared_refusal_that_answers_first_is_refused() -> Result<()> {
    let (control_dir, control_baseline) =
        repository(|tree| with_worker_limit(tree, &format!("{WORKERS}{TERMINAL_REFUSALS}")))?;
    let (control, _) = reviewed_with_workers(control_dir.path(), &control_baseline)?;
    drop(control_dir);

    let (dir, baseline) =
        repository(|tree| with_worker_limit(tree, &format!("{TERMINAL_REFUSALS}{WORKERS}")))?;
    let root = dir.path();
    let (update_goal, entries) = reviewed_with_workers(root, &baseline)?;
    assert_eq!(
        update_goal, control,
        "the two declaration orders must generate the same behaviour"
    );
    let reviewed_satisfied = reviewed(root, "satisfied")?;
    match drop_workers(root, &baseline, &entries)? {
        Ok(summary) => {
            assert_eq!(reviewed(root, "satisfied")?, reviewed_satisfied);
            panic!(
                "the acknowledgement of satisfied, reviewed while too-many-workers answered every \
                 update asking for more than 100 workers first, admitted satisfied answering those \
                 updates of a Satisfied goal itself after too-many-workers was dropped; the same \
                 drop is refused when too-many-workers is declared before satisfied, with the same \
                 generated update_goal: {summary}"
            )
        }
        Err(error) => assert!(
            error.contains(&format!("  {SATISFIED_ID} (callers")),
            "{error}"
        ),
    }
    Ok(())
}
