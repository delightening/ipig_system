use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{
        AdminRejectRequest, CreateDocumentRequest, DocumentListItem, DocumentQuery,
        DocumentWithLines, PoReceiptStatus, UpdateDocumentRequest,
    },
    require_permission,
    services::{DocumentService, NotificationService},
    AppError, AppState, Result,
};

use super::partner::DeleteQuery;

/// 建立文件
#[utoipa::path(
    post,
    path = "/api/v1/documents",
    request_body = CreateDocumentRequest,
    responses(
        (status = 200, description = "建立成功", body = DocumentWithLines),
        (status = 400, description = "驗證失敗"),
        (status = 401, description = "未認證"),
    ),
    tag = "單據管理",
    security(("bearer" = []))
)]
pub async fn create_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<DocumentWithLines>> {
    // 統一使用 erp.document.create 權限（所有單據類型共用）
    require_permission!(current_user, "erp.document.create");
    req.validate()?;

    // #61：銷貨單 / 銷貨出庫（SO / DO）的「該計畫 SD / 研究主持人 / admin」開立授權，
    // 已下沉至 DocumentService::create（service 層 single source of truth，含計畫層級 SD 判斷）。

    // Audit 已收進 service 層（DOC_CREATE，tx 內含 document + lines snapshot）
    let actor = ActorContext::User(current_user.clone());
    let document = DocumentService::create(&state.db, &actor, &req).await?;
    Ok(Json(document))
}

/// 列出所有文件
#[utoipa::path(
    get,
    path = "/api/v1/documents",
    params(DocumentQuery),
    responses(
        (status = 200, description = "單據清單", body = Vec<DocumentListItem>),
        (status = 401, description = "未認證"),
    ),
    tag = "單據管理",
    security(("bearer" = []))
)]
pub async fn list_documents(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<DocumentQuery>,
) -> Result<Json<Vec<DocumentListItem>>> {
    require_permission!(current_user, "erp.document.view");

    // 資安稽核 M-4：WM / admin 具全域監督權，其餘持有 view 權限的角色（如 PURCHASING）
    // 僅能列出自己建立的單據，避免 list 端點洩漏跨建立者的財務摘要（金額 / 夥伴 / 計畫號）。
    // 授權邊界與 get_document 的 check_access（creator / WM / admin）一致。
    let created_by_scope = if current_user.has_permission("erp.document.view_all") {
        None
    } else {
        Some(current_user.id)
    };

    let documents = DocumentService::list(&state.db, &query, created_by_scope).await?;
    Ok(Json(documents))
}

/// 取得單個文件
#[utoipa::path(
    get,
    path = "/api/v1/documents/{id}",
    params(("id" = Uuid, Path, description = "單據 ID")),
    responses(
        (status = 200, description = "單據詳細", body = DocumentWithLines),
        (status = 401, description = "未認證"),
        (status = 403, description = "無權存取"),
        (status = 404, description = "找不到單據"),
    ),
    tag = "單據管理",
    security(("bearer" = []))
)]
pub async fn get_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentWithLines>> {
    require_permission!(current_user, "erp.document.view");

    let document = DocumentService::get_by_id(&state.db, id).await?;
    DocumentService::check_access(&current_user, document.document.created_by)?;
    Ok(Json(document))
}

/// 更新文件
#[utoipa::path(
    put,
    path = "/api/v1/documents/{id}",
    params(("id" = Uuid, Path, description = "單據 ID")),
    request_body = UpdateDocumentRequest,
    responses(
        (status = 200, description = "更新成功", body = DocumentWithLines),
        (status = 400, description = "驗證失敗"),
        (status = 401, description = "未認證"),
        (status = 403, description = "無權存取"),
        (status = 404, description = "找不到單據"),
    ),
    tag = "單據管理",
    security(("bearer" = []))
)]
pub async fn update_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateDocumentRequest>,
) -> Result<Json<DocumentWithLines>> {
    require_permission!(current_user, "erp.document.edit");
    req.validate()?;

    let existing = DocumentService::get_by_id(&state.db, id).await?;
    DocumentService::check_access(&current_user, existing.document.created_by)?;

    // Audit 已收進 service 層（DOC_UPDATE，tx 內含 before/after snapshot）
    let actor = ActorContext::User(current_user.clone());
    let document = DocumentService::update(&state.db, &actor, id, &req).await?;
    Ok(Json(document))
}

