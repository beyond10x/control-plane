use std::{
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn wall_ms() -> u64 {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the machine clock reads after the epoch")
            .as_millis(),
    )
    .expect("epoch milliseconds fit in u64")
}

/// The report AEP imports as conformance evidence states when the run happened, so its
/// `completed_at` is the machine's clock at the end of the run, not the runner's 2023 default.
#[test]
fn conformance_report_carries_run_instant() -> anyhow::Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let before = wall_ms();
    let run = Command::new(env!("CARGO_BIN_EXE_control-plane-xtask"))
        .arg("conformance")
        .output()?;
    let after = wall_ms();
    assert!(
        run.status.success(),
        "conformance run failed: {}\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let report: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
        root.join(".scratch/conformance/report.json"),
    )?)?;
    let completed_at = report["completed_at"]
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("report carries no completed_at: {report}"))?;
    assert!(
        (before..=after).contains(&completed_at),
        "completed_at {completed_at} lies outside the run [{before}, {after}]"
    );
    Ok(())
}
