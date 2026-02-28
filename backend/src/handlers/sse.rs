// 安全警報即時推送（SSE）
// AlertBroadcaster：使用 tokio::sync::broadcast 進行安全警報廣播
// SSE Handler：Server-Sent Events 端點

use axum::{
    extract::State,
    response::sse::{Event, Sse},
    Extension,
};
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, time::Duration};
use tokio::sync::broadcast;

use crate::{
    middleware::CurrentUser,
    AppState,
};

/// 安全警報事件 payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub alert_type: String,
    pub severity: String,
    pub title: String,
    pub description: String,
}

/// AlertBroadcaster — 全域廣播器
#[derive(Clone)]
pub struct AlertBroadcaster {
    sender: broadcast::Sender<AlertEvent>,
}

impl Default for AlertBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertBroadcaster {
    /// 建立新的 AlertBroadcaster（容量 64 條訊息）
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(64);
        Self { sender }
    }

    /// 發送警報事件
    pub fn send(&self, event: AlertEvent) {
        // 即使沒有接收者也不算錯誤
        let _ = self.sender.send(event);
    }

    /// 取得新的接收器
    pub fn subscribe(&self) -> broadcast::Receiver<AlertEvent> {
        self.sender.subscribe()
    }
}

/// SSE 端點 — 管理員即時接收安全警報
pub async fn sse_security_alerts(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.alert_broadcaster.subscribe();

    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(alert) => {
                    let data = serde_json::to_string(&alert).unwrap_or_default();
                    yield Ok(Event::default()
                        .event("security_alert")
                        .data(data));
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("SSE client lagged by {} messages", n);
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("ping"),
    )
}