/// 提交文件
#[utoipa::path(
    post,
    path = "/api/v1/documents/{id}/submit",
    params(("id" = Uuid, Path, description = "單據 ID")),
    responses(
        (status = 200, description = "提交成功", body = DocumentWithLines),
        (status = 401, description = "未認證"),
        (status = 403, description = "無權存取"),
        (status = 404, description = "找不到單據"),
    ),
    tag = "單據管理",
    security(("bearer" = []))
)]
pub async fn submit_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentWithLines>> {
    require_permission!(current_user, "erp.document.submit");

    let existing = DocumentService::get_by_id(&state.db, id).await?;
    DocumentService::check_access(&current_user, existing.document.created_by)?;

    // Audit 已收進 service 層（DOC_SUBMIT，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let document = DocumentService::submit(&state.db, &actor, id).await?;

    // 非同步通知 WAREHOUSE_MANAGER
    let db = state.db.clone();
    let doc_id = document.document.id;
    let doc_no = document.document.doc_no.clone();
    let doc_type = document.document.doc_type.prefix().to_string();
    let creator_name = document.created_by_name.clone();
    tokio::spawn(async move {
        let svc = NotificationService::new(db);
        if let Err(e) = svc
            .notify_document_submitted(doc_id, &doc_no, &doc_type, &creator_name)
            .await
        {
            tracing::warn!("發送單據提交通知失敗: {e}");
        }
    });

    Ok(Json(document))
}

/// 核准文件
#[utoipa::path(
    post,
    path = "/api/v1/documents/{id}/approve",
    params(("id" = Uuid, Path, description = "單據 ID")),
    responses(
        (status = 200, description = "核准成功", body = DocumentWithLines),
        (status = 401, description = "未認證"),
        (status = 403, description = "僅倉庫管理員可核准"),
        (status = 404, description = "找不到單據"),
    ),
    tag = "單據管理",
    security(("bearer" = []))
)]
pub async fn approve_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentWithLines>> {
    require_permission!(current_user, "erp.document.approve");

    // 僅 WAREHOUSE_MANAGER (倉庫管理員) 可核准單據
    if !current_user
        .roles
        .contains(&crate::constants::ROLE_WAREHOUSE_MANAGER.to_string())
    {
        return Err(AppError::Forbidden("僅倉庫管理員可核准單據".to_string()));
    }

    // Audit 已收進 service 層（DOC_APPROVE / DOC_WM_APPROVE + GRN 自動建立 DOC_CREATE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let document = DocumentService::approve(&state.db, &actor, id).await?;

    // 非同步通知建立者（已核准）
    let db = state.db.clone();
    let doc_id = document.document.id;
    let doc_no = document.document.doc_no.clone();
    let doc_type = document.document.doc_type.prefix().to_string();
    let creator_id = document.document.created_by;
    tokio::spawn(async move {
        let svc = NotificationService::new(db);
        if let Err(e) = svc
            .notify_document_decided(doc_id, &doc_no, &doc_type, true, creator_id)
            .await
        {
            tracing::warn!("發送單據決定通知失敗: {e}");
        }
    });

    Ok(Json(document))
}

/// ADMIN 最終核准（大金額 ADJ 調整單）
#[utoipa::path(
    post,
    path = "/api/v1/documents/{id}/admin-approve",
    params(("id" = Uuid, Path, description = "單據 ID")),
    responses(
        (status = 200, description = "ADMIN 核准成功", body = DocumentWithLines),
        (status = 401, description = "未認證"),
        (status = 403, description = "僅管理員可執行最終核准"),
        (status = 404, description = "找不到單據"),
    ),
    tag = "單據管理",
    security(("bearer" = []))
)]
pub async fn admin_approve_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentWithLines>> {
    // R97-1c：終審改由業務授權把關，不再看「是不是系統管理員」。
    //
    // ⚠️ 這裡**不再要求 `erp.document.approve`**：那是倉管階段的核准權，
    // 而終審者（負責人）刻意不具備倉管的操作權限（不建單、不送審）。
    // 若兩者都要求，DIRECTOR 會在到達終審閘之前就被擋下。
    require_permission!(current_user, "erp.document.final_approve");

    // Audit 已收進 service 層（DOC_ADMIN_APPROVE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let document = DocumentService::admin_approve(&state.db, &actor, id).await?;

    // 非同步通知建立者（已核准）
    let db = state.db.clone();
    let doc_id = document.document.id;
    let doc_no = document.document.doc_no.clone();
    let doc_type = document.document.doc_type.prefix().to_string();
    let creator_id = document.document.created_by;
    tokio::spawn(async move {
        let svc = NotificationService::new(db);
        if let Err(e) = svc
            .notify_document_decided(doc_id, &doc_no, &doc_type, true, creator_id)
            .await
        {
            tracing::warn!("發送單據決定通知失敗: {e}");
        }
    });

    Ok(Json(document))
}

