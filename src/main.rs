use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "femtoclaw",
    version,
    about = "FemtoClaw — Industrial Agent Runtime"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run an interactive REPL loop (stdin -> agent -> stdout).
    Run,
    /// Run a single prompt (useful for CI).
    Once {
        /// Prompt text
        prompt: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "full")]
    {
        tracing_subscriber::fmt().with_ansi(true).init();
    }

    let cli = Cli::parse();
    match cli.cmd {
        Command::Run => femtoclaw::app::run_repl().await,
        Command::Once { prompt } => femtoclaw::app::run_once(&prompt).await,
    }
}
