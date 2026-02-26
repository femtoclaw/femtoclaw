//! Azure OpenAI brain.

use async_trait::async_trait;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;

use crate::{brain::Brain, types::Message};

pub struct AzureBrain {
    client: reqwest::Client,
    endpoint: String,
    api_key: String,
    deployment: String,
    api_version: String,
}

impl AzureBrain {
    pub fn from_env() -> anyhow::Result<Self> {
        let endpoint = std::env::var("FEMTO_AZURE_ENDPOINT").map_err(|_| {
            anyhow::anyhow!("FEMTO_AZURE_ENDPOINT is required when FEMTO_BRAIN=azure")
        })?;
        let api_key = std::env::var("FEMTO_AZURE_API_KEY").map_err(|_| {
            anyhow::anyhow!("FEMTO_AZURE_API_KEY is required when FEMTO_BRAIN=azure")
        })?;
        let deployment =
            std::env::var("FEMTO_AZURE_DEPLOYMENT").unwrap_or_else(|_| "gpt-4".to_string());
        let api_version = std::env::var("FEMTO_AZURE_API_VERSION")
            .unwrap_or_else(|_| "2024-02-15-preview".to_string());

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()?;

        Ok(Self {
            client,
            endpoint,
            api_key,
            deployment,
            api_version,
        })
    }

    fn get_url(&self) -> String {
        format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.endpoint.trim_end_matches('/'),
            self.deployment,
            self.api_version
        )
    }
}

#[async_trait]
impl Brain for AzureBrain {
    async fn think(&self, messages: &[Message]) -> anyhow::Result<String> {
        let url = self.get_url();

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
            "messages": final_msgs,
            "temperature": 0.2,
            "response_format": {"type": "json_object"}
        });

        let resp = self
            .client
            .post(&url)
            .header(CONTENT_TYPE, "application/json")
            .header("api-key", &self.api_key)
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("azure request failed: {status} {text}"));
        }

        let v: serde_json::Value = resp.json().await?;
        let content = v["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("unexpected azure response shape"))?;

        Ok(content.to_string())
    }
}
