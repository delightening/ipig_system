//! PI 顯示來源的共用 SQL 片段建構器（DRY）。
//!
//! 外部 PI 匯入計畫的 `pi_user_id` FK 可能掛在匯入者，真實 PI（客人）身分存於
//! `working_content.basic.pi`。所有「顯示計畫 PI」之查詢一律以 basic.pi 為準、
//! fallback 至 FK 使用者，避免到處重複相同 JSON 路徑（CodeRabbit DRY）。
//!
//! 使用前提：查詢中 protocols 別名為 `p`。`fallback` 為 FK 使用者欄位運算式，
//! 例：`"u.display_name"`、`"u.display_name, u.email"`、`"u.name"`。

/// PI 顯示名片段：`COALESCE(NULLIF(basic.pi.name,''), <fallback>)`。
pub fn pi_display_name(fallback: &str) -> String {
    format!("COALESCE(NULLIF(p.working_content->'basic'->'pi'->>'name', ''), {fallback})")
}

/// 委託單位名片段：`COALESCE(NULLIF(basic.sponsor.name,''), <fallback>)`。
pub fn pi_sponsor_org(fallback: &str) -> String {
    format!("COALESCE(NULLIF(p.working_content->'basic'->'sponsor'->>'name', ''), {fallback})")
}
