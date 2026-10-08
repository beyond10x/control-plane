//! Adversarial case for story:spec-owned-admission, pass 2: the rebuilt acknowledgements.
//!
//! Seven `outcome-added` reasons in `ess/spec-acknowledgements.json` end "The committed history
//! replays, and the re-recorded fixture records this refusal." The committed fixture
//! (`tests/fixtures/recorded-history.db`) is unchanged by this unit, and `recorded_history.rs`
//! says the opposite ("A refusal the committed fixture predates is acknowledged as
//! `outcome-added` and recorded by `declared_admission_refusals_are_recorded_and_replay`
//! instead"). A reason is the record a reviewer reads; this case holds it to the fixture.
use anyhow::{Context, Result};
use eventlog_core::{EventStore, StreamId, TenantId};
use eventlog_sqlite::SqliteEventStore;
use serde_json::Value;
use std::{collections::BTreeSet, fs, path::Path};

const HISTORY: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/recorded-history.db"
);
const ACKNOWLEDGEMENTS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../ess/spec-acknowledgements.json"
);
const CLAIM: &str = "the re-recorded fixture records this refusal";

async fn answered(path: &Path) -> Result<BTreeSet<(String, String)>> {
    let log = SqliteEventStore::open_existing(
        path.to_str().context("history path is not UTF-8")?,
        "control_plane",
    )
    .await?;
    let stream = StreamId::new(TenantId::new("local")?, "host", "state")?;
    let mut answered = BTreeSet::new();
    let mut read = 0u64;
    loop {
        let slice = log.read_stream(&stream, read, 1000).await?;
        for event in slice.events {
            read += 1;
            if let (Some(command), Some(outcome)) = (
                event.data["command"].as_str(),
                event.data["outcome"]["outcome"].as_str(),
            ) {
                answered.insert((command.to_owned(), outcome.to_owned()));
            }
        }
        if slice.end_of_stream {
            break;
        }
    }
    Ok(answered)
}

#[tokio::test]
async fn acknowledged_reasons_that_cite_the_fixture_are_in_the_fixture() -> Result<()> {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR"))?;
    let copy = dir.path().join("state.sqlite");
    fs::copy(HISTORY, &copy)?;
    let answered = answered(&copy).await?;
    assert!(
        answered.iter().any(|(_, outcome)| outcome == "applied"),
        "the fixture reader found no applied answer; it reads nothing"
    );
    let file: Value = serde_json::from_slice(&fs::read(ACKNOWLEDGEMENTS)?)?;
    let citing: Vec<(String, String)> = file["acknowledged"]
        .as_array()
        .context("acknowledged is an array")?
        .iter()
        .filter(|entry| entry["reason"].as_str().is_some_and(|r| r.contains(CLAIM)))
        .map(|entry| {
            (
                entry["change"]["subject"]
                    .as_str()
                    .unwrap_or_default()
                    .trim_start_matches("controlplane.host.")
                    .to_owned(),
                entry["change"]["changed"]["outcome"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
            )
        })
        .collect();
    let absent: Vec<_> = citing
        .iter()
        .filter(|pair| !answered.contains(*pair))
        .map(|(command, outcome)| format!("{command} {outcome}"))
        .collect();
    assert!(
        absent.is_empty(),
        "{} of {} acknowledgements say \"{CLAIM}\", and the committed fixture records none of \
         these: {absent:?}",
        absent.len(),
        citing.len()
    );
    Ok(())
}
