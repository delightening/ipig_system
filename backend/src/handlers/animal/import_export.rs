// 匯入匯出 Handlers

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::Response,
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{AnimalImportBatch, ExportRequest, ImportResult},
    require_permission,
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AnimalImportExportService, AnimalMedicalService, AnimalService, AuditService, PdfService,
    },
    AppError, AppState, Result,
};
use axum::extract::Multipart;

/// 匯出動物的醫療資料
pub async fn export_animal_medical_data(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<ExportRequest>,
) -> Result<Response> {
    require_permission!(current_user, "animal.export.medical");

    let data = AnimalMedicalService::get_animal_medical_data(&state.db, animal_id).await?;
    let _record = AnimalMedicalService::create_export_record(
        &state.db,
        Some(animal_id),
        None,
        req.export_type,
        req.format,
        Some("pending"),
        current_user.id,
    )
    .await?;

    let export_display = match AnimalService::get_by_id(&state.db, animal_id).await {
        Ok(animal) => {
            let iacuc = animal.iacuc_no.as_deref().unwrap_or("未指派");
            format!("[{}] {}", iacuc, animal.ear_tag)
        }
        _ => format!("匯出醫療資料 (animal: {})", animal_id),
    };

    let actor = ActorContext::User(current_user.clone());
    if let Err(e) = AuditService::log_activity_oneshot(
        &state.db,
        &actor,
        ActivityLogEntry {
            event_category: "ANIMAL",
            event_type: "EXPORT_MEDICAL",
            entity: Some(AuditEntity::new("animal", animal_id, &export_display)),
            data_diff: None,
            request_context: None,
        },
    )
    .await
    {
        tracing::error!("寫入 user_activity_logs 失敗 (MEDICAL_EXPORT): {}", e);
    }

    match req.format {
        crate::models::ExportFormat::Pdf => {
            let pdf_bytes = PdfService::generate_medical_pdf(&data)?;
            let filename = format!("medical_record_{}.pdf", animal_id);
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/pdf")
                .header(
                    header::CONTENT_DISPOSITION,
                    crate::utils::http::content_disposition_header(&filename),
                )
                .body(Body::from(pdf_bytes))
                .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))?)
        }
        _ => Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::to_vec(&serde_json::json!({
                    "data": data, "format": req.format, "export_type": req.export_type,
                }))
                .map_err(|e| AppError::Internal(format!("serialize error: {e}")))?,
            ))
            .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))?),
    }
}

/// 匯出專案的醫療資料
pub async fn export_project_medical_data(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(iacuc_no): Path<String>,
    Json(req): Json<ExportRequest>,
) -> Result<Response> {
    require_permission!(current_user, "animal.export.medical");

    let data = AnimalMedicalService::get_project_medical_data(&state.db, &iacuc_no).await?;
    let _record = AnimalMedicalService::create_export_record(
        &state.db,
        None,
        Some(&iacuc_no),
        req.export_type,
        req.format,
        Some("pending"),
        current_user.id,
    )
    .await?;

    match req.format {
        crate::models::ExportFormat::Pdf => {
            let pdf_bytes = PdfService::generate_project_medical_pdf(&iacuc_no, &data)?;
            let filename = format!("project_medical_{}.pdf", iacuc_no);
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/pdf")
                .header(
                    header::CONTENT_DISPOSITION,
                    crate::utils::http::content_disposition_header(&filename),
                )
                .body(Body::from(pdf_bytes))
                .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))?)
        }
        _ => Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::to_vec(&serde_json::json!({
                    "data": data, "format": req.format, "export_type": req.export_type,
                }))
                .map_err(|e| AppError::Internal(format!("serialize error: {e}")))?,
            ))
            .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))?),
    }
}

