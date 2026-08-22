use axum::extract::Multipart;
use axum::{
    body::Body,
    extract::{ConnectInfo, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::Response,
    Extension, Json,
};
use std::net::SocketAddr;
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::{extract_real_ip_with_trust, ActorContext, CurrentUser},
    models::{
        CreateWarehouseRequest, UpdateWarehouseRequest, Warehouse, WarehouseImportResult,
        WarehouseQuery, WarehouseReportData, WarehouseTreeNode,
    },
    require_permission,
    services::{audit::RequestContext, pdf_service_client::DocxRenderFormat, WarehouseService},
    AppError, AppState, Result,
};

/// 倉庫現況報表 PDF 檔名固定後綴（與 frontend `WarehouseReportPage.tsx` 對齊）。
const WAREHOUSE_REPORT_FILENAME_SUFFIX: &str = "_倉庫現況報表.pdf";

/// 取 User-Agent header（缺漏或非 ASCII 時為 None，與 auth handler 一致）。
fn user_agent_of(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
}

/// 建立倉庫
#[utoipa::path(post, path = "/api/v1/warehouses", request_body = CreateWarehouseRequest, responses((status = 200, description = "建立成功", body = Warehouse)), tag = "倉儲管理", security(("bearer" = [])))]
pub async fn create_warehouse(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<CreateWarehouseRequest>,
) -> Result<Json<Warehouse>> {
    require_permission!(current_user, "erp.warehouse.create");
    req.validate()?;

    let ip = extract_real_ip_with_trust(&headers, &addr, state.config.trust_proxy_headers);
    let ctx = RequestContext {
        ip_address: Some(&ip),
        user_agent: user_agent_of(&headers),
    };
    let actor = ActorContext::User(current_user.clone());
    let warehouse = WarehouseService::create(&state.db, &actor, &req, Some(ctx)).await?;

    Ok(Json(warehouse))
}

