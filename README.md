# 🦅 FemtoClaw — Industrial Agent Runtime

[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
[![Tier](https://img.shields.io/badge/Tier-Enterprise-green.svg)]()

**FemtoClaw** is a lightweight, deterministic agent runtime designed for industrial, enterprise, and high-stakes production use. It enforces a strict separation between inference (probabilistic intent) and execution (deterministic authority).

## 🚀 Key Features

- **Autonomous Execution (Spec 18):** Iterative "Think-Execute-Repeat" loop with safety-bounded iteration limits.
- **Persistent Storage & WAL (Spec 20):** Durable, crash-resilient memory backed by a high-performance Write-Ahead Log.
- **Distributed Coordination (Spec 41):** Reference-tier support for cluster-wide state synchronization and peer discovery.
- **Deny-by-Default Security:** Capability-gated tool execution with mandatory authorization and strict allowlists.
- **Minimal TCB:** Small binary footprint (<5MB) and minimal dependencies for a reduced attack surface.

## 🛠 Quickstart

### 1. One-liner Installation (macOS/Linux)
```bash
curl -fsSL https://femtoclaw.org/install.sh | sh
```

### 2. Run Interactively
```bash
femtoclaw run
```

### 3. Integration (Rust)
```rust
use femtoclaw::{Agent, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = Agent::new(Config::default())?;
    let response = agent.run("Perform a multi-step audit of the filesystem.").await?;
    println!("Final Response: {}", response);
    Ok(())
}
```

## 🧠 Brain Backends
FemtoClaw is model-agnostic. Configure your brain via environment variables:
- `echo` (default): Safe, deterministic JSON responses for testing.
- `openai`: OpenAI-compatible API (GPT-4o, etc.).
- `anthropic`, `gemini`, `ollama`, `grok`, and more.

## 🏗 Industrial Security Posture
- **No Markdown Execution:** Strictly enforces JSON-serialized tool calls to prevent parsing ambiguities.
- **Argv-only Shell:** Process execution is limited to explicitly allowlisted binaries with no shell interpolation.
- **Audit Trails:** Every decision, authorization, and execution is recorded in a tamper-evident audit log.

## 📦 Project Structure
- `femtoclaw-protocol`: Strict JSON schema validation for agent messages.
- `femtoclaw-storage`: High-performance persistence and WAL.
- `femtoclaw-remote`: Distributed cluster support and API server.
- `femtoclaw-policy`: Deterministic authorization engine.

## 📄 License
Apache 2.0 — see [LICENSE](LICENSE).
