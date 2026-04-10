//! Tool/Claw Module.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub mod shell;
pub mod web_get;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    async fn execute(&self, args: Value) -> anyhow::Result<String>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        self.tools.insert(tool.name().to_string(), Arc::new(tool));
    }

    pub async fn execute(&self, name: &str, args: Value) -> anyhow::Result<String> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("tool not found: {}", name))?;
        tool.execute(args).await
    }

    pub fn list(&self) -> Vec<(String, String)> {
        self.tools
            .iter()
            .map(|(name, tool)| (name.clone(), tool.description().to_string()))
            .collect()
    }

    pub fn has(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
