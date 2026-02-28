//! Deterministic brain for offline use and tests.
//!
//! It always produces a strict JSON message response (no tool calls).

use async_trait::async_trait;

use crate::{brain::Brain, types::Message};

#[derive(Default)]
pub struct EchoBrain;

#[async_trait]
impl Brain for EchoBrain {
    async fn think(&self, messages: &[Message]) -> anyhow::Result<String> {
        let last = messages
            .iter()
            .rev()
            .find(|m| matches!(m.role, crate::types::Role::User));
        let content = last.map(|m| m.content.as_str()).unwrap_or("");
        let reply = format!(
            "{{\"message\":{{\"content\":\"ACK: {}\"}}}}",
            escape_json_string(content)
        );
        Ok(reply)
    }
}

fn escape_json_string(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect(),
            '\n' => "\\n".chars().collect(),
            '\r' => "\\r".chars().collect(),
            '\t' => "\\t".chars().collect(),
            _ => vec![c],
        })
        .collect()
}
