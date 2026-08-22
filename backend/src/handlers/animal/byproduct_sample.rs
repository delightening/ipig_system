// R53-4: 廢棄物再利用紀錄 Handlers
//
// URL 設計：euthanasia_id 走 path（建立時不放 body），單筆操作用全域 id。
// - POST   /euthanasia/:euth_id/byproduct-samples       建立
// - GET    /euthanasia/:euth_id/byproduct-samples       列某 euthanasia 下
// - GET    /animals/:animal_id/byproduct-samples        列某動物
// - GET    /protocols/:protocol_id/byproduct-samples    列某計畫
// - GET    /byproduct-samples/:id                       取單筆
// - PATCH  /byproduct-samples/:id                       更新
// - DELETE /byproduct-samples/:id                       軟刪除
//
// Permission：
//   *.view 控所有 GET；*.write 控 POST / PATCH / DELETE
// PI / GUEST 無 view 權限 → 看不到列表 / 單筆（R53-6 audit blacklist 配套）

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    constants::ROLE_SYSTEM_ADMIN,
    middleware::{ActorContext, CurrentUser},
    models::{CreateNotificationRequest, NotificationType},
    require_permission,
    services::{
        ByproductSample, ByproductSampleService, CreateByproductSampleRequest, NotificationService,
        UpdateByproductSampleRequest,
    },
    AppState, Result,
};

/// POST /euthanasia/:euth_id/byproduct-samples body — 不含 euthanasia_id（path 提供）
/// POST body —— **IDOR 守衛**：不接受 `animal_id` / `source_protocol_id`，
/// service 從 path 的 `euthanasia_id` 內部推導。
///
/// R53-14：requester 改成雙層（org + contact），billing 三欄（special_equipment_used /
/// work_started_at / work_ended_at）為老闆指定的月結報表欄位。
#[derive(Debug, Deserialize)]
pub struct CreateByproductSampleHttpRequest {
    pub sampled_at: DateTime<Utc>,
    pub sample_content: String,
    pub requester_user_id: Option<Uuid>,
    pub requester_org_name: Option<String>,
    pub requester_contact_name: Option<String>,
    pub notes: Option<String>,
    pub special_equipment_used: Option<String>,
    pub work_started_at: Option<DateTime<Utc>>,
    pub work_ended_at: Option<DateTime<Utc>>,
}

/// 建立廢棄物再利用紀錄
pub async fn create_byproduct_sample(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(euthanasia_id): Path<Uuid>,
    Json(body): Json<CreateByproductSampleHttpRequest>,
) -> Result<Json<ByproductSample>> {
    require_permission!(current_user, "animal.byproduct_sample.write");

    let req = http_body_to_create_req(body, Some(euthanasia_id));
    let actor = ActorContext::User(current_user.clone());
    let row = ByproductSampleService::create(&state.db, &actor, &req).await?;
    Ok(Json(row))
}

/// 共用：把 HTTP body 轉成 service 的 CreateByproductSampleRequest。
fn http_body_to_create_req(
    body: CreateByproductSampleHttpRequest,
    euthanasia_id: Option<Uuid>,
) -> CreateByproductSampleRequest {
    CreateByproductSampleRequest {
        euthanasia_id,
        sampled_at: body.sampled_at,
        sample_content: body.sample_content,
        requester_user_id: body.requester_user_id,
        requester_org_name: body.requester_org_name,
        requester_contact_name: body.requester_contact_name,
        notes: body.notes,
        special_equipment_used: body.special_equipment_used,
        work_started_at: body.work_started_at,
        work_ended_at: body.work_ended_at,
    }
}

