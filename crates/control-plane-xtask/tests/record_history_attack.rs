//! Adversarial case for `record-history`.
//!
//! The unit's contract: a committed history that no longer replays is evidence, a replay failure
//! after a specification change is "a migration to write, not a fixture to refresh", and
//! `record-history` "refuses to replace a committed history that no longer replays"
//! (`crates/control-plane-xtask/README.md`). `recorded_history_replays` defines replaying as
//! `Store::open` succeeding *and* every view matching `recorded-history.views.json`.
//! `record-history` only checks the first half.
//!
//! A change that `Store::open` tolerates but that replays stored decisions into different state
//! reaches exactly that half-way point: ESS classifies `RepairAssignment`'s `attempt` increment
//! going from 1 to 2 as `outcome-sets-changed`, history `compatible`, so `spec-history-check`
//! passes it, the recorded outcomes still match, and only the views differ. This case builds that
//! state by recording views the current code does not replay to.
#[allow(dead_code)]
#[path = "../src/history.rs"]
mod history;

use anyhow::Result;
use std::{fs, path::Path};

#[test]
fn record_history_refuses_a_fixture_whose_replay_no_longer_matches_its_views() -> Result<()> {
    // Outside every home directory and Git work tree, as `record-history` requires.
    let base = Path::new(env!("CARGO_TARGET_TMPDIR"));
    fs::create_dir_all(base)?;
    let scratch = tempfile::Builder::new()
        .prefix("record-history-attack-")
        .tempdir_in(base)?;
    let root = scratch.path().join("root");
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for file in [history::HISTORY, history::VIEWS] {
        let to = root.join(file);
        fs::create_dir_all(to.parent().unwrap())?;
        fs::copy(repository.join(file), to)?;
    }
    let views_path = root.join(history::VIEWS);
    let mut views: serde_json::Value = serde_json::from_slice(&fs::read(&views_path)?)?;
    let repaired = views["AssignmentList"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["state"] == "Merged" && row["attempt"] == 2)
        .expect("a repaired, merged assignment in the recorded views");
    repaired["attempt"] = serde_json::json!(3);
    let mut doctored = serde_json::to_vec_pretty(&views)?;
    doctored.push(b'\n');
    fs::write(&views_path, &doctored)?;
    let history_before = fs::read(root.join(history::HISTORY))?;

    let result = history::run(&root, &scratch.path().join("work"));

    let history_after = fs::read(root.join(history::HISTORY))?;
    let views_after = fs::read(&views_path)?;
    assert!(
        result.is_err(),
        "record-history replaced a committed history whose replay no longer matches its recorded views"
    );
    assert!(
        history_after == history_before && views_after == doctored,
        "record-history rewrote the committed evidence"
    );
    Ok(())
}
