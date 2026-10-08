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

/// The one mutant the audit admits unwitnessed: its id, and the single synthesis refusal (code
/// and scenario) its suite must have gained. Negating SatisfyGoal's `stale-revision` guard,
/// `when: defined(receipt_revision)`, leaves that outcome reading an input the guard now requires
/// absent, so ESS refuses to synthesize its scenario (ESS-SYNTH-003) and reports the mutant
/// unwitnessed (ESS-MUTATE-004). Remove this exemption when the ESS release ships a witness for
/// a negated `defined()` guard.
pub const UNWITNESSED_EXEMPTION: (&str, &str, &str) = (
    "guard-negate/controlplane.host.SatisfyGoal/stale-revision",
    "ESS-SYNTH-003",
    "controlplane.host.SatisfyGoal/outcome/stale-revision",
);

/// Whether `mutant` is exactly the exempted unwitnessed mutant with exactly its refusal.
fn exempt(mutant: &serde_json::Value) -> bool {
    let (id, code, scenario) = UNWITNESSED_EXEMPTION;
    mutant["id"] == id
        && mutant["verdict"] == "unwitnessed"
        && mutant["added_refusals"].as_array().is_some_and(|refusals| {
            matches!(refusals.as_slice(), [only] if only["code"] == code && only["scenario"] == scenario)
        })
}

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

/// The counts of an `ess-mutation-report/3` document. It is refused unless every listed mutant is
/// killed by its own record or is the [`UNWITNESSED_EXEMPTION`], the counts agree with the list,
/// and a guard-class mutant was killed.
#[derive(Debug, PartialEq)]
pub struct Verdict {
    pub mutants: u64,
    pub killed: u64,
    /// Mutants admitted unwitnessed by [`UNWITNESSED_EXEMPTION`]: 0 or 1.
    pub exempt: u64,
    pub guard_mutants: usize,
}