/// 取得倉庫樹（含貨架，供庫存查詢樹狀選單）
#[utoipa::path(get, path = "/api/v1/warehouses/with-shelves", responses((status = 200, description = "倉庫樹含貨架", body = Vec<WarehouseTreeNode>)), tag = "倉儲管理", security(("bearer" = [])))]
pub async fn list_warehouses_with_shelves(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<WarehouseTreeNode>>> {
    require_permission!(current_user, "erp.warehouse.view");

    let tree = WarehouseService::list_with_shelves(&state.db).await?;
    Ok(Json(tree))
}

/// 列出所有倉庫
#[utoipa::path(get, path = "/api/v1/warehouses", responses((status = 200, description = "倉庫清單", body = Vec<Warehouse>)), tag = "倉儲管理", security(("bearer" = [])))]
pub async fn list_warehouses(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<WarehouseQuery>,
) -> Result<Json<Vec<Warehouse>>> {
    require_permission!(current_user, "erp.warehouse.view");

    let warehouses = WarehouseService::list(&state.db, &query).await?;
    Ok(Json(warehouses))
}

/// 取得單個倉庫
#[utoipa::path(get, path = "/api/v1/warehouses/{id}", params(("id" = Uuid, Path, description = "倉庫 ID")), responses((status = 200, description = "倉庫資訊", body = Warehouse)), tag = "倉儲管理", security(("bearer" = [])))]
pub async fn get_warehouse(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Warehouse>> {
    require_permission!(current_user, "erp.warehouse.view");

    let warehouse = WarehouseService::get_by_id(&state.db, id).await?;
    Ok(Json(warehouse))
}

/// 更新倉庫
#[utoipa::path(put, path = "/api/v1/warehouses/{id}", params(("id" = Uuid, Path, description = "倉庫 ID")), request_body = UpdateWarehouseRequest, responses((status = 200, description = "更新成功", body = Warehouse)), tag = "倉儲管理", security(("bearer" = [])))]
pub async fn update_warehouse(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<UpdateWarehouseRequest>,
) -> Result<Json<Warehouse>> {
    require_permission!(current_user, "erp.warehouse.edit");
    req.validate()?;

    let ip = extract_real_ip_with_trust(&headers, &addr, state.config.trust_proxy_headers);
    let ctx = RequestContext {
        ip_address: Some(&ip),
        user_agent: user_agent_of(&headers),
    };
    let actor = ActorContext::User(current_user.clone());
    let warehouse = WarehouseService::update(&state.db, &actor, id, &req, Some(ctx)).await?;

    Ok(Json(warehouse))
}

/// 刪除倉庫（軟刪除）
/// DELETE /warehouses/:id 與 POST /warehouses/:id/delete 均支援，避免部分代理/tunnel 對 DELETE 回傳 405
#[utoipa::path(delete, path = "/api/v1/warehouses/{id}", params(("id" = Uuid, Path, description = "倉庫 ID")), responses((status = 200, description = "刪除成功")), tag = "倉儲管理", security(("bearer" = [])))]
pub async fn delete_warehouse(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "erp.warehouse.delete");

    let ip = extract_real_ip_with_trust(&headers, &addr, state.config.trust_proxy_headers);
    let ctx = RequestContext {
        ip_address: Some(&ip),
        user_agent: user_agent_of(&headers),
    };
    let actor = ActorContext::User(current_user.clone());
    WarehouseService::delete(&state.db, &actor, id, Some(ctx)).await?;
    Ok(Json(
        serde_json::json!({ "message": "Warehouse deleted successfully" }),
    ))
}

/// 取得倉庫現況報表資料
#[utoipa::path(get, path = "/api/v1/warehouses/{id}/report", params(("id" = Uuid, Path, description = "倉庫 ID")), responses((status = 200, description = "倉庫現況報表", body = WarehouseReportData)), tag = "倉儲管理", security(("bearer" = [])))]
pub async fn get_warehouse_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<WarehouseReportData>> {
    require_permission!(current_user, "erp.warehouse.view");
    let report = WarehouseService::get_report_data(&state.db, id).await?;
    Ok(Json(report))
}

/// R35-4: PDF 匯出 query — `?inline=1` 走 `Content-Disposition: inline`（瀏覽器
/// 內預覽，PDF viewer 用 filename 當分頁標題）；省略或其他值仍預設 `attachment`
/// （下載行為不變）。
#[derive(Debug, serde::Deserialize, Default)]
pub struct ReportPdfQuery {
    #[serde(default)]
    pub inline: Option<u8>,
}

/// 匯出倉庫現況報表 PDF
#[utoipa::path(get, path = "/api/v1/warehouses/{id}/report/pdf", params(("id" = Uuid, Path, description = "倉庫 ID")), responses((status = 200, description = "PDF 檔案")), tag = "倉儲管理", security(("bearer" = [])))]
pub async fn export_warehouse_report_pdf(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Query(q): Query<ReportPdfQuery>,
) -> Result<Response> {
    require_permission!(current_user, "erp.warehouse.view");
    let report = WarehouseService::get_report_data(&state.db, id).await?;
    // R32-A8g：改走 docx → Word COM PDF（取代 services/pdf/service.rs printpdf 1236 行手刻）。
    let exporter_name =
        crate::repositories::user::find_user_display_name_by_id(&state.db, current_user.id)
            .await?
            .unwrap_or_else(|| current_user.email.clone());
    let mut payload = serde_json::to_value(&report)
        .map_err(|e| AppError::Internal(format!("Failed to serialize warehouse report: {}", e)))?;
    let obj = payload.as_object_mut().ok_or_else(|| {
        AppError::Internal(
            "Serialized warehouse report not a JSON object — cannot inject exporter_name".into(),
        )
    })?;
    obj.insert(
        "exporter_name".to_string(),
        serde_json::Value::String(exporter_name),
    );
    let (pdf_bytes, renderer) = state
        .pdf_service
        .render_warehouse_from_report_data(&payload, DocxRenderFormat::Pdf)
        .await?;
    // 用 name（更可讀）；trim 後仍空才 fallback code（與 frontend
    // `WarehouseReportPage.tsx::fileLabel` 的 `name?.trim() || code` 對齊）
    let label = if report.warehouse.name.trim().is_empty() {
        report.warehouse.code.as_str()
    } else {
        report.warehouse.name.trim()
    };
    let filename = format!("{}{}", label, WAREHOUSE_REPORT_FILENAME_SUFFIX);
    crate::utils::http::file_response_with_renderer(
        pdf_bytes,
        "application/pdf",
        &filename,
        q.inline == Some(1),
        renderer,
    )
    .map_err(AppError::Internal)
}

/// 匯入倉庫（CSV 或 Excel）
pub async fn import_warehouses(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    mut multipart: Multipart,
) -> Result<Json<WarehouseImportResult>> {
    require_permission!(current_user, "erp.warehouse.create");

    let (file_data, file_name) = parse_warehouse_import_file(&mut multipart).await?;
    if file_data.len() > 10 * 1024 * 1024 {
        return Err(AppError::Validation("檔案大小不能超過 10MB".to_string()));
    }

    let actor = ActorContext::User(current_user.clone());
    let result =
        WarehouseService::import_warehouses(&state.db, &actor, &file_data, &file_name).await?;

    // service 層 N+1 audit（per-row WAREHOUSE_CREATE + summary WAREHOUSE_IMPORT）
    Ok(Json(result))
}

/// 下載倉庫匯入模板
pub async fn download_warehouse_import_template() -> Result<Response> {
    let data = WarehouseService::generate_import_template()
        .map_err(|e| AppError::Internal(format!("產生模板失敗: {}", e)))?;
    let filename = "warehouse_import_template.xlsx";
    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header(
            header::CONTENT_DISPOSITION,
            crate::utils::http::content_disposition_header(filename),
        )
        .body(Body::from(data))
        .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))
}

async fn parse_warehouse_import_file(multipart: &mut Multipart) -> Result<(Vec<u8>, String)> {
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
