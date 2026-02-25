//! Allowlisted process execution (argv only).
//!
//! Args:
//! { "bin": "ls", "argv": ["-la"] }
//!
//! Security:
//! - bin must be in allowlist
//! - argv length and each arg length are bounded
//! - executed via Command without shell interpolation

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use tokio::process::Command;

use crate::tools::Tool;

#[derive(Clone)]
pub struct ShellTool {
    allowlist: Vec<&'static str>,
    max_args: usize,
    max_arg_len: usize,
}

impl ShellTool {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for ShellTool {
    fn default() -> Self {
        Self {
            allowlist: vec![
                "ls", "cat", "pwd", "whoami", "git", "echo", "head", "tail", "wc", "grep",
            ],
            max_args: 16,
            max_arg_len: 256,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ShellArgs {
    bin: String,
    #[serde(default)]
    argv: Vec<String>,
}

#[async_trait]
impl Tool for ShellTool {
    fn name(&self) -> &'static str {
        "shell"
    }
    fn description(&self) -> &'static str {
        "Allowlisted argv process execution (no shell)."
    }

    async fn execute(&self, args: Value) -> anyhow::Result<String> {
        let parsed: ShellArgs =
            serde_json::from_value(args).map_err(|e| anyhow::anyhow!("shell args invalid: {e}"))?;

        if !self.allowlist.iter().any(|&b| b == parsed.bin) {
            return Err(anyhow::anyhow!("shell bin not allowed"));
        }
        if parsed.argv.len() > self.max_args {
            return Err(anyhow::anyhow!("too many argv items"));
        }
        for a in &parsed.argv {
            if a.len() > self.max_arg_len {
                return Err(anyhow::anyhow!("arg too long"));
            }
        }

        let out = Command::new(&parsed.bin)
            .args(&parsed.argv)
            .output()
            .await?;

        let mut s = String::new();
        s.push_str(&String::from_utf8_lossy(&out.stdout));
        s.push_str(&String::from_utf8_lossy(&out.stderr));

        // Trim to keep responses bounded
        const MAX_OUT: usize = 32_768;
        if s.len() > MAX_OUT {
            s.truncate(MAX_OUT);
            s.push_str("\n…(truncated)…");
        }
        Ok(s)
    }
}
