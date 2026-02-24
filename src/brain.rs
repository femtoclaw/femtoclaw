use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[async_trait::async_trait]
pub trait Brain: Send + Sync {
    async fn think(&self, messages: Vec<Message>) -> Result<String>;
}

pub struct RemoteBrain {
    client: async_openai::Client,
    model: String,
}

impl RemoteBrain {
    pub fn new(_api_key: String, _base_url: Option<String>, model: String) -> Self {
        let client = async_openai::Client::new();
        
        // Note: For custom base URLs, use OPENAI_BASE_URL env var or set it directly
        // The async-openai 0.10.x doesn't have with_base_url method
        
        Self { client, model }
    }
}

#[async_trait::async_trait]
impl Brain for RemoteBrain {
    async fn think(&self, messages: Vec<Message>) -> Result<String> {
        let req = async_openai::types::CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(messages.into_iter().map(|m| {
                async_openai::types::ChatCompletionRequestMessageArgs::default()
                    .role(async_openai::types::Role::User)
                    .content(m.content)
                    .build()
                    .unwrap()
            }).collect::<Vec<_>>())
            .build()?;

        let response = self.client.chat().create(req).await?;
        Ok(response.choices[0].message.content.clone())
    }
}

pub struct LocalBrain {
    url: String,
    model: String,
}

impl LocalBrain {
    pub fn new(url: String, model: String) -> Self {
        Self { url, model }
    }
}

#[async_trait::async_trait]
impl Brain for LocalBrain {
    async fn think(&self, messages: Vec<Message>) -> Result<String> {
        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "stream": false
        });

        let resp = client.post(&format!("{}/api/chat", self.url))
            .json(&payload)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok(resp["message"]["content"].as_str().unwrap_or("No response").to_string())
    }
}
