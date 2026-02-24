mod agent;
mod brain;
mod memory;
mod tools;
mod config;

use anyhow::Result;
use std::sync::Arc;
use femtoclaw::agent::FemtoClaw;
use femtoclaw::brain::{Brain, LocalBrain, RemoteBrain};
use femtoclaw::memory::ConversationMemory;
use femtoclaw::tools::{Claw, FetchClaw, ShellClaw};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    // 1. Initialize Brain (Switch based on ENV)
    let brain: Arc<dyn Brain> = if std::env::var("USE_LOCAL").unwrap_or_default() == "true" {
        Arc::new(LocalBrain::new(
            std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string()),
            std::env::var("MODEL").unwrap_or_else(|_| "llama3".to_string()),
        ))
    } else {
        Arc::new(RemoteBrain::new(
            std::env::var("OPENAI_API_KEY").expect("API_KEY required"),
            std::env::var("OPENAI_BASE_URL").ok(),
            std::env::var("MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string()),
        ))
    };

    // 2. Initialize Memory (Keep last 20 messages)
    let memory = Arc::new(ConversationMemory::new(20));

    // 3. Initialize Claws
    let tools: Vec<Arc<dyn Claw>> = vec![
        Arc::new(FetchClaw),
        Arc::new(ShellClaw),
    ];

    // 4. Spawn Agent
    let agent = FemtoClaw::new(brain, memory, tools);

    println!("🦅 FemtoClaw Ready.");
    let response = agent.run("Fetch https://example.com").await?;
    println!("📝 {}", response);

    Ok(())
}