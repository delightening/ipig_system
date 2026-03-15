use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::Response,
    Extension, Json,
};
use axum::extract::Multipart;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{
        CreatePartnerRequest, GenerateCodeResponse, Partner, PartnerImportResult, PartnerQuery,
        SupplierCategory, UpdatePartnerRequest,
    },
    require_permission,
    services::{AuditService, PartnerService},
    AppError, AppState, Result,
};

/// 建立合作夥伴
#[utoipa::path(
    post,
    path = "/api/partners",
    request_body = CreatePartnerRequest,
    responses(
        (status = 200, description = "建立成功", body = Partner),
        (status = 400, description = "驗證失敗"),
        (status = 401, description = "未認證"),
    ),
    tag = "合作夥伴管理",
    security(("bearer" = []))
)]
pub async fn create_partner(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreatePartnerRequest>,
) -> Result<Json<Partner>> {
    require_permission!(current_user, "erp.partner.create");
    req.validate()?;
    
    let partner = PartnerService::create(&state.db, &req).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ERP", "PARTNER_CREATE",
        Some("partner"), Some(partner.id), Some(&partner.name),
        None,
        Some(serde_json::json!({
            "name": partner.name,
            "partner_type": format!("{:?}", partner.partner_type),
        })),
        None, None,
    ).await {
        tracing::error!("寫入審計日誌失敗 (PARTNER_CREATE): {}", e);
    }

    Ok(Json(partner))
}

/// 列出所有合作夥伴
#[utoipa::path(
    get,
    path = "/api/partners",
    params(PartnerQuery),
    responses(
        (status = 200, description = "夥伴清單", body = Vec<Partner>),
        (status = 401, description = "未認證"),
    ),
    tag = "合作夥伴管理",
    security(("bearer" = []))
)]
pub async fn list_partners(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<PartnerQuery>,
) -> Result<Json<Vec<Partner>>> {
    require_permission!(current_user, "erp.partner.view");
    
    let partners = PartnerService::list(&state.db, &query).await?;
    Ok(Json(partners))
}

/// 取得單個合作夥伴
#[utoipa::path(
    get,
    path = "/api/partners/{id}",
    params(("id" = Uuid, Path, description = "夥伴 ID")),
    responses(
        (status = 200, description = "夥伴詳細", body = Partner),
        (status = 401, description = "未認證"),
        (status = 404, description = "找不到夥伴"),
    ),
    tag = "合作夥伴管理",
    security(("bearer" = []))
)]
pub async fn get_partner(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Partner>> {
    require_permission!(current_user, "erp.partner.view");
    
    let partner = PartnerService::get_by_id(&state.db, id).await?;
    Ok(Json(partner))
}

/// 更新合作夥伴
#[utoipa::path(
    put,
    path = "/api/partners/{id}",
    params(("id" = Uuid, Path, description = "夥伴 ID")),
    request_body = UpdatePartnerRequest,
    responses(
        (status = 200, description = "更新成功", body = Partner),
        (status = 400, description = "驗證失敗"),
        (status = 401, description = "未認證"),
        (status = 404, description = "找不到夥伴"),
    ),
    tag = "合作夥伴管理",
    security(("bearer" = []))
)]
pub async fn update_partner(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePartnerRequest>,
) -> Result<Json<Partner>> {
    require_permission!(current_user, "erp.partner.edit");
    req.validate()?;
    
    let partner = PartnerService::update(&state.db, id, &req).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ERP", "PARTNER_UPDATE",
        Some("partner"), Some(id), Some(&partner.name),
        None, None, None, None,
    ).await {
        tracing::error!("寫入審計日誌失敗 (PARTNER_UPDATE): {}", e);
    }

    Ok(Json(partner))
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct DeleteQuery {
    pub hard: Option<bool>,
}

/// 刪除合作夥伴
#[utoipa::path(
    delete,
    path = "/api/partners/{id}",
    params(
        ("id" = Uuid, Path, description = "夥伴 ID"),
        DeleteQuery
    ),
    responses(
        (status = 200, description = "刪除成功"),
        (status = 401, description = "未認證"),
        (status = 404, description = "找不到夥伴"),
    ),
    tag = "合作夥伴管理",
    security(("bearer" = []))
)]
pub async fn delete_partner(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Query(params): Query<DeleteQuery>,
    Json(body): Json<DeleteQuery>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "erp.partner.delete");
    
    let is_hard = (params.hard.unwrap_or(false) || body.hard.unwrap_or(false)) && current_user.is_admin();

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ERP", if is_hard { "PARTNER_HARD_DELETE" } else { "PARTNER_DELETE" },
        Some("partner"), Some(id), None,
        None, None, None, None,
    ).await {
        tracing::error!("寫入審計日誌失敗 (PARTNER_DELETE): {}", e);
    }

    PartnerService::delete(&state.db, id, is_hard).await?;
    Ok(Json(serde_json::json!({ 
        "message": if is_hard { "Partner permanently deleted" } else { "Partner deleted successfully" } 
    })))
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct GenerateCodeQuery {
    pub partner_type: Option<String>,
    pub category: Option<String>,
}