/// R84-5 發起沖銷：對已核准單據建立一張待 ADMIN 核准的沖銷草稿
#[utoipa::path(
    post,
    path = "/api/v1/documents/{id}/reverse",
    params(("id" = Uuid, Path, description = "被沖銷的原單 ID")),
    responses(
        (status = 200, description = "沖銷單建立成功（待 ADMIN 核准）", body = DocumentWithLines),
        (status = 400, description = "原單非已核准 / 已被沖銷 / 沖銷單不可再沖銷"),
        (status = 401, description = "未認證"),
        (status = 403, description = "僅倉庫管理員或管理員可發起沖銷"),
        (status = 404, description = "找不到單據"),
    ),
    tag = "單據管理",
    security(("bearer" = []))
)]
pub async fn reverse_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentWithLines>> {
    require_permission!(current_user, "erp.document.approve");

    // Audit 已收進 service 層（DOC_REVERSAL_CREATE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let document = DocumentService::create_reversal(&state.db, &actor, id).await?;
    Ok(Json(document))
}

/// R84-5 ADMIN 最終核准沖銷單：執行庫存與會計的反向鏡射
#[utoipa::path(
    post,
    path = "/api/v1/documents/{id}/reverse-approve",
    params(("id" = Uuid, Path, description = "沖銷單 ID")),
    responses(
        (status = 200, description = "沖銷核准成功（庫存與會計已反向鏡射）", body = DocumentWithLines),
        (status = 400, description = "非沖銷單 / 狀態不符 / 庫存不足無法沖銷"),
        (status = 401, description = "未認證"),
        (status = 403, description = "僅管理員可核准，且發起人不得自行核准"),
        (status = 404, description = "找不到單據"),
    ),
    tag = "單據管理",
    security(("bearer" = []))
)]
pub async fn reverse_approve_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentWithLines>> {
    // R97-1c：沖銷核准同屬業務決策——它直接推翻已生效的庫存與帳務，
    // 風險高於大額調整單。理由與 `admin_approve_document` 相同，
    // 亦不再要求倉管階段的 `erp.document.approve`。
    require_permission!(current_user, "erp.document.reverse_approve");

    // Audit 已收進 service 層（DOC_REVERSAL_APPROVE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let document = DocumentService::approve_reversal(&state.db, &actor, id).await?;
    Ok(Json(document))
}

/// ADMIN 駁回（大金額 ADJ 調整單，退回草稿）
#[utoipa::path(
    post,
    path = "/api/v1/documents/{id}/admin-reject",
    params(("id" = Uuid, Path, description = "單據 ID")),
    request_body = AdminRejectRequest,
    responses(
        (status = 200, description = "ADMIN 駁回成功", body = DocumentWithLines),
        (status = 401, description = "未認證"),
        (status = 403, description = "僅管理員可駁回"),
        (status = 404, description = "找不到單據"),
    ),
    tag = "單據管理",
    security(("bearer" = []))
)]
pub async fn admin_reject_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<AdminRejectRequest>,
) -> Result<Json<DocumentWithLines>> {
    // R97-1c：駁回是終審的另一半（同一顆決策的反面），與核准同權限。
    // 刻意**不受**職務分離守衛限制——駁回只是把單退回草稿、不寫庫存也不過帳，
    // 屬「不作為」，且是這條流程唯一的逃生口（見 R97-1 前端註解）。
    require_permission!(current_user, "erp.document.final_approve");

    // Audit 已收進 service 層（DOC_ADMIN_REJECT，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let document = DocumentService::admin_reject(&state.db, &actor, id, &req.reason).await?;

    // 非同步通知建立者（已駁回）
    let db = state.db.clone();
    let doc_id = document.document.id;
    let doc_no = document.document.doc_no.clone();
    let doc_type = document.document.doc_type.prefix().to_string();
    let creator_id = document.document.created_by;
    tokio::spawn(async move {
        let svc = NotificationService::new(db);
        if let Err(e) = svc
            .notify_document_decided(doc_id, &doc_no, &doc_type, false, creator_id)
            .await
        {
            tracing::warn!("發送單據決定通知失敗: {e}");
        }
    });

    Ok(Json(document))
}

