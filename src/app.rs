use std::collections::HashMap;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};

use crate::brain::{Brain, BrainKind};
use crate::memory::stm::ShortTermMemory;
use crate::memory::Memory;
use crate::schema::AgentOutput;
use crate::tools::{self, Claw};
use crate::types::Message;

pub struct Agent {
    brain: BrainKind,
    mem: ShortTermMemory,
    tools: HashMap<&'static str, Box<dyn Claw>>,
}

impl Agent {
    pub fn new() -> anyhow::Result<Self> {
        let brain = BrainKind::from_env()?;
        let mut tools_map: HashMap<&'static str, Box<dyn Claw>> = HashMap::new();

        // Capability-gated tools: enable explicitly here.
        tools_map.insert("shell", Box::new(tools::shell::ShellClaw::new_default()));
        tools_map.insert(
            "web_get",
            Box::new(tools::web_get::WebGetClaw::new(64 * 1024, 20)?),
        );

        let mut mem = ShortTermMemory::new(24);
        mem.push(Message::system(
            "FemtoClaw — Industrial Agent Runtime. Strict JSON tool calling.",
        ));

        Ok(Self {
            brain,
            mem,
            tools: tools_map,
        })
    }

    pub async fn step(&mut self, user_text: &str) -> anyhow::Result<String> {
        self.mem.push(Message::user(user_text));

        let raw = self.brain.think(self.mem.history()).await?;
        let out = AgentOutput::parse_strict(&raw)?;

        match out {
            AgentOutput::Message { message } => {
                self.mem.push(Message::assistant(message.content.clone()));
                Ok(message.content)
            }
            AgentOutput::ToolCall { tool_call } => {
                let tool_name = tool_call.tool;
                let tool = self
                    .tools
                    .get(tool_name.as_str())
                    .ok_or_else(|| anyhow::anyhow!("tool not enabled: {tool_name}"))?;
                let result = tool.execute(tool_call.args).await?;

                // Store tool result as tool message and ask brain for final message
                self.mem
                    .push(Message::tool(format!("tool:{tool_name}\n{result}")));
                let raw2 = self.brain.think(self.mem.history()).await?;
                let out2 = AgentOutput::parse_strict(&raw2)?;
                match out2 {
                    AgentOutput::Message { message } => {
                        self.mem.push(Message::assistant(message.content.clone()));
                        Ok(message.content)
                    }
                    AgentOutput::ToolCall { .. } => Err(anyhow::anyhow!(
                        "tool chaining is disabled in Industrial Runtime (expected message)"
                    )),
                }
            }
        }
    }
}

pub async fn run_repl() -> anyhow::Result<()> {
    let mut agent = Agent::new()?;
    let stdin = io::BufReader::new(io::stdin());
    let mut lines = stdin.lines();
    let mut stdout = io::stdout();

    stdout.write_all(b"FemtoClaw \xE2\x80\x94 Industrial Agent Runtime \xE2\x80\x94 type a prompt and press Enter.\n").await?;
    stdout.write_all(b"> ").await?;
    stdout.flush().await?;

    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            stdout.write_all(b"> ").await?;
            stdout.flush().await?;
            continue;
        }
        if line == "/quit" || line == "/exit" {
            break;
        }
        match agent.step(line).await {
            Ok(reply) => {
                stdout.write_all(reply.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
            }
            Err(e) => {
                stdout.write_all(format!("ERROR: {e}\n").as_bytes()).await?;
            }
        }
        stdout.write_all(b"> ").await?;
        stdout.flush().await?;
    }
    Ok(())
}

pub async fn run_once(prompt: &str) -> anyhow::Result<()> {
    let mut agent = Agent::new()?;
    let reply = agent.step(prompt).await?;
    println!("{reply}");
    Ok(())
}
