//! R22-3: 安全回應記錄 middleware
//!
//! 攔截 403 Forbidden 回應，將 permission denied 事件寫入 user_activity_logs。
//! 放在 auth middleware 外層，確保能讀取 CurrentUser extension。

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::constants::SEC_EVENT_PERMISSION_DENIED;
use crate::middleware::CurrentUser;
use crate::services::AuditService;
use crate::AppState;

pub async fn security_response_logger(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let user_id = request
        .extensions()
        .get::<CurrentUser>()
        .map(|u| u.id);

    let response = next.run(request).await;

    if response.status() == StatusCode::FORBIDDEN {
        let db = state.db.clone();
        tokio::spawn(async move {
            let _ = AuditService::log_security_event(
                &db,
                SEC_EVENT_PERMISSION_DENIED,
                None,
                None,
                Some(&path),
                Some(&method),
                serde_json::json!({
                    "user_id": user_id,
                    "path": path,
                    "method": method,
                }),
            )
            .await;
        });
    }

    response
}
