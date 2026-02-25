use async_trait::async_trait;
use serde_json::Value;

pub mod shell;
pub mod web_get;

#[async_trait]
pub trait Claw: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    async fn execute(&self, args: Value) -> anyhow::Result<String>;
}
