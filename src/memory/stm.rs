use crate::types::Message;

/// Short-Term Memory (STM) - simple sliding-window memory.
pub struct Stm {
    max_messages: usize,
    buf: Vec<Message>,
}

impl Stm {
    pub fn new(max_messages: usize) -> Self {
        Self {
            max_messages,
            buf: Vec::with_capacity(max_messages.min(64)),
        }
    }
}

impl crate::memory::Memory for Stm {
    fn push(&mut self, msg: Message) {
        self.buf.push(msg);
        if self.buf.len() > self.max_messages {
            let overflow = self.buf.len() - self.max_messages;
            self.buf.drain(0..overflow);
        }
    }

    fn history(&self) -> &[Message] {
        &self.buf
    }

    fn clear(&mut self) {
        self.buf.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Memory;
    use crate::types::{Message, Role};

    fn user_msg(s: &str) -> Message {
        Message::user(s)
    }

    fn assistant_msg(s: &str) -> Message {
        Message::assistant(s)
    }

    #[test]
    fn test_stm_eviction() {
        let mut stm = Stm::new(3);
        for i in 0..5 {
            stm.push(user_msg(&format!("u{}", i)));
            stm.push(assistant_msg(&format!("a{}", i)));
        }
        assert!(stm.history().len() <= 3);
    }

    #[test]
    fn test_stm_no_eviction_under_max() {
        let mut stm = Stm::new(4);
        for i in 0..2 {
            stm.push(user_msg(&format!("u{}", i)));
            stm.push(assistant_msg(&format!("a{}", i)));
        }
        assert_eq!(stm.history().len(), 4);
    }
}
