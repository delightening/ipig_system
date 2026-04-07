//! 全庫 IDXF 匯出 / 匯入 Handler
//!
//! GET /admin/data-export
//! POST /admin/data-import

use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::Response,
    Extension, Json,
};
use axum::extract::Multipart;
use serde::Deserialize;

use crate::{
    middleware::CurrentUser,
    require_permission,
    services::{
        export_full_database, import_idxf, AuditService, ExportFormat, ExportParams, ImportMode,
        ImportResult,
    },
    startup::ensure_admin_user_after_import,
    time,
    AppError, AppState, Result,
};

use super::user::require_reauth_token;

#[derive(Debug, Deserialize)]
pub struct DataExportQuery {
    #[serde(rename = "include_audit")]
    pub include_audit: Option<bool>,
    #[serde(default)]
    pub format: Option<String>,
}


/// 一鍵匯出全庫
/// GET /admin/data-export?include_audit=false&format=json|zip
pub async fn full_database_export(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    headers: HeaderMap,
    Query(params): Query<DataExportQuery>,
) -> Result<Response> {
    require_permission!(current_user, "admin.data.export");
    // C5: 全庫匯出屬高度敏感操作，強制要求 reauth（二次密碼確認）
    require_reauth_token(&headers, &state, &current_user)?;

    let include_audit = params.include_audit.unwrap_or(false);
    let format = match params.format.as_deref() {
        Some("zip") => ExportFormat::Zip,
        _ => ExportFormat::Json,
    };
    let export_params = ExportParams {
        include_audit,
        format,
    };

    let bytes = export_full_database(&state.db, export_params).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ADMIN",
        "DATA_EXPORT",
        None,
        None,
        Some(&format!("全庫匯出 (include_audit={}, format={:?})", include_audit, format)),
        None,
        Some(serde_json::json!({ "include_audit": include_audit, "format": format!("{:?}", format), "bytes": bytes.len() })),
        None,
        None,
    )
    .await { tracing::error!("審計日誌寫入失敗: {e}"); }

    let (ext, content_type) = match format {
        ExportFormat::Zip => ("zip", "application/zip"),
        ExportFormat::Json => ("json", "application/json; charset=utf-8"),
    };
    let filename = format!("ipig_export_{}.{}", time::now_taiwan().format("%Y%m%d_%H%M%S"), ext);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            crate::utils::http::content_disposition_header(&filename),
        )
        .header(header::CONTENT_LENGTH, bytes.len())
        .body(Body::from(bytes))
        .map_err(|e| AppError::Internal(format!("Build response: {e}")))
}

/// 匯入 IDXF JSON 或 Zip
/// POST /admin/data-import (multipart, field: file)
/// 遇重複則取代（ON CONFLICT DO UPDATE）
pub async fn full_database_import(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    mut multipart: Multipart,
) -> Result<Json<ImportResult>> {
    require_permission!(current_user, "admin.data.import");

    let (file_data, _file_name) = parse_import_file(&mut multipart).await?;
    let result = import_idxf(&state.db, &file_data, ImportMode::Append).await?;

    // 匯入後確保 admin 可登入（重設密碼為 ADMIN_INITIAL_PASSWORD）
    let _ = ensure_admin_user_after_import(&state.db, &state.config).await;

    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ADMIN",
        "DATA_IMPORT",
        None,
        None,
        Some(&format!(
            "全庫匯入: {} 表, {} 筆新增, {} 筆略過",
            result.tables_processed, result.rows_inserted, result.rows_skipped
        )),
        None,
        Some(serde_json::json!({
            "tables_processed": result.tables_processed,
            "rows_inserted": result.rows_inserted,
            "rows_skipped": result.rows_skipped,
            "errors": result.errors,
            "skipped_details": result.skipped_details
        })),
        None,
        None,
    )
    .await { tracing::error!("審計日誌寫入失敗: {e}"); }

    Ok(Json(result))
}

async fn parse_import_file(multipart: &mut Multipart) -> Result<(Vec<u8>, String)> {
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
    if file_data.len() > crate::constants::FILE_MAX_DATA_IMPORT {
        return Err(AppError::Validation(format!(
            "檔案大小不能超過 {} MB",
            crate::constants::FILE_MAX_DATA_IMPORT / 1024 / 1024
        )));
    }
    Ok((file_data, file_name))
}
