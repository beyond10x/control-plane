//! The admission rules of the design review (review-result:ess-design-review-2026-10-06) are
//! either declared in the specification, with synthesized scenarios, or listed as host facts in
//! `ess/README.md`; and the Supervisor holds only the grants the runtime uses.
use std::{collections::BTreeSet, fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &str) -> String {
    fs::read_to_string(root().join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// The rows the review marks `missing`, each with the scenarios that declare it. A row with no
/// scenario is a host fact; a row with scenarios may still list the part ESS cannot hold.
const ROWS: [(u32, &[&str]); 15] = [
    (1, &[]),
    (2, &[]),
    (3, &[]),
    (4, &[]),
    (
        5,
        &["controlplane.host.QueueAssignment/outcome/goal-not-current"],
    ),
    (
        6,
        &[
            "controlplane.host.ReadyAssignment/outcome/reviewer-missing",
            "controlplane.host.ReadyAssignment/outcome/review-not-independent",
        ],
    ),
    (
        7,
        &[
            "controlplane.host.ReviewAssignment/outcome/tests-not-current",
            "controlplane.host.ReadyAssignment/outcome/evidence-not-current",
        ],
    ),
    (8, &[]),
    (9, &[]),
    (
        10,
        &[
            "controlplane.host.CompleteAssignment/outcome/receipt-missing",
            "controlplane.host.ReconcileAssignment/outcome/receipt-missing",
        ],
    ),
    (11, &[]),
    (14, &[]),
    (15, &[]),
    (16, &[]),
    (
        17,
        &[
            "controlplane.host.CreateGoal/outcome/workers-invalid",
            "controlplane.host.CreateGoal/outcome/attempts-invalid",
            "controlplane.host.CreateGoal/outcome/minutes-invalid",
        ],
    ),
];

/// The host-facts section of `ess/README.md`: the rows of its table, by review row number.
fn host_facts() -> BTreeSet<u32> {
    let readme = read("ess/README.md");
    let section = readme
        .split("\n## Host facts\n")
        .nth(1)
        .expect("ess/README.md has a `## Host facts` section");
    let section = section.split("\n## ").next().unwrap_or(section);
    section
        .lines()
        .filter_map(|line| line.strip_prefix("| "))
        .filter_map(|line| line.split(" |").next()?.trim().parse().ok())
        .collect()
}

#[test]
fn admission_rules_are_declared() {
    let suite: serde_json::Value =
        serde_json::from_str(&read("generated/conformance.json")).unwrap();
    let scenarios = suite["scenarios"].as_object().unwrap();
    let listed = host_facts();
    for (row, declared) in ROWS {
        for id in declared {
            assert!(
                scenarios.contains_key(*id),
                "row {row}: {id} is not a synthesized scenario"
            );
        }
        // Every row but the fully declared ones (6 and 7) keeps a part only the host enforces.
        let fully_declared = matches!(row, 6 | 7);
        assert_eq!(
            listed.contains(&row),
            !fully_declared,
            "row {row}: host-facts listing in ess/README.md"
        );
    }
    let known: BTreeSet<u32> = ROWS.iter().map(|(row, _)| *row).collect();
    assert!(
        listed.is_subset(&known),
        "host facts name rows the review does not mark missing: {:?}",
        listed.difference(&known).collect::<Vec<_>>()
    );
}

/// Every command a declared actor's `may:` list names in `ess/domains/host.yaml`.
fn grants(actor: &str) -> BTreeSet<String> {
    let spec = read("ess/domains/host.yaml");
    let start = format!("- name: controlplane.host.{actor}\n  may:\n");
    let list = spec
        .split(&start)
        .nth(1)
        .unwrap_or_else(|| panic!("{actor} declares no may: list"));
    list.lines()
        .map_while(|line| line.strip_prefix("  - controlplane.host."))
        .map(str::to_owned)
        .collect()
}

/// Every declared command name that the runtime's production source (everything before its
/// `#[cfg(test)] mod`) spells as a string literal. The runtime sends every command as the
/// Supervisor; the test also refuses any production use of the Operator actor there.
fn runtime_commands(commands: &BTreeSet<String>) -> BTreeSet<String> {
    let dir = root().join("crates/control-plane-runtime/src");
    let mut sent = BTreeSet::new();
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let source = fs::read_to_string(&path).unwrap();
        let production = source
            .split("#[cfg(test)]\nmod ")
            .next()
            .unwrap_or_default();
        assert!(
            !production.contains("Actor::Operator"),
            "{} sends a command as the Operator",
            path.display()
        );
        for command in commands {
            if production.contains(&format!("\"{command}\"")) {
                sent.insert(command.clone());
            }
        }
    }
    sent
}

#[test]
fn supervisor_grants_are_least_privilege() {
    let spec = read("ess/domains/host.yaml");
    let commands: BTreeSet<String> = spec
        .split("\ncommands:\n")
        .nth(1)
        .and_then(|commands| commands.split("\nevents:\n").next())
        .unwrap()
        .lines()
        .filter_map(|line| line.strip_prefix("- name: controlplane.host."))
        .map(str::to_owned)
        .collect();
    let mut sent = runtime_commands(&commands);
    // The host records planning progress for the runtime (`Store::record_activity`).
    sent.insert("RecordPlanningProgress".into());
    assert_eq!(grants("Supervisor"), sent);
}
