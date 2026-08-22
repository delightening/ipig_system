//! PI 帳號開通 + admin 核准寄送開通信 handlers。
//!
//! - `provision_pi_account`：建立者/SD/admin 為外部 PI 建帳號 + relink（不寄信）。
//! - `list_pi_account_invites` / `approve_send_pi_invite`：admin 檢視待核准開通信並核准寄送。

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::{ActorContext, CurrentUser},
    require_permission,
    services::{access, ProtocolService},
    AppState, Result,
};

#[derive(Serialize)]
pub struct ProvisionPiResponse {
    pub pi_user_id: Uuid,
    pub email: String,
    pub created_new_account: bool,
}

/// 開通計畫外部 PI 帳號 + relink（不寄信）。限可管理補登者（建立者 / 計劃負責人）
/// 或系統管理員。M4：admin 不受 import_pending 限制 —— 補登中漏開通、finalize 後
/// 仍可由 admin 補開，避免外部 PI 永遠無法取得帳號的死路。
pub async fn provision_pi_account(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProvisionPiResponse>> {
    if !(current_user.is_admin()
        || access::can_manage_import_pending(&state.db, id, &current_user).await?)
    {
        return Err(AppError::Forbidden(
            "無權開通此計畫的 PI 帳號（限建立者 / 計劃負責人 / 系統管理員）".into(),
        ));
    }
    let actor = ActorContext::User(current_user.clone());
    let (pi_user_id, email, created_new_account) =
        ProtocolService::provision_pi_account(&state.db, &actor, id).await?;
    Ok(Json(ProvisionPiResponse {
        pi_user_id,
        email,
        created_new_account,
    }))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PiAccountInviteItem {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub protocol_no: String,
    /// 官方 IACUC 編號（前端顯示用，對齊主計畫清單；匯入計畫於匯入時必填）。
    pub iacuc_no: Option<String>,
    pub protocol_title: String,
    pub pi_user_id: Uuid,
    pub pi_name: String,
    pub email: String,
    pub status: String,
    pub provisioned_by_name: Option<String>,
    pub provisioned_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct PiInviteQuery {
    pub status: Option<String>,
}

/// 列出 PI 開通信（admin only）。預設 pending。
pub async fn list_pi_account_invites(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(q): Query<PiInviteQuery>,
) -> Result<Json<Vec<PiAccountInviteItem>>> {
    if !current_user.is_admin() {
        return Err(AppError::Forbidden("僅系統管理員可檢視 PI 開通信".into()));
    }
    let status = q.status.unwrap_or_else(|| "pending".to_string());
    let items: Vec<PiAccountInviteItem> = sqlx::query_as(
        r#"SELECT i.id, i.protocol_id, p.protocol_no, p.iacuc_no, p.title as protocol_title,
                  i.pi_user_id, u.display_name as pi_name, i.email, i.status,
                  pb.display_name as provisioned_by_name, i.provisioned_at
           FROM pi_account_invites i
           JOIN protocols p ON i.protocol_id = p.id
           JOIN users u ON i.pi_user_id = u.id
           LEFT JOIN users pb ON i.provisioned_by = pb.id
           WHERE i.status = $1
           ORDER BY i.provisioned_at DESC"#,
    )
    .bind(&status)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(items))
}

/// 核准並寄送 PI 開通（設定密碼）信。需 aup.pi_invite.approve 權限（admin 短路具備）。
pub async fn approve_send_pi_invite(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(invite_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "aup.pi_invite.approve");
    let actor = ActorContext::User(current_user.clone());
    let email =
        ProtocolService::approve_send_pi_invite(&state.db, &actor, &state.config, invite_id)
            .await?;
    Ok(Json(serde_json::json!({ "ok": true, "email": email })))
}
