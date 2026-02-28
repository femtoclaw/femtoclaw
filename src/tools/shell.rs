//! Allowlisted process execution (argv only).
//!
//! Args (preferred):
//! { "bin": "ls", "argv": ["-la"] }
//!
//! Compatibility shapes accepted:
//! { "command": "ls", "args": ["-la"] }
//! { "args": { "bin": "ls", "argv": ["-la"] } }
//! { "bin": "powershell -NoProfile -Command ..." }  // auto-split
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
                "ls",
                "cat",
                "pwd",
                "whoami",
                "git",
                "echo",
                "head",
                "tail",
                "wc",
                "grep",
                "powershell",
                "powershell.exe",
                "notepad",
                "notepad.exe",
            ],
            max_args: 16,
            max_arg_len: 4096,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ShellArgs {
    #[serde(default, alias = "command", alias = "cmd", alias = "program", alias = "executable")]
    bin: Option<String>,
    #[serde(default, alias = "args", alias = "arguments", alias = "params")]
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

    async fn execute(&self, mut args: Value) -> anyhow::Result<String> {
        // Some models wrap payload as {"args": {...}} or {"parameters": {...}}
        if args.get("bin").is_none() {
            if let Some(inner) = args.get("args").cloned().filter(|v| v.is_object()) {
                args = inner;
            } else if let Some(inner) = args
                .get("parameters")
                .cloned()
                .filter(|v| v.is_object())
            {
                args = inner;
            }
        }

        let parsed: ShellArgs = serde_json::from_value(args.clone())
            .map_err(|e| anyhow::anyhow!("shell args invalid: {e}"))?;

        let raw_bin = parsed.bin.ok_or_else(|| {
            let keys = args
                .as_object()
                .map(|o| o.keys().cloned().collect::<Vec<_>>().join(", "))
                .unwrap_or_else(|| "<non-object>".to_string());
            anyhow::anyhow!(
                "shell args invalid: missing bin/command field; received keys: {}",
                keys
            )
        })?;

        let (bin, argv) = normalize_command(raw_bin, parsed.argv)?;

        if !self.allowlist.iter().any(|&b| b == bin) {
            return Err(anyhow::anyhow!("shell bin not allowed"));
        }

        if argv.len() > self.max_args {
            return Err(anyhow::anyhow!("too many argv items"));
        }

        for a in &argv {
            if a.len() > self.max_arg_len {
                return Err(anyhow::anyhow!("arg too long"));
            }
        }

        let out = Command::new(&bin).args(&argv).output().await?;

        let mut s = String::new();
        s.push_str(&String::from_utf8_lossy(&out.stdout));
        s.push_str(&String::from_utf8_lossy(&out.stderr));

        // Trim to keep responses bounded
        const MAX_OUT: usize = 32_768;
        if s.len() > MAX_OUT {
            s.truncate(MAX_OUT);
            s.push_str("\n...(truncated)...");
        }

        Ok(s)
    }
}

fn normalize_command(raw_bin: String, mut argv: Vec<String>) -> anyhow::Result<(String, Vec<String>)> {
    if !raw_bin.chars().any(char::is_whitespace) {
        return Ok((raw_bin, argv));
    }

    // Common model mistake: returns one command string instead of bin+argv.
    let mut parts = split_command_line(&raw_bin)?;
    if parts.is_empty() {
        return Err(anyhow::anyhow!("shell args invalid: empty command"));
    }

    let bin = parts.remove(0);
    if !parts.is_empty() {
        parts.append(&mut argv);
        Ok((bin, parts))
    } else {
        Ok((bin, argv))
    }
}

fn split_command_line(input: &str) -> anyhow::Result<Vec<String>> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let mut escape = false;

    for ch in input.chars() {
        if escape {
            cur.push(ch);
            escape = false;
            continue;
        }

        if ch == '\\' {
            escape = true;
            continue;
        }

        match quote {
            Some(q) => {
                if ch == q {
                    quote = None;
                } else {
                    cur.push(ch);
                }
            }
            None => {
                if ch == '"' || ch == '\'' {
                    quote = Some(ch);
                } else if ch.is_whitespace() {
                    if !cur.is_empty() {
                        out.push(std::mem::take(&mut cur));
                    }
                } else {
                    cur.push(ch);
                }
            }
        }
    }

    if escape {
        cur.push('\\');
    }

    if quote.is_some() {
        return Err(anyhow::anyhow!("shell args invalid: unterminated quote in command"));
    }

    if !cur.is_empty() {
        out.push(cur);
    }

    Ok(out)
}

