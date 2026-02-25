//! Bounded HTTP GET.
//!
//! Args:
//! { "url": "https://example.com" }
//!
//! Security:
//! - timeout enforced
//! - max bytes enforced
//! - only http/https schemes
//! - response body is truncated to max_bytes

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;

use crate::tools::Claw;

#[derive(Clone)]
pub struct WebGetClaw {
    client: reqwest::Client,
    max_bytes: usize,
}

impl WebGetClaw {
    pub fn new(max_bytes: usize, timeout_secs: u64) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()?;
        Ok(Self { client, max_bytes })
    }
}

#[derive(Debug, Deserialize)]
struct WebGetArgs {
    url: String,
}

#[async_trait]
impl Claw for WebGetClaw {
    fn name(&self) -> &'static str { "web_get" }
    fn description(&self) -> &'static str { "Bounded HTTP GET (timeout + max bytes)." }

    async fn execute(&self, args: Value) -> anyhow::Result<String> {
        let parsed: WebGetArgs = serde_json::from_value(args)
            .map_err(|e| anyhow::anyhow!("web_get args invalid: {e}"))?;

        let url = parsed.url;
        if !(url.starts_with("https://") || url.starts_with("http://")) {
            return Err(anyhow::anyhow!("url must be http(s)"));
        }

        let resp = self.client.get(url).send().await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;
        let mut out = bytes.to_vec();
        if out.len() > self.max_bytes {
            out.truncate(self.max_bytes);
        }
        let text = String::from_utf8_lossy(&out).to_string();

        Ok(format!("STATUS: {status}\n{text}"))
    }
}
