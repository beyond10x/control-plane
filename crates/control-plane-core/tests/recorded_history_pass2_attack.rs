//! Adversarial case for the recorded-history fixture's refusal coverage, pass 2.
//!
//! The correction claims the recorded history "answers every declared refusal of every command"
//! and reads those refusals from the generated OpenAPI contract. `recorded_history_replays`
//! counts one answered outcome per refusal *response status*, though: a status that carries
//! several declared outcomes (`DeleteGoal` 409: `paused`, `running`, `satisfied`) is satisfied by
//! any one of them, and the outcomes it does not record are invisible to replay.
use anyhow::{Context, Result};
use controlplane_model::server::control_plane::{OPENAPI, ROUTES};
use eventlog_core::{EventStore, StreamId, TenantId};
use eventlog_sqlite::SqliteEventStore;
use serde_json::Value;
use std::{collections::BTreeSet, fs, path::Path};

const HISTORY: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/recorded-history.db"
);

/// The `(command, outcome)` of each `outcome-added` change acknowledged in
/// `ess/spec-acknowledgements.json`.
fn acknowledged_added_outcomes() -> Result<BTreeSet<(String, String)>> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../ess/spec-acknowledgements.json"
    );
    let file: Value = serde_json::from_slice(&fs::read(path)?)?;
    Ok(file["acknowledged"]
        .as_array()
        .context("acknowledged is an array")?
        .iter()
        .filter(|entry| entry["change"]["changed"]["kind"] == "outcome-added")
        .filter_map(|entry| {
            Some((
                entry["change"]["subject"]
                    .as_str()?
                    .trim_start_matches("controlplane.host.")
                    .to_owned(),
                entry["change"]["changed"]["outcome"].as_str()?.to_owned(),
            ))
        })
        .collect())
}

fn references(value: &Value, found: &mut BTreeSet<String>) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                match (key.as_str(), value) {
                    ("$ref", Value::String(target)) => {
                        found.insert(target.clone());
                    }
                    _ => references(value, found),
                }
            }
        }
        Value::Array(values) => values.iter().for_each(|value| references(value, found)),
        _ => {}
    }
}

/// Every declared refusal outcome of every command, one entry per outcome.
fn declared_refusal_outcomes() -> Result<BTreeSet<(String, String)>> {
    let contract: Value = serde_json::from_str(OPENAPI)?;
    let mut declared = BTreeSet::new();
    for command in ROUTES
        .iter()
        .filter(|(method, _)| *method == "POST")
        .filter_map(|(_, path)| path.strip_prefix("/host/commands/"))
    {
        let responses = contract["paths"][format!("/host/commands/{command}")]["post"]["responses"]
            .as_object()
            .with_context(|| format!("no responses for {command}"))?;
        let prefix = format!("#/components/schemas/controlplane.host.{command}.");
        for (status, response) in responses {
            if status.starts_with('2') || matches!(status.as_str(), "403" | "501") {
                continue;
            }
            let mut found = BTreeSet::new();
            references(response, &mut found);
            for target in found {
                if let Some(outcome) = target
                    .strip_prefix(&prefix)
                    .and_then(|rest| rest.strip_suffix(".Response"))
                {
                    declared.insert((command.to_owned(), outcome.to_owned()));
                }
            }
        }
    }
    Ok(declared)
}

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
async fn recorded_history_answers_every_declared_refusal_outcome() -> Result<()> {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR"))?;
    let copy = dir.path().join("state.sqlite");
    fs::copy(HISTORY, &copy)?;
    let declared = declared_refusal_outcomes()?;
    assert!(
        !declared.is_empty(),
        "the contract declares no refusal outcomes"
    );
    let mut answered = answered(&copy).await?;
    // A refusal the committed fixture predates is acknowledged as `outcome-added` and recorded,
    // with its replay, by `declared_admission_refusals_are_recorded_and_replay` (src/tests.rs),
    // which requires its refusals to equal that acknowledged set.
    answered.extend(acknowledged_added_outcomes()?);
    let unrecorded: Vec<_> = declared.difference(&answered).collect();
    assert!(
        unrecorded.is_empty(),
        "the recorded history answers {} of {} declared refusal outcomes; never answered: {unrecorded:?}",
        declared.len() - unrecorded.len(),
        declared.len()
    );
    Ok(())
}
