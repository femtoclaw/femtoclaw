# FemtoClaw — Industrial Agent Runtime (Rust)

**FemtoClaw** is a lightweight, deterministic agent runtime designed for industrial/enterprise/production use.
The **Industrial Runtime** prioritizes:
- Small binaries (release profile optimized)
- Fast startup
- Strict JSON tool calling (no markdown)
- Capability-gated tools with allowlists

## Quickstart

### 1. One-liner (macOS/Linux)

```bash
curl -fsSL https://femtoclaw.org/install.sh | sh
```

### 2. From Source

```bash
cargo build --release
./target/release/femtoclaw run
```

Type an instruction and press Enter. Use `Ctrl+C` to exit.

## Brains

FemtoClaw ships with two brains:
- `echo` (default): returns a safe, deterministic JSON response (great for tests / offline)
- `openai`: OpenAI-compatible Chat Completions endpoint (optional)

Configure via environment:
```bash
export FEMTO_BRAIN=echo
# or
export FEMTO_BRAIN=openai
export FEMTO_OPENAI_BASE_URL="https://api.openai.com/v1"
export FEMTO_OPENAI_API_KEY="..."
export FEMTO_OPENAI_MODEL="gpt-4.1-mini"
```

## Tools (Claws)

- `shell` — allowlisted process execution (argv only, no shell interpolation)
- `web_get` — bounded HTTP GET with size + timeout limits

Tools are deny-by-default: you must enable them in code (see `src/app.rs`).

## Security posture

- Structured tool calls only (JSON schema-like validation)
- Unknown tools rejected
- Shell tool uses allowlist and argv execution (no `sh -c`)
- Network tool is bounded (timeout + max bytes)

## Build profiles

Industrial runtime (default):
```bash
cargo build --release
```

Full (adds tracing logs):
```bash
cargo build --release --features full
```

## License

Apache 2.0 — see [LICENSE](LICENSE).
