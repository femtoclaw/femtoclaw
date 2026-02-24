use anyhow::Result;

pub trait Claw: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, args: &str) -> Result<String>;
}

pub struct FetchClaw;

impl Claw for FetchClaw {
    fn name(&self) -> &str {
        "fetch_url"
    }

    fn description(&self) -> &str {
        "Fetches content from a given URL"
    }

    fn execute(&self, args: &str) -> Result<String> {
        // In production: use reqwest to actually fetch
        Ok(format!("Fetched content from {}", args))
    }
}

pub struct ShellClaw;

impl Claw for ShellClaw {
    fn name(&self) -> &str {
        "run_command"
    }

    fn description(&self) -> &str {
        "Executes a safe shell command"
    }

    fn execute(&self, args: &str) -> Result<String> {
        // In production: use std::process::Command with strict whitelisting
        Ok(format!("Command executed: {}", args))
    }
}