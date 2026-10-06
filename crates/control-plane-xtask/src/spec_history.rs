//! Specification changes that could break replay of stored history fail unless acknowledged.
//!
//! `Store::open` re-invokes every recorded command with its recorded body and actor through the
//! current generated contract and refuses to start when any answer differs from the recorded one.
//! A change can therefore break a store whatever ESS's `history` column says, so the gate asks the
//! released ESS CLI to classify every change between a baseline and `ess/` and refuses each change
//! that is not `compatible` for callers, readers and history alike, unless it is purely additive
//! (see [`additive`]). An acknowledgement in `ess/spec-acknowledgements.json` admits exactly the
//! change it reviewed: the id and the `change` object ESS printed for it.
//!
//! The baseline is the merge base of `HEAD` with `origin/main` (`main` in a clone without that
//! remote) when that commit holds `ess/ess-inputs.yaml`, otherwise the commit the acknowledgement
//! file records; when both exist, the descendant of the other wins.
use anyhow::{Context, Result, bail, ensure};
use serde_json::{Map, Value, json};
use std::{fmt::Write as _, fs, path::Path, process::Command};

pub const ACKNOWLEDGEMENTS: &str = "ess/spec-acknowledgements.json";
const FORMAT: &str = "control-plane-spec-acknowledgements/1";
const MAINLINES: [&str; 2] = ["origin/main", "main"];

pub fn run(root: &Path) -> Result<()> {
    println!("{}", check(root)?);
    Ok(())
}

struct Acknowledgement {
    id: String,
    /// The `change` object `ess verify diff` printed for the reviewed change.
    change: Value,
}

