// Activity Logger Middleware
// 自動記錄 API 請求到 user_activity_logs

use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{Method, Request},
    middleware::Next,
    response::Response,
};
use chrono::Utc;
use std::net::SocketAddr;
use uuid::Uuid;

use crate::AppState;

/// 活動記錄 Middleware
/// 自動記錄重要 API 操作到 user_activity_logs 表
pub async fn activity_logger_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let start = std::time::Instant::now();
    
    // 提取請求資訊
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let ip_address = addr.ip().to_string();
    
    // 從 extension 取得 user_id（如果有的話）
    let user_id = request
        .extensions()
        .get::<Uuid>()
        .cloned();
    
    // 決定事件類別和類型
    let (event_category, event_type, entity_type) = categorize_request(&method, &path);
    
    // 判斷是否需要記錄
    let should_log = should_log_request(&method, &path);
    
    // 執行請求
    let response = next.run(request).await;
    
    // 如果需要記錄，異步寫入資料庫
    if should_log {
        let duration_ms = start.elapsed().as_millis() as i32;
        let status_code = response.status().as_u16() as i32;
        let is_success = response.status().is_success();
        let is_suspicious = check_suspicious(&method, &path, status_code);
        
        let pool = state.db.clone();
        let partition_date = Utc::now().date_naive();
        
        // 異步記錄，不阻塞請求
        tokio::spawn(async move {
            let log_id = Uuid::new_v4();
            let _ = sqlx::query(
                r#"
                INSERT INTO user_activity_logs (
                    id, partition_date, actor_user_id,
                    event_category, event_type, event_severity,
                    entity_type, request_path, request_method, response_status,
                    ip_address, user_agent, duration_ms,
                    is_suspicious, created_at
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, NOW()
                )
                "#,
            )
            .bind(log_id)
            .bind(partition_date)
            .bind(user_id)
            .bind(&event_category)
            .bind(&event_type)
            .bind(if is_success { "info" } else { "warning" })
            .bind(&entity_type)
            .bind(&path)
            .bind(method.as_str())
            .bind(status_code)
            .bind(&ip_address)
            .bind(&user_agent)
            .bind(duration_ms)
            .bind(is_suspicious)
            .execute(&pool)
            .await;
        });
    }
    
    response
}

/// 分類請求
fn categorize_request(method: &Method, path: &str) -> (String, String, Option<String>) {
    // 從路徑解析實體類型
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    
    let entity_type = if segments.len() >= 2 {
        match segments.get(1) {
            Some(&"users") => Some("user".to_string()),
            Some(&"roles") => Some("role".to_string()),
            Some(&"products") => Some("product".to_string()),
            Some(&"partners") => Some("partner".to_string()),
            Some(&"documents") => Some("document".to_string()),
            Some(&"protocols") => Some("protocol".to_string()),
            Some(&"pigs") => Some("pig".to_string()),
            Some(&"leaves") => Some("leave_request".to_string()),
            Some(&"overtime") => Some("overtime".to_string()),
            Some(&"attendance") => Some("attendance".to_string()),
            _ => None,
        }
    } else {
        None
    };
    
    // 決定事件類別
    let event_category = match segments.get(1) {
        Some(&"auth") => "authentication",
        Some(&"admin") => "admin",
        Some(&"hr") => "hr",
        Some(&"users") | Some(&"roles") => "user_management",
        Some(&"products") | Some(&"partners") | Some(&"warehouses") => "master_data",
        Some(&"documents") | Some(&"inventory") => "transaction",
        Some(&"protocols") => "aup",
        Some(&"pigs") => "animal_management",
        Some(&"reports") => "report",
        Some(&"notifications") => "notification",
        _ => "system",
    }
    .to_string();
    
    // 決定事件類型
    let event_type = match (method.as_str(), segments.len()) {
        ("GET", _) if path.contains("export") => "export",
        ("GET", _) if path.contains("download") => "download",
        ("GET", 2) => "list",      // GET /api/users
        ("GET", _) => "view",      // GET /api/users/123
        ("POST", _) if path.contains("login") => "login",
        ("POST", _) if path.contains("logout") => "logout",
        ("POST", _) if path.contains("submit") => "submit",
        ("POST", _) if path.contains("approve") => "approve",
        ("POST", _) if path.contains("reject") => "reject",
        ("POST", _) if path.contains("cancel") => "cancel",
        ("POST", _) if path.contains("clock-in") => "clock_in",
        ("POST", _) if path.contains("clock-out") => "clock_out",
        ("POST", _) => "create",
        ("PUT", _) | ("PATCH", _) => "update",
        ("DELETE", _) => "delete",
        _ => "unknown",
    }
    .to_string();
    
    (event_category, event_type, entity_type)
}

/// 判斷是否需要記錄
fn should_log_request(method: &Method, path: &str) -> bool {
    // 不記錄的路徑
    let skip_paths = [
        "/api/me",
        "/api/notifications/unread-count",
        "/health",
        "/api/auth/refresh",
    ];
    
    if skip_paths.iter().any(|p| path.starts_with(p)) {
        return false;
    }
    
    // 只記錄修改操作和重要的讀取操作
    match method.as_str() {
        "GET" => {
            // 只記錄 export、download、敏感資料存取
            path.contains("export")
                || path.contains("download")
                || path.contains("audit")
                || path.contains("admin")
        }
        "POST" | "PUT" | "PATCH" | "DELETE" => true,
        _ => false,
    }
}

/// 檢查是否為可疑活動
fn check_suspicious(method: &Method, path: &str, status_code: i32) -> bool {
    // 多次失敗的請求
    if status_code == 401 || status_code == 403 {
        return true;
    }
    
    // 敏感操作
    if path.contains("/admin/") && method != Method::GET {
        return false; // 記錄但不標記為可疑（admin 修改是正常的）
    }
    
    // 大量刪除
    if method == Method::DELETE && path.contains("batch") {
        return true;
    }
    
    // 強制登出他人
    if path.contains("force") && path.contains("logout") {
        return true;
    }
    
    false
}
