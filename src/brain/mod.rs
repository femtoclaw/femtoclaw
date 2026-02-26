use async_trait::async_trait;

use crate::types::Message;

pub mod echo;
pub mod openai;

#[async_trait]
pub trait Brain: Send + Sync {
    async fn think(&self, messages: &[Message]) -> anyhow::Result<String>;
}

pub enum BrainKind {
    Echo(echo::EchoBrain),
    OpenAI(openai::OpenAIBrain),
}

impl BrainKind {
    pub fn from_env() -> anyhow::Result<Self> {
        let kind = std::env::var("FEMTO_BRAIN").unwrap_or_else(|_| "echo".to_string());
        match kind.as_str() {
            "echo" => Ok(Self::Echo(echo::EchoBrain::default())),
            "openai" => Ok(Self::OpenAI(openai::OpenAIBrain::from_env()?)),
            other => Err(anyhow::anyhow!(
                "unknown FEMTO_BRAIN='{other}' (use 'echo' or 'openai')"
            )),
        }
    }
}

#[async_trait]
impl Brain for BrainKind {
    async fn think(&self, messages: &[Message]) -> anyhow::Result<String> {
        match self {
            BrainKind::Echo(b) => b.think(messages).await,
            BrainKind::OpenAI(b) => b.think(messages).await,
        }
    }
}
