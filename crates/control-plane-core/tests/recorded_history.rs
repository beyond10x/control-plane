//! A host history recorded through `Store::execute` and committed under `tests/fixtures/` must
//! keep opening with `Store::open`, which replays every recorded decision through current
//! generated behavior and refuses any outcome that differs.
//!
//! The fixture is evidence about what earlier releases stored, so no test or gate writes it.
//! Regenerate it only deliberately, with
//! `cargo run --locked -p control-plane-xtask -- record-history --work-dir <DIR>`
//! (see `crates/control-plane-xtask/README.md`). A replay failure after a specification change is
//! a migration to write, not a fixture to refresh.
use anyhow::{Context, Result, ensure};
use control_plane_core::Store;
use controlplane_model::server::control_plane::{OPENAPI, ROUTES};
use eventlog_core::{EventStore, StreamId, TenantId};
use eventlog_sqlite::SqliteEventStore;
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

const HISTORY: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/recorded-history.db"
);
const VIEWS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/recorded-history.views.json"
);
const DISAGREES: &str = "generated behavior disagrees with durable history";

fn declared(method: &str, prefix: &str) -> BTreeSet<&'static str> {
    ROUTES
        .iter()
        .filter(|(m, _)| *m == method)
        .filter_map(|(_, path)| path.strip_prefix(prefix))
        .collect()
}

/// Every declared refusal response of every command, with the outcomes it may carry, read from
/// the generated contract. `2xx` answers are successes; `403` (no grant) and `501` (unfinished
/// realization) carry no declared outcome. Any other response must name its outcomes.
fn declared_refusals() -> Result<Vec<(&'static str, String, BTreeSet<String>)>> {
    fn references<'a>(value: &'a Value, found: &mut Vec<&'a str>) {
        match value {
            Value::Object(object) => {
                for (key, value) in object {
                    match (key.as_str(), value) {
                        ("$ref", Value::String(target)) => found.push(target),
                        _ => references(value, found),
                    }
                }
            }
            Value::Array(values) => values.iter().for_each(|value| references(value, found)),
            _ => {}
        }
    }
    let contract: Value = serde_json::from_str(OPENAPI)?;
    let mut refusals = Vec::new();
    for command in declared("POST", "/host/commands/") {
        let responses = contract["paths"][format!("/host/commands/{command}")]["post"]["responses"]
            .as_object()
            .with_context(|| format!("the contract declares no responses for {command}"))?;
        let prefix = format!("#/components/schemas/controlplane.host.{command}.");
        for (status, response) in responses {
            if status.starts_with('2') || matches!(status.as_str(), "403" | "501") {
                continue;
            }
            let mut found = Vec::new();
            references(response, &mut found);
            let outcomes: BTreeSet<String> = found
                .iter()
                .filter_map(|target| target.strip_prefix(&prefix)?.strip_suffix(".Response"))
                .map(str::to_owned)
                .collect();
            ensure!(
                !outcomes.is_empty(),
                "{command} {status} names no declared outcome this test can read"
            );
            refusals.push((command, status.clone(), outcomes));
        }
    }
    Ok(refusals)
}

/// A private copy: opening a store takes its lock and may write, and the fixture is evidence.
fn copy_history(dir: &Path) -> Result<PathBuf> {
    let copy = dir.join("state.sqlite");
    fs::copy(HISTORY, &copy).with_context(|| format!("read committed history {HISTORY}"))?;
    Ok(copy)
}

/// The recorded host decisions, read beneath `Store` straight from its Eventlog stream.
async fn decisions(path: &Path) -> Result<Vec<Value>> {
    let log = SqliteEventStore::open_existing(
        path.to_str().context("history path is not UTF-8")?,
        "control_plane",
    )
    .await?;
    let stream = StreamId::new(TenantId::new("local")?, "host", "state")?;
    let mut recorded = Vec::new();
    loop {
        let slice = log
            .read_stream(&stream, recorded.len() as u64, 1000)
            .await?;
        for event in slice.events {
            ensure!(
                event.name == "HostDecision" && event.schema_version == 1,
                "unexpected recorded event {} v{}",
                event.name,
                event.schema_version
            );
            recorded.push(event.data);
        }
        if slice.end_of_stream {
            break;
        }
    }
    Ok(recorded)
}

