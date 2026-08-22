//! MCP Tool 實作
//!
//! 每個 tool 接受 `&serde_json::Value`（來自 tools/call arguments），
//! 呼叫現有 services，回傳 `Result<serde_json::Value>`。

use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::middleware::{ActorContext, CurrentUser};
use crate::models::{ChangeStatusRequest, CreateCommentRequest, ProtocolStatus};
use crate::services::{EmailService, ProtocolService};
use crate::{AppError, AppState, Result};

// ── 輔助：從 args 解析字串欄位 ──

fn str_arg<'a>(args: &'a Value, key: &str) -> Result<&'a str> {
    args.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest(format!("Missing argument: {key}")))
}

// ── notify_secretary 每位呼叫者每日寄信上限（CSO-r3 #7）──
//
// 防 MCP key 外流後被當「公司名義」對內外狂寄釣魚信。以 caller `user.id` 計
// （一人通常持一把 MCP key，等同 per-key）。in-process 計數、backend 重啟歸零
// （與 health.rs 的 daemon-down 告警限流、jwt_blacklist 同屬 in-memory 防濫用）。
const NOTIFY_SECRETARY_DAILY_LIMIT: u32 = 30;
const NOTIFY_SECRETARY_WINDOW: std::time::Duration = std::time::Duration::from_secs(86_400);
#[allow(clippy::type_complexity)]
static NOTIFY_SECRETARY_RATE: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<Uuid, (std::time::Instant, u32)>>,
> = std::sync::OnceLock::new();

/// 檢查並遞增 caller 的 notify_secretary 每日配額；超限回 Forbidden。
fn check_notify_secretary_rate(user_id: Uuid) -> Result<()> {
    let map = NOTIFY_SECRETARY_RATE
        .get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    // poisoned mutex 不該發生（hold time 極短、無 panic），保險起見 recover
    let mut guard = map.lock().unwrap_or_else(|p| p.into_inner());
    // Gemini #532：清掉已過窗口的舊條目，避免 map 隨不同 caller 無限增長（記憶體洩漏）
    guard.retain(|_, (start, _)| start.elapsed() < NOTIFY_SECRETARY_WINDOW);
    let entry = guard
        .entry(user_id)
        .or_insert_with(|| (std::time::Instant::now(), 0));
    if entry.0.elapsed() >= NOTIFY_SECRETARY_WINDOW {
        *entry = (std::time::Instant::now(), 0);
    }
    if entry.1 >= NOTIFY_SECRETARY_DAILY_LIMIT {
        return Err(AppError::TooManyRequests(format!(
            "notify_secretary 已達每日上限（{NOTIFY_SECRETARY_DAILY_LIMIT} 封/日），請明日再試或聯絡管理員"
        )));
    }
    entry.1 += 1;
    Ok(())
}

fn uuid_arg(args: &Value, key: &str) -> Result<Uuid> {
    let s = str_arg(args, key)?;
    // 支援 UUID 或計畫書編號（AUP-YYYY-NNN）
    if let Ok(id) = s.parse::<Uuid>() {
        return Ok(id);
    }
    Err(AppError::BadRequest(format!(
        "Invalid UUID for argument: {key}"
    )))
}

// ── 1. list_protocols ──

pub async fn list_protocols(state: &AppState, user: &CurrentUser, args: &Value) -> Result<Value> {
    let status_filter = args.get("status").and_then(|v| v.as_str());
    let limit = args
        .get("limit")
        .and_then(|v| v.as_i64())
        .unwrap_or(20)
        .min(100);
    let list = list_visible_protocols(&state.db, user, status_filter, limit).await?;
    Ok(serde_json::json!({ "protocols": list, "count": list.len() }))
}

