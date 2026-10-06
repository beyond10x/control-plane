//! Specification changes that could break replay of stored history fail unless acknowledged.
//!
//! `ess/spec-acknowledgements.json` records a baseline commit: the specification whose stored
//! decisions this tree must still replay. The gate materialises the baseline's `ess/` tree from
//! Git into scratch and asks the released ESS CLI to classify every change against `ess/`. A change
//! whose `history` verdict is `breaking` or `unknown` fails unless the file acknowledges its id
//! with a reason, and an acknowledgement that no longer names such a change fails as stale.
//! Moving the baseline is the deliberate reset that empties the acknowledgements.
use anyhow::{Context, Result, bail, ensure};
use serde_json::{Map, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    fs,
    path::Path,
    process::Command,
};

pub const ACKNOWLEDGEMENTS: &str = "ess/spec-acknowledgements.json";
const FORMAT: &str = "control-plane-spec-acknowledgements/1";

pub fn run(root: &Path) -> Result<()> {
    println!("{}", check(root)?);
    Ok(())
}

struct Acknowledgements {
    baseline: String,
    /// Acknowledged change id and the reviewer's reason.
    acknowledged: BTreeMap<String, String>,
}

fn closed<'a>(value: &'a Value, what: &str, fields: &[&str]) -> Result<&'a Map<String, Value>> {
    let object = value
        .as_object()
        .with_context(|| format!("{what} must be a JSON object"))?;
    for key in object.keys() {
        ensure!(
            fields.contains(&key.as_str()),
            "{what} has unknown field `{key}`"
        );
    }
    Ok(object)
}

fn text<'a>(object: &'a Map<String, Value>, what: &str, field: &str) -> Result<&'a str> {
    object
        .get(field)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .with_context(|| format!("{what} needs a nonempty string `{field}`"))
}

