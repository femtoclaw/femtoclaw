//! OpenAI-compatible Chat Completions brain.
//!
//! Env:
//! - FEMTO_OPENAI_BASE_URL (default: https://api.openai.com/v1)
//! - FEMTO_OPENAI_API_KEY (required)
//! - FEMTO_OPENAI_MODEL (default: gpt-4.1-mini)
//!
//! IMPORTANT: For Industrial Runtime, we request the model to output strict JSON only.

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;

use crate::{brain::Brain, types::Message};

pub struct OpenAIBrain {
    client: reqwest::Client,
    base_url: String,
    model: String,
}

impl OpenAIBrain {
    pub fn from_env() -> anyhow::Result<Self> {
        let base_url = std::env::var("FEMTO_OPENAI_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let api_key = std::env::var("FEMTO_OPENAI_API_KEY")
            .map_err(|_| anyhow::anyhow!("FEMTO_OPENAI_API_KEY is required when FEMTO_BRAIN=openai"))?;
        let model = std::env::var("FEMTO_OPENAI_MODEL")
            .unwrap_or_else(|_| "gpt-4.1-mini".to_string());

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {api_key}"))?);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self { client, base_url, model })
    }
}

#[async_trait]
impl Brain for OpenAIBrain {
    async fn think(&self, messages: &[Message]) -> anyhow::Result<String> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let msgs = messages.iter().map(|m| {
            let role = match m.role {
                crate::types::Role::System => "system",
                crate::types::Role::User => "user",
                crate::types::Role::Assistant => "assistant",
                crate::types::Role::Tool => "tool",
            };
            json!({"role": role, "content": m.content})
        }).collect::<Vec<_>>();

        let system_guard = "You are FemtoClaw — Industrial Agent Runtime. Output STRICT JSON only, no markdown.             Output exactly one of: {"message":{"content":"..."}} OR {"tool_call":{"tool":"...","args":{...}}}.             Do not include extra keys.";

        let mut final_msgs = vec![json!({"role":"system","content":system_guard})];
        final_msgs.extend(msgs);

        let body = json!({
            "model": self.model,
            "messages": final_msgs,
            "temperature": 0.2,
            "response_format": { "type": "json_object" }
        });

        let resp = self.client.post(url).json(&body).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("openai request failed: {status} {text}"));
        }
        let v: serde_json::Value = resp.json().await?;
        let content = v["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("unexpected openai response shape"))?;
        Ok(content.to_string())
    }
}