/// 依使用者可見範圍列出計畫（供 list_protocols 使用，抽出以利測試）。
///
/// SEC-IDOR: MCP 讀取須對齊使用者可見範圍（與 HTTP 一致）。具 `aup.protocol.view_all`
/// 者看全部；其餘僅列使用者關聯的計畫——關聯來源與 `access::has_any_protocol_role`
/// 一致（PI / user_protocols / review_assignments / vet_review_assignments），
/// 改動任一端時兩處須同步。
pub async fn list_visible_protocols(
    db: &sqlx::PgPool,
    user: &CurrentUser,
    status_filter: Option<&str>,
    limit: i64,
) -> Result<Vec<Value>> {
    // 防禦：負數 limit 會讓 PostgreSQL 拋 "LIMIT must not be negative" → 500。
    let limit = limit.max(0);
    type ProtocolRow = (
        Uuid,
        String,
        Option<String>,
        String,
        String,
        Option<String>,
        DateTime<Utc>,
    );
    // 修正：users 表欄位為 display_name（原碼誤用不存在的 u.name；因 MCP 認證
    // C1 bug 從未跑到此查詢而未被發現）。
    let pi_name_sql = crate::utils::pi_sql::pi_display_name("u.display_name");
    let scoped = !crate::services::access::has_protocol_view_all(user);

    let mut qb = sqlx::QueryBuilder::new(format!(
        "SELECT p.id, p.protocol_no, p.iacuc_no, p.title, p.status::text, \
         {pi_name_sql} AS pi_name, p.updated_at \
         FROM protocols p LEFT JOIN users u ON p.pi_user_id = u.id \
         WHERE p.status != 'DELETED'"
    ));
    if let Some(status) = status_filter {
        qb.push(" AND p.status::text = ")
            .push_bind(status.to_string());
    }
    if scoped {
        qb.push(" AND (p.pi_user_id = ").push_bind(user.id);
        qb.push(" OR EXISTS (SELECT 1 FROM user_protocols up WHERE up.protocol_id = p.id AND up.user_id = ").push_bind(user.id);
        qb.push(") OR EXISTS (SELECT 1 FROM review_assignments ra WHERE ra.protocol_id = p.id AND ra.reviewer_id = ").push_bind(user.id);
        qb.push(") OR EXISTS (SELECT 1 FROM vet_review_assignments vra WHERE vra.protocol_id = p.id AND vra.vet_id = ").push_bind(user.id);
        qb.push("))");
    }
    qb.push(" ORDER BY p.updated_at DESC LIMIT ")
        .push_bind(limit);

    let rows: Vec<ProtocolRow> = qb.build_query_as().fetch_all(db).await?;

    Ok(rows
        .into_iter()
        .map(|(id, no, iacuc_no, title, status, pi_name, updated_at)| {
            serde_json::json!({
                "id": id,
                "protocol_no": no,
                "iacuc_no": iacuc_no,
                "title": title,
                "status": status,
                "pi_name": pi_name,
                "updated_at": updated_at.to_rfc3339()
            })
        })
        .collect())
}

// ── 2. read_protocol（含稽核日誌） ──

