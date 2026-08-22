// 通知服務模組
// 拆分自原始 notification.rs（1,737 行）

mod alert;
mod amendment;
mod animal;
mod crud;
mod dispatch;
pub(crate) mod dispatcher;
mod equipment;
mod erp;
mod euthanasia;
mod expiry_config;
pub(crate) mod expiry_monthly;
mod helpers;
mod hr;
mod protocol;
mod reconcile;
mod report;
mod resolvers;
mod routing;
mod send_window;

use std::sync::OnceLock;

use sqlx::PgPool;

// dispatch 的 StaffEmail / DispatchOutcome 供 scheduler 等同 crate 呼叫端建構參數用。
pub use dispatch::{DispatchOutcome, StaffEmail};
// 統一派送層型別：供各 notify_* 與呼叫端建構事件上下文 / 內容用。
pub use dispatcher::{EventContext, NotificationPayload};
// 置頂待辦對帳：供一次性修補 bin 與定期排程共用。
pub use reconcile::{OrphanPinnedRow, ReconcileReport};

/// 程序級全域 app_url，供 dispatch_event 渲染通知 email（與 holiday::global 同風格，
/// 避免將 config 逐層 thread 進只持有 db 的通知服務）。
static APP_URL: OnceLock<String> = OnceLock::new();

/// 初始化全域 app_url（main.rs 啟動時呼叫一次）。重複呼叫忽略。
pub fn init_app_url(url: String) {
    let _ = APP_URL.set(url);
}

/// 取得全域 app_url（未初始化回 None；測試 / bin 未設時 dispatch_event 跳過 email）。
pub(crate) fn app_url() -> Option<String> {
    APP_URL.get().cloned()
}

pub struct NotificationService {
    pub(crate) db: PgPool,
}

impl NotificationService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}
