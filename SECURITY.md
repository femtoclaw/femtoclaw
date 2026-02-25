# FemtoClaw Security Model (Industrial Runtime)

FemtoClaw is a production-minded agent runtime. The Industrial Runtime is intentionally strict:

- **Structured outputs only:** Brain responses must be valid JSON and match the protocol.
- **No markdown:** prevents accidental tool triggering ambiguity.
- **Deny-by-default tools:** tools must be enabled in `src/app.rs`.
- **Shell tool is allowlisted:** argv execution only; no shell interpolation.
- **Network tool is bounded:** timeout + max bytes; explicit http(s).

If you discover a vulnerability, open a private issue or contact the maintainers.