pub async fn read_protocol(state: &AppState, user: &CurrentUser, args: &Value) -> Result<Value> {
    // 支援 UUID 或 protocol_no（AUP-YYYY-NNN）
    let id_str = str_arg(args, "protocol_id")?;
    let protocol = if let Ok(id) = id_str.parse::<Uuid>() {
        sqlx::query_as::<_, crate::models::Protocol>(
            "SELECT * FROM protocols WHERE id = $1 AND status != 'DELETED'",
        )
        .bind(id)
        .fetch_optional(&state.db)
        .await?
    } else {
        sqlx::query_as::<_, crate::models::Protocol>(
            "SELECT * FROM protocols WHERE protocol_no = $1 AND status != 'DELETED'",
        )
        .bind(id_str)
        .fetch_optional(&state.db)
        .await?
    }
    .ok_or_else(|| AppError::NotFound("計畫書不存在".to_string()))?;

    // SEC-IDOR: 對齊使用者可見範圍（與 HTTP get-protocol 一致）。view_all 角色放行；
    // 其餘須與該計畫有關聯（PI / 成員 / 指派審查委員 / 指派獸醫），否則 403。
    crate::services::access::require_protocol_related_access(&state.db, user, protocol.id).await?;

    // PI 資訊
    let pi_info: Option<(String, String, Option<String>)> =
        sqlx::query_as("SELECT display_name, email, organization FROM users WHERE id = $1")
            .bind(protocol.pi_user_id)
            .fetch_optional(&state.db)
            .await?;

    // PI 歷史退件次數（PRE_REVIEW_REVISION_REQUIRED 次數）
    let past_returns: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM protocols
        WHERE pi_user_id = $1
          AND id != $2
          AND status IN ('PRE_REVIEW_REVISION_REQUIRED', 'REVISION_REQUIRED')
        "#,
    )
    .bind(protocol.pi_user_id)
    .bind(protocol.id)
    .fetch_one(&state.db)
    .await?;

    // 若為 REVIEWER 或 VET，記錄稽核日誌
    let is_reviewer_or_vet = user.roles.iter().any(|r| {
        [crate::constants::ROLE_REVIEWER, crate::constants::ROLE_VET].contains(&r.as_str())
    });
    if is_reviewer_or_vet {
        record_mcp_read(&state.db, protocol.id, user.id).await;
    }

    let (mut pi_name, mut pi_email, mut pi_org) = pi_info.unwrap_or_default();
    // PI（客人）/ 委託單位以研究資料 basic 為準（外部 PI 匯入時與 FK 不同），fallback FK 使用者
    if let Some(basic) = protocol
        .working_content
        .as_ref()
        .and_then(|w| w.get("basic"))
    {
        let nonempty = |v: Option<&serde_json::Value>| {
            v.and_then(|x| x.as_str())
                .map(str::trim)
                .filter(|t| !t.is_empty())
                .map(String::from)
        };
        if let Some(n) = nonempty(basic.get("pi").and_then(|p| p.get("name"))) {
            pi_name = n;
        }
        if let Some(e) = nonempty(basic.get("pi").and_then(|p| p.get("email"))) {
            pi_email = e;
        }
        if let Some(o) = nonempty(basic.get("sponsor").and_then(|s| s.get("name"))) {
            pi_org = Some(o);
        }
    }

    Ok(serde_json::json!({
        "id": protocol.id,
        "protocol_no": protocol.protocol_no,
        "iacuc_no": protocol.iacuc_no,
        "title": protocol.title,
        "status": protocol.status.as_str(),
        "pi": {
            "name": pi_name,
            "email": pi_email,
            "organization": pi_org
        },
        "pi_history": {
            "past_return_count": past_returns
        },
        "content": protocol.working_content
    }))
}

async fn record_mcp_read(db: &sqlx::PgPool, protocol_id: Uuid, user_id: Uuid) {
    let _ = sqlx::query(
        r#"
        INSERT INTO protocol_activities (id, protocol_id, activity_type, performed_by, metadata)
        VALUES (gen_random_uuid(), $1, 'MCP_READ', $2, '{}')
        "#,
    )
    .bind(protocol_id)
    .bind(user_id)
    .execute(db)
    .await;
}

// ── 3. create_review_flag ──

pub async fn create_review_flag(
    state: &AppState,
    user: &CurrentUser,
    args: &Value,
) -> Result<Value> {
    let protocol_id = uuid_arg(args, "protocol_id")?;
    let flag_type = str_arg(args, "flag_type")?;
    let section = str_arg(args, "section")?;
    let message = str_arg(args, "message")?;
    let suggestion = str_arg(args, "suggestion")?;

    // 驗證 flag_type
    if !["needs_attention", "concern", "suggestion"].contains(&flag_type) {
        return Err(AppError::BadRequest(
            "flag_type 必須為 needs_attention / concern / suggestion".to_string(),
        ));
    }

    // 寫入 protocol_ai_reviews（staff_pre_review 類型）
    sqlx::query(
        r#"
        INSERT INTO protocol_ai_reviews (
            id, protocol_id, review_type, rule_result, ai_result,
            total_errors, total_warnings, triggered_by
        ) VALUES (
            gen_random_uuid(), $1, 'staff_pre_review',
            '{"errors":[],"warnings":[],"passed":[]}'::jsonb,
            $2::jsonb,
            $3, $4, $5
        )
        "#,
    )
    .bind(protocol_id)
    .bind(serde_json::json!({
        "flags": [{
            "flag_type": flag_type,
            "section": section,
            "message": message,
            "suggestion": suggestion
        }]
    }))
    .bind(if flag_type == "needs_attention" {
        1i32
    } else {
        0i32
    })
    .bind(if flag_type == "concern" { 1i32 } else { 0i32 })
    .bind(user.id)
    .execute(&state.db)
    .await?;

    Ok(serde_json::json!({
        "success": true,
        "flag": {
            "flag_type": flag_type,
            "section": section,
            "message": message,
            "suggestion": suggestion
        }
    }))
}

