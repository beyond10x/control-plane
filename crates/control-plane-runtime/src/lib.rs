//! The local planner and its trusted repository adapters.
use anyhow::Result;
use control_plane_core::Store;
use serde_json::Value;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Mutex;
mod context;
mod engine;
mod ess_reference;
mod fleet;
mod governance;
mod loom_model;
mod model;
pub mod process;
mod read_request;
mod refusal;
mod supervisor;
pub use loom_model::CodexAgentModel;
pub use supervisor::Supervisor;

pub type SharedStore = Arc<Mutex<Store>>;

#[derive(Clone, Debug)]
pub struct ModelRequest {
    pub role: String,
    pub execution_context: String,
    pub model: String,
    pub instructions: String,
    pub prompt: String,
    pub schema: Value,
    pub timeout: std::time::Duration,
}

pub trait AgentModel: Send + Sync {
    fn respond(&self, request: &ModelRequest) -> Result<Value>;
    fn respond_in(&self, request: &ModelRequest, environment: &ModelEnvironment) -> Result<Value> {
        let _ = environment;
        self.respond(request)
    }
}

/// Host bindings for Loom. The model never supplies these values.
pub struct ModelEnvironment {
    pub workspace: PathBuf,
    pub max_turns: u64,
    pub cancel: tokio_util::sync::CancellationToken,
    pub progress: Arc<dyn Fn(Value) -> Result<()> + Send + Sync>,
    /// New host observations for an existing Loom session; the first turn uses the full brief.
    pub continuation: Option<String>,
}

#[derive(Clone)]
pub struct RuntimeConfig {
    /// Explicitly isolated local eval repositories; never enables local commits for other repositories.
    pub local_eval_root: Option<PathBuf>,
    /// Added to every host-started process, including commands that run model-written code.
    pub environment: Vec<(String, String)>,
    /// Added only to the trusted commit and publish commands; never to checks or model tools.
    pub credentials: Vec<(String, String)>,
    pub aep_protocols: String,
    pub commit_command: Vec<String>,
    pub max_steps: usize,
    pub process_timeout: std::time::Duration,
    pub poll_interval: std::time::Duration,
    /// How long a publication whose publisher exited without its merge on the target waits for
    /// the candidate while the target does not move, before it is closed as not published.
    pub publication_grace: std::time::Duration,
}

/// The default [`RuntimeConfig::publication_grace`]: ten minutes.
pub const PUBLICATION_GRACE: std::time::Duration = std::time::Duration::from_secs(600);

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            local_eval_root: None,
            environment: Vec::new(),
            credentials: Vec::new(),
            aep_protocols:
                "git+https://github.com/beyond10x/aep#6d7a44d3607d2d9a6ffdf0a165993c546c43d0db"
                    .into(),
            commit_command: ["b10x-gates", "bot", "--repo", ".", "--", "commit", "-m"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            max_steps: 64,
            process_timeout: std::time::Duration::from_secs(120),
            poll_interval: std::time::Duration::from_secs(15),
            publication_grace: PUBLICATION_GRACE,
        }
    }
}

impl RuntimeConfig {
    /// The runner for a trusted commit or publish command: `runner` plus the credentials.
    pub fn credentialed(&self, runner: &process::ProcessRunner) -> process::ProcessRunner {
        let mut runner = runner.clone();
        runner.environment.extend(self.credentials.iter().cloned());
        runner
    }
    pub fn local_eval(
        &self,
        path: &std::path::Path,
        runner: &process::ProcessRunner,
    ) -> Result<bool> {
        let Some(root) = &self.local_eval_root else {
            return Ok(false);
        };
        let root = root.canonicalize()?;
        let common = runner.command(path, "git", &["rev-parse", "--git-common-dir"])?;
        if !path.join(common.trim()).canonicalize()?.starts_with(&root) {
            return Ok(false);
        }
        let origin = runner.command(path, "git", &["remote", "get-url", "origin"])?;
        let origin = std::path::Path::new(origin.trim());
        anyhow::ensure!(
            origin.is_absolute() && origin.canonicalize()?.starts_with(&root),
            "eval repository must use a local origin inside the eval root"
        );
        Ok(true)
    }
    pub fn commit_for(
        &self,
        path: &std::path::Path,
        runner: &process::ProcessRunner,
    ) -> Result<Vec<String>> {
        Ok(if self.local_eval(path, runner)? {
            vec!["git".into(), "commit".into(), "-m".into()]
        } else {
            self.commit_command.clone()
        })
    }
}

#[derive(Default, Debug)]
pub struct TickReport {
    pub planned: usize,
    pub queued: usize,
    pub blockers: Vec<String>,
    pub merged: usize,
    pub satisfied: usize,
}

/// The retained location and immutable commit of one validated engineering plan.
#[derive(Debug)]
pub struct PlanSnapshot {
    pub worktree_id: String,
    pub path: PathBuf,
    pub commit: String,
    pub stories: Vec<String>,
}

#[cfg(test)]
mod eval_policy_tests {
    use super::*;
    #[test]
    fn go_and_local_commits_require_both_local_repository_and_local_origin() -> Result<()> {
        let root = tempfile::tempdir()?;
        let repo = root.path().join("repo");
        let origin = root.path().join("origin.git");
        std::fs::create_dir(&repo)?;
        std::fs::create_dir(&origin)?;
        let runner = process::ProcessRunner {
            environment: vec![],
            timeout: std::time::Duration::from_secs(5),
            cancel: Default::default(),
        };
        runner.command(&repo, "git", &["init"])?;
        runner.command(
            &repo,
            "git",
            &["remote", "add", "origin", origin.to_str().unwrap()],
        )?;
        let config = RuntimeConfig {
            local_eval_root: Some(root.path().into()),
            ..Default::default()
        };
        assert!(config.local_eval(&repo, &runner)?);
        assert_eq!(config.commit_for(&repo, &runner)?, ["git", "commit", "-m"]);
        assert!(!RuntimeConfig::default().local_eval(&repo, &runner)?);
        let elsewhere = tempfile::tempdir()?;
        let outside = RuntimeConfig {
            local_eval_root: Some(elsewhere.path().into()),
            ..Default::default()
        };
        assert_eq!(outside.commit_for(&repo, &runner)?, outside.commit_command);
        runner.command(
            &repo,
            "git",
            &[
                "remote",
                "set-url",
                "origin",
                "https://example.invalid/repo",
            ],
        )?;
        assert!(config.local_eval(&repo, &runner).is_err());
        Ok(())
    }
}
