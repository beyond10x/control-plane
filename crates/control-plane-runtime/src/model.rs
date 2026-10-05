use crate::{AgentModel, ModelRequest};
use anyhow::{Context, Result};
use b10x_llm_tool_call::{call_tool, codex_model};
use llm_core::{Item, ToolName, ToolSpec};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::Duration;

/// Existing Codex login, accessed through LLM's read-only credential resolver.
pub struct CodexAgentModel {
    pub timeout: Duration,
}

impl Default for CodexAgentModel {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(600),
        }
    }
}

impl AgentModel for CodexAgentModel {
    fn respond(&self, request: &ModelRequest) -> Result<Value> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let model = codex_model(&request.model)?;
        let tool = ToolSpec {
            name: ToolName::new("planner_response")?,
            description: format!("{} response", request.role),
            input_schema: request.schema.clone(),
        };
        runtime.block_on(async {
            tokio::time::timeout(
                self.timeout.min(request.timeout),
                call_tool(
                    &model,
                    &request.instructions,
                    vec![Item::user(&request.prompt)],
                    tool,
                ),
            )
            .await
            .context("model deadline exceeded")?
            .map_err(Into::into)
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum PlannerAction {
    Read {
        paths: Vec<String>,
    },
    WriteSpecification {
        path: String,
        contents: String,
    },
    Aep {
        args: Vec<String>,
        body: Option<String>,
    },
    Finish {
        stories: Vec<String>,
        summary: String,
    },
}

impl PlannerAction {
    pub fn protocol_action(&self) -> &'static str {
        match self {
            Self::Read { .. } => "repository.inspect",
            Self::WriteSpecification { .. } | Self::Aep { .. } => "plan.edit",
            Self::Finish { .. } => "plan.validate",
        }
    }
}

pub fn planner_schema() -> Value {
    json!({"oneOf":[
        {"type":"object","properties":{"action":{"const":"read"},"paths":{"type":"array","items":{"type":"string"}}},"required":["action","paths"],"additionalProperties":false},
        {"type":"object","properties":{"action":{"const":"write_specification"},"path":{"type":"string"},"contents":{"type":"string"}},"required":["action","path","contents"],"additionalProperties":false},
        {"type":"object","properties":{"action":{"const":"aep"},"args":{"type":"array","items":{"type":"string"}},"body":{"type":["string","null"]}},"required":["action","args","body"],"additionalProperties":false},
        {"type":"object","properties":{"action":{"const":"finish"},"stories":{"type":"array","items":{"type":"string"}},"summary":{"type":"string"}},"required":["action","stories","summary"],"additionalProperties":false}
    ]})
}

pub fn critique_schema() -> Value {
    json!({"type":"object","properties":{"approved":{"type":"boolean"},"reason":{"type":"string"}},"required":["approved","reason"],"additionalProperties":false})
}
