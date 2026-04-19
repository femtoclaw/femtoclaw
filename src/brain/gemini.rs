//! Google Gemini brain.

use async_trait::async_trait;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;

use crate::{brain::Brain, types::Message};

pub struct GeminiBrain {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl GeminiBrain {
    pub fn from_env() -> anyhow::Result<Self> {
        let api_key = std::env::var("FEMTO_GEMINI_API_KEY").map_err(|_| {
            anyhow::anyhow!("FEMTO_GEMINI_API_KEY is required when FEMTO_BRAIN=gemini")
        })?;
        let model =
            std::env::var("FEMTO_GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.0-flash".to_string());

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
impl Brain for GeminiBrain {
    async fn think(&self, messages: &[Message]) -> anyhow::Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let system_guard = r#"You are FemtoClaw — Industrial Agent Runtime. Output STRICT JSON only. Output exactly one of: {"message":{"content":"..."}} OR {"tool_call":{"tool":"...","args":{...}}}. Do not include extra keys."#;

        let contents: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    crate::types::Role::System => "user",
                    crate::types::Role::User => "user",
                    crate::types::Role::Assistant => "model",
                    crate::types::Role::Tool => "user",
                };
                json!({"role": role, "parts": [{"text": m.content}]})
            })
            .collect();

        let body = json!({
            "systemInstruction": {"parts": [{"text": system_guard}]},
            "contents": contents,
            "generationConfig": {
                "temperature": 0.2,
                "responseMimeType": "application/json"
            }
        });

        let resp = self
            .client
            .post(&url)
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("gemini request failed: {status} {text}"));
        }

        let v: serde_json::Value = resp.json().await?;
        let content = v["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("unexpected gemini response shape"))?;

        Ok(content.to_string())
    }
}
