use crate::types::Message;

pub mod stm;
pub mod wal;

pub trait Memory: Send + Sync {
    fn push(&mut self, msg: Message);
    fn history(&self) -> &[Message];
    fn clear(&mut self);
    /// Synchronize state with remote messages (Reference Tier).
    fn sync(&mut self, messages: &[Message]);
}
