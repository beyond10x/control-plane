//! Eval commit rules: only a repository inside the configured eval root whose origin is an
//! absolute local path inside that root commits with plain `git commit`.
use control_plane_runtime::{RuntimeConfig, process::ProcessRunner};
use std::path::Path;

fn runner() -> ProcessRunner {
    ProcessRunner {
        environment: vec![],
        timeout: std::time::Duration::from_secs(5),
        cancel: Default::default(),
    }
}

/// A repository at `root/repos/case` whose origin is `origin`.
fn repository(root: &Path, origin: &str) -> std::path::PathBuf {
    let repo = root.join("repos/case");
    std::fs::create_dir_all(&repo).unwrap();
    let runner = runner();
    runner.command(&repo, "git", &["init"]).unwrap();
    runner
        .command(&repo, "git", &["remote", "add", "origin", origin])
        .unwrap();
    repo
}

fn has_eval_rules(config: &RuntimeConfig, repo: &Path) -> bool {
    let runner = runner();
    let local = config.local_eval(repo, &runner);
    let commit = config.commit_for(repo, &runner);
    let eval = ["git", "commit", "-m"].map(str::to_owned).to_vec();
    local.is_ok_and(|local| local) || commit.is_ok_and(|commit| commit == eval)
}

#[test]
fn eval_origin_outside_root_is_refused() {
    let root = tempfile::tempdir().unwrap();
    let config = RuntimeConfig {
        local_eval_root: Some(root.path().into()),
        ..Default::default()
    };

    // The control: an absolute origin inside the root gets eval rules.
    let inside = root.path().join("remotes/case.git");
    std::fs::create_dir_all(&inside).unwrap();
    let repo = repository(root.path(), inside.to_str().unwrap());
    assert!(has_eval_rules(&config, &repo));
    std::fs::remove_dir_all(root.path().join("repos")).unwrap();

    // An existing local origin outside the eval root.
    let elsewhere = tempfile::tempdir().unwrap();
    let outside = elsewhere.path().join("case.git");
    std::fs::create_dir_all(&outside).unwrap();
    let repo = repository(root.path(), outside.to_str().unwrap());
    assert!(
        !has_eval_rules(&config, &repo),
        "an origin outside the eval root got eval commit rules"
    );
    std::fs::remove_dir_all(root.path().join("repos")).unwrap();

    // A relative origin, even one that resolves inside the root from the repository.
    let repo = repository(root.path(), "../../remotes/case.git");
    assert!(repo.join("../../remotes/case.git").is_dir());
    assert!(
        !has_eval_rules(&config, &repo),
        "a relative origin got eval commit rules"
    );
}