/// 建立廢棄物再利用紀錄 — **計劃內犧牲路徑**（#445）。
/// path 用 `animal_id`（不需安樂死單）；service 驗動物已犧牲 + 從 animal 推導來源計畫。
pub async fn create_byproduct_sample_for_animal(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(body): Json<CreateByproductSampleHttpRequest>,
) -> Result<Json<ByproductSample>> {
    require_permission!(current_user, "animal.byproduct_sample.write");

    let req = http_body_to_create_req(body, None);
    let actor = ActorContext::User(current_user.clone());
    let row = ByproductSampleService::create_for_animal(&state.db, &actor, animal_id, &req).await?;
    Ok(Json(row))
}

/// 列某安樂死下的所有採樣紀錄
pub async fn list_byproduct_samples_by_euthanasia(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(euthanasia_id): Path<Uuid>,
) -> Result<Json<Vec<ByproductSample>>> {
    require_permission!(current_user, "animal.byproduct_sample.view");
    let rows = ByproductSampleService::list_by_euthanasia(&state.db, euthanasia_id).await?;
    Ok(Json(rows))
}

/// 列某動物的所有採樣紀錄（跨多 euthanasia 通常只一筆）
pub async fn list_byproduct_samples_by_animal(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<Vec<ByproductSample>>> {
    require_permission!(current_user, "animal.byproduct_sample.view");
    let rows = ByproductSampleService::list_by_animal(&state.db, animal_id).await?;
    Ok(Json(rows))
}

/// 列某來源計畫的所有採樣紀錄
pub async fn list_byproduct_samples_by_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(protocol_id): Path<Uuid>,
) -> Result<Json<Vec<ByproductSample>>> {
    require_permission!(current_user, "animal.byproduct_sample.view");
    let rows = ByproductSampleService::list_by_protocol(&state.db, protocol_id).await?;
    Ok(Json(rows))
}

/// 取單筆採樣紀錄
pub async fn get_byproduct_sample(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ByproductSample>> {
    require_permission!(current_user, "animal.byproduct_sample.view");
    let row = ByproductSampleService::get(&state.db, id).await?;
    Ok(Json(row))
}

/// 更新採樣紀錄
pub async fn update_byproduct_sample(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateByproductSampleRequest>,
) -> Result<Json<ByproductSample>> {
    require_permission!(current_user, "animal.byproduct_sample.write");

    let actor = ActorContext::User(current_user.clone());
    let row = ByproductSampleService::update(&state.db, &actor, id, &req).await?;

    notify_admins_byproduct_change(state.db.clone(), current_user.id, row.id, "更新");
    Ok(Json(row))
}

/// 軟刪除採樣紀錄
pub async fn delete_byproduct_sample(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.byproduct_sample.write");

    let actor = ActorContext::User(current_user.clone());
    ByproductSampleService::delete(&state.db, &actor, id).await?;

    notify_admins_byproduct_change(state.db.clone(), current_user.id, id, "刪除");
    Ok(Json(serde_json::json!({
        "message": "Byproduct sample deleted successfully"
    })))
}

fn notify_admins_byproduct_change(
    db: sqlx::PgPool,
    operator_id: Uuid,
    sample_id: Uuid,
    action: &'static str,
) {
    tokio::spawn(async move {
        let svc = NotificationService::new(db);
        let admins = match svc.get_users_by_role(ROLE_SYSTEM_ADMIN).await {
            Ok(a) => a,
            Err(e) => {
                tracing::warn!("查詢 admin 失敗: {e}");
                return;
            }
        };
        for (user_id, _, _) in &admins {
            if *user_id == operator_id {
                continue;
            }
            if let Err(e) = svc
                .create_notification(CreateNotificationRequest {
                    user_id: *user_id,
                    notification_type: NotificationType::SystemAlert,
                    title: format!("採樣紀錄{action}通知"),
                    content: Some(format!("廢棄物再利用採樣紀錄已{action}（ID: {sample_id}）")),
                    related_entity_type: Some("byproduct_sample".to_string()),
                    related_entity_id: Some(sample_id),
                })
                .await
            {
                tracing::warn!("建立採樣{action}通知失敗: {e}");
            }
        }
    });
}
