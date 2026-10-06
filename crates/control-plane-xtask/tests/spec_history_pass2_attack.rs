//! Adversarial cases for `spec-history-check`, pass 2: baseline selection and the lifetime of an
//! acknowledgement.
//!
//! The gate module is compiled in by path so the cases drive `check` exactly as the binary does;
//! that also re-runs the module's own unit tests in this binary.
#[allow(dead_code)]
#[path = "../src/spec_history.rs"]
mod spec_history;

use anyhow::{Result, ensure};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

const HOST: &str = "ess/domains/host.yaml";
const REPAIR: &str = "entity/controlplane.host.Assignment/transition-route-changed/repair";
const REMOVED: &str =
    "command/controlplane.host.DisableRepositoryRegistration/outcome-removed/wrong-state";
const ADDED: &str =
    "command/controlplane.host.DisableRepositoryRegistration/outcome-added/wrong-state";
/// `DisableRepositoryRegistration` with and without its declared `wrong-state` refusal.
const WITH_WRONG_STATE: &str = "        repository_id: input.repository_id\n  - name: wrong-state\n    wrong_state: true\n    error: controlplane.host.RepositoryRegistrationStateConflict\n  - name: not-found\n    unknown_instance: true\n    error: controlplane.host.RepositoryRegistrationNotFound\n- name: controlplane.host.EnableRepositoryRegistration\n";
const WITHOUT_WRONG_STATE: &str = "        repository_id: input.repository_id\n  - name: not-found\n    unknown_instance: true\n    error: controlplane.host.RepositoryRegistrationNotFound\n- name: controlplane.host.EnableRepositoryRegistration\n";

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

/// `main` whose only commit is this tree's specification.
fn repository() -> Result<(tempfile::TempDir, String)> {
    let scratch = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.scratch/spec-history-pass2-attack-tests");
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
    git(root, &["init", "--quiet", "--initial-branch=main"])?;
    git(root, &["add", "ess"])?;
    git(
        root,
        &["commit", "--quiet", "--message", "Baseline specification"],
    )?;
    let baseline = git(root, &["rev-parse", "HEAD"])?;
    Ok((dir, baseline))
}

