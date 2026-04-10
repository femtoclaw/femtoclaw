//! DeepSeek brain.

use async_trait::async_trait;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;

use crate::{brain::Brain, types::Message};

pub struct DeepSeekBrain {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl DeepSeekBrain {
    pub fn from_env() -> anyhow::Result<Self> {
        let api_key = std::env::var("FEMTO_DEEPSEEK_API_KEY").map_err(|_| {
            anyhow::anyhow!("FEMTO_DEEPSEEK_API_KEY is required when FEMTO_BRAIN=deepseek")
        })?;
        let model =
            std::env::var("FEMTO_DEEPSEEK_MODEL").unwrap_or_else(|_| "deepseek-chat".to_string());

        let client = reqwest::Client::builder()
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
impl Brain for DeepSeekBrain {
    async fn think(&self, messages: &[Message]) -> anyhow::Result<String> {
        let url = "https://api.deepseek.com/v1/chat/completions";

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
            "temperature": 0.2,
            "response_format": {"type": "json_object"}
        });

        let resp = self
            .client
            .post(url)
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("deepseek request failed: {status} {text}"));
        }

        let v: serde_json::Value = resp.json().await?;
        let content = v["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("unexpected deepseek response shape"))?;

        Ok(content.to_string())
    }
}
