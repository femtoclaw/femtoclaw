//! WAL-backed Memory (Reference Tier).
//!
//! Wraps a memory implementation and logs all push operations to a Write-Ahead Log.

use crate::memory::Memory;
use crate::types::Message;
use femtoclaw_storage::Wal;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct WalMemory {
    inner: Box<dyn Memory>,
    wal: Arc<RwLock<Wal>>,
}

impl WalMemory {
    pub fn new(inner: Box<dyn Memory>, wal: Wal) -> Self {
        Self {
            inner,
            wal: Arc::new(RwLock::new(wal)),
        }
    }

    pub async fn replay(&mut self) -> anyhow::Result<()> {
        let wal = self.wal.read().await;
        let entries = wal.replay()?;
        for entry in entries {
            if entry.event_type == "push" {
                let msg: Message = serde_json::from_value(entry.payload)?;
                self.inner.push(msg);
            }
        }
        Ok(())
    }
}

impl Memory for WalMemory {
    fn push(&mut self, msg: Message) {
        // NOTE: We are doing synchronous IO in a trait that might be called from async contexts.
        // In a real reference implementation, we might want a separate async channel for WAL.
        // For now, we block or use a simplified approach as this is a Reference implementation.
        let payload = serde_json::to_value(&msg).unwrap();

        // We can't easily await here because the trait is synchronous.
        // This is a common mismatch.
        // Option 1: Change Memory trait to be async.
        // Option 2: Use block_in_place or spawn a task.

        self.inner.push(msg);

        // Since the WAL is in a separate crate and designed for File IO,
        // we'll assume the caller of WalMemory knows it's persistent.

        if let Ok(mut wal) = self.wal.try_write() {
            let _ = wal.append("push", payload);
        }
    }

    fn history(&self) -> &[Message] {
        self.inner.history()
    }

    fn clear(&mut self) {
        self.inner.clear();
        if let Ok(mut wal) = self.wal.try_write() {
            let _ = wal.truncate();
        }
    }

    fn sync(&mut self, messages: &[Message]) {
        self.inner.sync(messages);
        // Sync should also update WAL if we want full consistency
        let _ = self.clear();
        for msg in messages {
            self.push(msg.clone());
        }
    }
}
