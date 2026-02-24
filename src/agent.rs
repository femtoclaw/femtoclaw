use crate::brain::{Brain, Message};
use crate::memory::Memory;
use crate::tools::Claw;
use anyhow::Result;
use std::sync::Arc;

pub struct FemtoClaw {
    brain: Arc<dyn Brain>,
    memory: Arc<dyn Memory>,
    tools: Vec<Arc<dyn Claw>>,
}

impl FemtoClaw {
    pub fn new(brain: Arc<dyn Brain>, memory: Arc<dyn Memory>, tools: Vec<Arc<dyn Claw>>) -> Self {
        Self { brain, memory, tools }
    }

    pub async fn run(&self, prompt: &str) -> Result<String> {
        // 1. Store User Input
        self.memory.add("user", prompt);

        // 2. Construct System Prompt with Tool Definitions
        let mut messages = self.memory.get_history();
        
        // Inject tool capabilities into context (Simple ReAct style)
        let tool_desc = self.tools.iter()
            .map(|t| format!("{}: {}", t.name(), t.description()))
            .collect::<Vec<_>>()
            .join("\n");
        
        let system_prompt = format!("You are FemtoClaw. You have access to these tools:\n{}", tool_desc);
        
        // Prepend system prompt temporarily for this turn
        let mut full_context = vec![Message { role: "system".into(), content: system_prompt }];
        full_context.append(&mut messages);

        // 3. Think
        let response = self.brain.think(full_context).await?;

        // 4. Store Assistant Output
        self.memory.add("assistant", &response);

        // 5. Check for Tool Usage (Simple keyword matching for skeleton)
        // In production: Use structured JSON output from LLM to trigger tools
        for tool in &self.tools {
            if response.contains(tool.name()) {
                let result = tool.execute(prompt)?;
                self.memory.add("system", &format!("Tool {} result: {}", tool.name(), result));
                return Ok(format!("Thought: {}\nAction: {} -> {}", response, tool.name(), result));
            }
        }

        Ok(response)
    }
}