// ── 4. batch_return_to_pi ──

pub async fn batch_return_to_pi(
    state: &AppState,
    user: &CurrentUser,
    args: &Value,
) -> Result<Value> {
    let protocol_id = uuid_arg(args, "protocol_id")?;

    // 驗證狀態
    let protocol =
        sqlx::query_as::<_, crate::models::Protocol>("SELECT * FROM protocols WHERE id = $1")
            .bind(protocol_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("計畫書不存在".to_string()))?;

    if protocol.status != ProtocolStatus::PreReview {
        return Err(AppError::BusinessRule(
            "僅可在行政預審（PRE_REVIEW）狀態下退回補件".to_string(),
        ));
    }

    // 取得最新 version_id
    let version_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM protocol_versions WHERE protocol_id = $1 ORDER BY version_no DESC LIMIT 1",
    )
    .bind(protocol_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Protocol version not found".to_string()))?;

    // 建立 review_comments
    let flags = args
        .get("flags")
        .and_then(|v| v.as_array())
        .ok_or_else(|| AppError::BadRequest("flags 為必填陣列".to_string()))?;

    let mut created = 0usize;
    for flag in flags {
        let section = flag.get("section").and_then(|v| v.as_str()).unwrap_or("—");
        let message = flag.get("message").and_then(|v| v.as_str()).unwrap_or("");
        let suggestion = flag
            .get("suggestion")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let content = format!("【行政預審補件】[{section}] {message}\n\n建議：{suggestion}");
        let req = CreateCommentRequest {
            protocol_version_id: version_id,
            content,
            review_stage: Some("PRE_REVIEW".to_string()),
        };
        ProtocolService::add_comment(&state.db, &req, user.id).await?;
        created += 1;
    }

    // 補充說明
    if let Some(note) = args.get("additional_note").and_then(|v| v.as_str()) {
        let trimmed = note.trim();
        if !trimmed.is_empty() {
            let req = CreateCommentRequest {
                protocol_version_id: version_id,
                content: trimmed.to_string(),
                review_stage: Some("PRE_REVIEW".to_string()),
            };
            ProtocolService::add_comment(&state.db, &req, user.id).await?;
            created += 1;
        }
    }

    // 變更狀態
    let status_req = ChangeStatusRequest {
        to_status: ProtocolStatus::PreReviewRevisionRequired,
        remark: args
            .get("additional_note")
            .and_then(|v| v.as_str())
            .map(String::from),
        reviewer_ids: None,
        vet_id: None,
    };
    let actor = ActorContext::User(user.clone());
    ProtocolService::change_status(&state.db, &actor, protocol_id, &status_req).await?;

    Ok(serde_json::json!({
        "created_comments": created,
        "status": "PRE_REVIEW_REVISION_REQUIRED"
    }))
}

// ── 5. get_review_history ──

pub async fn get_review_history(state: &AppState, args: &Value) -> Result<Value> {
    let pi_user_id = uuid_arg(args, "pi_user_id")?;
    let limit = args
        .get("limit")
        .and_then(|v| v.as_i64())
        .unwrap_or(5)
        .min(20);

    let rows: Vec<(String, String, Option<String>)> = sqlx::query_as(
        r#"
        SELECT p.protocol_no, p.title, p.status::text
        FROM protocols p
        WHERE p.pi_user_id = $1
          AND p.status IN (
              'PRE_REVIEW_REVISION_REQUIRED', 'REVISION_REQUIRED',
              'APPROVED', 'REJECTED', 'DEFERRED'
          )
        ORDER BY p.updated_at DESC
        LIMIT $2
        "#,
    )
    .bind(pi_user_id)
    .bind(limit)
    .fetch_all(&state.db)
    .await?;

    let history: Vec<Value> = rows
        .into_iter()
        .map(|(no, title, status)| {
            serde_json::json!({
                "protocol_no": no,
                "title": title,
                "status": status
            })
        })
        .collect();

    Ok(serde_json::json!({ "history": history }))
}

