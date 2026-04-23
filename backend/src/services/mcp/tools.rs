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

fn uuid_arg(args: &Value, key: &str) -> Result<Uuid> {
    let s = str_arg(args, key)?;
    // 支援 UUID 或計畫書編號（AUP-YYYY-NNN）
    if let Ok(id) = s.parse::<Uuid>() {
        return Ok(id);
    }
    Err(AppError::BadRequest(format!("Invalid UUID for argument: {key}")))
}

// ── 1. list_protocols ──

pub async fn list_protocols(
    state: &AppState,
    _user: &CurrentUser,
    args: &Value,
) -> Result<Value> {
    let status_filter = args.get("status").and_then(|v| v.as_str());
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(20).min(100);

    type ProtocolRow = (Uuid, String, Option<String>, String, String, Option<String>, DateTime<Utc>);
    let rows: Vec<ProtocolRow> = if let Some(status) = status_filter {
        sqlx::query_as(
            r#"
            SELECT p.id, p.protocol_no, p.iacuc_no, p.title, p.status::text,
                   u.name AS pi_name, p.updated_at
            FROM protocols p
            LEFT JOIN users u ON p.pi_user_id = u.id
            WHERE p.status::text = $1 AND p.status != 'DELETED'
            ORDER BY p.updated_at DESC
            LIMIT $2
            "#,
        )
        .bind(status)
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as(
            r#"
            SELECT p.id, p.protocol_no, p.iacuc_no, p.title, p.status::text,
                   u.name AS pi_name, p.updated_at
            FROM protocols p
            LEFT JOIN users u ON p.pi_user_id = u.id
            WHERE p.status != 'DELETED'
            ORDER BY p.updated_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    };

    let list: Vec<Value> = rows
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
        .collect();

    Ok(serde_json::json!({ "protocols": list, "count": list.len() }))
}

// ── 2. read_protocol（含稽核日誌） ──

pub async fn read_protocol(
    state: &AppState,
    user: &CurrentUser,
    args: &Value,
) -> Result<Value> {
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

    // PI 資訊
    let pi_info: Option<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT name, email, organization FROM users WHERE id = $1",
    )
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

    let (pi_name, pi_email, pi_org) = pi_info.unwrap_or_default();

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
        return Err(AppError::BadRequest("flag_type 必須為 needs_attention / concern / suggestion".to_string()));
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
    .bind(if flag_type == "needs_attention" { 1i32 } else { 0i32 })
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
    let protocol = sqlx::query_as::<_, crate::models::Protocol>(
        "SELECT * FROM protocols WHERE id = $1",
    )
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
    let flags = args.get("flags")
        .and_then(|v| v.as_array())
        .ok_or_else(|| AppError::BadRequest("flags 為必填陣列".to_string()))?;

    let mut created = 0usize;
    for flag in flags {
        let section = flag.get("section").and_then(|v| v.as_str()).unwrap_or("—");
        let message = flag.get("message").and_then(|v| v.as_str()).unwrap_or("");
        let suggestion = flag.get("suggestion").and_then(|v| v.as_str()).unwrap_or("");
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
        remark: args.get("additional_note").and_then(|v| v.as_str()).map(String::from),
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
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(5).min(20);

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
        .map(|(no, title, status)| serde_json::json!({
            "protocol_no": no,
            "title": title,
            "status": status
        }))
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

    let is_admin = user.roles.iter().any(|r| r == crate::constants::ROLE_SYSTEM_ADMIN);
    if !is_assigned && !is_admin {
        return Err(AppError::Forbidden("您未被指派審查此計畫書".to_string()));
    }

    let items = args.get("items")
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

pub async fn notify_secretary(
    state: &AppState,
    user: &CurrentUser,
    args: &Value,
) -> Result<Value> {
    let subject = str_arg(args, "subject")?;
    let body_html = str_arg(args, "body_html")?;
    let body_plain = args.get("body_plain").and_then(|v| v.as_str()).unwrap_or(subject);

    // to_email 為選填：未提供時從 system_settings.iacuc_notify_emails 讀取
    let to_raw = match args.get("to_email").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
        Some(v) => v.to_string(),
        None => {
            sqlx::query_scalar::<_, serde_json::Value>(
                "SELECT value FROM system_settings WHERE key = 'iacuc_notify_emails'",
            )
            .fetch_optional(&state.db)
            .await?
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default()
        }
    };

    let recipients: Vec<&str> = to_raw.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
    if recipients.is_empty() {
        return Err(AppError::BadRequest(
            "未指定收件人，且系統通知信箱（iacuc_notify_emails）尚未設定".to_string(),
        ));
    }

    let smtp = EmailService::resolve_smtp(&state.db, &state.config).await;
    for addr in &recipients {
        EmailService::send_email_smtp(&smtp, addr, "IACUC 執行秘書", subject, body_plain, body_html)
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
