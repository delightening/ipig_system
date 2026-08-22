//! R30-3a Channel Adapter — outbox event 對外送出邏輯。
//!
//! Worker 從 outbox claim 一筆 OutboxEvent 後，依 `channel` 路由到對應 adapter。
//! Adapter 只回 `Ok(())` 或 `Err(...)`；retry / next_attempt_at 由 OutboxService::mark_failed 管。

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use super::OutboxEvent;
use crate::{AppError, Result};

/// Outbox row status — Rust 型別封裝（DB 是 TEXT + CHECK constraint）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutboxStatus {
    Pending,
    Sending,
    Done,
    Failed,
    Dead,
}

impl OutboxStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Sending => "SENDING",
            Self::Done => "DONE",
            Self::Failed => "FAILED",
            Self::Dead => "DEAD",
        }
    }
}

/// Channel adapter trait — 每個 outbound channel 一個實作。
///
/// `send` **只負責對外送出**，不要碰 outbox row 狀態（status / attempt_count
/// 等由 OutboxService 唯一 caller 管）。
#[async_trait]
pub trait ChannelAdapter: Send + Sync {
    /// Channel name — 用來註冊到 ChannelRegistry，必須與 OutboxEvent.channel 對應。
    fn channel(&self) -> &'static str;

    /// 送出事件。失敗回 Err，由 worker 標記 mark_failed。
    async fn send(&self, event: &OutboxEvent) -> Result<()>;
}

/// 註冊所有 channel adapter，依 OutboxEvent.channel 路由。
#[derive(Default)]
pub struct ChannelRegistry {
    adapters: HashMap<&'static str, Arc<dyn ChannelAdapter>>,
}

impl ChannelRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 註冊 channel adapter；重複 channel name → panic（startup-time fail-fast，
    /// 避免靜默覆蓋導致路由錯亂）。
    pub fn register<A: ChannelAdapter + 'static>(mut self, adapter: A) -> Self {
        let key = adapter.channel();
        if self.adapters.insert(key, Arc::new(adapter)).is_some() {
            panic!(
                "outbox: duplicate ChannelAdapter registration for channel '{}'",
                key
            );
        }
        self
    }

    /// 路由到對應 adapter。未註冊 channel → 回 Err（worker 會 mark_failed）。
    pub async fn send(&self, event: &OutboxEvent) -> Result<()> {
        let adapter = self.adapters.get(event.channel.as_str()).ok_or_else(|| {
            AppError::Internal(format!(
                "outbox: no adapter registered for channel '{}'",
                event.channel
            ))
        })?;
        adapter.send(event).await
    }

    pub fn registered_channels(&self) -> Vec<&'static str> {
        self.adapters.keys().copied().collect()
    }
}
