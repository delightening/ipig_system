use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{AppError, Result};

/// 查詢使用者顯示名稱（出現在 document/blood_test/hr/signature 等多處）
pub async fn find_user_display_name_by_id(pool: &PgPool, user_id: Uuid) -> Result<Option<String>> {
    let name = sqlx::query_scalar("SELECT display_name FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(name)
}

/// 查詢**在職**使用者的顯示名稱，在呼叫端的交易內執行。
///
/// 與 [`find_user_display_name_by_id`] 的兩點差異：
/// 1. 走呼叫端的 `tx`——供「業務狀態轉換與通知建立必須同一個 tx」的流程使用
///    （見 `NotificationService::create_pinned_notification_tx` 的文件）。
/// 2. 過濾 `is_active` / `deleted_at`：用於要寫進通知標題給別人看的名字，
///    已停用或已刪除的帳號不應被當成有效的顯示名沿用。
pub async fn find_active_user_display_name_by_id_tx(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
) -> Result<Option<String>> {
    let name = sqlx::query_scalar(
        "SELECT display_name FROM users \
         WHERE id = $1 AND is_active = true AND deleted_at IS NULL",
    )
    .bind(user_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::Database)?;
    Ok(name)
}

/// 批次查詢多個使用者的顯示名稱。
///
/// N+1 修復：供「一份清單逐列補上姓名」的場景使用（簽章清單、記錄附註），
/// 取代在迴圈中逐一呼叫 [`find_user_display_name_by_id`]。
/// `display_name` 為 NULL 或使用者不存在者不會出現在結果中，呼叫端查無即 `None`。
pub async fn find_user_display_names_by_ids(
    pool: &PgPool,
    user_ids: &[Uuid],
) -> Result<std::collections::HashMap<Uuid, String>> {
    if user_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let rows: Vec<(Uuid, Option<String>)> =
        sqlx::query_as("SELECT id, display_name FROM users WHERE id = ANY($1::uuid[])")
            .bind(user_ids)
            .fetch_all(pool)
            .await
            .map_err(AppError::Database)?;
    Ok(rows
        .into_iter()
        .filter_map(|(id, name)| name.map(|n| (id, n)))
        .collect())
}

/// 查詢使用者帳號狀態 (is_active, expires_at)。
///
/// 使用情境：auth_middleware 每請求驗證帳號狀態。
/// 回傳 `None` 表示使用者不存在；caller 應視為 Unauthorized。
///
/// R27-4：原本 SQL 寫在 middleware 內違反分層職責（CLAUDE.md「Middleware 禁業務邏輯」），
/// 移到 repository 層讓 services/access.rs 等其他模組也能復用。
pub async fn find_user_active_status_by_id(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<(bool, Option<DateTime<Utc>>)>> {
    let row: Option<(bool, Option<DateTime<Utc>>)> =
        sqlx::query_as("SELECT is_active, expires_at FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)?;
    Ok(row)
}

/// 查詢使用者的 `tokens_valid_after`（access token 有效起點；NULL = 無限制）。
///
/// CSO-r3 #3/#4：auth_middleware 每請求比對 JWT iat < tokens_valid_after，
/// 撤銷改密碼 / 重設 / 角色變更前簽發的舊 access token（含其 roles 快照）。
pub async fn find_tokens_valid_after(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<DateTime<Utc>>> {
    let row: Option<(Option<DateTime<Utc>>,)> =
        sqlx::query_as("SELECT tokens_valid_after FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)?;
    Ok(row.and_then(|r| r.0))
}

/// 查詢使用者所有有效權限 code（4-table JOIN: permissions × role_permissions ×
/// user_roles × roles，篩 `r.is_active = true`）。
///
/// 使用情境：auth_middleware 的 permission_cache loader（H2 single-flight）。
/// R27-4：從 middleware 抽到 repository 層；同 SELECT 也能被 services/access.rs
/// 等其他模組復用。
pub async fn list_permission_codes_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<String>> {
    let perms = sqlx::query_scalar::<_, String>(
        r#"SELECT DISTINCT p.code FROM permissions p
           INNER JOIN role_permissions rp ON p.id = rp.permission_id
           INNER JOIN user_roles ur ON rp.role_id = ur.role_id
           INNER JOIN roles r ON r.id = ur.role_id
           WHERE ur.user_id = $1 AND r.is_active = true"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        // Gemini PR #218 Medium：與 find_user_active_status_by_id 一致，
        // 用 AppError::Database 保留具體 SQL 錯誤 context。
        tracing::error!(
            "[repositories::user] 載入使用者 {} 權限失敗: {}",
            user_id,
            e
        );
        AppError::Database(e)
    })?;
    Ok(perms)
}