// ── 6. submit_vet_review ──

pub async fn submit_vet_review(
    state: &AppState,
    user: &CurrentUser,
    args: &Value,
) -> Result<Value> {
    let protocol_id = uuid_arg(args, "protocol_id")?;

    // 驗證 VET 被指派到此計畫書
    let is_assigned: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM vet_review_assignments WHERE protocol_id = $1 AND vet_id = $2)",
    )
    .bind(protocol_id)
    .bind(user.id)
    .fetch_one(&state.db)
    .await?;

    let is_admin = user
        .roles
        .iter()
        .any(|r| r == crate::constants::ROLE_SYSTEM_ADMIN);
    if !is_assigned && !is_admin {
        return Err(AppError::Forbidden("您未被指派審查此計畫書".to_string()));
    }

    let items = args
        .get("items")
        .ok_or_else(|| AppError::BadRequest("items 為必填".to_string()))?;
    let vet_signature = args.get("vet_signature").and_then(|v| v.as_str());

    // 組合 review_form JSON（對應 VetReviewForm struct）
    let review_form = serde_json::json!({
        "items": items,
        "vet_signature": vet_signature,
        "signed_at": chrono::Utc::now().to_rfc3339()
    });

    // 更新 vet_review_assignments
    sqlx::query(
        r#"
        UPDATE vet_review_assignments
        SET review_form = $1,
            completed_at = NOW()
        WHERE protocol_id = $2 AND vet_id = $3
        "#,
    )
    .bind(&review_form)
    .bind(protocol_id)
    .bind(user.id)
    .execute(&state.db)
    .await?;

    Ok(serde_json::json!({
        "success": true,
        "protocol_id": protocol_id,
        "items_count": items.as_array().map(|a| a.len()).unwrap_or(0),
        "signed_by": user.email
    }))
}

// ── 7. notify_secretary ──
// 讓排程 agent 呼叫，透過 iPig SMTP 發送通知信給執行秘書
// 僅 STAFF / CHAIR / ADMIN 角色可呼叫

