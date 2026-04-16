//! R22-16: 蜜罐路由
//!
//! 常見攻擊者探測的路徑，放在 /api/v1 外層。

use axum::{routing::get, Router};
use crate::handlers::honeypot::honeypot_handler;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/.env", get(honeypot_handler).post(honeypot_handler))
        .route("/wp-login.php", get(honeypot_handler).post(honeypot_handler))
        .route("/wp-admin", get(honeypot_handler))
        .route("/phpmyadmin", get(honeypot_handler))
        .route("/xmlrpc.php", get(honeypot_handler).post(honeypot_handler))
        .route("/admin/backup", get(honeypot_handler))
}
