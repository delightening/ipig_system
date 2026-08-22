use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::{handlers, AppState};

/// R40-A 站內信路由
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/messages/threads",
            get(handlers::messaging::list_threads).post(handlers::messaging::create_thread),
        )
        .route(
            "/messages/threads/{id}",
            get(handlers::messaging::get_thread).post(handlers::messaging::send_message),
        )
        .route(
            "/messages/threads/{id}/read",
            post(handlers::messaging::mark_thread_read),
        )
        .route(
            "/messages/{id}",
            delete(handlers::messaging::delete_message),
        )
        .route(
            "/messages/unread-count",
            get(handlers::messaging::unread_count),
        )
        .route(
            "/messages/recipients",
            get(handlers::messaging::list_recipients),
        )
        // 注意：附件「上傳」POST /messages/attachments 已移至 routes/upload.rs，
        // 改套較嚴的 upload_rate_limit（30/min）而非 write_rate_limit（120/min）。
        // 資安稽核 L-4：原本走錯限流層 → 可 4 倍速率灌檔塞爆磁碟。下載 GET 留於此。
        .route(
            "/messages/attachments/{id}/download",
            get(handlers::messaging::download_attachment),
        )
}
