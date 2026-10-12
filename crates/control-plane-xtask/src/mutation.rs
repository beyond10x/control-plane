//! The ESS mutation audit, run against the durable generated store below the host rules
//! ([`DurableTarget::new`]).
//!
//! `ess verify conform mutate --emit` writes the baseline suite and one suite per specification
//! mutant; each runs here against [`DurableTarget`], whose report is written beside its suite;
//! `--collect` then scores the reports. A mutant is killed when its suite fails against the
//! unchanged implementation; a survivor is a declared rule no scenario pins down.
use crate::target::{DurableTarget, run};
use anyhow::{Context, Result, ensure};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

/// The classes that mutate declared guards.
pub const GUARD_CLASSES: [&str; 4] = [
    "guard-boundary",
    "guard-negate",
    "guard-connective",
    "precedence-swap",
];

/// The only report format the verdict judges: what the pinned ESS 0.59.0 `--collect` writes.
pub const REPORT_FORMAT: &str = "ess-mutation-report/5";

/// The committed `--collect` report of the last audit of the specification, checked by the fast
/// test against the specification digest of `generated/conformance.json`. `task mutation --
/// --keep` writes a fresh one into the kept directory; copy its `mutation-report.json` here.
#[cfg(test)]
pub const RECORDED_REPORT: &str =
    "crates/control-plane-xtask/tests/fixtures/guard-mutation-report.json";

/// What `--collect` printed and scored.
pub struct Audit {
    pub summary: String,
    pub report: serde_json::Value,
    pub passed: bool,
    /// Whether the baseline suite executed and passed every scenario against the target.
    pub baseline_passed: bool,
    /// The emitted directory, when it was kept.
    pub kept: Option<PathBuf>,
}

/// The counts of an [`REPORT_FORMAT`] document. It is refused unless every listed mutant is
/// killed or is [`evidenced_equivalent`], the counts agree with the list, and a guard-class
/// mutant was killed. No mutant is admitted unwitnessed, survived, inconclusive or stillborn.
#[derive(Debug, PartialEq)]
pub struct Verdict {
    pub mutants: u64,
    pub killed: u64,
    /// Mutants admitted as equivalent on ESS's own evidence.
    pub equivalent: u64,
    pub guard_mutants: usize,
}

/// Whether `mutant` is an `equivalent` record carrying the evidence ESS 0.59.0 writes for that
/// verdict (`ess-mutation-report/5`). The `ESS-MUTATE-005` code itself appears only in the text
/// summary of `--collect`; the record carries one of two fields instead:
///
/// - `unsatisfiable_guard`: the non-empty mutated guard, with the stored predicate beside it, that
///   no input satisfies (ESS 0.57.0, "no input satisfies …"; on any guard class, and on a
///   `precedence-swap` whose two guards never overlap);
/// - `identical_answer`: on a `precedence-swap` only, the non-empty description of the one answer
///   both swapped branches give (ESS 0.57.0, beyond10x/ess#517; ESS refuses it on another class).
///
/// An `equivalent` record with neither, with an empty or non-text value, or with
/// `identical_answer` on another class, is not evidenced.
fn evidenced_equivalent(mutant: &serde_json::Value) -> bool {
    let text = |key: &str| mutant[key].as_str().is_some_and(|value| !value.is_empty());
    mutant["verdict"] == "equivalent"
        && (text("unsatisfiable_guard")
            || (mutant["class"] == "precedence-swap" && text("identical_answer")))
}

pub fn verdict(report: &serde_json::Value) -> Result<Verdict> {
    ensure!(
        report["format"] == REPORT_FORMAT,
        "not an {REPORT_FORMAT} document: {}",
        report["format"]
    );
    let counts = &report["counts"];
    let count = |key: &str| {
        counts[key]
            .as_u64()
            .with_context(|| format!("no {key} count"))
    };
    let mutants = report["mutants"].as_array().context("no mutants array")?;
    for key in ["survived", "inconclusive", "unwitnessed"] {
        ensure!(count(key)? == 0, "{} mutant(s) {key}", count(key)?);
    }
    let stillborn = counts["stillborn"].as_u64().unwrap_or(0);
    ensure!(stillborn == 0, "{stillborn} mutant(s) stillborn");
    // Each mutant is judged by its own record; the counts must agree with the list.
    let unkilled: Vec<String> = mutants
        .iter()
        .filter(|mutant| mutant["verdict"] != "killed" && !evidenced_equivalent(mutant))
        .map(|mutant| format!("{} ({})", mutant["id"], mutant["verdict"]))
        .collect();
    ensure!(
        unkilled.is_empty(),
        "mutant(s) neither killed nor equivalent on ESS's evidence: {}",
        unkilled.join(", ")
    );
    let equivalent = mutants
        .iter()
        .filter(|mutant| evidenced_equivalent(mutant))
        .count() as u64;
    let killed = mutants.len() as u64 - equivalent;
    let counted_equivalent = counts["equivalent"].as_u64().unwrap_or(0);
    ensure!(
        count("mutants")? == mutants.len() as u64
            && count("killed")? == killed
            && counted_equivalent == equivalent,
        "the counts ({} mutants, {} killed, {counted_equivalent} equivalent) disagree with the {} \
         mutant(s) listed ({killed} killed, {equivalent} equivalent)",
        count("mutants")?,
        count("killed")?,
        mutants.len()
    );
    let guard_mutants = mutants
        .iter()
        .filter(|mutant| {
            mutant["verdict"] == "killed"
                && mutant["class"]
                    .as_str()
                    .is_some_and(|class| GUARD_CLASSES.contains(&class))
        })
        .count();
    ensure!(guard_mutants > 0, "no guard-class mutant was killed");
    Ok(Verdict {
        mutants: count("mutants")?,
        killed,
        equivalent,
        guard_mutants,
    })
}

