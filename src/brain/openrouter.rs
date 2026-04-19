//! OpenRouter.ai brain.
//!
//! OpenRouter provides unified API to many LLM providers.
//! See https://openrouter.ai
//!
//! Env:
//! - FEMTO_OPENROUTER_API_KEY (required)
//! - FEMTO_OPENROUTER_MODEL (default: openai/gpt-4.1-mini)
//! - FEMTO_OPENROUTER_TIMEOUT_SECS (optional, default: 180)

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;

use crate::{brain::Brain, types::Message};

pub struct OpenRouterBrain {
    client: reqwest::Client,
    api_key: String,
    model: String,
    timeout_secs: u64,
}

impl OpenRouterBrain {
    pub fn from_env() -> anyhow::Result<Self> {
        let api_key = std::env::var("FEMTO_OPENROUTER_API_KEY").map_err(|_| {
            anyhow::anyhow!("FEMTO_OPENROUTER_API_KEY is required when FEMTO_BRAIN=openrouter")
        })?;

        let model = std::env::var("FEMTO_OPENROUTER_MODEL")
            .unwrap_or_else(|_| "openai/gpt-4.1-mini".to_string());

        let timeout_secs = std::env::var("FEMTO_OPENROUTER_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(180)
            .clamp(10, 600);

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "HTTP-Referer",
            HeaderValue::from_static("https://femtoclaw.ai"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .connect_timeout(std::time::Duration::from_secs(20))
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()?;

        Ok(Self {
            client,
            api_key,
            model,
            timeout_secs,
        })
    }

    pub fn popular_models() -> Vec<&'static str> {
        vec![
            "openai/gpt-4.1",
            "openai/gpt-4.1-mini",
            "anthropic/claude-sonnet-4-5",
            "google/gemini-2.0-flash",
            "meta-llama/llama-3.3-70b-instruct",
            "deepseek/deepseek-chat",
            "mistralai/mistral-small",
            "xai/grok-2",
        ]
    }
}

#[async_trait]
impl Brain for OpenRouterBrain {
    async fn think(&self, messages: &[Message]) -> anyhow::Result<String> {
        let url = "https://openrouter.ai/api/v1/chat/completions";

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

        let system_guard = r#"You are FemtoClaw - Industrial Agent Runtime. Output STRICT JSON only. Output exactly one of: {\"message\":{\"content\":\"...\"}} OR {\"tool_call\":{\"tool\":\"...\",\"args\":{...}}}. Do not include extra keys."#;

        let mut final_msgs = vec![json!({"role":"system","content":system_guard})];
        final_msgs.extend(msgs);

        let body_json_mode = json!({
            "model": self.model,
            "messages": final_msgs,
            "temperature": 0.2,
            "response_format": { "type": "json_object" }
        });

        let body_plain = json!({
            "model": self.model,
            "messages": final_msgs,
            "temperature": 0.2
        });

        let mut use_json_mode = true;

        // Retry once on timeout; if model rejects json mode, fallback without it.
        for attempt in 0..3 {
            let body = if use_json_mode {
                &body_json_mode
            } else {
                &body_plain
            };

            let resp = match self
                .client
                .post(url)
                .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
                .json(body)
                .send()
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    if e.is_timeout() && attempt < 2 {
                        continue;
                    }
                    return Err(anyhow::anyhow!("openrouter request failed: {e}"));
                }
            };

            let status = resp.status();
            let text = match resp.text().await {
                Ok(text) => text,
                Err(e) => {
                    if e.is_timeout() && attempt < 2 {
                        continue;
                    }
                    return Err(anyhow::anyhow!(
                        "failed to read openrouter response body: {e}; consider increasing FEMTO_OPENROUTER_TIMEOUT_SECS (current={})",
                        self.timeout_secs
                    ));
                }
            };

            if !status.is_success() {
                if use_json_mode && is_response_format_unsupported(status, &text) {
                    use_json_mode = false;
                    continue;
                }

                return Err(anyhow::anyhow!(
                    "openrouter request failed: {status} {text}"
                ));
            }

            let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
                anyhow::anyhow!(
                    "unexpected openrouter JSON response: {e}; body={}",
                    truncate(&text, 400)
                )
            })?;

            let content = v["choices"][0]["message"]["content"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("unexpected openrouter response shape"))?;

            return Ok(content.to_string());
        }

        Err(anyhow::anyhow!(
            "openrouter request timed out; use a faster model, shorter prompt, or increase FEMTO_OPENROUTER_TIMEOUT_SECS (current={})",
            self.timeout_secs
        ))
    }
}

fn is_response_format_unsupported(status: reqwest::StatusCode, body: &str) -> bool {
    if status != reqwest::StatusCode::BAD_REQUEST {
        return false;
    }

    let b = body.to_lowercase();
    b.contains("response_format") && b.contains("json_object") && b.contains("not supported")
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut out = s[..max].to_string();
    out.push_str("...(truncated)");
    out
}
