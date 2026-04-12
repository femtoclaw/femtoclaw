//! Mock brain for testing autonomous execution.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{brain::Brain, types::Message};

#[derive(Default, Clone)]
pub struct MockBrain {
    responses: Arc<Mutex<Vec<String>>>,
}

impl MockBrain {
    pub fn new(responses: Vec<String>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses)),
        }
    }
}

#[async_trait]
impl Brain for MockBrain {
    async fn think(&self, _messages: &[Message]) -> anyhow::Result<String> {
        let mut responses = self.responses.lock().await;
        if responses.is_empty() {
            return Ok(r#"{"message":{"content":"No more responses in mock"}}"#.to_string());
        }
        Ok(responses.remove(0))
    }
}
