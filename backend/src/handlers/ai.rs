//! AI 接口 Handlers
//!
//! 提供兩組路由：
//! 1. 管理端（需管理員 JWT 認證）：API key CRUD
//! 2. AI 端（需 AI API key 認證）：資料查詢

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::{ai_auth::AiCaller, ActorContext, CurrentUser},
    models::ai::*,
    require_permission,
    services::AiService,
    AppError, AppState, Result,
};

// ════════════════════════════════════════
// 管理端 Handlers（JWT 認證）
// ════════════════════════════════════════

/// 建立 AI API key（僅管理員）
#[utoipa::path(
    post,
    path = "/api/v1/ai/admin/keys",
    request_body = CreateAiApiKeyRequest,
    responses(
        (status = 201, description = "API key 建立成功", body = CreateAiApiKeyResponse),
        (status = 403, description = "權限不足")
    ),
    tag = "AI 接口管理",
    security(("bearer" = []))
)]
pub async fn create_ai_api_key(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<CreateAiApiKeyRequest>,
) -> Result<(axum::http::StatusCode, Json<CreateAiApiKeyResponse>)> {
    require_permission!(user, "system.admin");

    // SECURITY audit 已收進 service 層（AI_API_KEY_CREATE，tx 內）
    let actor = ActorContext::User(user.clone());
    let resp = AiService::create_api_key(&state.db, &actor, &req).await?;

    Ok((axum::http::StatusCode::CREATED, Json(resp)))
}

/// 列出所有 AI API keys
#[utoipa::path(
    get,
    path = "/api/v1/ai/admin/keys",
    responses(
        (status = 200, description = "API key 清單", body = [AiApiKeyInfo])
    ),
    tag = "AI 接口管理",
    security(("bearer" = []))
)]
pub async fn list_ai_api_keys(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> Result<Json<Vec<AiApiKeyInfo>>> {
    require_permission!(user, "system.admin");
    let keys = AiService::list_api_keys(&state.db).await?;
    Ok(Json(keys))
}

/// 停用 / 啟用 AI API key
#[utoipa::path(
    put,
    path = "/api/v1/ai/admin/keys/{id}/toggle",
    params(("id" = Uuid, Path, description = "API key ID")),
    responses(
        (status = 200, description = "狀態更新成功")
    ),
    tag = "AI 接口管理",
    security(("bearer" = []))
)]
pub async fn toggle_ai_api_key(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(body): Json<ToggleActiveRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(user, "system.admin");

    // SECURITY audit 已收進 service 層（AI_API_KEY_ENABLE/DISABLE，tx 內）
    let actor = ActorContext::User(user.clone());
    AiService::toggle_api_key(&state.db, &actor, id, body.is_active).await?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// 刪除 AI API key
#[utoipa::path(
    delete,
    path = "/api/v1/ai/admin/keys/{id}",
    params(("id" = Uuid, Path, description = "API key ID")),
    responses(
        (status = 200, description = "刪除成功")
    ),
    tag = "AI 接口管理",
    security(("bearer" = []))
)]
pub async fn delete_ai_api_key(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(user, "system.admin");

    // SECURITY audit 已收進 service 層（AI_API_KEY_DELETE，tx 內）
    let actor = ActorContext::User(user.clone());
    AiService::delete_api_key(&state.db, &actor, id).await?;

    Ok(Json(serde_json::json!({ "success": true })))
}

// ════════════════════════════════════════
// AI 端 Handlers（API Key 認證）
// ════════════════════════════════════════

/// 取得系統概覽（AI 進入系統的第一個端點）
#[utoipa::path(
    get,
    path = "/api/v1/ai/overview",
    responses(
        (status = 200, description = "系統概覽", body = AiSystemOverview)
    ),
    tag = "AI 資料查詢",
    security(("ai_api_key" = []))
)]
pub async fn ai_system_overview(
    State(state): State<AppState>,
    Extension(caller): Extension<AiCaller>,
) -> Result<Json<AiSystemOverview>> {
    let start = std::time::Instant::now();
    let overview = AiService::get_system_overview(&state.db).await?;
    let duration = start.elapsed().as_millis() as i32;

    log_ai_query(&state, &caller, "/api/v1/ai/overview", "GET", None, 200, duration).await;

    Ok(Json(overview))
}

/// 取得 API schema（告訴 AI 可以查什麼）
#[utoipa::path(
    get,
    path = "/api/v1/ai/schema",
    responses(
        (status = 200, description = "API schema", body = AiSchemaResponse)
    ),
    tag = "AI 資料查詢",
    security(("ai_api_key" = []))
)]
pub async fn ai_schema(
    State(state): State<AppState>,
    Extension(caller): Extension<AiCaller>,
) -> Result<Json<AiSchemaResponse>> {
    log_ai_query(&state, &caller, "/api/v1/ai/schema", "GET", None, 200, 0).await;
    Ok(Json(AiService::get_schema()))
}

/// 執行資料查詢
#[utoipa::path(
    post,
    path = "/api/v1/ai/query",
    request_body = AiQueryRequest,
    responses(
        (status = 200, description = "查詢結果", body = AiQueryResponse),
        (status = 400, description = "查詢參數錯誤")
    ),
    tag = "AI 資料查詢",
    security(("ai_api_key" = []))
)]
pub async fn ai_query(
    State(state): State<AppState>,
    Extension(caller): Extension<AiCaller>,
    Json(req): Json<AiQueryRequest>,
) -> Result<Json<AiQueryResponse>> {
    // 檢查 scope 權限
    let required_scope = match &req.domain {
        AiQueryDomain::Animals
        | AiQueryDomain::Observations
        | AiQueryDomain::Surgeries
        | AiQueryDomain::Weights => "animal.read",
        AiQueryDomain::Protocols => "protocol.read",
        AiQueryDomain::Facilities => "facility.read",
        AiQueryDomain::Stock => "stock.read",
        AiQueryDomain::HrSummary => "hr.read",
    };

    if !caller.has_scope(required_scope) {
        return Err(AppError::Forbidden(format!(
            "API key 缺少權限：{}",
            required_scope
        )));
    }

    let start = std::time::Instant::now();
    let result = AiService::execute_query(&state.db, &req).await;
    let duration = start.elapsed().as_millis() as i32;

    let status = if result.is_ok() { 200 } else { 400 };
    let summary = serde_json::json!({
        "domain": format!("{:?}", req.domain),
        "page": req.page,
        "per_page": req.per_page,
    });

    log_ai_query(
        &state,
        &caller,
        "/api/v1/ai/query",
        "POST",
        Some(&summary),
        status,
        duration,
    )
    .await;

    Ok(Json(result?))
}

// ── Helper ──

/// toggle 請求
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct ToggleActiveRequest {
    pub is_active: bool,
}

async fn log_ai_query(
    state: &AppState,
    caller: &AiCaller,
    endpoint: &str,
    method: &str,
    summary: Option<&serde_json::Value>,
    status: i16,
    duration_ms: i32,
) {
    let db = state.db.clone();
    let api_key_id = caller.api_key_id;
    let endpoint = endpoint.to_string();
    let method = method.to_string();
    let summary = summary.cloned();

    tokio::spawn(async move {
        AiService::log_query(
            &db,
            api_key_id,
            &endpoint,
            &method,
            summary.as_ref(),
            status,
            duration_ms,
            None,
        )
        .await;
    });
}
