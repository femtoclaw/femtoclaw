# 🦅 FemtoClaw

> **The Lightweight AI Agent. Rust Core. Modular Brain. Token Incentives.**

[![Crates.io](https://img.shields.io/crates/v/femtoclaw.svg)](https://crates.io/crates/femtoclaw)
[![Build Status](https://github.com/femtoclaw/femtoclaw/workflows/CI/badge.svg)](https://github.com/femtoclaw/femtoclaw/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Discord](https://img.shields.io/discord/123456789?label=discord&logo=discord)](https://discord.gg/femtoclaw)

**FemtoClaw** is the third-generation evolution of the Claw automation lineage (OpenClaw → PicoClaw → **FemtoClaw**). It is a privacy-first, Rust-based AI agent designed for edge devices, CI/CD pipelines, and decentralized autonomous operations.

Unlike heavy Python-based agents, FemtoClaw is compiled, memory-safe, and under 5MB. It features a modular architecture where the **Brain** (LLM), **Memory** (Context), and **Claws** (Tools) are decoupled, allowing you to swap components without rewriting core logic.

---

## 🚀 Features

-   **🦀 Rust Core:** Blazing fast, memory-safe, and portable across Linux, macOS, Windows, and WASM.
-   **🧠 Modular Brain:** Switch between local inference (Ollama, Llama.cpp) and remote APIs (OpenAI, Anthropic) via configuration.
-   **🪶 Persistent Memory:** Sliding window context for short-term memory + vector store integration for long-term recall.
-   **🔗 Crypto-Native:** Optional integration with the **$FEMTO** token for paying for premium Claws, staking for reputation, and accessing decentralized compute.
-   **🛡️ Safety Sandbox:** Strict capability-based security model for tool execution (Shell, Network, Filesystem).

---

## 📦 Installation

### From Crates.io (Recommended)

```bash
cargo install femtoclaw
```

### From Source

```bash
git clone https://github.com/femtoclaw/femtoclaw.git
cd femtoclaw
cargo build --release
```

### Docker

```bash
docker pull femtoclaw/agent:latest
docker run -it --rm femtoclaw/agent:latest
```

---

## ⚡ Quick Start

### 1. Initialize Configuration

```bash
femtoclaw init
```
This creates a `.femtoclaw/` directory with `config.toml` and `.env`.

### 2. Configure Your Brain

Edit `.femtoclaw/.env`:

```ini
# Local Brain (Privacy First)
USE_LOCAL=true
OLLAMA_URL=http://localhost:11434
MODEL=llama3

# OR Remote Brain (Capability First)
# USE_LOCAL=false
# OPENAI_API_KEY=sk-...
# MODEL=gpt-4-turbo
```

### 3. Run the Agent

```bash
# Interactive Mode
femtoclaw run

# One-off Command
femtoclaw exec "Summarize the latest Rust blog posts"

# With Specific Tool
femtoclaw exec --tool web_fetch "https://rust-lang.org"
```

---

## 🏗 Architecture

FemtoClaw is built on three core traits:

### 1. The Brain (`src/brain.rs`)
Handles inference. Implements `Brain` trait.
```rust
pub trait Brain: Send + Sync {
    async fn think(&self, messages: Vec<Message>) -> Result<String>;
}
```
*Implementations:* `LocalBrain` (Ollama), `RemoteBrain` (OpenAI).

### 2. The Memory (`src/memory.rs`)
Handles context storage. Implements `Memory` trait.
```rust
pub trait Memory: Send + Sync {
    fn add(&mut self, role: &str, content: &str);
    fn get_history(&self) -> Vec<Message>;
}
```
*Implementations:* `ConversationMemory` (RAM), `VectorMemory` (Qdrant/SQLite).

### 3. The Claws (`src/tools.rs`)
Handles actions. Implements `Claw` trait.
```rust
pub trait Claw: Send + Sync {
    fn name(&self) -> &str;
    fn execute(&self, args: &str) -> Result<String>;
}
```
*Implementations:* `WebClaw`, `ShellClaw`, `FileClaw`, `TokenClaw` (Crypto).

---

## 🔐 Safety & Security

FemtoClaw can execute system commands. **Use with caution.**

1.  **Whitelisting:** By default, `ShellClaw` is disabled. Enable it explicitly in `config.toml` and define allowed binaries.
    ```toml
    [security]
    allow_shell = true
    allowed_commands = ["git", "ls", "cat", "cargo"]
    ```
2.  **Sandboxing:** For untrusted prompts, run FemtoClaw inside a Docker container or Firecracker microVM.
3.  **Key Management:** Never commit `.env` files. Use system keychains or secret managers for API keys.

---

## 🪶 Token Integration ($FEMTO)

FemtoClaw supports optional cryptoeconomic features via the `femtoclaw-sdk`.

-   **Pay for Claws:** Use $FEMTO to access premium community tools.
-   **Stake for Memory:** Lock tokens to reserve decentralized vector storage.
-   **Earn Rewards:** Provide compute power to the network and earn $FEMTO.

*See the [Protocol Repository](https://github.com/femtoclaw/protocol) for smart contract details.*

---

## 📅 Roadmap

| Phase | Status | Goals |
|-------|--------|-------|
| **Alpha** | 🟢 Complete | Core Agent, Local/Remote Brain, Basic Claws |
| **Beta** | 🟡 In Progress | Vector Memory, Token Integration, CLI Polish |
| **RC1** | ⚪ Planned | ZK Privacy Proofs, WASM Plugin System |
| **Mainnet** | ⚪ Planned | Decentralized Agent Swarm, Governance DAO |

---

## 🤝 Contributing

We welcome contributions from the Rust, AI, and Crypto communities.

1.  **Fork** the repository.
2.  **Create** a feature branch (`git checkout -b feat/amazing-claw`).
3.  **Commit** your changes (`git commit -m 'Add amazing claw'`).
4.  **Push** to the branch (`git push origin feat/amazing-claw`).
5.  **Open** a Pull Request.

Please read our [Contributing Guidelines](CONTRIBUTING.md) and [Code of Conduct](CODE_OF_CONDUCT.md) first.

**Development Requirements:**
-   Rust 1.70+
-   Docker (for integration tests)
-   Ollama (for local brain testing)

---

## 📄 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

*Smart contract integrations are licensed under **GPL-3.0**.*

---

## ⚠️ Disclaimer

**FemtoClaw is experimental software.**
-   **AI Risk:** LLMs may hallucinate or generate incorrect commands. Review actions before execution in production.
-   **Crypto Risk:** $FEMTO token integration involves financial risk. This is not financial advice.
-   **Security:** You are responsible for securing your API keys and system access.

---

<p align="center">
  <strong>🦀 Built with Rust | 🧠 Powered by AI | 🪶 Lightweight by Design</strong><br>
  <sub>Part of the OpenClaw → PicoClaw → FemtoClaw lineage</sub>
</p>

---

## 📬 Contact

-   **Website:** [femtoclaw.github.io](https://femtoclaw.github.io)
-   **Discord:** [discord.gg/femtoclaw](https://discord.gg/femtoclaw)
-   **Twitter:** [@femtoclaw](https://twitter.com/femtoclaw)
-   **Email:** core@femtoclaw.xyz