/// 列出所有匯入批次
pub async fn list_import_batches(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<AnimalImportBatch>>> {
    require_permission!(current_user, "animal.animal.import");
    let batches = AnimalMedicalService::list_import_batches(&state.db, 50).await?;
    Ok(Json(batches))
}

/// 下載動物基礎資料匯入範本
pub async fn download_basic_import_template(
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Response> {
    require_permission!(current_user, "animal.animal.import");
    let format = params.get("format").map(|s| s.as_str()).unwrap_or("xlsx");
    let (data, filename, content_type) = if format == "csv" {
        (
            AnimalImportExportService::generate_basic_import_template_csv()?,
            "animal_basic_import_template.csv",
            "text/csv; charset=utf-8",
        )
    } else {
        (
            AnimalImportExportService::generate_basic_import_template()?,
            "animal_basic_import_template.xlsx",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            crate::utils::http::content_disposition_header(filename),
        )
        .body(Body::from(data))
        .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))
}

/// 下載動物體重匯入範本
pub async fn download_weight_import_template(
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Response> {
    require_permission!(current_user, "animal.animal.import");
    let format = params.get("format").map(|s| s.as_str()).unwrap_or("xlsx");
    let (data, filename, content_type) = if format == "csv" {
        (
            AnimalImportExportService::generate_weight_import_template_csv()?,
            "animal_weight_import_template.csv",
            "text/csv; charset=utf-8",
        )
    } else {
        (
            AnimalImportExportService::generate_weight_import_template()?,
            "animal_weight_import_template.xlsx",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            crate::utils::http::content_disposition_header(filename),
        )
        .body(Body::from(data))
        .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))
}

/// 匯入動物基礎資料
pub async fn import_basic_data(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    mut multipart: Multipart,
) -> Result<Json<ImportResult>> {
    require_permission!(current_user, "animal.animal.import");
    let (file_data, file_name) = parse_import_file(&mut multipart).await?;
    let actor = ActorContext::User(current_user.clone());
    let result = AnimalImportExportService::import_basic_data(
        &state.db,
        &actor,
        &file_data,
        &file_name,
    )
    .await?;
    let display = format!(
        "匯入動物基礎資料: {} (成功: {}, 失敗: {})",
        file_name, result.success_count, result.error_count
    );
    if let Err(e) = AuditService::log_activity_oneshot(
        &state.db,
        &actor,
        ActivityLogEntry {
            event_category: "ANIMAL",
            event_type: "ANIMAL_IMPORT",
            entity: Some(AuditEntity::new("animal", Uuid::nil(), &display)),
            data_diff: None,
            request_context: None,
        },
    )
    .await
    {
        tracing::error!("寫入 user_activity_logs 失敗 (ANIMAL_IMPORT): {}", e);
    }
    Ok(Json(result))
}

/// 匯入動物體重資料
pub async fn import_weight_data(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    mut multipart: Multipart,
) -> Result<Json<ImportResult>> {
    require_permission!(current_user, "animal.animal.import");
    let (file_data, file_name) = parse_import_file(&mut multipart).await?;
    let actor = ActorContext::User(current_user.clone());
    let result = AnimalImportExportService::import_weight_data(
        &state.db,
        &actor,
        &file_data,
        &file_name,
    )
    .await?;
    let display = format!(
        "匯入體重資料: {} (成功: {}, 失敗: {})",
        file_name, result.success_count, result.error_count
    );
    if let Err(e) = AuditService::log_activity_oneshot(
        &state.db,
        &actor,
        ActivityLogEntry {
            event_category: "ANIMAL",
            event_type: "WEIGHT_IMPORT",
            entity: Some(AuditEntity::new("animal_weight", Uuid::nil(), &display)),
            data_diff: None,
            request_context: None,
        },
    )
    .await
    {
        tracing::error!("寫入 user_activity_logs 失敗 (WEIGHT_IMPORT): {}", e);
    }
    Ok(Json(result))
}

/// 共用的匯入檔案解析
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
    if file_data.len() > crate::constants::FILE_MAX_ANIMAL_PHOTO {
        return Err(AppError::Validation("檔案大小不能超過 10MB".to_string()));
    }
    Ok((file_data, file_name))
}
