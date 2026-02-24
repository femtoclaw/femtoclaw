use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String, // "system", "user", "assistant"
    pub content: String,
}

#[async_trait::async_trait]
pub trait Brain: Send + Sync {
    async fn think(&self, messages: Vec<Message>) -> Result<String>;
}

// --- Remote Brain (OpenAI Compatible) ---
pub struct RemoteBrain {
    client: async_openai::Client<async_openai::config::OpenAIConfig>,
    model: String,
}

impl RemoteBrain {
    pub fn new(api_key: String, base_url: Option<String>, model: String) -> Self {
        let mut config = async_openai::config::OpenAIConfig::new().with_api_key(api_key);
        if let Some(url) = base_url {
            config = config.with_api_base(url);
        }
        Self {
            client: async_openai::Client::with_config(config),
            model,
        }
    }
}

#[async_trait::async_trait]
impl Brain for RemoteBrain {
    async fn think(&self, messages: Vec<Message>) -> Result<String> {
        // Convert to async-openai format
        let req = async_openai::types::CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(messages.into_iter().map(|m| {
                async_openai::types::ChatCompletionRequestMessage::User(
                    async_openai::types::ChatCompletionRequestUserMessageArgs::default()
                        .content(m.content)
                        .build()
                        .unwrap()
                )
            }).collect::<Vec<_>>())
            .build()?;

        let response = self.client.chat().create(req).await?;
        Ok(response.choices[0].message.content.clone().unwrap_or_default())
    }
}

// --- Local Brain (Ollama Placeholder) ---
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
        // Simplified Ollama call logic
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