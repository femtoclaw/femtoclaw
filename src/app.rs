//! FemtoClaw Application.

use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use crate::agent::Agent;
use crate::config::Config;

pub async fn run_repl() -> anyhow::Result<()> {
    let agent = Agent::new(Config::default())?;
    let stdin = tokio::io::stdin();
    let mut lines = tokio::io::BufReader::new(stdin).lines();
    let mut stdout = tokio::io::stdout();

    stdout
        .write_all(b"FemtoClaw \xE2\x80\x94 Industrial Agent Runtime\n")
        .await?;
    stdout
        .write_all(b"Type a prompt and press Enter. /quit to exit.\n\n")
        .await?;
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

        match agent.run(line).await {
            Ok(reply) => {
                stdout.write_all(reply.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
            }
            Err(e) => {
                stdout
                    .write_all(format!("ERROR: {}\n", e).as_bytes())
                    .await?;
            }
        }
        stdout.write_all(b"> ").await?;
        stdout.flush().await?;
    }
    Ok(())
}

pub async fn run_once(prompt: &str) -> anyhow::Result<()> {
    let agent = Agent::new(Config::default())?;
    let reply = agent.run(prompt).await?;
    println!("{}", reply);
    Ok(())
}
