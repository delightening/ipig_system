// 動物管理 Handlers

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{
        Animal, AnimalListItem, AnimalQuery, AnimalStatsResponse, AnimalsByPen,
        AvailablePigListResponse, AvailablePigQuery, BatchAssignRequest, CreateAnimalRequest,
        DeleteRequest, PaginatedResponse, UpdateAnimalRequest,
    },
    require_permission,
    services::{access, animal_excel_export, AnimalService},
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
        return Err(AppError::Forbidden(
            "需要 animal.animal.view_all 或 animal.animal.view_project 權限".to_string(),
        ));
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
            pen_counts_by_building: std::collections::HashMap::new(),
            total: 0,
        }));
    }

    let stats = AnimalService::stats(&state.db).await?;
    Ok(Json(stats))
}

/// R47 — 可用豬隻快速查詢（庫存盤點）
///
/// 回 JSON `AvailablePigListResponse`；若 query `export=xlsx` 則回 Excel 檔案下載。
/// 權限：沿用 `animal.animal.view_all` 或 `animal.animal.view_project`（與 list_animals 同）。
/// 範圍：只有 view_project（無 view_all）者收斂為「已指派計畫」豬隻，與 list_animals 一致，
/// 避免外部 PI 列舉/匯出全院未指派庫存豬（#386）。
#[utoipa::path(
    get,
    path = "/api/v1/animals/available",
    params(AvailablePigQuery),
    responses(
        (status = 200, description = "可用豬隻列表 + 統計", body = AvailablePigListResponse),
        (status = 401, description = "未授權"),
        (status = 403, description = "無權限")
    ),
    tag = "動物管理",
    security(("bearer" = []))
)]
pub async fn list_available_pigs(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<AvailablePigQuery>,
) -> Result<axum::response::Response> {
    use axum::{
        body::Body,
        http::{header, StatusCode},
        response::IntoResponse,
    };

    let has_view_all = current_user.has_permission("animal.animal.view_all");
    let has_view = has_view_all || current_user.has_permission("animal.animal.view_project");
    if !has_view {
        return Err(AppError::Forbidden(
            "需要 animal.animal.view_all 或 animal.animal.view_project 權限".to_string(),
        ));
    }
    // 只有 view_project（無 view_all）→ 收斂為已指派計畫豬隻（#386）。
    let restrict_to_assigned = !has_view_all;

    let result =
        AnimalService::list_available_pigs(&state.db, &query, restrict_to_assigned).await?;

    if query.export.as_deref() == Some("xlsx") {
        let bytes = animal_excel_export::build_available_pigs_xlsx(&result.animals)?;
        let filename = format!(
            "available_pigs_{}.xlsx",
            chrono::Local::now().format("%Y%m%d_%H%M%S")
        );
        let response = (
            StatusCode::OK,
            [
                (
                    header::CONTENT_TYPE,
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                ),
                (
                    header::CONTENT_DISPOSITION,
                    &format!("attachment; filename=\"{filename}\""),
                ),
            ],
            Body::from(bytes),
        )
            .into_response();
        return Ok(response);
    }

    Ok(Json(result).into_response())
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
    access::require_animal_read_access(&state.db, &current_user, id).await?;
    let animal = AnimalService::get_by_id(&state.db, id).await?;
    Ok(Json(animal))
}

/// 取得動物客戶/委託資訊（狀態相依，供匯入體重驗證資訊行）
#[utoipa::path(
    get,
    path = "/api/v1/animals/{id}/client-info",
    params(("id" = Uuid, Path, description = "動物 ID")),
    responses(
        (status = 200, body = crate::models::AnimalClientInfoResponse),
        (status = 401),
        (status = 404)
    ),
    tag = "動物管理",
    security(("bearer" = []))
)]
pub async fn get_animal_client_info(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<crate::models::AnimalClientInfoResponse>> {
    // IDOR：驗證使用者對此動物所屬計畫的存取權限
    access::require_animal_read_access(&state.db, &current_user, id).await?;
    let info = AnimalService::get_client_info(&state.db, id).await?;
    Ok(Json(info))
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

    // Audit 已收進 service 層（ANIMAL_CREATE with create_only diff，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let animal = AnimalService::create(&state.db, &actor, &req).await?;
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

    // Audit 已收進 service 層（IACUC_CHANGE + ANIMAL_UPDATE 皆在 tx 內）
    let actor = ActorContext::User(current_user.clone());
    let (animal, _iacuc_change) = AnimalService::update(&state.db, &actor, id, &req).await?;
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
    require_permission!(current_user, "animal.animal.delete");
    let scope =
        access::Scoped::<access::AnimalWrite>::authorize(&state.db, &current_user, id).await?;
    req.validate()?;

    // Audit 已收進 service 層（ANIMAL_DELETE with delete_only diff，tx 內）
    let actor = ActorContext::User(current_user.clone());
    AnimalService::delete_with_reason(&state.db, &actor, scope, &req.reason).await?;

    // 清理相關附件檔案
    if let Err(e) = crate::services::FileService::delete_by_entity(&state.db, "animal", &id).await {
        tracing::warn!("清理動物附件失敗 (non-fatal): {}", e);
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

    // Audit 已收進 service 層（N+1：per-row ANIMAL_ASSIGN + summary ANIMAL_BATCH_ASSIGN，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let animals = AnimalService::batch_assign(&state.db, &actor, &req).await?;
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
    // R75-2: 補動物存取守衛（原僅權限檢查，可對任意 id 標記；vet 具 view_all 仍放行，另獲 404 存在檢查）
    let scope =
        access::Scoped::<access::AnimalRead>::authorize(&state.db, &current_user, id).await?;

    AnimalService::mark_vet_read(&state.db, scope).await?;
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
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AnimalEvent>>> {
    // SEC-IDOR: v2 審計發現 — 動物活動事件需驗證計畫歸屬
    access::require_animal_read_access(&state.db, &current_user, id).await?;
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
