// 通知服務模組
// 拆分自原始 notification.rs（1,737 行）

mod crud;
mod protocol;
mod animal;
mod alert;
mod report;
mod euthanasia;
mod hr;
mod amendment;
mod erp;
mod helpers;

use sqlx::PgPool;

// 子模組均為 impl NotificationService block，不需要 re-export

pub struct NotificationService {
    pub(crate) db: PgPool,
}

impl NotificationService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}
