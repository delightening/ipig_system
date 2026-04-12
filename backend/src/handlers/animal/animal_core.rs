// 動物管理 Handlers

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{
        Animal, AnimalListItem, AnimalQuery, AnimalStatsResponse, AnimalsByPen,
        BatchAssignRequest, CreateAnimalRequest, DeleteRequest, PaginatedResponse,
        UpdateAnimalRequest,
    },
    require_permission,
    services::{access, AnimalService, AuditService},
    AppError, AppState, Result,
};

/// 列出所有動物
#[utoipa::path(
    get,
    path = "/api/v1/animals",
    responses(
        (status = 200, description = "成功獲取動物列表", body = [AnimalListItem]),
        (status = 401, description = "未授權")
    ),
    tag = "動物管理",
    security(("bearer" = []))
)]
pub async fn list_animals(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<AnimalQuery>,
) -> Result<Json<PaginatedResponse<AnimalListItem>>> {
    let has_view_all = current_user.has_permission("animal.animal.view_all");
    let has_view_project = current_user.has_permission("animal.animal.view_project");

    if !has_view_all && !has_view_project {
        return Err(AppError::Forbidden("需要 animal.animal.view_all 或 animal.animal.view_project 權限".to_string()));
    }

    let mut result = AnimalService::list(&state.db, &query).await.map_err(|e| {
        tracing::error!(
            "list_animals failed: status={:?} breed={:?} keyword={:?} page={:?} per_page={:?} error={:?}",
            query.status,
            query.breed,
            query.keyword,
            query.page,
            query.per_page,
            e
        );
        e
    })?;

    if !has_view_all {
        let before_len = result.data.len();
        result.data.retain(|a| a.iacuc_no.is_some());
        let removed = before_len - result.data.len();
        result.total -= removed as i64;
        result.total_pages = (result.total as f64 / result.per_page as f64).ceil() as i64;
    }

    Ok(Json(result))
}