fn suites(dir: &Path, found: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            suites(&path, found)?;
        } else if path.file_name().is_some_and(|name| name == "suite.json") {
            found.push(path);
        }
    }
    Ok(())
}

fn ess(root: &Path, args: &[&str]) -> Result<(bool, String)> {
    let output = Command::new("ess")
        .current_dir(root)
        .args(args)
        .output()
        .context("start released ESS CLI")?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok((output.status.success(), text))
}

/// Removes the emitted directory when the audit ends, however it ends, unless it is kept.
struct Emitted {
    dir: PathBuf,
    keep: bool,
}

impl Drop for Emitted {
    fn drop(&mut self) {
        if !self.keep {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }
}

/// Emit the mutants of `classes` into a new directory under `<root>/.scratch`, run every suite
/// against the durable generated store and collect the scores. The directory is removed at the
/// end unless `keep`.
pub fn audit(root: &Path, classes: &[String], keep: bool) -> Result<Audit> {
    let scratch = root.join(".scratch");
    fs::create_dir_all(&scratch)?;
    let emitted = Emitted {
        dir: tempfile::Builder::new()
            .prefix("mutation-")
            .tempdir_in(&scratch)?
            .keep(),
        keep,
    };
    let dir = emitted.dir.join("emitted");
    let path = dir.to_str().context("mutation path is not UTF-8")?;
    let mut emit = vec![
        "verify", "conform", "mutate", "--path", "ess", "--emit", path,
    ];
    for class in classes {
        emit.extend(["--class", class.as_str()]);
    }
    let (ok, text) = ess(root, &emit)?;
    ensure!(ok, "ess mutate --emit failed:\n{text}");
    let mut found = Vec::new();
    suites(&dir, &mut found)?;
    found.sort();
    ensure!(!found.is_empty(), "ess mutate --emit wrote no suite");
    let target = DurableTarget::new(&emitted.dir.join("target"))?;
    for suite in &found {
        let (report, _) = run(&fs::read_to_string(suite)?, &target)
            .with_context(|| format!("run {}", suite.display()))?;
        fs::write(
            suite.with_file_name("report.json"),
            report.to_canonical_json()?,
        )?;
    }
    let baseline: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join("baseline/report.json"))?)?;
    let baseline_passed = baseline["execution_status"] == "passed"
        && baseline["counts"]["failed"] == 0
        && baseline["counts"]["error"] == 0;
    let report = emitted.dir.join("mutation-report.json");
    let written = report.to_str().context("report path is not UTF-8")?;
    let (passed, summary) = ess(
        root,
        &[
            "verify",
            "conform",
            "mutate",
            "--collect",
            path,
            "--report-out",
            written,
        ],
    )?;
    let report = match fs::read(&report) {
        Ok(bytes) => serde_json::from_slice(&bytes)?,
        Err(_) => serde_json::Value::Null,
    };
    Ok(Audit {
        summary,
        report,
        passed,
        baseline_passed,
        kept: keep.then(|| emitted.dir.clone()),
    })
}

