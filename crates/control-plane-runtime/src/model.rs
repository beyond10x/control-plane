use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum PlannerAction {
    Read {
        paths: Vec<String>,
    },
    ReadRange {
        path: String,
        start_line: usize,
        line_count: usize,
    },
    ReadBytes {
        path: String,
        start_byte: usize,
        byte_count: usize,
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
            Self::Read { .. } | Self::ReadRange { .. } | Self::ReadBytes { .. } => {
                "repository.inspect"
            }
            Self::WriteSpecification { .. } | Self::Aep { .. } => "plan.edit",
            Self::Finish { .. } => "plan.validate",
        }
    }
}

pub fn planner_schema() -> Value {
    json!({"oneOf":[
        {"type":"object","properties":{"action":{"const":"read"},"paths":{"type":"array","items":{"type":"string"}}},"required":["action","paths"],"additionalProperties":false},
        {"type":"object","properties":{"action":{"const":"read_range"},"path":{"type":"string"},"start_line":{"type":"integer","minimum":1},"line_count":{"type":"integer","minimum":1,"maximum":200}},"required":["action","path","start_line","line_count"],"additionalProperties":false},
        {"type":"object","properties":{"action":{"const":"read_bytes"},"path":{"type":"string"},"start_byte":{"type":"integer","minimum":0},"byte_count":{"type":"integer","minimum":1,"maximum":12288}},"required":["action","path","start_byte","byte_count"],"additionalProperties":false},
        {"type":"object","properties":{"action":{"const":"write_specification"},"path":{"type":"string"},"contents":{"type":"string"}},"required":["action","path","contents"],"additionalProperties":false},
        {"type":"object","properties":{"action":{"const":"aep"},"args":{"type":"array","items":{"type":"string"}},"body":{"type":["string","null"]}},"required":["action","args","body"],"additionalProperties":false},
        {"type":"object","properties":{"action":{"const":"finish"},"stories":{"type":"array","items":{"type":"string"}},"summary":{"type":"string"}},"required":["action","stories","summary"],"additionalProperties":false}
    ]})
}

pub fn critique_schema() -> Value {
    json!({"type":"object","properties":{"approved":{"type":"boolean"},"reason":{"type":"string"}},"required":["approved","reason"],"additionalProperties":false})
}