/// 生成供應商代碼
#[utoipa::path(
    get,
    path = "/api/partners/generate-code",
    params(GenerateCodeQuery),
    responses(
        (status = 200, description = "代碼", body = GenerateCodeResponse),
        (status = 400, description = "驗證失敗"),
        (status = 401, description = "未認證"),
    ),
    tag = "合作夥伴管理",
    security(("bearer" = []))
)]
pub async fn generate_partner_code(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<GenerateCodeQuery>,
) -> Result<Json<GenerateCodeResponse>> {
    require_permission!(current_user, "erp.partner.create");
    
    let partner_type = match params.partner_type.as_deref() {
        Some("supplier") => crate::models::PartnerType::Supplier,
        Some("customer") => crate::models::PartnerType::Customer,
        None => crate::models::PartnerType::Supplier,
        _ => return Err(AppError::Validation("Invalid partner_type. Must be supplier or customer".to_string())),
    };
    
    let category = if partner_type == crate::models::PartnerType::Supplier {
        match params.category.as_deref() {
            Some("drug") => Some(SupplierCategory::Drug),
            Some("consumable") => Some(SupplierCategory::Consumable),
            Some("feed") => Some(SupplierCategory::Feed),
            Some("equipment") => Some(SupplierCategory::Equipment),
            _ => return Err(AppError::Validation("Invalid category for supplier. Must be one of: drug, consumable, feed, equipment".to_string())),
        }
    } else {
        None
    };
    
    let code = PartnerService::generate_code(&state.db, partner_type, category).await?;
    Ok(Json(GenerateCodeResponse { code }))
}

/// 匯入夥伴（供應商/客戶）
#[utoipa::path(
    post,
    path = "/api/partners/import",
    responses(
        (status = 200, description = "匯入結果 (multipart/form-data, field file: CSV/Excel)", body = PartnerImportResult),
        (status = 400, description = "驗證失敗或檔案格式錯誤"),
        (status = 401, description = "未認證"),
    ),
    tag = "合作夥伴管理",
    security(("bearer" = []))
)]
pub async fn import_partners(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    mut multipart: Multipart,
) -> Result<Json<PartnerImportResult>> {
    require_permission!(current_user, "erp.partner.create");

    let (file_data, file_name) = parse_partner_import_file(&mut multipart).await?;
    if file_data.len() > 10 * 1024 * 1024 {
        return Err(AppError::Validation("檔案大小不能超過 10MB".to_string()));
    }

    let result = PartnerService::import_partners(&state.db, &file_data, &file_name).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ERP",
        "PARTNER_IMPORT",
        Some("partner"),
        None,
        Some(&format!(
            "匯入夥伴: {} (成功: {}, 失敗: {})",
            file_name, result.success_count, result.error_count
        )),
        None,
        Some(serde_json::json!({
            "file_name": file_name,
            "success_count": result.success_count,
            "error_count": result.error_count
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("寫入審計日誌失敗 (PARTNER_IMPORT): {}", e);
    }

    Ok(Json(result))
}

/// 下載夥伴匯入模板
#[utoipa::path(
    get,
    path = "/api/partners/import/template",
    responses(
        (status = 200, description = "Excel 模板檔案"),
        (status = 401, description = "未認證"),
    ),
    tag = "合作夥伴管理",
    security(("bearer" = []))
)]
pub async fn download_partner_import_template() -> Result<Response> {
    let data = PartnerService::generate_import_template()
        .map_err(|e| AppError::Internal(format!("產生模板失敗: {}", e)))?;
    let filename = "partner_import_template.xlsx";
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(Body::from(data))
        .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))
}

async fn parse_partner_import_file(multipart: &mut Multipart) -> Result<(Vec<u8>, String)> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name = String::from("unknown");
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("解析檔案欄位失敗: {}", e)))?
    {
        if field.name() == Some("file") {
            file_name = field
                .file_name()
                .map(String::from)
                .unwrap_or_else(|| "unknown".to_string());
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::Validation(format!("讀取檔案資料失敗: {}", e)))?;
            file_data = Some(data.to_vec());
        }
    }
    let file_data = file_data.ok_or_else(|| AppError::Validation("未找到檔案".to_string()))?;
    Ok((file_data, file_name))
}