pub fn run_verb(root: &Path, classes: Vec<String>, keep: bool) -> Result<()> {
    let classes = if classes.is_empty() {
        GUARD_CLASSES.map(String::from).to_vec()
    } else {
        classes
    };
    let audit = audit(root, &classes, keep)?;
    print!("{}", audit.summary);
    if let Some(dir) = &audit.kept {
        println!("kept: {}", dir.display());
    }
    // Both judges must pass: `--collect`, which fails an audit with an unwitnessed mutant, and
    // the verdict, over a baseline that passed every scenario.
    ensure!(
        audit.passed && audit.baseline_passed,
        "the mutation audit did not pass"
    );
    let verdict = verdict(&audit.report)?;
    println!(
        "{} mutant(s), {} killed, {} equivalent on ESS's evidence, {} of the guard classes; no \
         survivor",
        verdict.mutants, verdict.killed, verdict.equivalent, verdict.guard_mutants
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
    }

    /// The recorded `--collect` report of `xtask mutate` (the gate step, `task mutation`) is of
    /// this specification, scores guard-class mutants, finds no survivor and holds no unwitnessed
    /// mutant: every mutant is killed or equivalent with ESS's own evidence.
    #[test]
    fn guard_mutants_are_killed() -> Result<()> {
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(root().join(RECORDED_REPORT))?)?;
        let suite: serde_json::Value =
            serde_json::from_slice(&fs::read(root().join("generated/conformance.json"))?)?;
        assert_eq!(
            report["spec_digest"], suite["provenance"]["spec_digest"],
            "{RECORDED_REPORT} audits another specification; run `task mutation -- --keep` and \
             copy the kept mutation-report.json there"
        );
        assert_eq!(report["counts"]["unwitnessed"], 0);
        let verdict = verdict(&report)?;
        assert!(
            verdict.guard_mutants > 0 && verdict.killed + verdict.equivalent == verdict.mutants
        );
        Ok(())
    }

    fn mutation_report(mutants: Vec<serde_json::Value>) -> serde_json::Value {
        let count = |verdict: &str| mutants.iter().filter(|m| m["verdict"] == verdict).count();
        serde_json::json!({"format": "ess-mutation-report/5",
            "counts": {"mutants": mutants.len(), "killed": count("killed"),
                "survived": count("survived"), "inconclusive": count("inconclusive"),
                "unwitnessed": count("unwitnessed"), "equivalent": count("equivalent"),
                "stillborn": count("stillborn")},
            "mutants": mutants})
    }

    fn killed_guard() -> serde_json::Value {
        serde_json::json!({"id": "guard-negate/controlplane.host.CreateGoal/workers-invalid",
            "class": "guard-negate", "verdict": "killed"})
    }

    /// A report with a survivor, or with no guard-class mutant, is refused.
    #[test]
    fn verdict_refuses_survivors_and_audits_without_guard_mutants() {
        let report = |survived: u64, class: &str| {
            serde_json::json!({"format": "ess-mutation-report/5",
                "counts": {"mutants": 1, "killed": 1 - survived, "survived": survived,
                    "inconclusive": 0, "unwitnessed": 0},
                "mutants": [{"class": class, "verdict": if survived == 0 { "killed" } else { "survived" }}]})
        };
        assert!(verdict(&report(0, "guard-negate")).is_ok());
        assert!(verdict(&report(1, "guard-negate")).is_err());
        assert!(verdict(&report(0, "emit-drop")).is_err());
    }

    /// Only `ess-mutation-report/5`, the format the pinned ESS 0.59.0 writes, is judged; the
    /// formats before it, `/3` included, and any later one are refused.
    #[test]
    fn verdict_judges_only_the_pinned_report_format() {
        for (format, admitted) in [
            ("ess-mutation-report/5", true),
            ("ess-mutation-report/3", false),
            ("ess-mutation-report/4", false),
            ("ess-mutation-report/6", false),
            ("ess-mutation-manifest/5", false),
        ] {
            let mut report = mutation_report(vec![killed_guard()]);
            report["format"] = format.into();
            assert_eq!(verdict(&report).is_ok(), admitted, "{format}");
        }
    }

    /// story:remove-stale-revision-mutation-exemption. No mutant is admitted unwitnessed: a `/5`
    /// report with one unwitnessed mutant fails the audit, whatever its id, the once-exempted
    /// negated `stale-revision` guard with its ESS-SYNTH-003 refusal included.
    #[test]
    fn mutation_audit_admits_no_exemption() {
        assert!(verdict(&mutation_report(vec![killed_guard()])).is_ok());
        let refusal = serde_json::json!({"code": "ESS-SYNTH-003",
            "scenario": "controlplane.host.SatisfyGoal/outcome/stale-revision",
            "subject": "outcome controlplane.host.SatisfyGoal/stale-revision"});
        for id in [
            "guard-negate/controlplane.host.SatisfyGoal/stale-revision",
            "guard-negate/controlplane.host.RepairAssignment/rebased",
        ] {
            let unwitnessed = serde_json::json!({"id": id, "class": "guard-negate",
                "verdict": "unwitnessed", "added_refusals": [refusal]});
            let report = mutation_report(vec![killed_guard(), unwitnessed]);
            assert!(verdict(&report).is_err(), "unwitnessed {id} was admitted");
        }
    }

    /// An `equivalent` mutant is admitted only with ESS 0.59.0's evidence on its own record: a
    /// guard mutant's `unsatisfiable_guard` (ESS-MUTATE-005, "no input satisfies …"), or a
    /// `precedence-swap`'s `identical_answer`. Without it, with an empty value, or with
    /// `identical_answer` on a mutant that is not a `precedence-swap`, it fails the audit.
    #[test]
    fn an_equivalent_mutant_is_admitted_only_with_its_evidence() {
        let guard = "(not (defined(receipt_revision)) and (state == Running and revision != input.receipt_revision))";
        let answer = "refuse with `controlplane.host.GoalLimitInvalid` and no payload; neither \
                      changes state, sets or emits anything";
        let equivalent = |class: &str, evidence: Option<(&str, serde_json::Value)>| {
            let mut mutant = serde_json::json!({"id": format!("{class}/controlplane.host.X/y"),
                "class": class, "verdict": "equivalent"});
            if let Some((key, value)) = evidence {
                mutant[key] = value;
            }
            mutation_report(vec![killed_guard(), mutant])
        };
        for (report, why) in [
            (
                equivalent("guard-negate", Some(("unsatisfiable_guard", guard.into()))),
                "a negated guard no input satisfies",
            ),
            (
                equivalent("precedence-swap", Some(("identical_answer", answer.into()))),
                "a swap of two branches that answer alike",
            ),
            (
                equivalent(
                    "precedence-swap",
                    Some(("unsatisfiable_guard", guard.into())),
                ),
                "a swap of two guards that never overlap",
            ),
        ] {
            assert!(verdict(&report).is_ok(), "{why} was refused: {report}");
        }
        for (report, why) in [
            (equivalent("guard-negate", None), "no evidence"),
            (equivalent("precedence-swap", None), "no evidence on a swap"),
            (
                equivalent("guard-negate", Some(("unsatisfiable_guard", "".into()))),
                "an empty unsatisfiable_guard",
            ),
            (
                equivalent("guard-negate", Some(("unsatisfiable_guard", true.into()))),
                "an unsatisfiable_guard that is not the guard ESS stored",
            ),
            (
                equivalent("guard-negate", Some(("identical_answer", answer.into()))),
                "identical_answer on a mutant that is not a precedence-swap",
            ),
            (
                equivalent("precedence-swap", Some(("identical_answer", true.into()))),
                "an identical_answer that names no answer",
            ),
        ] {
            assert!(verdict(&report).is_err(), "{why} was admitted: {report}");
        }
    }

    /// The `equivalent` and `stillborn` counts agree with the records: a count with no record
    /// behind it, or a record the count leaves out, fails; a stillborn mutant fails.
    #[test]
    fn equivalent_and_stillborn_counts_agree_with_the_records() {
        let guard = "(not (defined(x)) and (state == Running))";
        let equivalent = serde_json::json!({"id": "guard-negate/controlplane.host.X/y",
            "class": "guard-negate", "verdict": "equivalent", "unsatisfiable_guard": guard});
        let admitted = mutation_report(vec![killed_guard(), equivalent.clone()]);
        assert!(verdict(&admitted).is_ok());
        for key in ["equivalent", "stillborn"] {
            let mut report = admitted.clone();
            report["counts"][key] = (report["counts"][key].as_u64().unwrap() + 1).into();
            assert!(
                verdict(&report).is_err(),
                "{key} count off by one was admitted"
            );
        }
        let mut report = admitted.clone();
        report["counts"]["equivalent"] = 0.into();
        assert!(
            verdict(&report).is_err(),
            "an uncounted equivalent record was admitted"
        );
        let stillborn = serde_json::json!({"id": "guard-boundary/controlplane.host.X/z",
            "class": "guard-boundary", "verdict": "stillborn"});
        assert!(verdict(&mutation_report(vec![killed_guard(), stillborn])).is_err());
    }

    /// The emitted directory is removed when the audit ends, and kept on request.
    #[test]
    fn emitted_directory_is_removed_unless_kept() -> Result<()> {
        let scratch = root().join(".scratch/mutation-cleanup-test");
        for keep in [false, true] {
            let dir = tempfile::Builder::new()
                .prefix("mutation-")
                .tempdir_in({
                    fs::create_dir_all(&scratch)?;
                    &scratch
                })?
                .keep();
            fs::write(dir.join("report.json"), "{}")?;
            drop(Emitted {
                dir: dir.clone(),
                keep,
            });
            assert_eq!(dir.exists(), keep);
        }
        fs::remove_dir_all(&scratch)?;
        Ok(())
    }
}
