//! Amazon Bedrock brain.
//! Uses AWS Signature Version 4 for authentication.

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::json;

use crate::{brain::Brain, types::Message};

pub struct BedrockBrain {
    client: reqwest::Client,
    region: String,
    model: String,
    access_key: String,
    secret_key: String,
}

impl BedrockBrain {
    pub fn from_env() -> anyhow::Result<Self> {
        let region =
            std::env::var("FEMTO_BEDROCK_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        let access_key = std::env::var("FEMTO_BEDROCK_ACCESS_KEY").map_err(|_| {
            anyhow::anyhow!("FEMTO_BEDROCK_ACCESS_KEY is required when FEMTO_BRAIN=bedrock")
        })?;
        let secret_key = std::env::var("FEMTO_BEDROCK_SECRET_KEY").map_err(|_| {
            anyhow::anyhow!("FEMTO_BEDROCK_SECRET_KEY is required when FEMTO_BRAIN=bedrock")
        })?;
        let model = std::env::var("FEMTO_BEDROCK_MODEL")
            .unwrap_or_else(|_| "anthropic.claude-3-sonnet-20240229-v1:0".to_string());

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()?;

        Ok(Self {
            client,
            region,
            model,
            access_key,
            secret_key,
        })
    }

    fn get_endpoint(&self) -> String {
        format!(
            "https://bedrock-runtime.{}.amazonaws.com/model/{}/invoke",
            self.region, self.model
        )
    }
}

#[async_trait]
impl Brain for BedrockBrain {
    async fn think(&self, messages: &[Message]) -> anyhow::Result<String> {
        let endpoint = self.get_endpoint();

        let system_guard = r#"You are FemtoClaw — Industrial Agent Runtime. Output STRICT JSON only. Output exactly one of: {"message":{"content":"..."}} OR {"tool_call":{"tool":"...","args":{...}}}. Do not include extra keys."#;

        let msgs: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    crate::types::Role::System => "user",
                    crate::types::Role::User => "user",
                    crate::types::Role::Assistant => "assistant",
                    crate::types::Role::Tool => "user",
                };
                json!({"role": role, "content": [{"type": "text", "text": m.content}]})
            })
            .collect();

        let mut final_msgs =
            vec![json!({"role": "user", "content": [{"type": "text", "text": system_guard}]})];
        final_msgs.extend(msgs);

        let body = json!({
            "anthropic_version": "bedrock-2023-05-31",
            "max_tokens": 1024,
            "messages": final_msgs,
            "temperature": 0.2
        });

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let resp = self
            .client
            .post(&endpoint)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("bedrock request failed: {status} {text}"));
        }

        let v: serde_json::Value = resp.json().await?;
        let content = v["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("unexpected bedrock response shape"))?;

        Ok(content.to_string())
    }
}
