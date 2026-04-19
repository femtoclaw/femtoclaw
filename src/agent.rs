//! FemtoClaw Agent Core.
//!
//! The primary execution controller that implements the runtime state machine
//! and coordinates all execution according to the architecture specification.

use std::sync::Arc;

use tokio::sync::RwLock;

use crate::brain::{Brain, BrainKind};
use crate::config::Config;
use crate::memory::Memory;
use crate::protocol::{ProtocolOutput, Validator};
use crate::tools::{Tool, ToolRegistry};
use crate::types::Message;

use async_trait::async_trait;
use femtoclaw_audit::{Event, Telemetry};
use femtoclaw_policy::{Capability, CapabilityGate, Policy, PolicyEngine, Rule};
use serde_json::Value;

// Import claws from femtoclaw-claws
use femtoclaw_claws::claws::fs::FsClaw;
use femtoclaw_claws::claws::net::NetClaw;
use femtoclaw_claws::claws::process::ProcessClaw;
use femtoclaw_claws::core::Claw;

/// Adapter that converts a Claw (from femtoclaw-claws) into a Tool.
struct ClawTool<C> {
    inner: C,
}

#[async_trait]
impl<C> Tool for ClawTool<C>
where
    C: Claw + Send + Sync + 'static,
{
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn description(&self) -> &'static str {
        self.inner.description()
    }

    async fn execute(&self, args: Value) -> anyhow::Result<String> {
        let result = self.inner.execute(args)?;
        Ok(serde_json::to_string(&result)?)
    }
}

pub struct Agent {
    brain: BrainKind,
    memory: Arc<RwLock<Box<dyn Memory>>>,
    tools: ToolRegistry,
    validator: Validator,
    gate: CapabilityGate,
    telemetry: Telemetry,
    max_iterations: usize,
}

impl Agent {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let brain = BrainKind::from_name(&config.brain.backend)?;
        let memory = Box::new(crate::memory::stm::Stm::new(config.max_memory));
        let max_iterations = config.max_iterations;

        let mut tools = ToolRegistry::new();
        tools.register(crate::tools::shell::ShellTool::new());
        tools.register(crate::tools::web_get::WebGetTool::new()?);
        // Register Claw-based tools: fs, net, process
        tools.register(ClawTool { inner: FsClaw });
        tools.register(ClawTool { inner: NetClaw });
        tools.register(ClawTool { inner: ProcessClaw });

        let mut gate = CapabilityGate::new();
        gate.register_capability(Capability::new("shell", "Execute shell commands"));
        gate.register_capability(Capability::new("web.get", "Fetch URLs"));
        gate.register_capability(Capability::new("fs", "Filesystem read operations"));
        gate.register_capability(Capability::new("net", "Network operations"));
        gate.register_capability(Capability::new(
            "process",
            "Process execution and inspection",
        ));

        let mut policy = Policy::new("default", "1.0");
        for capability in &config.allowed_capabilities {
            policy = policy.with_rule(Rule::allow(capability.clone()));
        }

        let mut engine = PolicyEngine::new();
        engine.add_policy(policy);

        let gate = gate.with_engine(engine);
        let telemetry = Telemetry::new();

        Ok(Self {
            brain,
            memory: Arc::new(RwLock::new(memory)),
            tools,
            validator: Validator::new(),
            gate,
            telemetry,
            max_iterations,
        })
    }

    pub fn with_brain(mut self, brain: BrainKind) -> Self {
        self.brain = brain;
        self
    }

    pub async fn run(&self, input: &str) -> anyhow::Result<String> {
        let user_message = Message::user(input);

        {
            let mut memory = self.memory.write().await;
            memory.push(user_message.clone());
        }

        for _ in 0..self.max_iterations {
            let messages = {
                let memory = self.memory.read().await;
                memory.history().to_vec()
            };

            let response = self.brain.think(&messages).await?;

            let protocol_output = self.validator.validate_str(&response)?;

            match protocol_output {
                ProtocolOutput::Message(msg) => {
                    let content = msg.message.content;
                    let assistant_message = Message::assistant(&content);
                    let mut memory = self.memory.write().await;
                    memory.push(assistant_message);
                    return Ok(content);
                }
                ProtocolOutput::ToolCall(tc) => {
                    let tool_name = tc.tool_call.tool.clone();
                    let args = tc.tool_call.args.clone();

                    let decision = self.gate.authorize(&tool_name, &args);
                    if !decision.is_allowed() {
                        return Err(anyhow::anyhow!("Capability denied: {}", decision));
                    }

                    // Record the tool call in memory
                    {
                        let mut memory = self.memory.write().await;
                        memory.push(Message::assistant(&response));
                    }

                    let result = self.tools.execute(&tool_name, args).await?;

                    let audit_event = Event::capability_execution_complete(&tool_name, &result);
                    self.telemetry.emit_and_log(audit_event).await;

                    // Record the tool result in memory
                    {
                        let mut memory = self.memory.write().await;
                        memory.push(Message::tool(&result));
                    }
                }
            }
        }

        Err(anyhow::anyhow!(
            "Maximum iterations reached ({}) without a final response",
            self.max_iterations
        ))
    }

    pub async fn reset(&self) {
        let mut memory = self.memory.write().await;
        memory.clear();
    }

    pub async fn history(&self) -> Vec<Message> {
        let memory = self.memory.read().await;
        memory.history().to_vec()
    }

    /// Synchronize agent memory with external state (Reference Tier).
    pub async fn sync_memory(&self, messages: &[Message]) {
        let mut memory = self.memory.write().await;
        memory.sync(messages);
    }

    /// Directly execute a tool by name with arguments, bypassing the brain.
    /// Authorization and audit are still enforced.
    pub async fn execute_tool(
        &self,
        tool: &str,
        args: serde_json::Value,
    ) -> anyhow::Result<String> {
        let decision = self.gate.authorize(tool, &args);
        if !decision.is_allowed() {
            return Err(anyhow::anyhow!("Capability denied: {}", decision));
        }

        let result = self.tools.execute(tool, args).await?;

        let audit_event = Event::capability_execution_complete(tool, &result);
        self.telemetry.emit_and_log(audit_event).await;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_execute_tool_unknown_denied() {
        let agent = Agent::new(Config::default()).expect("agent creation failed");
        let result = agent.execute_tool("unknown_tool", json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_tool_shell_denied_by_default() {
        let agent = Agent::new(Config::default()).expect("agent creation failed");
        let result = agent
            .execute_tool("shell", json!({"bin": "echo", "argv": ["hello"]}))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_tool_shell_allowed_when_configured() {
        let mut config = Config::default();
        config.allowed_capabilities = vec!["shell".to_string()];

        let agent = Agent::new(config).expect("agent creation failed");
        let result = agent
            .execute_tool("shell", json!({"bin": "echo", "argv": ["hello"]}))
            .await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("hello"));
    }
}
