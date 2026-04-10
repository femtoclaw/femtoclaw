//! FemtoClaw — Industrial Agent Runtime.
//!
//! The core runtime that orchestrates:
//! - Agent Core: execution controller
//! - Brain Interface: inference abstraction  
//! - Protocol Validator: validates inference output
//! - Capability Gate: authorization enforcement
//! - Capability Execution: tool/claw execution
//! - Memory: state management
//! - Observability: telemetry

pub mod agent;
pub mod app;
pub mod brain;
pub mod config;
pub mod memory;
pub mod protocol;
pub mod tools;
pub mod types;

pub use agent::Agent;
pub use config::Config;
pub use types::{Message, Role};
