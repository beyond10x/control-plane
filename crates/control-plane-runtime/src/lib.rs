//! The local planner and its trusted repository adapters.
use anyhow::Result;
use control_plane_core::Store;
use serde_json::Value;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Mutex;
mod engine;
mod model;
pub mod process;
mod supervisor;
pub use model::CodexAgentModel;
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
}

#[derive(Clone)]
pub struct RuntimeConfig {
    pub environment: Vec<(String, String)>,
    pub aep_protocols: String,
    pub commit_command: Vec<String>,
    pub max_steps: usize,
    pub process_timeout: std::time::Duration,
    pub poll_interval: std::time::Duration,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            environment: Vec::new(),
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
        }
    }
}

#[derive(Default, Debug)]
pub struct TickReport {
    pub planned: usize,
    pub queued: usize,
    pub blockers: Vec<String>,
}

/// The retained location and immutable commit of one validated engineering plan.
#[derive(Debug)]
pub struct PlanSnapshot {
    pub worktree_id: String,
    pub path: PathBuf,
    pub commit: String,
    pub stories: Vec<String>,
}