fn acknowledgements(root: &Path) -> Result<Acknowledgements> {
    let path = root.join(ACKNOWLEDGEMENTS);
    let bytes = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
    let document: Value = serde_json::from_slice(&bytes)
        .with_context(|| format!("{ACKNOWLEDGEMENTS} is not JSON"))?;
    let object = closed(
        &document,
        ACKNOWLEDGEMENTS,
        &["format", "baseline", "acknowledged"],
    )?;
    ensure!(
        object.get("format").and_then(Value::as_str) == Some(FORMAT),
        "{ACKNOWLEDGEMENTS} must declare format `{FORMAT}`"
    );
    let baseline = text(object, ACKNOWLEDGEMENTS, "baseline")?;
    ensure!(
        baseline.len() == 40
            && baseline
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
        "{ACKNOWLEDGEMENTS} baseline must be a full lowercase commit id, not a moving name: {baseline}"
    );
    let mut acknowledged = BTreeMap::new();
    for entry in object
        .get("acknowledged")
        .and_then(Value::as_array)
        .with_context(|| format!("{ACKNOWLEDGEMENTS} needs an `acknowledged` array"))?
    {
        let what = "an acknowledgement";
        let entry = closed(entry, what, &["id", "reason"])?;
        let id = text(entry, what, "id")?;
        let reason = text(entry, what, "reason")?;
        ensure!(
            acknowledged
                .insert(id.to_owned(), reason.to_owned())
                .is_none(),
            "{ACKNOWLEDGEMENTS} acknowledges {id} twice"
        );
    }
    Ok(Acknowledgements {
        baseline: baseline.to_owned(),
        acknowledged,
    })
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .context("start git")?;
    ensure!(
        output.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(output.stdout)
}

/// Write the baseline commit's `ess/` tree under `into`, byte for byte, regular files only.
/// ESS itself selects the files `ess-inputs.yaml` lists and reads nothing else.
fn materialise(root: &Path, baseline: &str, into: &Path) -> Result<()> {
    git(root, &["cat-file", "-e", &format!("{baseline}^{{commit}}")]).with_context(|| {
        format!("baseline commit {baseline} is not in this clone; fetch the full history")
    })?;
    let listing = git(root, &["ls-tree", "-r", "-z", baseline, "--", "ess/"])?;
    let mut inputs = false;
    for entry in listing.split(|byte| *byte == 0).filter(|e| !e.is_empty()) {
        let entry = std::str::from_utf8(entry).context("baseline path is not UTF-8")?;
        let (meta, path) = entry
            .split_once('\t')
            .context("unreadable git ls-tree entry")?;
        let mut meta = meta.split(' ');
        let (mode, kind, object) = (meta.next(), meta.next(), meta.next());
        ensure!(
            matches!(mode, Some("100644" | "100755")) && kind == Some("blob"),
            "baseline {path} is not a regular file"
        );
        let object = object.context("unreadable git ls-tree entry")?;
        let relative = path.strip_prefix("ess/").context("path outside ess/")?;
        ensure!(
            relative
                .split('/')
                .all(|part| !part.is_empty() && part != "." && part != ".."),
            "baseline path is not a plain relative path: {path}"
        );
        inputs |= relative == "ess-inputs.yaml";
        let destination = into.join(relative);
        fs::create_dir_all(destination.parent().context("baseline file parent")?)?;
        fs::write(destination, git(root, &["cat-file", "blob", object])?)?;
    }
    ensure!(
        inputs,
        "baseline commit {baseline} has no ess/ess-inputs.yaml"
    );
    Ok(())
}

struct Change {
    history: String,
    detail: String,
}

/// Every change ESS reports between the baseline and `ess/`, by id. Unknown vocabulary refuses.
fn changes(root: &Path, baseline: &Path) -> Result<BTreeMap<String, Change>> {
    let from = baseline.to_str().context("baseline path is not UTF-8")?;
    let output = Command::new("ess")
        .current_dir(root)
        .args(["verify", "diff", "--from", from, "--to", "ess"])
        .args(["--compatibility", "--format", "json"])
        .output()
        .context("start released ESS CLI")?;
    ensure!(
        output.status.success(),
        "ess verify diff failed ({}):\n{}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let delta: Value =
        serde_json::from_slice(&output.stdout).context("ess verify diff did not print JSON")?;
    ensure!(
        delta["format"]
            .as_str()
            .is_some_and(|format| format.starts_with("ess-diff/")),
        "ess verify diff printed an unexpected format: {}",
        delta["format"]
    );
    let mut changes = BTreeMap::new();
    for change in delta["changes"]
        .as_array()
        .context("ess verify diff printed no `changes` array")?
    {
        let id = change["id"].as_str().context("a change has no id")?;
        let history = change["compatibility"]["history"]
            .as_str()
            .filter(|verdict| matches!(*verdict, "compatible" | "unknown" | "breaking"))
            .with_context(|| format!("{id} has no recognised history verdict"))?;
        let changed = &change["change"]["changed"];
        let detail = match (changed["before"].as_str(), changed["after"].as_str()) {
            (Some(before), Some(after)) => format!("{before} => {after}"),
            _ => change["change"].to_string(),
        };
        ensure!(
            changes
                .insert(
                    id.to_owned(),
                    Change {
                        history: history.to_owned(),
                        detail,
                    },
                )
                .is_none(),
            "ess verify diff reported {id} twice"
        );
    }
    Ok(changes)
}

/// Run the gate; the summary names the baseline and the acknowledged changes.
pub fn check(root: &Path) -> Result<String> {
    let acknowledgements = acknowledgements(root)?;
    let baseline = &acknowledgements.baseline;
    let scratch = root.join(".scratch");
    fs::create_dir_all(&scratch)?;
    let fresh = tempfile::Builder::new()
        .prefix("spec-history-")
        .tempdir_in(scratch)?;
    let from = fresh.path().join("ess");
    materialise(root, baseline, &from)?;
    let changes = changes(root, &from)?;
    let gating: BTreeMap<_, _> = changes
        .iter()
        .filter(|(_, change)| change.history != "compatible")
        .collect();

    let mut refusal = String::new();
    let unacknowledged: Vec<_> = gating
        .iter()
        .filter(|(id, _)| !acknowledgements.acknowledged.contains_key(**id))
        .collect();
    if !unacknowledged.is_empty() {
        writeln!(
            refusal,
            "specification changes since baseline {baseline} could break stored history:"
        )?;
        for (id, change) in &unacknowledged {
            writeln!(
                refusal,
                "  {id} (history: {}): {}",
                change.history, change.detail
            )?;
        }
        writeln!(
            refusal,
            "Restore the specification, or migrate stored history and acknowledge each id with a reason in {ACKNOWLEDGEMENTS}."
        )?;
    }
    let ids: BTreeSet<&String> = gating.keys().copied().collect();
    let stale: Vec<_> = acknowledgements
        .acknowledged
        .keys()
        .filter(|id| !ids.contains(id))
        .collect();
    for id in &stale {
        let why = match changes.get(*id) {
            Some(change) => format!("history is {} and needs no acknowledgement", change.history),
            None => format!("no longer in the diff against baseline {baseline}"),
        };
        writeln!(
            refusal,
            "stale acknowledgement in {ACKNOWLEDGEMENTS}: {id}: {why}; remove it"
        )?;
    }
    if !refusal.is_empty() {
        bail!("{}", refusal.trim_end());
    }
    Ok(format!(
        "specification history gate passed: {} change(s) since baseline {baseline}, {} could affect stored history, {} acknowledged",
        changes.len(),
        gating.len(),
        acknowledgements.acknowledged.len()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, process::Command};

    const REPAIR: &str = "entity/controlplane.host.Assignment/transition-route-changed/repair";
    const SPECIFICATION: [(&str, &str); 4] = [
        (
            "ess-inputs.yaml",
            include_str!("../../../ess/ess-inputs.yaml"),
        ),
        ("system.yaml", include_str!("../../../ess/system.yaml")),
        (
            "components.yaml",
            include_str!("../../../ess/components.yaml"),
        ),
        (
            "domains/host.yaml",
            include_str!("../../../ess/domains/host.yaml"),
        ),
    ];

    fn git(root: &Path, args: &[&str]) -> Result<String> {
        let output = Command::new("git").current_dir(root).args(args).output()?;
        anyhow::ensure!(
            output.status.success(),
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(String::from_utf8(output.stdout)?.trim().to_owned())
    }

    /// A repository whose only commit is the current specification: the recorded baseline.
    fn repository() -> Result<(tempfile::TempDir, String)> {
        let scratch =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.scratch/spec-history-tests");
        fs::create_dir_all(&scratch)?;
        let dir = tempfile::tempdir_in(scratch)?;
        let root = dir.path();
        fs::create_dir_all(root.join("ess/domains"))?;
        for (name, content) in SPECIFICATION {
            fs::write(root.join("ess").join(name), content)?;
        }
        git(root, &["init", "--quiet", "--initial-branch=main"])?;
        git(root, &["add", "ess"])?;
        git(
            root,
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "core.hooksPath=/dev/null",
                "commit",
                "--quiet",
                "--message",
                "Baseline specification",
            ],
        )?;
        let baseline = git(root, &["rev-parse", "HEAD"])?;
        Ok((dir, baseline))
    }

    fn acknowledge(root: &Path, baseline: &str, ids: &[&str]) -> Result<()> {
        let acknowledged: Vec<_> = ids
            .iter()
            .map(|id| serde_json::json!({"id": id, "reason": "reviewed in the gate test"}))
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

    /// The change the ESS hardening review classified `unknown`: `Blocked` leaves `repair.from`.
    fn remove_blocked_from_repair(root: &Path) -> Result<()> {
        let path = root.join("ess/domains/host.yaml");
        let before = fs::read_to_string(&path)?;
        let route = "    - name: repair\n      from:\n      - Reviewing\n      - Blocked\n";
        assert_eq!(before.matches(route).count(), 1, "repair route not found");
        fs::write(
            &path,
            before.replace(
                route,
                "    - name: repair\n      from:\n      - Reviewing\n",
            ),
        )?;
        Ok(())
    }

    #[test]
    fn unacknowledged_spec_change_fails_gate() -> Result<()> {
        let (dir, baseline) = repository()?;
        let root = dir.path();
        acknowledge(root, &baseline, &[])?;
        check(root)?;
        remove_blocked_from_repair(root)?;
        let error = format!("{:#}", check(root).unwrap_err());
        assert!(error.contains(REPAIR), "{error}");
        assert!(error.contains("history: unknown"), "{error}");
        // The same edit changes RepairAssignment's outcome subject, which ESS classifies
        // `compatible` for history: callers and readers are not this gate's concern.
        assert!(
            !error.contains("command/controlplane.host.RepairAssignment"),
            "{error}"
        );
        eprintln!("{error}");
        Ok(())
    }

    #[test]
    fn acknowledged_spec_change_passes_gate() -> Result<()> {
        let (dir, baseline) = repository()?;
        let root = dir.path();
        acknowledge(root, &baseline, &[REPAIR])?;
        remove_blocked_from_repair(root)?;
        let summary = check(root)?;
        assert!(summary.contains("1 acknowledged"), "{summary}");
        eprintln!("{summary}");
        Ok(())
    }

    #[test]
    fn stale_acknowledgement_fails_gate() -> Result<()> {
        let (dir, baseline) = repository()?;
        let root = dir.path();
        acknowledge(root, &baseline, &[REPAIR])?;
        let error = format!("{:#}", check(root).unwrap_err());
        assert!(error.contains("stale acknowledgement"), "{error}");
        assert!(error.contains(REPAIR), "{error}");
        eprintln!("{error}");
        Ok(())
    }
}
