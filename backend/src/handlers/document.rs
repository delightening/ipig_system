use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{
        CreateDocumentRequest, DocumentListItem, DocumentQuery, DocumentWithLines,
        UpdateDocumentRequest,
    },
    require_permission,
    services::DocumentService,
    AppError, AppState, Result,
};

/// 建立單據
pub async fn create_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<DocumentWithLines>> {
    // 根據單據類型檢查權限
    let permission = format!("{}.create", req.doc_type.prefix().to_lowercase());
    require_permission!(current_user, &permission);
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let document = DocumentService::create(&state.db, &req, current_user.id).await?;
    Ok(Json(document))
}

/// 取得單據列表
pub async fn list_documents(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<DocumentQuery>,
) -> Result<Json<Vec<DocumentListItem>>> {
    // 基本讀取權限
    require_permission!(current_user, "document.read");
    
    let documents = DocumentService::list(&state.db, &query).await?;
    Ok(Json(documents))
}

/// 取得單一單據
pub async fn get_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentWithLines>> {
    require_permission!(current_user, "document.read");
    
    let document = DocumentService::get_by_id(&state.db, id).await?;
    Ok(Json(document))
}

/// 更新單據
pub async fn update_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateDocumentRequest>,
) -> Result<Json<DocumentWithLines>> {
    require_permission!(current_user, "document.update");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let document = DocumentService::update(&state.db, id, &req).await?;
    Ok(Json(document))
}

/// 送審
pub async fn submit_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentWithLines>> {
    require_permission!(current_user, "document.submit");
    
    let document = DocumentService::submit(&state.db, id).await?;
    Ok(Json(document))
}

/// 核准
pub async fn approve_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentWithLines>> {
    require_permission!(current_user, "document.approve");
    
    let document = DocumentService::approve(&state.db, id, current_user.id).await?;
    Ok(Json(document))
}

/// 作廢
pub async fn cancel_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentWithLines>> {
    require_permission!(current_user, "document.cancel");
    
    let document = DocumentService::cancel(&state.db, id).await?;
    Ok(Json(document))
}
