//! Anthropic-compatible API brain.
//!
//! Env:
//! - FEMTO_ANTHROPIC_API_KEY (required)
//! - FEMTO_ANTHROPIC_MODEL (default: claude-3-5-sonnet-20241022)
//!
//! IMPORTANT: For Industrial Runtime, we request the model to output strict JSON only.

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::json;

use crate::{brain::Brain, types::Message};

pub struct AnthropicBrain {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl AnthropicBrain {
    pub fn from_env() -> anyhow::Result<Self> {
        let api_key = std::env::var("FEMTO_ANTHROPIC_API_KEY").map_err(|_| {
            anyhow::anyhow!("FEMTO_ANTHROPIC_API_KEY is required when FEMTO_BRAIN=anthropic")
        })?;
        let model = std::env::var("FEMTO_ANTHROPIC_MODEL")
            .unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string());

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("x-api-key", HeaderValue::from_str(&api_key)?);
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(60))
            .build()?;

        Ok(Self {
            client,
            api_key,
            model,
        })
    }
}

#[async_trait]
impl Brain for AnthropicBrain {
    async fn think(&self, messages: &[Message]) -> anyhow::Result<String> {
        let url = "https://api.anthropic.com/v1/messages";

        let system_guard = r#"You are FemtoClaw — Industrial Agent Runtime. Output STRICT JSON only, no markdown. Output exactly one of: {"message":{"content":"..."}} OR {"tool_call":{"tool":"...","args":{...}}}. Do not include extra keys."#;

        let msgs: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    crate::types::Role::System => "user", // Anthropic doesn't have system in messages
                    crate::types::Role::User => "user",
                    crate::types::Role::Assistant => "assistant",
                    crate::types::Role::Tool => "user",
                };
                json!({"role": role, "content": m.content})
            })
            .collect();

        let body = json!({
            "model": self.model,
            "max_tokens": 1024,
            "system": system_guard,
            "messages": msgs,
        });

        let resp = self.client.post(url).json(&body).send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("anthropic request failed: {status} {text}"));
        }

        let v: serde_json::Value = resp.json().await?;
        let content = v["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("unexpected anthropic response shape"))?;

        Ok(content.to_string())
    }
}
