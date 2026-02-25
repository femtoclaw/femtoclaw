use crate::types::Message;

/// Simple sliding-window memory (short-term).
pub struct ShortTermMemory {
    max_messages: usize,
    buf: Vec<Message>,
}

impl ShortTermMemory {
    pub fn new(max_messages: usize) -> Self {
        Self { max_messages, buf: Vec::with_capacity(max_messages.min(64)) }
    }
}

impl crate::memory::Memory for ShortTermMemory {
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