struct Acknowledgements {
    baseline: String,
    acknowledged: Vec<Acknowledgement>,
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

fn parse_acknowledgements(bytes: &[u8], origin: &str) -> Result<Acknowledgements> {
    let document: Value =
        serde_json::from_slice(bytes).with_context(|| format!("{origin} is not JSON"))?;
    let object = closed(&document, origin, &["format", "baseline", "acknowledged"])?;
    ensure!(
        object.get("format").and_then(Value::as_str) == Some(FORMAT),
        "{origin} must declare format `{FORMAT}`"
    );
    let baseline = text(object, origin, "baseline")?;
    ensure!(
        baseline.len() == 40
            && baseline
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
        "{origin} baseline must be a full lowercase commit id, not a moving name: {baseline}"
    );
    let mut acknowledged: Vec<Acknowledgement> = Vec::new();
    for entry in object
        .get("acknowledged")
        .and_then(Value::as_array)
        .with_context(|| format!("{origin} needs an `acknowledged` array"))?
    {
        let entry = closed(entry, "an acknowledgement", &["id", "change", "reason"])?;
        let id = text(entry, "an acknowledgement", "id")?;
        text(entry, "an acknowledgement", "reason")?;
        let change = entry
            .get("change")
            .filter(|change| change.is_object())
            .with_context(|| {
                format!(
                    "the acknowledgement of {id} must record the reviewed `change` object exactly as \
                     `ess verify diff --compatibility --format json` printed it"
                )
            })?;
        ensure!(
            acknowledged.iter().all(|other| other.id != id),
            "{origin} acknowledges {id} twice"
        );
        acknowledged.push(Acknowledgement {
            id: id.to_owned(),
            change: change.clone(),
        });
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

/// Whether `git` succeeds; failure is an answer here, not an error.
fn git_succeeds(root: &Path, args: &[&str]) -> Result<bool> {
    Ok(Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .context("start git")?
        .status
        .success())
}

fn commit_of(root: &Path, revision: &str) -> Result<String> {
    Ok(
        String::from_utf8(git(root, &["rev-parse", "--verify", revision])?)?
            .trim()
            .to_owned(),
    )
}

/// Where the baseline came from, for the summary and for every refusal.
struct Baseline {
    commit: String,
    source: String,
}

/// The newest specification `HEAD` is known to descend from: the mainline merge base when it holds
/// a specification, the recorded baseline otherwise, the descendant when both exist.
fn baseline(root: &Path, recorded: &str) -> Result<Baseline> {
    ensure!(
        git_succeeds(root, &["cat-file", "-e", &format!("{recorded}^{{commit}}")])?,
        "recorded baseline commit {recorded} is not in this clone; fetch the full history"
    );
    let mut merge_base = None;
    for mainline in MAINLINES {
        if !git_succeeds(
            root,
            &[
                "rev-parse",
                "--verify",
                "--quiet",
                &format!("{mainline}^{{commit}}"),
            ],
        )? {
            continue;
        }
        if let Ok(base) = git(root, &["merge-base", "HEAD", mainline]) {
            let base = String::from_utf8(base)?.trim().to_owned();
            if git_succeeds(
                root,
                &["cat-file", "-e", &format!("{base}:ess/ess-inputs.yaml")],
            )? {
                merge_base = Some((base, mainline));
            }
        }
        break;
    }
    let recorded = Baseline {
        commit: commit_of(root, &format!("{recorded}^{{commit}}"))?,
        source: format!("recorded in {ACKNOWLEDGEMENTS}"),
    };
    let Some((base, mainline)) = merge_base else {
        return Ok(recorded);
    };
    let merged = Baseline {
        commit: base,
        source: format!("merge base with {mainline}"),
    };
    if git_succeeds(
        root,
        &[
            "merge-base",
            "--is-ancestor",
            &recorded.commit,
            &merged.commit,
        ],
    )? {
        Ok(merged)
    } else if git_succeeds(
        root,
        &[
            "merge-base",
            "--is-ancestor",
            &merged.commit,
            &recorded.commit,
        ],
    )? {
        Ok(recorded)
    } else {
        bail!(
            "the recorded baseline {} and the {} {} do not descend from one another; move the recorded baseline",
            recorded.commit,
            merged.source,
            merged.commit
        )
    }
}

/// Write the baseline commit's `ess/` tree under `into`, byte for byte, regular files only.
/// ESS itself selects the files `ess-inputs.yaml` lists and reads nothing else.
fn materialise(root: &Path, baseline: &str, into: &Path) -> Result<()> {
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

/// One change as `ess verify diff --compatibility --format json` reports it.
struct Change {
    id: String,
    relation: String,
    callers: String,
    readers: String,
    history: String,
    change: Value,
}

impl Change {
    fn category(&self) -> &str {
        self.change["category"].as_str().unwrap_or_default()
    }
    fn changed(&self, field: &str) -> &str {
        self.change["changed"][field].as_str().unwrap_or_default()
    }
    fn compatible(&self) -> bool {
        [&self.callers, &self.readers, &self.history]
            .iter()
            .all(|verdict| *verdict == "compatible")
    }
    fn detail(&self) -> String {
        let changed = &self.change["changed"];
        match (changed["before"].as_str(), changed["after"].as_str()) {
            (Some(before), Some(after)) => format!("{before} => {after}"),
            _ => changed.to_string(),
        }
    }
}

/// A purely additive change that cannot alter the answer to an already recorded call: a new
/// enum or union variant, or a new `Optional<…>` command input, entity field or view field.
/// Recorded calls never carry the new value, and whatever existing outcome would start reading
/// it is reported as a change of its own. New commands, views, events, entities, errors, types,
/// actors and grants are already `compatible` in every dimension.
fn additive(change: &Change) -> bool {
    let optional = change.changed("type_ref").starts_with("Optional<");
    match (change.category(), change.changed("kind")) {
        ("type", "variant-added") => change.relation == "expanded",
        ("command", "input-added") | ("entity" | "view", "field-added") => optional,
        _ => false,
    }
}

/// The transition change that a command's `outcome-subject-changed` merely restates: the outcome
/// still moves the same entity through the same transition, and only that transition's route moved.
fn restated<'a>(change: &Change, changes: &'a [Change]) -> Option<&'a Change> {
    if change.category() != "command" || change.changed("kind") != "outcome-subject-changed" {
        return None;
    }
    let (before, after) = (change.changed("before"), change.changed("after"));
    changes.iter().find(|route| {
        route.category() == "entity" && route.changed("kind") == "transition-route-changed" && {
            let subject = route.change["subject"].as_str().unwrap_or_default();
            let via = |side: &str| {
                format!(
                    "moves {subject} via {} ({})",
                    route.changed("transition"),
                    route.changed(side)
                )
            };
            let (from, to) = (via("before"), via("after"));
            before.contains(&from) && before.replacen(&from, &to, 1) == after
        }
    })
}

/// Every change ESS reports between the baseline and `ess/`. Unknown vocabulary refuses.
fn changes(root: &Path, baseline: &Path) -> Result<Vec<Change>> {
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
    let mut changes: Vec<Change> = Vec::new();
    for change in delta["changes"]
        .as_array()
        .context("ess verify diff printed no `changes` array")?
    {
        let id = change["id"].as_str().context("a change has no id")?;
        let verdict = |dimension: &str| {
            change["compatibility"][dimension]
                .as_str()
                .filter(|verdict| matches!(*verdict, "compatible" | "unknown" | "breaking"))
                .map(str::to_owned)
                .with_context(|| format!("{id} has no recognised {dimension} verdict"))
        };
        let relation = change["relation"]
            .as_str()
            .filter(|relation| matches!(*relation, "expanded" | "narrowed" | "changed"))
            .with_context(|| format!("{id} has no recognised relation"))?;
        ensure!(change["change"].is_object(), "{id} has no `change` object");
        ensure!(
            changes.iter().all(|other| other.id != id),
            "ess verify diff reported {id} twice"
        );
        changes.push(Change {
            id: id.to_owned(),
            relation: relation.to_owned(),
            callers: verdict("callers")?,
            readers: verdict("readers")?,
            history: verdict("history")?,
            change: change["change"].clone(),
        });
    }
    Ok(changes)
}

/// The acknowledgements published with the baseline itself; they need no change in the diff.
fn published(root: &Path, baseline: &str) -> Vec<Acknowledgement> {
    git(root, &["show", &format!("{baseline}:{ACKNOWLEDGEMENTS}")])
        .ok()
        .and_then(|bytes| parse_acknowledgements(&bytes, "the baseline's acknowledgements").ok())
        .map(|file| file.acknowledged)
        .unwrap_or_default()
}

/// Run the gate; the summary names the baseline and the acknowledged changes.
pub fn check(root: &Path) -> Result<String> {
    let path = root.join(ACKNOWLEDGEMENTS);
    let acknowledgements = parse_acknowledgements(
        &fs::read(&path).with_context(|| format!("read {}", path.display()))?,
        ACKNOWLEDGEMENTS,
    )?;
    let baseline = baseline(root, &acknowledgements.baseline)?;
    let scratch = root.join(".scratch");
    fs::create_dir_all(&scratch)?;
    let fresh = tempfile::Builder::new()
        .prefix("spec-history-")
        .tempdir_in(scratch)?;
    let from = fresh.path().join("ess");
    materialise(root, &baseline.commit, &from)?;
    let changes = changes(root, &from)?;
    let gating: Vec<&Change> = changes
        .iter()
        .filter(|change| !change.compatible() && !additive(change))
        .collect();
    // A restatement is decided by the change it restates.
    let reviewed: Vec<&Change> = gating
        .iter()
        .copied()
        .filter(|change| restated(change, &changes).is_none_or(|route| route.compatible()))
        .collect();
    let restating = |route: &Change| -> Vec<&str> {
        gating
            .iter()
            .filter(|change| restated(change, &changes).is_some_and(|r| r.id == route.id))
            .map(|change| change.id.as_str())
            .collect()
    };
    let at = format!("baseline {} ({})", baseline.commit, baseline.source);

    let mut refusal = String::new();
    let mut acknowledged = 0;
    for change in &reviewed {
        let entry = acknowledgements
            .acknowledged
            .iter()
            .find(|entry| entry.id == change.id);
        if entry.is_some_and(|entry| entry.change == change.change) {
            acknowledged += 1;
            continue;
        }
        if refusal.is_empty() {
            writeln!(
                refusal,
                "specification changes since {at} could break replay of stored decisions:"
            )?;
        }
        writeln!(
            refusal,
            "  {} (callers: {}, readers: {}, history: {}): {}",
            change.id,
            change.callers,
            change.readers,
            change.history,
            change.detail()
        )?;
        for restatement in restating(change) {
            writeln!(refusal, "    restated by {restatement}")?;
        }
        if let Some(entry) = entry {
            writeln!(
                refusal,
                "    the acknowledgement of this id reviewed a different change: {}",
                entry.change
            )?;
        }
        writeln!(
            refusal,
            "    acknowledge after review: {}",
            json!({"id": change.id, "change": change.change, "reason": "<why stored history still replays>"})
        )?;
    }
    if !refusal.is_empty() {
        writeln!(
            refusal,
            "Restore the specification, or migrate stored history and acknowledge each reviewed change in {ACKNOWLEDGEMENTS}."
        )?;
    }
    let inherited = published(root, &baseline.commit);
    for entry in &acknowledgements.acknowledged {
        let current = reviewed.iter().find(|change| change.id == entry.id);
        if current.is_some()
            || inherited
                .iter()
                .any(|old| old.id == entry.id && old.change == entry.change)
        {
            continue;
        }
        let why = match changes.iter().find(|change| change.id == entry.id) {
            Some(change) if restated(change, &changes).is_some() => {
                "it restates a transition change; acknowledge that change instead".to_owned()
            }
            Some(_) => "the change needs no acknowledgement".to_owned(),
            None => format!("no longer in the diff against {at}"),
        };
        writeln!(
            refusal,
            "stale acknowledgement in {ACKNOWLEDGEMENTS}: {}: {why}; remove it",
            entry.id
        )?;
    }
    if !refusal.is_empty() {
        bail!("{}", refusal.trim_end());
    }
    Ok(format!(
        "specification history gate passed: {} change(s) since {at}, {} could break replay, {acknowledged} acknowledged",
        changes.len(),
        gating.len(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn run_git(root: &Path, args: &[&str]) -> Result<String> {
        let mut command = vec![
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
        ];
        command.extend_from_slice(args);
        Ok(String::from_utf8(git(root, &command)?)?.trim().to_owned())
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
        run_git(root, &["init", "--quiet", "--initial-branch=main"])?;
        run_git(root, &["add", "ess"])?;
        run_git(
            root,
            &["commit", "--quiet", "--message", "Baseline specification"],
        )?;
        let baseline = run_git(root, &["rev-parse", "HEAD"])?;
        Ok((dir, baseline))
    }

    fn acknowledge(root: &Path, baseline: &str, reviewed: &[(&str, Value)]) -> Result<()> {
        let acknowledged: Vec<_> = reviewed
            .iter()
            .map(|(id, change)| json!({"id": id, "change": change, "reason": "reviewed in the gate test"}))
            .collect();
        let document = json!({
            "format": FORMAT,
            "baseline": baseline,
            "acknowledged": acknowledged,
        });
        fs::write(
            root.join(ACKNOWLEDGEMENTS),
            serde_json::to_vec_pretty(&document)?,
        )?;
        Ok(())
    }

    /// The `change` ESS prints for `Blocked` leaving `repair.from`.
    fn repair_without_blocked() -> Value {
        json!({"category": "entity", "subject": "controlplane.host.Assignment", "changed": {
            "kind": "transition-route-changed", "transition": "repair",
            "before": "Blocked, Reviewing -> Implementing", "after": "Reviewing -> Implementing"}})
    }

    fn edit(root: &Path, from: &str, to: &str) -> Result<()> {
        let path = root.join("ess/domains/host.yaml");
        let before = fs::read_to_string(&path)?;
        assert_eq!(before.matches(from).count(), 1, "edit anchor not unique");
        fs::write(&path, before.replacen(from, to, 1))?;
        Ok(())
    }

    /// The change the ESS hardening review classified `unknown`: `Blocked` leaves `repair.from`.
    fn remove_blocked_from_repair(root: &Path) -> Result<()> {
        edit(
            root,
            "    - name: repair\n      from:\n      - Reviewing\n      - Blocked\n",
            "    - name: repair\n      from:\n      - Reviewing\n",
        )
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
        // The command outcome that moves through `repair` restates the route change and is
        // decided with it, so one reviewed acknowledgement covers both.
        assert!(
            error.contains(
                "restated by command/controlplane.host.RepairAssignment/outcome-subject-changed/applied"
            ),
            "{error}"
        );
        eprintln!("{error}");
        Ok(())
    }

    #[test]
    fn acknowledged_spec_change_passes_gate() -> Result<()> {
        let (dir, baseline) = repository()?;
        let root = dir.path();
        acknowledge(root, &baseline, &[(REPAIR, repair_without_blocked())])?;
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
        acknowledge(root, &baseline, &[(REPAIR, repair_without_blocked())])?;
        let error = format!("{:#}", check(root).unwrap_err());
        assert!(error.contains("stale acknowledgement"), "{error}");
        assert!(error.contains(REPAIR), "{error}");
        eprintln!("{error}");
        Ok(())
    }

    /// Additions no recorded call can observe pass unacknowledged; ESS rates each of them
    /// non-compatible for some dimension, so each one exercises the exemption.
    #[test]
    fn additive_changes_pass_gate() -> Result<()> {
        let (dir, baseline) = repository()?;
        let root = dir.path();
        acknowledge(root, &baseline, &[])?;
        edit(
            root,
            "  - Validated\n  - Queued\n  - Blocked\n",
            "  - Validated\n  - Queued\n  - Blocked\n  - Abandoned\n",
        )?;
        edit(
            root,
            "- name: controlplane.host.PauseGoal\n  input:\n  - name: goal_id\n    type: Uuid\n",
            "- name: controlplane.host.PauseGoal\n  input:\n  - name: goal_id\n    type: Uuid\n  - name: note\n    type: Optional<String>\n",
        )?;
        edit(
            root,
            "  - name: name\n    type: String\n  lifecycle:\n    initial: Registered\n    states:\n    - Registered\n    - Archived\n",
            "  - name: name\n    type: String\n  - name: note\n    type: Optional<String>\n  lifecycle:\n    initial: Registered\n    states:\n    - Registered\n    - Archived\n",
        )?;
        edit(
            root,
            "- name: controlplane.host.WorkspaceList\n  source: controlplane.host.Workspace\n  consistency: read_your_writes\n  fields:\n",
            "- name: controlplane.host.WorkspaceList\n  source: controlplane.host.Workspace\n  consistency: read_your_writes\n  fields:\n  - name: note\n    type: Optional<String>\n",
        )?;
        let summary = check(root)?;
        assert!(
            summary.contains("4 change(s)") && summary.contains("0 could break replay"),
            "{summary}"
        );
        // A required input is not additive: every recorded call lacks it.
        edit(
            root,
            "    type: Uuid\n  - name: note\n    type: Optional<String>\n",
            "    type: Uuid\n  - name: note\n    type: String\n",
        )?;
        let error = format!("{:#}", check(root).unwrap_err());
        assert!(
            error.contains("command/controlplane.host.PauseGoal/input-added/note"),
            "{error}"
        );
        Ok(())
    }

    /// Once a reviewed change is published with its baseline, the diff no longer holds it; the
    /// acknowledgement the baseline itself carries is history, not stale.
    #[test]
    fn acknowledgement_published_with_the_baseline_is_not_stale() -> Result<()> {
        let (dir, baseline) = repository()?;
        let root = dir.path();
        acknowledge(root, &baseline, &[(REPAIR, repair_without_blocked())])?;
        remove_blocked_from_repair(root)?;
        check(root)?;
        run_git(root, &["add", "ess"])?;
        run_git(root, &["commit", "--quiet", "--message", "Publish"])?;
        let summary = check(root)?;
        assert!(summary.contains("merge base with main"), "{summary}");
        assert!(summary.contains("0 change(s)"), "{summary}");
        Ok(())
    }
}
