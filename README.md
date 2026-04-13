# 🦅 FemtoClaw: The Industrial Agent Runtime

[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
[![Tier](https://img.shields.io/badge/Tier-Enterprise-green.svg)]()
[![Documentation](https://img.shields.io/badge/docs-normative-blue.svg)](https://femtoclaw.org/docs)

**FemtoClaw** is a high-performance, deterministic AI agent runtime engineered for industrial automation, enterprise systems, and high-stakes production environments. 

Unlike traditional agent frameworks that bundle inference and execution, FemtoClaw defines a new class of infrastructure: the **Execution Authority**. It treats AI models as probabilistic "Brains" while maintaining absolute, rule-based control over system interactions.

---

## 🏗 Architecture: Separation of Authority

FemtoClaw operates on the principle that **Inference produces Intent, while the Runtime preserves Authority.**

```text
┌───────────────────────┐      ┌──────────────────────────────────┐
│   Probabilistic       │      │      Deterministic Authority     │
│      BRAIN            │      │             RUNTIME              │
├───────────────────────┤      ├──────────────────────────────────┤
│ Generate Intent       │      │ 1. Validate Protocol (Schema)    │
│ (Tool Call / Message) │ ───> │ 2. Authorize (Policy Engine)     │
│                       │      │ 3. Execute (Capability Claws)    │
│                       │      │ 4. Record (Audit & WAL)          │
└───────────────────────┘      └──────────────────────────────────┘
           ^                                     │
           └─────────────────────────────────────┘
                 Autonomous Feedback Loop
```

### The 10-Phase Execution Lifecycle (FC-02)
1.  **Input**: Receive user prompt or system trigger.
2.  **Think**: Brain generates next action (Inference).
3.  **Validate**: Strict JSON schema enforcement (FC-03).
4.  **Authorize**: Policy Engine checks capability permissions (FC-05).
5.  **Mediate**: Sanitize and prepare arguments for execution.
6.  **Execute**: Deterministic execution within a "Claw" sandbox.
7.  **Record**: Mutation appended to the Write-Ahead Log (FC-20).
8.  **Update**: History and memory updated with execution results.
9.  **Evaluate**: Check for task completion or autonomous continuation (Spec 18).
10. **Respond**: Return final output to the user/caller.

---

## 🚀 Key Features

- **🛡️ Deny-by-Default Security**: No capability (shell, net, fs) can execute without explicit policy authorization.
- **🔄 Autonomous Execution (Spec 18)**: Built-in support for multi-step reasoning loops with safety-bounded iteration limits.
- **💾 Durable State (Spec 20)**: Every state mutation is logged to a high-performance Write-Ahead Log (WAL), enabling perfect crash recovery.
- **🌐 Distributed Clusters (Spec 41)**: Reference-tier synchronization for multi-node agent coordination.
- **⚡ Industrial Performance**: Zero-overhead Rust core with <5ms dispatch latency and minimal binary footprint.

---

## 🛠️ Quickstart

### Installation
```bash
# Automated install script (recommended)
curl -fsSL https://femtoclaw.org/install.sh | sh
```

### Interactive REPL
Launch the autonomous agent loop:
```bash
femtoclaw run
```

### Embedding in Rust
```rust
use femtoclaw::{Agent, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize with default industrial configuration
    let agent = Agent::new(Config::default())?;

    // 2. Execute an autonomous goal
    let result = agent.run("Audit /tmp for large files and list the top 3.").await?;

    println!("Audit Complete: {}", result);
    Ok(())
}
```

---

## ⚙️ Configuration & Governance

FemtoClaw is governed by standard environment variables and `Config` objects.

| Variable | Description | Default |
|----------|-------------|---------|
| `FEMTO_BRAIN` | Backend provider (echo, openai, ollama, etc.) | `echo` |
| `FEMTO_MAX_ITERATIONS` | Maximum steps in an autonomous loop | `10` |
| `FEMTO_WAL_PATH` | Path to the durable Write-Ahead Log | `~/.femtoclaw/log.wal` |
| `FEMTO_MEMORY_LIMIT` | Maximum messages preserved in context | `1000` |

---

## 📦 Workspace Ecosystem

FemtoClaw is a modular ecosystem of specialized crates:

- **[femtoclaw-protocol](./femtoclaw-protocol)**: The normative message schema.
- **[femtoclaw-policy](./femtoclaw-policy)**: The rule-based authorization engine.
- **[femtoclaw-storage](./femtoclaw-storage)**: WAL and persistence backends.
- **[femtoclaw-claws](./femtoclaw-claws)**: Standard capability set (Shell, Net, FS).
- **[femtoclaw-remote](./femtoclaw-remote)**: API server and distributed sync.

---

## 📄 License & Standards
FemtoClaw is open-source under the **Apache License 2.0**. It conforms to the [FemtoClaw Engineering Specification Suite](./femtoclaw-spec).

Copyright © 2026 FemtoClaw Project.
