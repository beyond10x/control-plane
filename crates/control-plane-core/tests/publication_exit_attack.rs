//! Adversarial cases for story:publication-exit, wave 4 pass 1.
//!
//! The generated contract (`ess/domains/host.yaml`, `controlplane.host.ClosePublication`) takes
//! `publication_id: Uuid` and `reason: String` and declares exactly three outcomes: `applied`,
//! `not-found` for an unknown instance and `wrong-state` for an intent that is not Prepared or
//! Uncertain. It declares no refusal for the text of `reason`.
use control_plane_core::{Actor, Store};
use serde_json::{Value, json};

fn identity(outcome: &Value, field: &str) -> String {
    outcome["published"][0]["payload"][field]
        .as_str()
        .unwrap()
        .to_owned()
}

/// An unknown publication is `not-found`, the declared outcome for an unknown instance,
/// whatever text the caller gave as its reason.
#[tokio::test]
async fn close_of_unknown_publication_answers_not_found_whatever_its_reason() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let unknown = "00000000-0000-4000-8000-000000000000";
    let named = store
        .execute(
            "ClosePublication",
            json!({"publication_id":unknown,"reason":"closed as not published"}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(named["outcome"], "not-found", "{named}");
    let empty = store
        .execute(
            "ClosePublication",
            json!({"publication_id":unknown,"reason":""}),
            Actor::Supervisor,
        )
        .await;
    assert!(
        empty
            .as_ref()
            .is_ok_and(|answer| answer["outcome"] == "not-found"),
        "the contract declares not-found for an unknown publication; got {empty:?}"
    );
}

/// A closed intent is `wrong-state`, the declared outcome for an intent that is not open,
/// whatever text the caller gave as its reason.
#[tokio::test]
async fn close_of_closed_publication_answers_wrong_state_whatever_its_reason() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    assert!(
        std::process::Command::new("git")
            .args(["init", "--quiet", "--initial-branch=main"])
            .arg(&repo)
            .status()
            .unwrap()
            .success()
    );
    let mut store = Store::open(temp.path().join("state.sqlite")).await.unwrap();
    let ws = identity(
        &store
            .execute(
                "RegisterWorkspace",
                json!({"path":temp.path(),"name":"attack"}),
                Actor::Operator,
            )
            .await
            .unwrap(),
        "workspace_id",
    );
    let repository = identity(&store.execute("RegisterRepository",json!({"workspace_id":ws,"path":repo,"name":"repo","common_dir":"untrusted","base_branch":"main","test_command":"cargo test","publish_command":"configured-helper"}),Actor::Operator).await.unwrap(),"repository_id");
    let goal = identity(&store.execute("CreateGoal",json!({"workspace_id":ws,"objective":"deliver change","acceptance":"tests and independent review","max_workers":3,"max_attempts":3,"max_minutes":60,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":true}),Actor::Operator).await.unwrap(),"goal_id");
    store
        .execute("StartGoal", json!({"goal_id":goal}), Actor::Operator)
        .await
        .unwrap();
    let assignment = identity(&store.execute("QueueAssignment",json!({"goal_id":goal,"repository_id":repository,"story_id":"story:attack","case_id":"case","worktree_id":"","candidate":"","attempt":0,"reason":"","implementor_run":"","reviewer_run":"","goal_revision":1}),Actor::Supervisor).await.unwrap(),"assignment_id");
    for (command, body) in [
        (
            "ClaimAssignment",
            json!({"assignment_id":assignment,"worktree_id":"tree","implementor_run":"impl","base_revision":"base"}),
        ),
        (
            "ReviewAssignment",
            json!({"assignment_id":assignment,"candidate":"cand","test_revision":"cand"}),
        ),
        (
            "ReadyAssignment",
            json!({"assignment_id":assignment,"reviewer_run":"reviewer","review_revision":"cand"}),
        ),
    ] {
        let answer = store
            .execute(command, body, Actor::Supervisor)
            .await
            .unwrap();
        assert_eq!(answer["outcome"], "applied", "{command}: {answer}");
    }
    let publication = identity(
        &store
            .execute(
                "PreparePublication",
                json!({"assignment_id":assignment,"candidate":"cand","target":"main","expected_base":"base"}),
                Actor::Supervisor,
            )
            .await
            .unwrap(),
        "publication_id",
    );
    for (command, body) in [
        ("MergeAssignment", json!({"assignment_id":assignment})),
        (
            "MarkPublicationUncertain",
            json!({"publication_id":publication}),
        ),
        (
            "ClosePublication",
            json!({"publication_id":publication,"reason":"closed as not published"}),
        ),
    ] {
        let answer = store
            .execute(command, body, Actor::Supervisor)
            .await
            .unwrap();
        assert_eq!(answer["outcome"], "applied", "{command}: {answer}");
    }
    let named = store
        .execute(
            "ClosePublication",
            json!({"publication_id":publication,"reason":"closed twice"}),
            Actor::Supervisor,
        )
        .await
        .unwrap();
    assert_eq!(named["outcome"], "wrong-state", "{named}");
    let blank = store
        .execute(
            "ClosePublication",
            json!({"publication_id":publication,"reason":"  "}),
            Actor::Supervisor,
        )
        .await;
    assert!(
        blank
            .as_ref()
            .is_ok_and(|answer| answer["outcome"] == "wrong-state"),
        "the contract declares wrong-state for a closed publication; got {blank:?}"
    );
}
