//! Adversarial cases for `spec-history-check`.
//!
//! `Store::open` does not fold stored events: it re-invokes every recorded command with its
//! recorded body and actor through the current generated contract and requires the recorded
//! outcome envelope byte for byte (`crates/control-plane-core/src/lib.rs`, `Store::open`). A
//! specification change that alters what a recorded call is admitted to do, or what outcome it
//! answers, therefore stops the service on its existing store, whatever ESS's `history` column
//! (which models readers of stored *events*) says about it. Each case below is such a change that
//! ESS 0.53.0 classifies `history: compatible`, so the gate lets it through.
//!
//! The gate module is compiled in by path so the cases drive `check` exactly as the binary does;
//! that also re-runs the module's own three unit tests in this binary.
#[allow(dead_code)]
#[path = "../src/spec_history.rs"]
mod spec_history;

use anyhow::Result;
use std::{fs, path::Path, process::Command};

const HOST: &str = "ess/domains/host.yaml";

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
    anyhow::ensure!(
        output.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

/// A repository whose first commit is this tree's specification: the recorded baseline.
fn repository() -> Result<(tempfile::TempDir, String)> {
    let scratch =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.scratch/spec-history-attack-tests");
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
    acknowledge(root, &baseline, &[])?;
    Ok((dir, baseline))
}

/// The change each acknowledged id was reviewed for, as `ess verify diff` prints it: an
/// acknowledgement binds the reviewed change, not the id alone.
fn reviewed(id: &str) -> serde_json::Value {
    let route = |transition: &str, before: &str, after: &str| {
        serde_json::json!({"category": "entity", "subject": "controlplane.host.Assignment",
            "changed": {"kind": "transition-route-changed", "transition": transition,
                "before": before, "after": after}})
    };
    match id {
        "entity/controlplane.host.Assignment/transition-route-changed/repair" => route(
            "repair",
            "Blocked, Reviewing -> Implementing",
            "Reviewing -> Implementing",
        ),
        "entity/controlplane.host.Assignment/transition-route-changed/ready" => route(
            "ready",
            "Reviewing -> ReadyToMerge",
            "Blocked, Reviewing -> ReadyToMerge",
        ),
        _ => panic!("no reviewed change recorded for {id}"),
    }
}

fn acknowledge(root: &Path, baseline: &str, ids: &[&str]) -> Result<()> {
    let acknowledged: Vec<_> = ids
        .iter()
        .map(|id| {
            serde_json::json!({"id": id, "change": reviewed(id), "reason": "reviewed in the attack test"})
        })
        .collect();
    let document = serde_json::json!({
        "format": "control-plane-spec-acknowledgements/1",
        "baseline": baseline,
        "acknowledged": acknowledged,
    });
    fs::write(
        root.join("ess/spec-acknowledgements.json"),
        serde_json::to_vec_pretty(&document)?,
    )?;
    Ok(())
}

/// Replace exactly one occurrence of `from` in the host domain.
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

/// The gate must refuse and name `id`; it returning `Ok` is the defect.
fn refused_naming(root: &Path, id: &str) {
    match spec_history::check(root) {
        Ok(summary) => panic!(
            "the gate passed a change that breaks replay of stored decisions ({id}): {summary}"
        ),
        Err(error) => {
            let error = format!("{error:#}");
            assert!(error.contains(id), "{error}");
        }
    }
}

/// An operator who disables a repository twice (two tabs, a retried request) records a declared
/// `wrong-state` refusal: `Store::execute` commits declared refusals, and the committed fixture
/// itself records one for `PauseGoal`. Removing that outcome leaves the recorded call with no
/// declared answer, so replay cannot reproduce it. ESS: callers/readers unknown, history compatible.
#[test]
fn removed_recorded_outcome_fails_gate() -> Result<()> {
    let (dir, _) = repository()?;
    let root = dir.path();
    edit(
        root,
        "        repository_id: input.repository_id\n  - name: wrong-state\n    wrong_state: true\n    error: controlplane.host.RepositoryRegistrationStateConflict\n  - name: not-found\n    unknown_instance: true\n    error: controlplane.host.RepositoryRegistrationNotFound\n- name: controlplane.host.EnableRepositoryRegistration\n",
        "        repository_id: input.repository_id\n  - name: not-found\n    unknown_instance: true\n    error: controlplane.host.RepositoryRegistrationNotFound\n- name: controlplane.host.EnableRepositoryRegistration\n",
    )?;
    refused_naming(
        root,
        "command/controlplane.host.DisableRepositoryRegistration/outcome-removed/wrong-state",
    );
    Ok(())
}

/// A recorded `wrong-state` answer carries the error's fields in its payload
/// (`{"error":…,"outcome":"wrong-state","payload":{"state":…}}`); a new error field changes the
/// envelope replay compares. ESS: callers compatible, readers unknown, history compatible.
#[test]
fn changed_recorded_error_shape_fails_gate() -> Result<()> {
    let (dir, _) = repository()?;
    let root = dir.path();
    edit(
        root,
        "- name: controlplane.host.RepositoryRegistrationStateConflict\n  summary: The command cannot act in the current state.\n  fields:\n",
        "- name: controlplane.host.RepositoryRegistrationStateConflict\n  summary: The command cannot act in the current state.\n  fields:\n  - name: hint\n    type: String\n",
    )?;
    refused_naming(
        root,
        "error/controlplane.host.RepositoryRegistrationStateConflict/field-added/hint",
    );
    Ok(())
}

/// Replay dispatches each recorded call with its recorded actor through the generated `admit`;
/// a revoked grant answers `not_granted` and `Store::open` fails ("generated command refused").
/// The committed fixture records an operator `PauseGoal`. ESS: callers breaking, history compatible.
#[test]
fn revoked_grant_that_replay_needs_fails_gate() -> Result<()> {
    let (dir, _) = repository()?;
    let root = dir.path();
    edit(
        root,
        "  - controlplane.host.StartGoal\n  - controlplane.host.PauseGoal\n  - controlplane.host.CancelGoal\n  - controlplane.host.DeleteGoal\n",
        "  - controlplane.host.StartGoal\n  - controlplane.host.CancelGoal\n  - controlplane.host.DeleteGoal\n",
    )?;
    refused_naming(
        root,
        "actor/controlplane.host.Operator/grant-removed/controlplane.host.PauseGoal",
    );
    Ok(())
}

/// The recorded outcome holds every published payload; mapping `reason` from another input
/// changes the envelope of every recorded repair. ESS: callers/readers unknown, history compatible.
#[test]
fn changed_recorded_payload_fails_gate() -> Result<()> {
    let (dir, _) = repository()?;
    let root = dir.path();
    edit(
        root,
        "      controlplane.host.RepairAssignmentApplied:\n        assignment_id: input.assignment_id\n        reason: input.reason\n",
        "      controlplane.host.RepairAssignmentApplied:\n        assignment_id: input.assignment_id\n        reason: input.implementor_run\n",
    )?;
    refused_naming(
        root,
        "command/controlplane.host.RepairAssignment/outcome-payload-changed/applied",
    );
    Ok(())
}

/// An acknowledgement binds an id only, so it keeps admitting whatever change later carries the
/// same id. Reviewed: `Blocked` leaves `repair.from`. Not reviewed: `repair` now lands in `Queued`
/// instead of `Implementing`, which silently replays every stored repair into another state.
#[test]
fn acknowledgement_does_not_admit_a_different_change_with_the_same_id() -> Result<()> {
    let id = "entity/controlplane.host.Assignment/transition-route-changed/repair";
    let (dir, baseline) = repository()?;
    let root = dir.path();
    acknowledge(root, &baseline, &[id])?;
    edit(
        root,
        "    - name: repair\n      from:\n      - Reviewing\n      - Blocked\n      to: Implementing\n",
        "    - name: repair\n      from:\n      - Reviewing\n      to: Implementing\n",
    )?;
    spec_history::check(root)?;
    edit(
        root,
        "    - name: repair\n      from:\n      - Reviewing\n      to: Implementing\n",
        "    - name: repair\n      from:\n      - Reviewing\n      to: Queued\n",
    )?;
    match spec_history::check(root) {
        Ok(summary) => panic!(
            "an acknowledgement reviewed for `Blocked, Reviewing -> Implementing` => `Reviewing -> Implementing` admitted `=> Reviewing -> Queued`: {summary}"
        ),
        Err(error) => assert!(format!("{error:#}").contains(id), "{error:#}"),
    }
    Ok(())
}

/// The baseline is fixed until someone moves it, while main keeps publishing. A route main added
/// after the baseline (acknowledged there, so stores on that release may hold `ready` from
/// `Blocked`) and a later branch removes is invisible: baseline and branch agree, the gate calls
/// the old acknowledgement stale and passes once it is dropped. Against the merge base with main
/// (the story's stated baseline) the removal is a `transition-route-changed`.
#[test]
fn route_published_after_baseline_then_removed_fails_gate() -> Result<()> {
    let id = "entity/controlplane.host.Assignment/transition-route-changed/ready";
    let (dir, baseline) = repository()?;
    let root = dir.path();
    let original = fs::read_to_string(root.join(HOST))?;
    edit(
        root,
        "    - name: ready\n      from:\n      - Reviewing\n",
        "    - name: ready\n      from:\n      - Reviewing\n      - Blocked\n",
    )?;
    acknowledge(root, &baseline, &[id])?;
    spec_history::check(root)?;
    git(root, &["add", "ess"])?;
    git(
        root,
        &[
            "commit",
            "--quiet",
            "--message",
            "Allow ready from Blocked (published)",
        ],
    )?;

    // The branch: restore the route and drop the now-"stale" acknowledgement.
    fs::write(root.join(HOST), original)?;
    acknowledge(root, &baseline, &[])?;
    match spec_history::check(root) {
        Ok(summary) => {
            panic!("removing a route published after the baseline passed the gate: {summary}")
        }
        Err(error) => assert!(format!("{error:#}").contains(id), "{error:#}"),
    }
    Ok(())
}