/// 取得動物狀態統計
#[utoipa::path(
    get,
    path = "/api/v1/animals/stats",
    responses(
        (status = 200, description = "動物狀態統計", body = AnimalStatsResponse),
        (status = 401, description = "未授權")
    ),
    tag = "動物管理",
    security(("bearer" = []))
)]
pub async fn get_animal_stats(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<AnimalStatsResponse>> {
    let has_view = current_user.has_permission("animal.animal.view_all")
        || current_user.has_permission("animal.animal.view_project");

    if !has_view {
        return Ok(Json(AnimalStatsResponse {
            status_counts: std::collections::HashMap::new(),
            pen_animals_count: 0,
            total: 0,
        }));
    }

    let stats = AnimalService::stats(&state.db).await?;
    Ok(Json(stats))
}

/// 按欄位列出所有動物
#[utoipa::path(
    get,
    path = "/api/v1/animals/by-pen",
    responses(
        (status = 200, description = "成功獲取按欄位分類的動物列表", body = [AnimalsByPen]),
        (status = 401, description = "未授權")
    ),
    tag = "動物管理",
    security(("bearer" = []))
)]
pub async fn list_animals_by_pen(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<AnimalsByPen>>> {
    require_permission!(current_user, "animal.animal.view_all");

    let animals = AnimalService::list_by_pen(&state.db).await?;
    Ok(Json(animals))
}

/// 取得單個動物的詳細資訊
#[utoipa::path(
    get,
    path = "/api/v1/animals/{id}",
    responses(
        (status = 200, description = "成功獲取動物詳情", body = Animal),
        (status = 404, description = "找不到動物"),
        (status = 401, description = "未授權")
    ),
    params(
        ("id" = Uuid, Path, description = "動物 ID")
    ),
    tag = "動物管理",
    security(("bearer" = []))
)]
pub async fn get_animal(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Animal>> {
    // C2: 驗證使用者對此動物所屬計畫的存取權限，防止 IDOR
    access::require_animal_access(&state.db, &current_user, id).await?;
    let animal = AnimalService::get_by_id(&state.db, id).await?;
    Ok(Json(animal))
}

/// 建立新動物
#[utoipa::path(
    post,
    path = "/api/v1/animals",
    request_body = CreateAnimalRequest,
    responses(
        (status = 201, description = "建立成功", body = Animal),
        (status = 400, description = "輸入資料驗證失敗"),
        (status = 401, description = "未授權")
    ),
    tag = "動物管理",
    security(("bearer" = []))
)]
pub async fn create_animal(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateAnimalRequest>,
) -> Result<Json<Animal>> {
    require_permission!(current_user, "animal.animal.create");

    tracing::debug!("Create animal request: ear_tag={}, breed={:?}, gender={:?}, entry_date={:?}, birth_date={:?}, entry_weight={:?}", 
        req.ear_tag, req.breed, req.gender, req.entry_date, req.birth_date, req.entry_weight);

    if let Err(validation_errors) = req.validate() {
        let error_messages: Vec<String> = validation_errors
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |e| {
                    let field_name: &str = match field.as_ref() {
                        "ear_tag" => "耳標",
                        "breed" => "品種",
                        "gender" => "性別",
                        "entry_date" => "入場日期",
                        "birth_date" => "出生日期",
                        "entry_weight" => "入場體重",
                        _ => field.as_ref(),
                    };
                    format!("{}: {}", field_name, e.message.as_ref().unwrap_or(&e.code))
                })
            })
            .collect();
        let error_msg = error_messages.join("; ");
        tracing::warn!("Validation failed: {}", error_msg);
        return Err(AppError::Validation(error_msg));
    }

    let animal = AnimalService::create(&state.db, &req, current_user.id).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ANIMAL",
        "CREATE",
        Some("animal"),
        Some(animal.id),
        Some(&animal.ear_tag),
        None,
        Some(serde_json::json!({
            "ear_tag": animal.ear_tag,
            "breed": format!("{:?}", req.breed),
            "gender": format!("{:?}", req.gender),
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("寫入 user_activity_logs 失敗 (ANIMAL_CREATE): {}", e);
    }

    Ok(Json(animal))
}

/// 更新動物資訊
#[utoipa::path(
    put,
    path = "/api/v1/animals/{id}",
    request_body = UpdateAnimalRequest,
    responses(
        (status = 200, description = "更新成功", body = Animal),
        (status = 404, description = "找不到動物"),
        (status = 400, description = "輸入資料驗證失敗"),
        (status = 401, description = "未授權")
    ),
    params(
        ("id" = Uuid, Path, description = "動物 ID")
    ),
    tag = "動物管理",
    security(("bearer" = []))
)]
pub async fn update_animal(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAnimalRequest>,
) -> Result<Json<Animal>> {
    require_permission!(current_user, "animal.animal.edit");
    access::require_animal_access(&state.db, &current_user, id).await?;
    req.validate()?;

    let (animal, iacuc_change) =
        AnimalService::update(&state.db, id, &req, current_user.id).await?;

    // 記錄 IACUC No. 變更審計事件（含 before/after 資料供時間軸顯示）
    if let Some(change) = &iacuc_change {
        let before_data = serde_json::json!({
            "iacuc_no": change.old_iacuc_no,
        });
        let after_data = serde_json::json!({
            "iacuc_no": change.new_iacuc_no,
        });
        if let Err(e) = AuditService::log_activity(
            &state.db,
            current_user.id,
            "ANIMAL",
            "IACUC_CHANGE",
            Some("animal"),
            Some(id),
            Some(&animal.ear_tag),
            Some(before_data),
            Some(after_data),
            None,
            None,
        )
        .await
        {
            tracing::error!("寫入 user_activity_logs 失敗 (IACUC_CHANGE): {}", e);
        }
    }

    // 記錄一般更新活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ANIMAL",
        "UPDATE",
        Some("animal"),
        Some(id),
        Some(&animal.ear_tag),
        None,
        None,
        None,
        None,
    )
    .await
    {
        tracing::error!("寫入 user_activity_logs 失敗 (ANIMAL_UPDATE): {}", e);
    }

    Ok(Json(animal))
}

