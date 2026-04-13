//! MCP Tool 執行邏輯
//!
//! 每個 tool 函式回傳 `Result<serde_json::Value>`，由 handler 包裝成 JSON-RPC response。

mod tools;

pub use tools::{
    batch_return_to_pi, create_review_flag, get_review_history, list_protocols, notify_secretary,
    read_protocol, submit_vet_review,
};
