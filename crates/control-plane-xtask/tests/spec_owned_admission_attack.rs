//! Adversarial cases for story:spec-owned-admission, pass 1.
//!
//! Acceptance `queue_guards_are_refused_in_conformance` was narrowed after this pass: a missing
//! repository on QueueAssignment stays a host fact, listed in `ess/README.md`, until
//! story:admission-conformance-target declares it. This case asserts that decision.
use std::path::PathBuf;

#[test]
fn queue_assignment_declares_a_missing_repository_refusal() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let replace = "replace this case when story:admission-conformance-target lands";
    let suite: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("generated/conformance.json")).unwrap(),
    )
    .unwrap();
    let queue: Vec<&String> = suite["scenarios"]
        .as_object()
        .unwrap()
        .keys()
        .filter(|id| id.starts_with("controlplane.host.QueueAssignment/outcome/"))
        .collect();
    assert!(
        !queue.iter().any(|id| id.contains("repository")),
        "QueueAssignment declares a repository-missing refusal, which story:admission-conformance-target \
         owns; {replace}. Its outcome scenarios: {queue:?}"
    );
    let readme = std::fs::read_to_string(root.join("ess/README.md")).unwrap();
    let facts = readme.split("\n## Host facts\n").nth(1).unwrap_or_default();
    assert!(
        facts.contains("repository missing"),
        "ess/README.md lists no repository-missing host fact for QueueAssignment until \
         story:admission-conformance-target declares it; {replace}"
    );
}
