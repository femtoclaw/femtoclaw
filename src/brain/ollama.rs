//! Ollama-compatible Chat Completions brain.
//!
//! Env:
//! - FEMTO_OLLAMA_BASE_URL (default: http://localhost:11434)
//! - FEMTO_OLLAMA_MODEL (default: llama3.2)
//!
//! IMPORTANT: For Industrial Runtime, we request the model to output strict JSON only.

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::json;

use crate::{brain::Brain, types::Message};

pub struct OllamaBrain {
    client: reqwest::Client,
    base_url: String,
    model: String,
}

impl OllamaBrain {
    pub fn from_env() -> anyhow::Result<Self> {
        let base_url = std::env::var("FEMTO_OLLAMA_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        let model = std::env::var("FEMTO_OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2".to_string());

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(60))
            .build()?;

        Ok(Self {
            client,
            base_url,
            model,
        })
    }

    pub async fn list_models(&self) -> anyhow::Result<Vec<String>> {
        let url = format!("{}/api/tags", self.base_url.trim_end_matches('/'));
        let resp = self.client.get(url).send().await?;

        if !resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "ollama list models failed: {}",
                resp.status()
            ));
        }

        let v: serde_json::Value = resp.json().await?;
        let models = v["models"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("unexpected ollama response"))?
            .iter()
            .filter_map(|m| m["name"].as_str().map(String::from))
            .collect();

        Ok(models)
    }
}

#[async_trait]
impl Brain for OllamaBrain {
    async fn think(&self, messages: &[Message]) -> anyhow::Result<String> {
        let url = format!("{}/api/chat", self.base_url.trim_end_matches('/'));

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

        let system_guard = r#"You are FemtoClaw — Industrial Agent Runtime. Output STRICT JSON only, no markdown. Output exactly one of: {"message":{"content":"..."}} OR {"tool_call":{"tool":"...","args":{...}}}. Do not include extra keys."#;

        let mut final_msgs = vec![json!({"role":"system","content":system_guard})];
        final_msgs.extend(msgs);

        let body = json!({
            "model": self.model,
            "messages": final_msgs,
            "temperature": 0.2,
            "format": "json",
            "stream": false
        });

        let resp = self.client.post(url).json(&body).send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("ollama request failed: {status} {text}"));
        }

        let v: serde_json::Value = resp.json().await?;
        let content = v["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("unexpected ollama response shape"))?;

        Ok(content.to_string())
    }
}
