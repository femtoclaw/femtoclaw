# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.1] - 2026-02-25

### Added
- Full modular architecture with 12 specialized crates
- Protocol integration with femtoclaw-protocol for strict JSON validation
- Policy integration with femtoclaw-policy for capability gating
- Audit integration with femtoclaw-audit for observability
- Storage integration with femtoclaw-storage for persistence
- Tool registry with Shell and WebGet capabilities
- REPL-based CLI interface
- Agent configuration management
- Support for remote execution via femtoclaw-remote
- WASM sandbox support via femtoclaw-wasm

### Changed
- Version bump from 1.0.0 to 1.0.1
- Modularized codebase into separate crates for maintainability

### Fixed
- Build errors in all dependent crates resolved

## [1.0.0] - 2026-02-25

### Added
- Initial release
- Core agent runtime
- Basic tool execution
- Configuration management

