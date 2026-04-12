# FemtoClaw Testing Guide

This document describes the test suite, its structure, and how to run it.

## Test Categories

FemtoClaw's tests are organized into several categories:

### 1. Unit Tests

Located within each crate's `src/` in `#[cfg(test)] mod tests`. These test individual components in isolation.

**Examples:**
- `femtoclaw/src/agent.rs`: `execute_tool` permission checks.
- `femtoclaw/src/memory/stm.rs`: Short‑Term Memory eviction.
- `femtoclaw-remote/src/server.rs`: HTTP handler logic.
- `femtoclaw-claws/src/core/mod.rs`: ClawRegistry behavior.

### 2. Integration Tests

Located in `tests/` directories of each crate. These test interactions between components.

**Key files:**
- `femtoclaw/tests/agent_integration.rs`: Agent end‑to‑end behavior (echo brain, memory persistence, reset, direct tool execution for shell, fs, net, process).
- `femtoclaw/tests/memory_tools.rs`: STM under pressure, tool allowlist enforcement, web.get network tolerance.
- `femtoclaw-compliance/src/tests/`: Protocol validation, runtime basics, capabilities, security, performance.

### 3. Compliance Tests

The `femtoclaw-compliance` crate provides a normative compliance test harness. Run all domains:

```bash
cargo test -p femtoclaw-compliance
```

Or a specific domain:

```bash
cargo test -p femtoclaw-compliance protocol
cargo test -p femtoclaw-compliance runtime
cargo test -p femtoclaw-compliance capability
cargo test -p femtoclaw-compliance security
cargo test -p femtoclaw-compliance performance
```

### 4. Benchmarks

The `femtoclaw-bench` crate provides portable and startup benchmarks.

```bash
cargo run -p femtoclaw-bench -- run --preset all --iters 1000
cargo run -p femtoclaw-bench -- startup --bin target/release/femtoclaw --iterations 50
```

## Running Tests

### Workspace-wide

```bash
# Build and run all unit + integration tests
cargo test --workspace

# With output
cargo test --workspace -- --nocapture
```

### Per Crate

```bash
cargo test -p femtoclaw
cargo test -p femtoclaw-remote
cargo test -p femtoclaw-claws
cargo test -p femtoclaw-compliance
```

### Specific Test

```bash
# Run a single integration test file
cargo test --test agent_integration

# Run a single test by name
cargo test test_agent_echo_brain
```

## Test Coverage

- **Agent**: creation, message history, reset, direct tool execution, memory eviction.
- **Tools**: shell allowlist, argument validation, output truncation; web.get timeout and size limits; claws (fs, net, process) integration.
- **Protocol**: strict JSON schema validation (message vs tool_call, unknown fields).
- **Policy**: capability registration, enable/disable, allow/deny rules.
- **Remote API**: health endpoint, chat handler, tool execution handler, WebSocket upgrade.
- **Compliance**: Conformance to FC‑03 (Protocol), FC‑05 (Policy), FC‑08 (Audit), FC‑STORAGE‑0001 (Storage), and performance thresholds.

## Notes

- Some tests (e.g., `web.get`) are tolerant of offline environments; they will pass if a network error occurs.
- The test suite is designed to be fast and deterministic; most run in under a second.
- Warnings about unused imports/fields in some brain modules are benign and can be addressed in a future cleanup.

## Continuous Integration

A CI pipeline should run:

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

---

For questions or issues, open an issue on GitHub.
