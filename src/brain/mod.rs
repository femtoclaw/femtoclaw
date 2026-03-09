use async_trait::async_trait;

use crate::types::Message;

pub mod anthropic;
pub mod azure;
pub mod bedrock;
pub mod deepseek;
pub mod echo;
pub mod gemini;
pub mod grok;
pub mod lmstudio;
pub mod mistral;
pub mod ollama;
pub mod openai;
pub mod openrouter;
pub mod perplexity;
pub mod zen;

#[async_trait]
pub trait Brain: Send + Sync {
    async fn think(&self, messages: &[Message]) -> anyhow::Result<String>;
}

pub enum BrainKind {
    Echo(echo::EchoBrain),
    OpenAI(openai::OpenAIBrain),
    Ollama(ollama::OllamaBrain),
    Anthropic(anthropic::AnthropicBrain),
    Grok(grok::GrokBrain),
    Gemini(gemini::GeminiBrain),
    Bedrock(bedrock::BedrockBrain),
    Azure(azure::AzureBrain),
    Mistral(mistral::MistralBrain),
    DeepSeek(deepseek::DeepSeekBrain),
    Perplexity(perplexity::PerplexityBrain),
    Zen(zen::ZenBrain),
    OpenRouter(openrouter::OpenRouterBrain),
    LmStudio(lmstudio::LmStudioBrain),
}

impl BrainKind {
    pub fn from_name(kind: &str) -> anyhow::Result<Self> {
        match kind {
            "echo" => Ok(Self::Echo(echo::EchoBrain::default())),
            "openai" => Ok(Self::OpenAI(openai::OpenAIBrain::from_env()?)),
            "ollama" => Ok(Self::Ollama(ollama::OllamaBrain::from_env()?)),
            "anthropic" => Ok(Self::Anthropic(anthropic::AnthropicBrain::from_env()?)),
            "grok" => Ok(Self::Grok(grok::GrokBrain::from_env()?)),
            "gemini" => Ok(Self::Gemini(gemini::GeminiBrain::from_env()?)),
            "bedrock" => Ok(Self::Bedrock(bedrock::BedrockBrain::from_env()?)),
            "azure" => Ok(Self::Azure(azure::AzureBrain::from_env()?)),
            "mistral" => Ok(Self::Mistral(mistral::MistralBrain::from_env()?)),
            "deepseek" => Ok(Self::DeepSeek(deepseek::DeepSeekBrain::from_env()?)),
            "perplexity" => Ok(Self::Perplexity(perplexity::PerplexityBrain::from_env()?)),
            "zen" => Ok(Self::Zen(zen::ZenBrain::from_env()?)),
            "openrouter" => Ok(Self::OpenRouter(openrouter::OpenRouterBrain::from_env()?)),
            "lmstudio" => Ok(Self::LmStudio(lmstudio::LmStudioBrain::from_env()?)),
            other => Err(anyhow::anyhow!("unknown FEMTO_BRAIN='{other}'")),
        }
    }

    pub fn from_env() -> anyhow::Result<Self> {
        let kind = std::env::var("FEMTO_BRAIN").unwrap_or_else(|_| "echo".to_string());
        Self::from_name(&kind)
    }
}

#[async_trait]
impl Brain for BrainKind {
    async fn think(&self, messages: &[Message]) -> anyhow::Result<String> {
        match self {
            BrainKind::Echo(b) => b.think(messages).await,
            BrainKind::OpenAI(b) => b.think(messages).await,
            BrainKind::Ollama(b) => b.think(messages).await,
            BrainKind::Anthropic(b) => b.think(messages).await,
            BrainKind::Grok(b) => b.think(messages).await,
            BrainKind::Gemini(b) => b.think(messages).await,
            BrainKind::Bedrock(b) => b.think(messages).await,
            BrainKind::Azure(b) => b.think(messages).await,
            BrainKind::Mistral(b) => b.think(messages).await,
            BrainKind::DeepSeek(b) => b.think(messages).await,
            BrainKind::Perplexity(b) => b.think(messages).await,
            BrainKind::Zen(b) => b.think(messages).await,
            BrainKind::OpenRouter(b) => b.think(messages).await,
            BrainKind::LmStudio(b) => b.think(messages).await,
        }
    }
}
