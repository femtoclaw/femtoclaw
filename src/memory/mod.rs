use crate::types::Message;

pub mod stm;

pub trait Memory: Send + Sync {
    fn push(&mut self, msg: Message);
    fn history(&self) -> &[Message];
    fn clear(&mut self);
}