pub fn verdict(report: &serde_json::Value) -> Result<Verdict> {
    ensure!(
        report["format"] == "ess-mutation-report/3",
        "not an ess-mutation-report/3 document: {}",
        report["format"]
    );
    let counts = &report["counts"];
    let count = |key: &str| {
        counts[key]
            .as_u64()
            .with_context(|| format!("no {key} count"))
    };
    let mutants = report["mutants"].as_array().context("no mutants array")?;
    let exempted = mutants.iter().filter(|mutant| exempt(mutant)).count() as u64;
    for key in ["survived", "inconclusive"] {
        ensure!(count(key)? == 0, "{} mutant(s) {key}", count(key)?);
    }
    ensure!(
        exempted <= 1,
        "the exempted mutant is listed {exempted} times"
    );
    // With the exemption, `run_verb` no longer relies on `--collect` passing, so this verdict
    // also refuses the counts ESS would otherwise judge.
    if exempted > 0 {
        for key in ["stillborn", "equivalent"] {
            let n = counts[key].as_u64().unwrap_or(0);
            ensure!(n == 0, "{n} mutant(s) {key} beside the exempted mutant");
        }
    }
    ensure!(
        count("unwitnessed")? == exempted,
        "{} mutant(s) unwitnessed, {exempted} of them exempted",
        count("unwitnessed")?
    );
    // Each mutant is judged by its own record; the counts must agree with the list.
    let unkilled: Vec<String> = mutants
        .iter()
        .filter(|mutant| mutant["verdict"] != "killed" && !exempt(mutant))
        .map(|mutant| format!("{} ({})", mutant["id"], mutant["verdict"]))
        .collect();
    ensure!(
        unkilled.is_empty(),
        "mutant(s) not killed: {}",
        unkilled.join(", ")
    );
    let killed = mutants.len() as u64 - exempted;
    ensure!(
        count("mutants")? == mutants.len() as u64 && count("killed")? == killed,
        "the counts ({} mutants, {} killed) disagree with the {} mutant(s) listed ({killed} killed)",
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
        exempt: exempted,
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
    // `--collect` fails an audit with an unwitnessed mutant. It is admitted only when the
    // verdict accepts the report with the exempted mutant in it and the baseline passed.
    let verdict = verdict(&audit.report);
    let exempted = matches!(&verdict, Ok(verdict) if verdict.exempt > 0);
    ensure!(
        audit.passed || (exempted && audit.baseline_passed),
        "the mutation audit did not pass"
    );
    let verdict = verdict?;
    println!(
        "{} mutant(s), {} killed, {} exempted unwitnessed, {} of the guard classes; no survivor",
        verdict.mutants, verdict.killed, verdict.exempt, verdict.guard_mutants
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
    /// this specification, scores guard-class mutants and finds no survivor.
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
        let verdict = verdict(&report)?;
        assert!(verdict.guard_mutants > 0 && verdict.killed + verdict.exempt == verdict.mutants);
        Ok(())
    }

    /// A report with a survivor, or with no guard-class mutant, is refused.
    #[test]
    fn verdict_refuses_survivors_and_audits_without_guard_mutants() {
        let report = |survived: u64, class: &str| {
            serde_json::json!({"format": "ess-mutation-report/3",
                "counts": {"mutants": 1, "killed": 1 - survived, "survived": survived,
                    "inconclusive": 0, "unwitnessed": 0},
                "mutants": [{"class": class, "verdict": if survived == 0 { "killed" } else { "survived" }}]})
        };
        assert!(verdict(&report(0, "guard-negate")).is_ok());
        assert!(verdict(&report(1, "guard-negate")).is_err());
        assert!(verdict(&report(0, "emit-drop")).is_err());
    }

    /// story:typed-satisfaction-receipt. The one unwitnessed mutant ESS 0.56.0 cannot witness,
    /// the negated `defined(receipt_revision)` guard of SatisfyGoal's `stale-revision`, is
    /// admitted only with its own ESS-SYNTH-003 refusal on that outcome's scenario. Another
    /// unwitnessed id, this id with another refusal, an extra refusal or another verdict, and a
    /// survivor beside it, are refused.
    #[test]
    fn only_the_declared_unwitnessed_mutant_is_admitted() {
        let id = "guard-negate/controlplane.host.SatisfyGoal/stale-revision";
        let scenario = "controlplane.host.SatisfyGoal/outcome/stale-revision";
        let refusal = |code: &str, scenario: &str| {
            serde_json::json!({"code": code, "scenario": scenario,
                "subject": "outcome controlplane.host.SatisfyGoal/stale-revision"})
        };
        let report = |unwitnessed: serde_json::Value, survived: u64| {
            let mut mutants = vec![
                serde_json::json!({"id": "guard-negate/controlplane.host.CreateGoal/workers-invalid",
                    "class": "guard-negate", "verdict": "killed"}),
                unwitnessed,
            ];
            if survived == 1 {
                mutants.push(
                    serde_json::json!({"id": "guard-boundary/x", "class": "guard-boundary",
                    "verdict": "survived"}),
                );
            }
            serde_json::json!({"format": "ess-mutation-report/3",
                "counts": {"mutants": mutants.len(), "killed": 1, "survived": survived,
                    "inconclusive": 0, "unwitnessed": 1},
                "mutants": mutants})
        };
        let mutant = |id: &str, verdict: &str, refusals: Vec<serde_json::Value>| {
            serde_json::json!({"id": id, "class": "guard-negate", "verdict": verdict,
                "added_refusals": refusals})
        };
        let admitted = verdict(&report(
            mutant(id, "unwitnessed", vec![refusal("ESS-SYNTH-003", scenario)]),
            0,
        ));
        assert_eq!(
            admitted
                .map(|verdict| (verdict.killed, verdict.exempt))
                .ok(),
            Some((1, 1))
        );
        for (refused, why) in [
            (
                report(
                    mutant(
                        "guard-negate/controlplane.host.RepairAssignment/rebased",
                        "unwitnessed",
                        vec![refusal("ESS-SYNTH-003", scenario)],
                    ),
                    0,
                ),
                "another unwitnessed id",
            ),
            (
                report(
                    mutant(id, "unwitnessed", vec![refusal("ESS-SYNTH-001", scenario)]),
                    0,
                ),
                "another refusal code",
            ),
            (
                report(
                    mutant(
                        id,
                        "unwitnessed",
                        vec![refusal(
                            "ESS-SYNTH-003",
                            "controlplane.host.SatisfyGoal/outcome/applied",
                        )],
                    ),
                    0,
                ),
                "another refused scenario",
            ),
            (
                report(
                    mutant(
                        id,
                        "unwitnessed",
                        vec![
                            refusal("ESS-SYNTH-003", scenario),
                            refusal(
                                "ESS-SYNTH-003",
                                "controlplane.host.SatisfyGoal/outcome/applied",
                            ),
                        ],
                    ),
                    0,
                ),
                "an extra refusal",
            ),
            (report(mutant(id, "unwitnessed", vec![]), 0), "no refusal"),
            (
                report(
                    mutant(id, "inconclusive", vec![refusal("ESS-SYNTH-003", scenario)]),
                    0,
                ),
                "another verdict",
            ),
            (
                report(
                    mutant(id, "unwitnessed", vec![refusal("ESS-SYNTH-003", scenario)]),
                    1,
                ),
                "a survivor beside it",
            ),
        ] {
            assert!(verdict(&refused).is_err(), "{why} was admitted: {refused}");
        }
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
