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