/// 刪除動物（軟刪除 + 刪除原因）- GLP 合規
#[utoipa::path(
    delete,
    path = "/api/v1/animals/{id}",
    request_body = DeleteRequest,
    responses(
        (status = 200, description = "刪除成功"),
        (status = 404, description = "找不到動物"),
        (status = 401, description = "未授權")
    ),
    params(
        ("id" = Uuid, Path, description = "動物 ID")
    ),
    tag = "動物管理",
    security(("bearer" = []))
)]
pub async fn delete_animal(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<DeleteRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.animal.edit");
    access::require_animal_access(&state.db, &current_user, id).await?;
    req.validate()?;

    AnimalService::delete_with_reason(&state.db, id, &req.reason, current_user.id).await?;

    // 清理相關附件檔案
    if let Err(e) = crate::services::FileService::delete_by_entity(&state.db, "animal", &id).await {
        tracing::warn!("清理動物附件失敗 (non-fatal): {}", e);
    }

    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ANIMAL",
        "ANIMAL_DELETE",
        Some("animal"),
        Some(id),
        Some(&format!("動物 {} (原因: {})", id, req.reason)),
        None,
        Some(serde_json::json!({ "reason": req.reason })),
        None,
        None,
    )
    .await
    {
        tracing::error!("寫入 user_activity_logs 失敗 (ANIMAL_DELETE): {}", e);
    }

    Ok(Json(
        serde_json::json!({ "message": "Animal deleted successfully" }),
    ))
}

/// 批次分配動物的耳標
#[utoipa::path(
    post,
    path = "/api/v1/animals/batch/assign",
    request_body = BatchAssignRequest,
    responses(
        (status = 200, description = "批次分配成功", body = [Animal]),
        (status = 401, description = "未授權")
    ),
    tag = "動物管理",
    security(("bearer" = []))
)]
pub async fn batch_assign_animals(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<BatchAssignRequest>,
) -> Result<Json<Vec<Animal>>> {
    require_permission!(current_user, "animal.info.assign");

    let animals = AnimalService::batch_assign(&state.db, &req, current_user.id).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ANIMAL",
        "ANIMAL_BATCH_ASSIGN",
        Some("animal"),
        None,
        Some(&format!(
            "批次分配 {} 隻至 {}",
            animals.len(),
            &req.iacuc_no
        )),
        None,
        Some(serde_json::json!({
            "count": animals.len(),
            "iacuc_no": &req.iacuc_no,
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("寫入 user_activity_logs 失敗 (ANIMAL_BATCH_ASSIGN): {}", e);
    }

    Ok(Json(animals))
}

/// 標記動物為獸醫已讀
#[utoipa::path(
    post,
    path = "/api/v1/animals/{id}/vet-read",
    responses(
        (status = 200, description = "標記成功"),
        (status = 404, description = "找不到動物"),
        (status = 401, description = "未授權")
    ),
    params(
        ("id" = Uuid, Path, description = "動物 ID")
    ),
    tag = "動物管理",
    security(("bearer" = []))
)]
pub async fn mark_animal_vet_read(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.vet.read");

    AnimalService::mark_vet_read(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Marked as read" })))
}

/// 動物事件回傳結構
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct AnimalEvent {
    pub id: String,
    pub event_type: String,
    pub actor_name: Option<String>,
    pub before_data: Option<serde_json::Value>,
    pub after_data: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 取得動物的 IACUC 變更事件（用於時間軸顯示）
#[utoipa::path(
    get,
    path = "/api/v1/animals/{id}/events",
    responses(
        (status = 200, description = "成功獲取事件列表", body = [AnimalEvent]),
        (status = 404, description = "找不到動物"),
        (status = 401, description = "未授權")
    ),
    params(
        ("id" = Uuid, Path, description = "動物 ID")
    ),
    tag = "動物管理",
    security(("bearer" = []))
)]
pub async fn get_animal_events(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AnimalEvent>>> {
    type EventRow = (
        String,
        String,
        Option<String>,
        Option<serde_json::Value>,
        Option<serde_json::Value>,
        chrono::DateTime<chrono::Utc>,
    );
    let rows: Vec<EventRow> = sqlx::query_as(
        r#"
        SELECT
            id::text,
            event_type,
            actor_display_name,
            before_data,
            after_data,
            created_at
        FROM user_activity_logs
        WHERE entity_type = 'animal'
          AND entity_id = $1
          AND event_type = 'IACUC_CHANGE'
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    let events = rows
        .into_iter()
        .map(
            |(eid, event_type, actor_name, before_data, after_data, created_at)| AnimalEvent {
                id: eid,
                event_type,
                actor_name,
                before_data,
                after_data,
                created_at,
            },
        )
        .collect();

    Ok(Json(events))
}