/// 取消文件
#[utoipa::path(
    post,
    path = "/api/v1/documents/{id}/cancel",
    params(("id" = Uuid, Path, description = "單據 ID")),
    responses(
        (status = 200, description = "取消成功", body = DocumentWithLines),
        (status = 401, description = "未認證"),
        (status = 403, description = "僅倉庫管理員可取消"),
        (status = 404, description = "找不到單據"),
    ),
    tag = "單據管理",
    security(("bearer" = []))
)]
pub async fn cancel_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentWithLines>> {
    require_permission!(current_user, "erp.document.cancel");

    // 僅 WAREHOUSE_MANAGER (倉庫管理員) 可取消/駁回單據
    if !current_user
        .roles
        .contains(&crate::constants::ROLE_WAREHOUSE_MANAGER.to_string())
    {
        return Err(AppError::Forbidden("僅倉庫管理員可取消單據".to_string()));
    }

    // Audit 已收進 service 層（DOC_CANCEL，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let document = DocumentService::cancel(&state.db, &actor, id).await?;

    // 非同步通知建立者（已駁回）
    let db = state.db.clone();
    let doc_id = document.document.id;
    let doc_no = document.document.doc_no.clone();
    let doc_type = document.document.doc_type.prefix().to_string();
    let creator_id = document.document.created_by;
    tokio::spawn(async move {
        let svc = NotificationService::new(db);
        if let Err(e) = svc
            .notify_document_decided(doc_id, &doc_no, &doc_type, false, creator_id)
            .await
        {
            tracing::warn!("發送單據決定通知失敗: {e}");
        }
    });

    Ok(Json(document))
}

/// 刪除文件
#[utoipa::path(
    delete,
    path = "/api/v1/documents/{id}",
    params(
        ("id" = Uuid, Path, description = "單據 ID"),
        DeleteQuery
    ),
    responses(
        (status = 200, description = "刪除成功"),
        (status = 401, description = "未認證"),
        (status = 403, description = "無權存取"),
        (status = 404, description = "找不到單據"),
    ),
    tag = "單據管理",
    security(("bearer" = []))
)]
pub async fn delete_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Query(params): Query<DeleteQuery>,
) -> Result<Json<()>> {
    require_permission!(current_user, "erp.document.delete");

    let is_hard = params.hard.unwrap_or(false) && current_user.is_admin();

    let existing = DocumentService::get_by_id(&state.db, id).await?;
    DocumentService::check_access(&current_user, existing.document.created_by)?;

    // Audit 已收進 service 層（DOC_DELETE / DOC_HARD_DELETE，tx 內含 before snapshot）
    let actor = ActorContext::User(current_user.clone());
    DocumentService::delete(&state.db, &actor, id, is_hard).await?;
    Ok(Json(()))
}

/// 取得採購單入庫狀態
#[utoipa::path(
    get,
    path = "/api/v1/documents/{id}/receipt-status",
    params(("id" = Uuid, Path, description = "採購單 ID")),
    responses(
        (status = 200, description = "入庫狀態", body = PoReceiptStatus),
        (status = 401, description = "未認證"),
        (status = 404, description = "找不到單據"),
    ),
    tag = "單據管理",
    security(("bearer" = []))
)]
pub async fn get_po_receipt_status(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<PoReceiptStatus>> {
    require_permission!(current_user, "erp.document.view");

    // SEC-AUDIT-006: 加入 ownership check，防止跨使用者查詢
    let existing = DocumentService::get_by_id(&state.db, id).await?;
    DocumentService::check_access(&current_user, existing.document.created_by)?;

    let status = DocumentService::get_po_receipt_status(&state.db, id).await?;
    Ok(Json(status))
}

/// 從採購單建立採購入庫單（部分入庫，預設帶剩餘數量）
#[utoipa::path(
    post,
    path = "/api/v1/documents/{id}/create-grn",
    params(("id" = Uuid, Path, description = "採購單 ID")),
    responses(
        (status = 200, description = "建立成功", body = DocumentWithLines),
        (status = 400, description = "採購單未核准 / 已完全入庫"),
        (status = 401, description = "未認證"),
        (status = 403, description = "無權限"),
        (status = 404, description = "找不到採購單"),
    ),
    tag = "單據管理",
    security(("bearer" = []))
)]
pub async fn create_grn_from_po(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentWithLines>> {
    require_permission!(current_user, "erp.document.create");

    // SEC-AUDIT-006: 比照 get_po_receipt_status，驗證使用者有權存取此 PO
    let existing = DocumentService::get_by_id(&state.db, id).await?;
    DocumentService::check_access(&current_user, existing.document.created_by)?;

    let grn = DocumentService::create_additional_grn(&state.db, id, current_user.id).await?;
    Ok(Json(grn))
}

/// 重新計算所有已核准 PO 的入庫狀態
#[utoipa::path(
    post,
    path = "/api/v1/documents/recalculate-receipt-status",
    responses(
        (status = 200, description = "重新計算完成", body = serde_json::Value),
        (status = 401, description = "未認證"),
        (status = 403, description = "無權限"),
    ),
    tag = "單據管理",
    security(("bearer" = []))
)]
pub async fn recalculate_receipt_status(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<serde_json::Value>> {
    if !current_user.is_admin() {
        return Err(AppError::Forbidden("僅系統管理員可執行批次重算".into()));
    }

    let count = DocumentService::recalculate_all_po_receipt_status(&state.db).await?;
    Ok(Json(serde_json::json!({ "updated": count })))
}
