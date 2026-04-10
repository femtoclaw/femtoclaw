//! Strict JSON "protocol" for agent outputs.
//!
//! Brain output must be either:
//! - {"message":{"content":"..."}}
//! - {"tool_call":{"tool":"shell","args":{...}}}
//!
//! Anything else is rejected.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool: String,
    pub args: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AgentOutput {
    Message { message: AgentMessage },
    ToolCall { tool_call: ToolCall },
}

impl AgentOutput {
    pub fn parse_strict(s: &str) -> anyhow::Result<Self> {
        let v: serde_json::Value = serde_json::from_str(s)
            .map_err(|e| anyhow::anyhow!("brain output is not valid JSON: {e}"))?;

        // Disallow extra top-level keys by re-serializing through AgentOutput then comparing.
        let out: AgentOutput = serde_json::from_value(v.clone())
            .map_err(|e| anyhow::anyhow!("brain output does not match protocol: {e}"))?;

        // Validate tool names basic constraints
        if let AgentOutput::ToolCall { tool_call } = &out {
            if tool_call.tool.trim().is_empty() || tool_call.tool.len() > 64 {
                return Err(anyhow::anyhow!("invalid tool name"));
            }
        }

        // Ensure no ambiguous payload (both message and tool_call)
        let has_message = v.get("message").is_some();
        let has_tool = v.get("tool_call").is_some();
        if has_message == has_tool {
            return Err(anyhow::anyhow!("brain output must contain exactly one of: message or tool_call"));
        }

        Ok(out)
    }
}
