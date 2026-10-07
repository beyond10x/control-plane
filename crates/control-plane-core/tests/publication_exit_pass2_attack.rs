//! Adversarial cases for story:publication-exit, wave 4 pass 2.
//!
//! The unit re-recorded `tests/fixtures/recorded-history.db` with its own code, so
//! `recorded_history_replays` now replays history written by the behaviour it checks. These cases
//! replay what the unit's base commit stored, read from Git, through the current code.
use control_plane_core::Store;
use serde_json::Value;
use std::{path::Path, process::Command};

/// The base commit of story:publication-exit (wave 4).
const BASE: &str = "e88ee4f";

/// A file as the base commit stored it.
fn at_base(path: &str) -> Vec<u8> {
    let output = Command::new("git")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["show", &format!("{BASE}:crates/control-plane-core/{path}")])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git show {BASE}:{path}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

/// History an operator's store holds from the release before this change must open with this
/// change and show the same views: `Store::open` replays every decision through current
/// generated behaviour and refuses an outcome that differs.
#[tokio::test]
async fn history_recorded_before_publication_exit_still_replays() {
    let temp = tempfile::tempdir().unwrap();
    let copy = temp.path().join("state.sqlite");
    std::fs::write(&copy, at_base("tests/fixtures/recorded-history.db")).unwrap();
    let store = match Store::open(Path::new(&copy)).await {
        Ok(store) => store,
        Err(error) => panic!("history recorded at {BASE} no longer opens: {error:#}"),
    };
    let expected: Value =
        serde_json::from_slice(&at_base("tests/fixtures/recorded-history.views.json")).unwrap();
    for (view, rows) in expected.as_object().unwrap() {
        assert_eq!(
            &store.query(view).unwrap(),
            rows,
            "{view} after replaying the history recorded at {BASE}"
        );
    }
}
