//! OpenCode Zen brain.
//!
//! Zen is a curated gateway with pre-tested, optimized models for coding agents.
//! See https://opencode.ai/zen
//!
//! Env:
//! - FEMTO_ZEN_API_KEY (required) - Get from https://opencode.ai/zen
//! - FEMTO_ZEN_MODEL (default: openai/gpt-4.1)

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;

use crate::{brain::Brain, types::Message};

pub struct ZenBrain {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl ZenBrain {
    pub fn from_env() -> anyhow::Result<Self> {
        let api_key = std::env::var("FEMTO_ZEN_API_KEY").map_err(|_| {
            anyhow::anyhow!("FEMTO_ZEN_API_KEY is required when FEMTO_BRAIN=zen. Get from https://opencode.ai/zen")
        })?;
        let model =
            std::env::var("FEMTO_ZEN_MODEL").unwrap_or_else(|_| "openai/gpt-4.1".to_string());

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

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

    pub fn available_models() -> Vec<&'static str> {
        vec![
            "openai/gpt-4.1",
            "openai/gpt-4.1-mini",
            "anthropic/claude-sonnet-4-20250514",
            "anthropic/claude-sonnet-4-5",
            "xai/grok-2",
            "google/gemini-2.5-pro-preview-06-05",
            "deepseek/deepseek-coder-v2",
            "mistral/codestral-2505",
        ]
    }
}

#[async_trait]
impl Brain for ZenBrain {
    async fn think(&self, messages: &[Message]) -> anyhow::Result<String> {
        let url = "https://opencode.ai/api/v0/chat";

        let msgs: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    crate::types::Role::System => "system",
                    crate::types::Role::User => "user",
                    crate::types::Role::Assistant => "assistant",
                    crate::types::Role::Tool => "tool",
                };
                json!({"role": role, "content": m.content})
            })
            .collect();

        let system_guard = r#"You are FemtoClaw — Industrial Agent Runtime. Output STRICT JSON only. Output exactly one of: {"message":{"content":"..."}} OR {"tool_call":{"tool":"...","args":{...}}}. Do not include extra keys."#;

        let mut final_msgs = vec![json!({"role":"system","content":system_guard})];
        final_msgs.extend(msgs);

        let body = json!({
            "model": self.model,
            "messages": final_msgs,
            "temperature": 0.2
        });

        let resp = self
            .client
            .post(url)
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("zen request failed: {status} {text}"));
        }

        let v: serde_json::Value = resp.json().await?;
        let content = v["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("unexpected zen response shape"))?;

        Ok(content.to_string())
    }
}
