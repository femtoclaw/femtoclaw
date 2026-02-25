use crate::types::Message;

pub mod stm;

pub trait Memory: Send {
    fn push(&mut self, msg: Message);
    fn history(&self) -> &[Message];
    fn clear(&mut self);
}