#[tokio::test]
async fn recorded_history_replays() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let copy = copy_history(dir.path())?;
    let recorded = decisions(&copy).await?;
    let applied: BTreeSet<&str> = recorded
        .iter()
        .filter(|decision| decision["outcome"].get("error").is_none())
        .filter_map(|decision| decision["command"].as_str())
        .collect();
    let missing: Vec<_> = declared("POST", "/host/commands/")
        .into_iter()
        .filter(|command| !applied.contains(command))
        .collect();
    ensure!(
        missing.is_empty(),
        "the recorded history never applies {missing:?}; extend record-history and regenerate"
    );
    let answered: BTreeSet<(&str, &str)> = recorded
        .iter()
        .filter_map(|decision| {
            Some((
                decision["command"].as_str()?,
                decision["outcome"]["outcome"].as_str()?,
            ))
        })
        .collect();
    // Every declared refusal outcome on its own: a response that carries several outcomes
    // (`DeleteGoal` 409: `paused`, `running`, `satisfied`) is not covered by one of them.
    let refusals: BTreeSet<(&str, String, String)> = declared_refusals()?
        .into_iter()
        .flat_map(|(command, status, outcomes)| {
            outcomes
                .into_iter()
                .map(move |outcome| (command, status.clone(), outcome))
        })
        .collect();
    let unrecorded: Vec<_> = refusals
        .iter()
        .filter(|(command, _, outcome)| !answered.contains(&(*command, outcome.as_str())))
        .map(|(command, status, outcome)| format!("{command} {status} {outcome}"))
        .collect();
    ensure!(
        unrecorded.is_empty(),
        "the recorded history never answers these declared refusal outcomes: {unrecorded:?}; extend record-history and regenerate"
    );
    let refused: BTreeSet<&str> = refusals.iter().map(|(command, _, _)| *command).collect();
    eprintln!(
        "{} decisions; {} commands applied; {} declared refusal outcomes of {} commands answered",
        recorded.len(),
        applied.len(),
        refusals.len(),
        refused.len()
    );

    let store = Store::open(&copy).await?;
    let expected: Value = serde_json::from_slice(&fs::read(VIEWS)?)?;
    let expected = expected
        .as_object()
        .context("recorded views are an object")?;
    let views = declared("GET", "/host/views/");
    assert_eq!(
        expected.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        views,
        "recorded views must name every declared view"
    );
    for view in views {
        assert_eq!(store.query(view)?, expected[view], "{view} after replay");
    }
    Ok(())
}

#[tokio::test]
async fn changed_recorded_outcome_fails_replay() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let original = copy_history(dir.path())?;
    let recorded = decisions(&original).await?;
    let bytes = fs::read(&original)?;
    let (from, to) = (br#""outcome":"applied""#, br#""outcome":"created""#);
    let altered = dir.path().join("altered.sqlite");
    let mut changed = None;
    // Patch one stored outcome in place and keep the first copy in which exactly one decision
    // reads back with a different outcome and nothing else changed; free SQLite pages may hold
    // stale row images that no reader sees.
    for at in (0..bytes.len() - from.len()).filter(|at| bytes[*at..].starts_with(from)) {
        let mut candidate = bytes.clone();
        candidate[at..at + to.len()].copy_from_slice(to);
        fs::write(&altered, &candidate)?;
        let reread = decisions(&altered).await?;
        let differing: Vec<_> = (0..recorded.len())
            .filter(|index| recorded[*index] != reread[*index])
            .collect();
        if let [index] = differing[..] {
            let (mut before, mut after) = (recorded[index].clone(), reread[index].clone());
            assert_eq!(before["outcome"]["outcome"], "applied");
            assert_eq!(after["outcome"]["outcome"], "created");
            before["outcome"]["outcome"] = Value::Null;
            after["outcome"]["outcome"] = Value::Null;
            assert_eq!(before, after, "only the recorded outcome may differ");
            changed = Some(index);
            break;
        }
    }
    let index = changed.context("no stored applied outcome could be altered")?;

    let error = match Store::open(&altered).await {
        Ok(_) => panic!("decision {index} was altered, but the history still opened"),
        Err(error) => format!("{error:#}"),
    };
    assert!(error.contains(DISAGREES), "{error}");
    Ok(())
}
