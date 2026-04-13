# FemtoClaw Developer Documentation

## 🦅 Overview
FemtoClaw is an industrial agent runtime engineered for deterministic execution authority. It separates inference (intent) from execution (authority), ensuring that AI-driven automation remains safe, observable, and auditable in production environments.

---

## 🏗 Architecture

### Core Design Principles
1. **Deny-by-Default:** No system interaction is permitted without explicit capability authorization.
2. **Deterministic Control Loop:** The execution lifecycle follows a strict state machine: `Think` → `Validate` → `Authorize` → `Execute` → `Record` → `Respond`.
3. **Capability Isolation:** Interaction with the OS and network is restricted to modular "Claws" (Capabilities).
4. **Minimal TCB:** The Trusted Computing Base is kept minimal to reduce attack surface and ensure auditability.

### Tiered Specification Model
- **Enterprise Tier (Specs 1–40):** Defines the production-grade runtime, safety gates, and persistence models.
- **Reference Tier (Specs 41–60):** Defines standards for distributed coordination, clustering, and high-availability.

---

## 🚀 Key Implementations

### Autonomous Agent Loop (Spec 18)
The runtime supports autonomous multi-step execution. The `Agent::run` method implements an iterative loop that continues until the inference system produces a final message or the `max_iterations` limit (default: 10) is reached. Intermediate tool results are persisted in memory to provide contextual continuity.

### Persistent Storage & WAL (Spec 20)
To ensure durability and crash resilience, FemtoClaw utilizes a **Write-Ahead Log (WAL)**. Every state mutation is recorded to durable storage before being applied to memory. The `WalMemory` implementation (Reference Tier) allows the agent to reconstruct its entire execution state by replaying the log.

### Distributed Coordination (Spec 41)
FemtoClaw supports cluster-wide state synchronization. The `ClusterManager` enables peer discovery and state reconciliation, ensuring that multiple runtime nodes can maintain a consistent view of the execution history.

---

## 📦 Publication & Deployment

### Mandatory Publication Order
Due to internal dependency requirements, crates MUST be published to the registry in the following specific order:

1. **Foundation Libraries**
   - `femtoclaw-protocol`: Protocol validation and schema enforcement.
   - `femtoclaw-policy`: Authorization engine and rule evaluation.
   - `femtoclaw-audit`: Telemetry and audit logging.
   - `femtoclaw-storage`: Persistence and Write-Ahead Logging.
2. **Capability Modules**
   - `femtoclaw-claws`: Standard capability set (Shell, FS, Net).
3. **Core Runtime**
   - `femtoclaw`: The primary execution authority.
4. **Integrations & Tooling**
   - `femtoclaw-cli`: Interactive REPL and command-line interface.
   - `femtoclaw-remote`: HTTP/WebSocket API server and cluster support.
   - `femtoclaw-sdk`: Developer integration kits.

### Versioning Policy
The project strictly adheres to **Semantic Versioning (SemVer)**. Ensure that all crates in the workspace are aligned (e.g., `v1.0.3`) before attempting a publication run.

---

## 🧪 Testing & Verification
- **Unit Tests:** Located within each crate's `src/` directory.
- **Integration Tests:** Located in the `tests/` directory of the core `femtoclaw` crate.
- **Compliance Suite:** Use the `femtoclaw-compliance` crate to verify runtime conformance against the engineering specifications.

---

## 🛡 Security Hardening
- **User Mode:** Run as a non-privileged OS user.
- **Capability Gate:** Keep all capabilities disabled by default in policy.
- **Audit Logs:** Direct logs to a tamper-proof SIEM or external logging authority.
