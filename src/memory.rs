use crate::brain::Message;

pub trait Memory: Send + Sync {
    fn add(&mut self, role: &str, content: &str);
    fn get_history(&self) -> Vec<Message>;
    fn clear(&mut self);
}

pub struct ConversationMemory {
    history: Vec<Message>,
    max_messages: usize,
}

impl ConversationMemory {
    pub fn new(max_messages: usize) -> Self {
        Self {
            history: Vec::new(),
            max_messages,
        }
    }
}

impl Memory for ConversationMemory {
    fn add(&mut self, role: &str, content: &str) {
        self.history.push(Message {
            role: role.to_string(),
            content: content.to_string(),
        });
        // Sliding window to prevent context overflow
        if self.history.len() > self.max_messages {
            self.history.remove(0);
        }
    }

    fn get_history(&self) -> Vec<Message> {
        self.history.clone()
    }

    fn clear(&mut self) {
        self.history.clear();
    }
}
