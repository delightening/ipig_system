use axum::{routing::post, Router};

use crate::{handlers, AppState};

/// MCP（Model Context Protocol）路由
/// 使用個人 MCP API Key 認證（Bearer mcp_xxxx_...），不走 JWT / CSRF
pub fn routes() -> Router<AppState> {
    Router::new().route("/mcp", post(handlers::mcp::mcp_message))
}
