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

/// 撱箇??格?
pub async fn create_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<DocumentWithLines>> {
    // ?寞??格?憿?瑼Ｘ甈?
    let permission = format!("{}.create", req.doc_type.prefix().to_lowercase());
    require_permission!(current_user, &permission);
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let document = DocumentService::create(&state.db, &req, current_user.id).await?;
    Ok(Json(document))
}

/// ???格??”
pub async fn list_documents(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<DocumentQuery>,
) -> Result<Json<Vec<DocumentListItem>>> {
    // ?箸霈????
    require_permission!(current_user, "erp.document.view");
    
    let documents = DocumentService::list(&state.db, &query).await?;
    Ok(Json(documents))
}

/// ???桐??格?
pub async fn get_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentWithLines>> {
    require_permission!(current_user, "erp.document.view");
    
    let document = DocumentService::get_by_id(&state.db, id).await?;
    Ok(Json(document))
}

/// ?湔?格?
pub async fn update_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateDocumentRequest>,
) -> Result<Json<DocumentWithLines>> {
    require_permission!(current_user, "erp.document.edit");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let document = DocumentService::update(&state.db, id, &req).await?;
    Ok(Json(document))
}

/// ?祟
pub async fn submit_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentWithLines>> {
    require_permission!(current_user, "erp.document.submit");
    
    let document = DocumentService::submit(&state.db, id).await?;
    Ok(Json(document))
}

/// ?詨?
pub async fn approve_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentWithLines>> {
    require_permission!(current_user, "erp.document.approve");
    
    let document = DocumentService::approve(&state.db, id, current_user.id).await?;
    Ok(Json(document))
}

/// 雿誥
pub async fn cancel_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentWithLines>> {
    require_permission!(current_user, "erp.document.cancel");
    
    let document = DocumentService::cancel(&state.db, id).await?;
    Ok(Json(document))
}

