//! Protocol Validation Module.
//!
//! Wraps femtoclaw-protocol for internal use.

pub use femtoclaw_protocol::{
    MessageForm, ProtocolOutput, ToolCallForm, ToolCallWrapper, ValidationError, Validator,
};

pub fn create_validator() -> Validator {
    Validator::new()
}
