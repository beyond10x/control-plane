//! Adversary cases for story:conformance-evidence-record: the conformance runner now reads the
//! machine's clock. Everything a report says apart from time must still be a function of what was
//! run, and the time it states must be a coherent interval inside the run.

use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

/// Every case here writes `.scratch/conformance`; cases in one binary run on parallel threads.
static RUN: Mutex<()> = Mutex::new(());

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn wall_ms() -> u64 {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the machine clock reads after the epoch")
            .as_millis(),
    )
    .expect("epoch milliseconds fit in u64")
}

/// One `xtask conformance` run: (report.json, diagnostics.json).
fn conformance() -> (serde_json::Value, serde_json::Value) {
    let run = Command::new(env!("CARGO_BIN_EXE_control-plane-xtask"))
        .arg("conformance")
        .output()
        .expect("xtask starts");
    assert!(
        run.status.success(),
        "conformance run failed: {}\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let read = |name: &str| -> serde_json::Value {
        serde_json::from_str(
            &std::fs::read_to_string(root().join(".scratch/conformance").join(name))
                .expect("conformance output exists"),
        )
        .expect("conformance output is JSON")
    };
    (read("report.json"), read("diagnostics.json"))
}

fn mask_report(mut report: serde_json::Value) -> serde_json::Value {
    report["completed_at"] = serde_json::Value::Null;
    report
}

fn mask_diagnostics(mut diagnostics: serde_json::Value) -> serde_json::Value {
    diagnostics["started_at"] = serde_json::Value::Null;
    diagnostics["completed_at"] = serde_json::Value::Null;
    if let Some(scenarios) = diagnostics["scenarios"].as_array_mut() {
        for scenario in scenarios {
            scenario["duration_ms"] = serde_json::Value::Null;
        }
    }
    diagnostics
}

/// Two runs of one suite produce the same count report once its one time field is masked: the
/// wall clock moved time, and nothing else.
#[test]
fn two_runs_agree_on_the_count_report_apart_from_completed_at() {
    let _guard = RUN.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let (first, _) = conformance();
    let (second, _) = conformance();
    assert_eq!(mask_report(first), mask_report(second));
}

/// The diagnostics a run writes beside the report agree between two runs once `started_at`,
/// `completed_at` and every `duration_ms` are masked.
#[test]
fn two_runs_agree_on_the_diagnostics_apart_from_time() {
    let _guard = RUN.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let (_, first) = conformance();
    let (_, second) = conformance();
    let (first, second) = (mask_diagnostics(first), mask_diagnostics(second));
    if first != second {
        let a = first["scenarios"].as_array().cloned().unwrap_or_default();
        let b = second["scenarios"].as_array().cloned().unwrap_or_default();
        let differing: Vec<_> = a
            .iter()
            .zip(&b)
            .filter(|(x, y)| x != y)
            .map(|(x, _)| x["scenario"].clone())
            .take(5)
            .collect();
        panic!(
            "diagnostics differ apart from time; first differing scenarios: {differing:?}; \
             top-level equal apart from scenarios: {}",
            {
                let mut x = first.clone();
                let mut y = second.clone();
                x["scenarios"] = serde_json::Value::Null;
                y["scenarios"] = serde_json::Value::Null;
                x == y
            }
        );
    }
}

/// The run interval the diagnostics state lies inside the wall-clock window around the run,
/// `started_at <= completed_at`, the report's `completed_at` is the diagnostics' one, and the
/// scenario durations fit inside the interval (the rule ESS applies to a detailed count run).
#[test]
fn the_stated_run_interval_is_coherent_and_inside_the_run() {
    let _guard = RUN.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let before = wall_ms();
    let (report, diagnostics) = conformance();
    let after = wall_ms();
    let started = diagnostics["started_at"].as_u64().expect("started_at");
    let completed = diagnostics["completed_at"].as_u64().expect("completed_at");
    assert_eq!(report["completed_at"].as_u64(), Some(completed));
    assert!(
        before <= started && started <= completed && completed <= after,
        "interval [{started}, {completed}] outside run [{before}, {after}]"
    );
    let durations: u64 = diagnostics["scenarios"]
        .as_array()
        .expect("scenarios")
        .iter()
        .map(|s| s["duration_ms"].as_u64().expect("duration_ms"))
        .sum();
    assert!(
        durations <= completed - started,
        "scenario durations {durations} exceed the run interval {}",
        completed - started
    );
}
