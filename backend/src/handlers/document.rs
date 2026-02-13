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
    services::{AuditService, DocumentService, NotificationService},
    AppError, AppState, Result,
};

/// 建立文件
pub async fn create_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<DocumentWithLines>> {
    // 統一使用 erp.document.create 權限（所有單據類型共用）
    require_permission!(current_user, "erp.document.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let document = DocumentService::create(&state.db, &req, current_user.id).await?;

    // 審計日誌
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ERP", "DOC_CREATE",
        Some("document"), Some(document.document.id), Some(&document.document.doc_no),
        None,
        Some(serde_json::json!({
            "doc_no": document.document.doc_no,
            "doc_type": format!("{:?}", document.document.doc_type),
        })),
        None, None,
    ).await {
        tracing::error!("寫入審計日誌失敗 (DOC_CREATE): {}", e);
    }

    Ok(Json(document))
}

/// 列出所有文件
pub async fn list_documents(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<DocumentQuery>,
) -> Result<Json<Vec<DocumentListItem>>> {
    require_permission!(current_user, "erp.document.view");
    
    let documents = DocumentService::list(&state.db, &query).await?;
    Ok(Json(documents))
}

/// 取得單個文件
pub async fn get_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentWithLines>> {
    require_permission!(current_user, "erp.document.view");
    
    let document = DocumentService::get_by_id(&state.db, id).await?;
    Ok(Json(document))
}

/// 更新文件
pub async fn update_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateDocumentRequest>,
) -> Result<Json<DocumentWithLines>> {
    require_permission!(current_user, "erp.document.edit");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let document = DocumentService::update(&state.db, id, &req).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ERP", "DOC_UPDATE",
        Some("document"), Some(id), Some(&document.document.doc_no),
        None, None, None, None,
    ).await {
        tracing::error!("寫入審計日誌失敗 (DOC_UPDATE): {}", e);
    }

    Ok(Json(document))
}

/// 提交文件
pub async fn submit_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentWithLines>> {
    require_permission!(current_user, "erp.document.submit");
    
    let document = DocumentService::submit(&state.db, id).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ERP", "DOC_SUBMIT",
        Some("document"), Some(id), Some(&document.document.doc_no),
        None,
        Some(serde_json::json!({ "status": "submitted" })),
        None, None,
    ).await {
        tracing::error!("寫入審計日誌失敗 (DOC_SUBMIT): {}", e);
    }

    // 非同步通知 WAREHOUSE_MANAGER
    let db = state.db.clone();
    let doc_id = document.document.id;
    let doc_no = document.document.doc_no.clone();
    let doc_type = document.document.doc_type.prefix().to_string();
    let creator_name = document.created_by_name.clone();
    tokio::spawn(async move {
        let svc = NotificationService::new(db);
        let _ = svc.notify_document_submitted(
            doc_id, &doc_no, &doc_type, &creator_name,
        ).await;
    });

    Ok(Json(document))
}

/// 核准文件
pub async fn approve_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentWithLines>> {
    require_permission!(current_user, "erp.document.approve");
    
    // 僅 WAREHOUSE_MANAGER (倉庫管理員) 可核准單據
    if !current_user.roles.contains(&"WAREHOUSE_MANAGER".to_string()) {
        return Err(AppError::Forbidden("僅倉庫管理員可核准單據".to_string()));
    }
    
    let document = DocumentService::approve(&state.db, id, current_user.id).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ERP", "DOC_APPROVE",
        Some("document"), Some(id), Some(&document.document.doc_no),
        None,
        Some(serde_json::json!({ "status": "approved" })),
        None, None,
    ).await {
        tracing::error!("寫入審計日誌失敗 (DOC_APPROVE): {}", e);
    }

    // 非同步通知建立者（已核准）
    let db = state.db.clone();
    let doc_id = document.document.id;
    let doc_no = document.document.doc_no.clone();
    let doc_type = document.document.doc_type.prefix().to_string();
    let creator_id = document.document.created_by;
    tokio::spawn(async move {
        let svc = NotificationService::new(db);
        let _ = svc.notify_document_decided(
            doc_id, &doc_no, &doc_type, true, creator_id,
        ).await;
    });

    Ok(Json(document))
}

/// 取消文件
pub async fn cancel_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentWithLines>> {
    require_permission!(current_user, "erp.document.cancel");
    
    // 僅 WAREHOUSE_MANAGER (倉庫管理員) 可取消/駁回單據
    if !current_user.roles.contains(&"WAREHOUSE_MANAGER".to_string()) {
        return Err(AppError::Forbidden("僅倉庫管理員可取消單據".to_string()));
    }
    
    let document = DocumentService::cancel(&state.db, id).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ERP", "DOC_CANCEL",
        Some("document"), Some(id), Some(&document.document.doc_no),
        None,
        Some(serde_json::json!({ "status": "cancelled" })),
        None, None,
    ).await {
        tracing::error!("寫入審計日誌失敗 (DOC_CANCEL): {}", e);
    }

    // 非同步通知建立者（已駁回）
    let db = state.db.clone();
    let doc_id = document.document.id;
    let doc_no = document.document.doc_no.clone();
    let doc_type = document.document.doc_type.prefix().to_string();
    let creator_id = document.document.created_by;
    tokio::spawn(async move {
        let svc = NotificationService::new(db);
        let _ = svc.notify_document_decided(
            doc_id, &doc_no, &doc_type, false, creator_id,
        ).await;
    });

    Ok(Json(document))
}

/// 刪除文件
pub async fn delete_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>> {
    require_permission!(current_user, "erp.document.delete");

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ERP", "DOC_DELETE",
        Some("document"), Some(id), None,
        None, None, None, None,
    ).await {
        tracing::error!("寫入審計日誌失敗 (DOC_DELETE): {}", e);
    }
    
    DocumentService::delete(&state.db, id).await?;
    Ok(Json(()))
}
