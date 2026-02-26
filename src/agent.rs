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
use crate::tools::ToolRegistry;
use crate::types::Message;

use femtoclaw_audit::{Event, Telemetry};
use femtoclaw_policy::{Capability, CapabilityGate, Policy, PolicyEngine, Rule};

pub struct Agent {
    brain: BrainKind,
    memory: Arc<RwLock<Box<dyn Memory>>>,
    tools: ToolRegistry,
    validator: Validator,
    config: Config,
    gate: CapabilityGate,
    telemetry: Telemetry,
}

impl Agent {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let brain = BrainKind::from_env()?;
        let memory = Box::new(crate::memory::stm::Stm::new(1000));

        let mut tools = ToolRegistry::new();
        tools.register(crate::tools::shell::ShellTool::new());
        tools.register(crate::tools::web_get::WebGetTool::new()?);

        let mut gate = CapabilityGate::new();
        gate.register_capability(Capability::new("shell", "Execute shell commands"));
        gate.register_capability(Capability::new("web.get", "Fetch URLs"));

        let mut engine = PolicyEngine::new();
        engine.add_policy(
            Policy::new("default", "1.0")
                .with_rule(Rule::allow("shell"))
                .with_rule(Rule::allow("web.get")),
        );

        let gate = gate.with_engine(engine);
        let telemetry = Telemetry::new();

        Ok(Self {
            brain,
            memory: Arc::new(RwLock::new(memory)),
            tools,
            validator: Validator::new(),
            config,
            gate,
            telemetry,
        })
    }

    pub async fn run(&self, input: &str) -> anyhow::Result<String> {
        let user_message = Message::user(input);

        {
            let mut memory = self.memory.write().await;
            memory.push(user_message.clone());
        }

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
                Ok(content)
            }
            ProtocolOutput::ToolCall(tc) => {
                let tool_name = tc.tool_call.tool.clone();
                let args = tc.tool_call.args.clone();

                let decision = self.gate.authorize(&tool_name, &args);
                if !decision.is_allowed() {
                    return Err(anyhow::anyhow!("Capability denied: {}", decision));
                }

                let result = self.tools.execute(&tool_name, args).await?;

                let audit_event = Event::capability_execution_complete(&tool_name, &result);
                self.telemetry.emit_and_log(audit_event).await;

                let assistant_message = Message::assistant(&result);
                let mut memory = self.memory.write().await;
                memory.push(assistant_message);

                Ok(result)
            }
        }
    }

    pub async fn reset(&self) {
        let mut memory = self.memory.write().await;
        memory.clear();
    }

    pub async fn history(&self) -> Vec<Message> {
        let memory = self.memory.read().await;
        memory.history().to_vec()
    }
}