/// Write the acknowledgement file with `(id, change, reviewed baseline)` entries, each `change`
/// exactly as `ess verify diff --compatibility --format json` printed it.
fn acknowledge(root: &Path, baseline: &str, reviewed: &[(&str, Value, String)]) -> Result<()> {
    let acknowledged: Vec<_> = reviewed
        .iter()
        .map(|(id, change, reviewed)| {
            let mut entry = json!({"id": id, "change": change, "baseline": reviewed, "reason": "reviewed in the pass-2 attack test"});
            if change["changed"]["kind"] == "outcome-added" {
                entry["outcome"] = wrong_state();
                entry["preceded_by"] = json!(["applied"]);
            }
            entry
        })
        .collect();
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

fn edit(root: &Path, from: &str, to: &str) -> Result<()> {
    let path = root.join(HOST);
    let before = fs::read_to_string(&path)?;
    assert_eq!(
        before.matches(from).count(),
        1,
        "edit anchor not unique: {from}"
    );
    fs::write(&path, before.replacen(from, to, 1))?;
    Ok(())
}

fn commit(root: &Path, message: &str) -> Result<String> {
    git(root, &["add", "ess"])?;
    git(root, &["commit", "--quiet", "--message", message])?;
    git(root, &["rev-parse", "HEAD"])
}

/// The `change` ESS 0.53.0 prints for each id used here (probed with
/// `ess verify diff --compatibility --format json`).
fn change(kind: &str) -> Value {
    json!({"category": "command", "subject": "controlplane.host.DisableRepositoryRegistration",
        "changed": {"kind": kind, "outcome": "wrong-state"}})
}

/// The `wrong-state` refusal as `ess specify compile` prints it: an `outcome-added`
/// acknowledgement records the outcome it reviewed.
fn wrong_state() -> Value {
    json!({"name": "wrong-state", "condition": {"kind": "wrong_state"}, "complete_refusal": true,
        "test_strategy": "arrange_state", "emits": [],
        "error": "controlplane.host.RepositoryRegistrationStateConflict"})
}

/// The acceptance: "on a branch that removes `Blocked` from `Assignment.repair.from`, `task check`
/// fails and names `…/transition-route-changed/repair`" unless an acknowledgement names it.
///
/// A branch commits that removal, then points the recorded baseline at its own commit. The
/// recorded baseline descends from the merge base with `main`, so "the descendant wins" picks it,
/// the diff from it is empty, and the gate passes with no acknowledgement at all. Nothing checks
/// that the recorded baseline was ever published on the mainline: the live baseline today,
/// `eaa37d4`, is itself a commit `origin/main` does not contain.
#[test]
fn recorded_baseline_moved_onto_the_branch_does_not_hide_its_change() -> Result<()> {
    let (dir, published) = repository()?;
    let root = dir.path();
    git(root, &["checkout", "--quiet", "-b", "feature"])?;
    acknowledge(root, &published, &[])?;
    edit(
        root,
        "    - name: repair\n      from:\n      - Reviewing\n      - Blocked\n",
        "    - name: repair\n      from:\n      - Reviewing\n",
    )?;
    let removal = commit(root, "Remove Blocked from repair")?;
    // The branch as committed is refused, as the acceptance requires.
    let error = format!(
        "{:#}",
        spec_history::check(root).expect_err("the committed removal must be refused")
    );
    assert!(error.contains(REPAIR), "{error}");

    // Move the recorded baseline onto the branch's own commit; no acknowledgement names anything.
    acknowledge(root, &removal, &[])?;
    match spec_history::check(root) {
        Ok(summary) => panic!(
            "a branch that removes Blocked from repair.from passed with no acknowledgement after \
             pointing the recorded baseline at its own commit {removal} (main is {published}): {summary}"
        ),
        Err(error) => assert!(format!("{error:#}").contains(REPAIR), "{error:#}"),
    }
    Ok(())
}

/// An acknowledgement published with its baseline is never stale, so nothing ever asks for it
/// to be removed and it keeps admitting its change for good.
///
/// `main` removes `DisableRepositoryRegistration`'s `wrong-state` refusal (reviewed: stores held
/// none), later restores it (reviewed separately), and from then on stores record operators
/// disabling a repository twice. A branch then removes `wrong-state` again. Every store written
/// since the restoration holds a decision replay can no longer reproduce (pass 1 measured
/// `Store::open` refusing exactly this), yet the first, years-old acknowledgement admits the new
/// removal: same id, same `change` object, and the gate let it stay in the file.
#[test]
fn published_acknowledgement_does_not_admit_a_later_repeat_of_its_change() -> Result<()> {
    let (dir, first) = repository()?;
    let root = dir.path();
    // Each entry records the baseline it was reviewed against (correction round 2, F7).
    let removal = (REMOVED, change("outcome-removed"), first.clone());

    edit(root, WITH_WRONG_STATE, WITHOUT_WRONG_STATE)?;
    acknowledge(root, &first, std::slice::from_ref(&removal))?;
    spec_history::check(root)?;
    let published = commit(root, "Remove the wrong-state refusal (published)")?;
    let restoration = (ADDED, change("outcome-added"), published);

    edit(root, WITHOUT_WRONG_STATE, WITH_WRONG_STATE)?;
    acknowledge(root, &first, &[removal.clone(), restoration.clone()])?;
    // The gate accepts the old acknowledgement staying in the file.
    spec_history::check(root)?;
    commit(root, "Restore the wrong-state refusal (published)")?;

    git(root, &["checkout", "--quiet", "-b", "feature"])?;
    edit(root, WITH_WRONG_STATE, WITHOUT_WRONG_STATE)?;
    match spec_history::check(root) {
        Ok(summary) => panic!(
            "an acknowledgement reviewed for the first removal of wrong-state admitted a second \
             removal, after stores had recorded wrong-state again: {summary}"
        ),
        Err(error) => assert!(format!("{error:#}").contains(REMOVED), "{error:#}"),
    }
    Ok(())
}