pub async fn notify_secretary(state: &AppState, user: &CurrentUser, args: &Value) -> Result<Value> {
    let subject = str_arg(args, "subject")?;
    let raw_html = str_arg(args, "body_html")?;
    let allowed: std::collections::HashSet<&str> = [
        "p", "br", "strong", "em", "b", "i", "ul", "ol", "li", "h3", "h4",
    ]
    .into();
    let body_html = ammonia::Builder::default()
        .tags(allowed)
        .clean(raw_html)
        .to_string();
    let body_html = &body_html;
    let body_plain = args
        .get("body_plain")
        .and_then(|v| v.as_str())
        .unwrap_or(subject);

    // CSO-r3 #7：per-caller 每日寄信上限（防 leaked MCP key 內部/對外釣魚）
    check_notify_secretary_rate(user.id)?;

    // 系統管理員設定的執行秘書信箱（system_settings.iacuc_notify_emails）—
    // 同時作為「預設收件人」與「隱含白名單」（管理員設定，視為可信）。
    let iacuc_emails = sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT value FROM system_settings WHERE key = 'iacuc_notify_emails'",
    )
    .fetch_optional(&state.db)
    .await?
    .and_then(|v| v.as_str().map(str::to_string))
    .unwrap_or_default();

    // to_email 為選填：未提供時退回 iacuc_notify_emails
    let to_raw = match args
        .get("to_email")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
    {
        Some(v) => v.to_string(),
        None => iacuc_emails.clone(),
    };

    // 去重（保留首次出現的原始大小寫，以 lowercase 判重）+ 數量上限：
    // 即使收件人都合法，單次也不該放大成大量寄送（PR #506 CodeRabbit）。
    const MAX_RECIPIENTS: usize = 20;
    let mut seen = std::collections::HashSet::new();
    let recipients: Vec<String> = to_raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter(|s| seen.insert(s.to_lowercase()))
        .map(str::to_string)
        .collect();
    if recipients.is_empty() {
        return Err(AppError::BadRequest(
            "未指定收件人，且系統通知信箱（iacuc_notify_emails）尚未設定".to_string(),
        ));
    }
    if recipients.len() > MAX_RECIPIENTS {
        return Err(AppError::BadRequest(format!(
            "單次收件人不可超過 {MAX_RECIPIENTS} 人（收到 {}）",
            recipients.len()
        )));
    }

    // SEC (CSO R63-C13): 收件人白名單 — 防 MCP key 外流後被當成「公司名義」對外狂寄釣魚信。
    // 允許條件（任一）：
    //   (1) 內部使用者（users.email，is_active；本專案 soft-delete 即 is_active=false + 匿名化 email，
    //       故 is_active 已涵蓋軟刪除，無獨立 deleted_at 欄）
    //   (2) 外部白名單（system_settings.mcp_notify_external_allowlist，逗號分隔，僅存於 DB 不上 git）
    //   (3) 管理員設定的 iacuc_notify_emails（執行秘書，視為可信）
    // 規範：見 docs/security/mcp_notify_allowlist.local.md（gitignored 本機工作表）。
    let mut allowed_external: std::collections::HashSet<String> =
        sqlx::query_scalar::<_, serde_json::Value>(
            "SELECT value FROM system_settings WHERE key = 'mcp_notify_external_allowlist'",
        )
        .fetch_optional(&state.db)
        .await?
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    for e in iacuc_emails
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        allowed_external.insert(e.to_lowercase());
    }

    // 非白名單收件人 → 一次批次查詢驗證是否為內部 active 使用者（取代 per-recipient N+1）。
    let non_external: Vec<String> = recipients
        .iter()
        .map(|a| a.to_lowercase())
        .filter(|a| !allowed_external.contains(a))
        .collect();
    if !non_external.is_empty() {
        let active_internal: std::collections::HashSet<String> = sqlx::query_scalar::<_, String>(
            "SELECT lower(email) FROM users WHERE lower(email) = ANY($1) AND is_active",
        )
        .bind(&non_external)
        .fetch_all(&state.db)
        .await?
        .into_iter()
        .collect();
        for addr in &non_external {
            if !active_internal.contains(addr) {
                return Err(AppError::Forbidden(format!(
                    "收件人 {addr} 不在允許清單：僅可寄給內部使用者或 mcp_notify_external_allowlist 白名單"
                )));
            }
        }
    }

    let smtp = EmailService::resolve_smtp(&state.db, &state.config).await;
    for addr in &recipients {
        EmailService::send_email_smtp(
            &smtp,
            addr,
            "IACUC 執行秘書",
            subject,
            body_plain,
            body_html,
        )
        .await
        .map_err(|e| AppError::Internal(format!("SMTP 發送失敗（{addr}）：{e}")))?;
    }

    tracing::info!(
        caller_id = %user.id,
        to = to_raw,
        subject = subject,
        recipients = recipients.len(),
        "MCP notify_secretary sent"
    );

    Ok(serde_json::json!({
        "success": true,
        "recipients": recipients,
        "subject": subject
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    // CSO-r3 #7：notify_secretary per-caller 每日上限到頂後應拒絕。
    #[test]
    fn notify_secretary_rate_limit_caps_per_caller() {
        let uid = Uuid::new_v4(); // 全新 id → 隔離 process-global static
        for _ in 0..NOTIFY_SECRETARY_DAILY_LIMIT {
            check_notify_secretary_rate(uid).expect("配額內應通過");
        }
        let over = check_notify_secretary_rate(uid);
        // 每日上限是「配額用罄」(429 TooManyRequests)，非「權限不足」(403)。
        // 用 403 會被 security_response_logger 當成 IDOR 探測計數（MCP 工具機器反覆呼叫易誤觸）。
        assert!(
            matches!(over, Err(AppError::TooManyRequests(_))),
            "超過每日上限應回 TooManyRequests，實得：{over:?}"
        );
    }
}
