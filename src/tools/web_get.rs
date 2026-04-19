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
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use tokio::net::lookup_host;

use crate::tools::Tool;

#[derive(Clone)]
pub struct WebGetTool {
    client: reqwest::Client,
    max_bytes: usize,
    max_redirects: usize,
}

impl WebGetTool {
    pub fn new() -> anyhow::Result<Self> {
        Self::with_limits(1024 * 1024, 30)
    }

    pub fn with_limits(max_bytes: usize, timeout_secs: u64) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        Ok(Self {
            client,
            max_bytes,
            max_redirects: 10,
        })
    }
}

impl Default for WebGetTool {
    fn default() -> Self {
        Self::new().expect("failed to create WebGetTool")
    }
}

#[derive(Debug, Deserialize)]
struct WebGetArgs {
    url: String,
}

#[async_trait]
impl Tool for WebGetTool {
    fn name(&self) -> &'static str {
        "web.get"
    }

    fn description(&self) -> &'static str {
        "Bounded HTTP GET (timeout + max bytes)."
    }

    async fn execute(&self, args: Value) -> anyhow::Result<String> {
        let parsed: WebGetArgs = serde_json::from_value(args)
            .map_err(|e| anyhow::anyhow!("web.get args invalid: {e}"))?;

        let mut url =
            reqwest::Url::parse(&parsed.url).map_err(|e| anyhow::anyhow!("url is invalid: {e}"))?;

        for redirect_count in 0..=self.max_redirects {
            validate_url(&url)?;
            resolve_and_validate(&url).await?;

            let resp = self.client.get(url.clone()).send().await?;
            let status = resp.status();

            if status.is_redirection() {
                if redirect_count == self.max_redirects {
                    return Err(anyhow::anyhow!("too many redirects"));
                }

                let location = resp
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .ok_or_else(|| anyhow::anyhow!("redirect missing location header"))?
                    .to_str()
                    .map_err(|e| anyhow::anyhow!("redirect location is invalid: {e}"))?;
                url = url
                    .join(location)
                    .map_err(|e| anyhow::anyhow!("redirect location is invalid: {e}"))?;
                continue;
            }

            let bytes = resp.bytes().await?;
            let mut out = bytes.to_vec();
            if out.len() > self.max_bytes {
                out.truncate(self.max_bytes);
            }
            let text = String::from_utf8_lossy(&out).to_string();

            return Ok(format!("STATUS: {status}\n{text}"));
        }

        Err(anyhow::anyhow!("request did not complete"))
    }
}

async fn resolve_and_validate(url: &reqwest::Url) -> anyhow::Result<()> {
    let host = url
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("url host is required"))?;
    let port = url
        .port_or_known_default()
        .ok_or_else(|| anyhow::anyhow!("url port is invalid"))?;
    let addrs = lookup_host((host, port))
        .await
        .map_err(|e| anyhow::anyhow!("host resolution failed: {e}"))?;

    let mut found_any = false;
    for addr in addrs {
        found_any = true;
        if is_blocked_ip(addr.ip()) {
            return Err(anyhow::anyhow!(
                "target address resolves to a blocked IP: {}",
                addr.ip()
            ));
        }
    }

    if !found_any {
        return Err(anyhow::anyhow!("host resolution returned no addresses"));
    }

    Ok(())
}

fn validate_url(url: &reqwest::Url) -> anyhow::Result<()> {
    match url.scheme() {
        "http" | "https" => {}
        _ => return Err(anyhow::anyhow!("url must be http(s)")),
    }

    if url.username() != "" || url.password().is_some() {
        return Err(anyhow::anyhow!("embedded credentials are not allowed"));
    }

    let host = url
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("url host is required"))?;

    if is_blocked_hostname(host) {
        return Err(anyhow::anyhow!("target host is not allowed"));
    }

    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_blocked_ip(ip) {
            return Err(anyhow::anyhow!("target address is not allowed"));
        }
    }

    Ok(())
}

fn is_blocked_hostname(host: &str) -> bool {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    host == "localhost"
        || host.ends_with(".localhost")
        || host.ends_with(".local")
        || host == "metadata.google.internal"
}

fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_broadcast()
                || ip.is_documentation()
                || ip.is_unspecified()
                || ip.octets()[0] == 0
                || ip.octets()[0] >= 224
                || ip == Ipv4Addr::new(100, 64, 0, 1)
                || (ip.octets()[0] == 100 && (64..=127).contains(&ip.octets()[1]))
        }
        IpAddr::V6(ip) => {
            ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_unique_local()
                || ip.is_unicast_link_local()
                || ip.is_documentation()
                || ip == Ipv6Addr::LOCALHOST
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{is_blocked_hostname, is_blocked_ip, validate_url};
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn rejects_local_targets() {
        assert!(is_blocked_hostname("localhost"));
        assert!(is_blocked_hostname("api.localhost"));
        assert!(is_blocked_hostname("printer.local"));
        assert!(is_blocked_ip(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));
        assert!(is_blocked_ip(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 8))));
        assert!(is_blocked_ip(IpAddr::V6(Ipv6Addr::LOCALHOST)));
    }

    #[test]
    fn rejects_private_redirect_targets() {
        let url = reqwest::Url::parse("http://169.254.169.254/latest/meta-data").unwrap();
        assert!(validate_url(&url).is_err());
    }

    #[test]
    fn allows_public_https_targets() {
        let url = reqwest::Url::parse("https://example.com").unwrap();
        assert!(validate_url(&url).is_ok());
    }

    #[test]
    fn rejects_cloud_metadata_hostname() {
        let url =
            reqwest::Url::parse("http://metadata.google.internal/computeMetadata/v1").unwrap();
        assert!(validate_url(&url).is_err());
    }
}
