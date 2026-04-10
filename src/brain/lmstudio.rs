//! LM Studio brain.
//!
//! LM Studio is a local model runner for macOS, Linux, and Windows.
//! See https://lmstudio.ai and https://lmstudio.ai/docs/developer
//!
//! Env:
//! - FEMTO_LMSTUDIO_BASE_URL (default: http://localhost:1234/v1)
//! - FEMTO_LMSTUDIO_MODEL (default: auto-detected)

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::json;

use crate::{brain::Brain, types::Message};

pub struct LmStudioBrain {
    client: reqwest::Client,
    base_url: String,
    model: String,
}

impl LmStudioBrain {
    pub fn from_env() -> anyhow::Result<Self> {
        let base_url = std::env::var("FEMTO_LMSTUDIO_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:1234/v1".to_string());
        let model =
            std::env::var("FEMTO_LMSTUDIO_MODEL").unwrap_or_else(|_| "local-model".to_string());

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(120))
            .build()?;

        Ok(Self {
            client,
            base_url,
            model,
        })
    }

    pub async fn list_models(&self) -> anyhow::Result<Vec<String>> {
        let url = format!("{}/models", self.base_url.trim_end_matches('/'));
        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "lmstudio list models failed: {}",
                resp.status()
            ));
        }

        let v: serde_json::Value = resp.json().await?;
        let models = v["data"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("unexpected lmstudio response"))?
            .iter()
            .filter_map(|m| m["id"].as_str().map(String::from))
            .collect();

        Ok(models)
    }
}

#[async_trait]
impl Brain for LmStudioBrain {
    async fn think(&self, messages: &[Message]) -> anyhow::Result<String> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

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

        let resp = self.client.post(&url).json(&body).send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("lmstudio request failed: {status} {text}"));
        }

        let v: serde_json::Value = resp.json().await?;
        let content = v["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("unexpected lmstudio response shape"))?;

        Ok(content.to_string())
    }
}
