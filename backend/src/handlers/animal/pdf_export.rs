// Gotenberg PDF 匯出 Handlers（動物病歷、計畫批次病歷、動物欄位巡視報告）

use std::collections::HashMap;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::Response,
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::ExportRequest,
    require_permission,
    services::{AnimalMedicalService, AnimalService, AuditService},
    AppError, AppState, Result,
};

/// 欄位資訊（用於巡視報告模板）
#[derive(serde::Serialize)]
struct PenInfo {
    ear_tags: String,
    has_animal: bool,
    has_experiment: bool,
}

/// 匯出單隻動物病歷 PDF（Gotenberg 版）
pub async fn export_animal_medical_pdf(
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

    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ANIMAL",
        "EXPORT_MEDICAL",
        Some("animal"),
        Some(animal_id),
        Some(&export_display),
        None,
        Some(serde_json::json!({
            "format": format!("{:?}", req.format),
            "export_type": format!("{:?}", req.export_type),
            "engine": "gotenberg",
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("寫入 user_activity_logs 失敗 (MEDICAL_EXPORT): {}", e);
    }

    match req.format {
        crate::models::ExportFormat::Pdf => {
            // 解析 JSON 資料為模板上下文
            let ctx = build_medical_context(&data);
            let html = state.templates.render("medical_record.html", &ctx)?;
            let pdf_bytes = state.gotenberg.html_to_pdf(&html).await?;
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
                    "data": data,
                    "format": req.format,
                    "export_type": req.export_type,
                }))
                .map_err(|e| AppError::Internal(format!("serialize error: {e}")))?,
            ))
            .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))?),
    }
}

/// 匯出計畫批次病歷 PDF（Gotenberg 版）
pub async fn export_project_medical_pdf(
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
            let ctx = build_project_medical_context(&iacuc_no, &data);
            let html = state.templates.render("project_medical.html", &ctx)?;
            let pdf_bytes = state.gotenberg.html_to_pdf(&html).await?;
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
                    "data": data,
                    "format": req.format,
                    "export_type": req.export_type,
                }))
                .map_err(|e| AppError::Internal(format!("serialize error: {e}")))?,
            ))
            .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))?),
    }
}

/// 匯出動物欄位巡視報告 PDF (AD-05-01-02C)
pub async fn export_pen_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Response> {
    require_permission!(current_user, "animal.animal.view");

    // 查詢所有動物及其欄位位置
    let animals: Vec<(String, String, String)> = sqlx::query_as(
        r#"SELECT ear_tag, COALESCE(pen_location, '') as pen_location, status::text
           FROM animals
           WHERE is_active = true
           ORDER BY pen_location, ear_tag"#,
    )
    .fetch_all(&state.db)
    .await?;

    // 按欄位分組
    let mut pens: HashMap<String, PenInfo> = HashMap::new();
    for (ear_tag, pen_location, status) in &animals {
        if pen_location.is_empty() {
            continue;
        }
        let entry = pens.entry(pen_location.clone()).or_insert(PenInfo {
            ear_tags: String::new(),
            has_animal: false,
            has_experiment: false,
        });
        entry.has_animal = true;
        if status == "in_experiment" {
            entry.has_experiment = true;
        }
        if !entry.ear_tags.is_empty() {
            entry.ear_tags.push('.');
        }
        entry.ear_tags.push_str(ear_tag);
    }

    let today = crate::time::now_taiwan().format("%Y-%m-%d").to_string();

    let mut ctx = tera::Context::new();
    ctx.insert("pens", &pens);
    ctx.insert("inspection_date", &today);
    ctx.insert("inspector_name", "");

    let html = state.templates.render("pen_inspection.html", &ctx)?;
    let pdf_bytes = state.gotenberg.html_to_pdf(&html).await?;

    let filename = format!("動物欄位巡視報告_{}.pdf", today);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(
            header::CONTENT_DISPOSITION,
            crate::utils::http::content_disposition_header(&filename),
        )
        .body(Body::from(pdf_bytes))
        .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))
}

// ─── 內部輔助函式 ───

/// 將單隻動物 JSON 資料轉為 Tera Context
fn build_medical_context(data: &serde_json::Value) -> tera::Context {
    let mut ctx = tera::Context::new();

    if let Some(animal) = data.get("animal") {
        ctx.insert("animal", animal);
    }

    // 疫苗紀錄
    let vaccinations = data
        .get("vaccinations")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    ctx.insert("vaccinations", &vaccinations);

    // 合併體重、手術、觀察紀錄為 records（依日期排序）
    let mut records: Vec<serde_json::Value> = Vec::new();

    if let Some(weights) = data.get("weights").and_then(|v| v.as_array()) {
        for w in weights {
            let date = w
                .get("measured_at")
                .or_else(|| w.get("date"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let weight = w
                .get("weight_kg")
                .or_else(|| w.get("weight"))
                .and_then(|v| {
                    v.as_f64()
                        .map(|f| f.to_string())
                        .or_else(|| v.as_str().map(|s| s.to_string()))
                })
                .unwrap_or_default();
            records.push(serde_json::json!({
                "date": date,
                "weight": weight,
                "observation": "",
            }));
        }
    }

    if let Some(observations) = data.get("observations").and_then(|v| v.as_array()) {
        for o in observations {
            let date = o
                .get("observed_at")
                .or_else(|| o.get("date"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let note = o
                .get("notes")
                .or_else(|| o.get("observation"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            records.push(serde_json::json!({
                "date": date,
                "weight": "",
                "observation": note,
            }));
        }
    }

    if let Some(surgeries) = data.get("surgeries").and_then(|v| v.as_array()) {
        for s in surgeries {
            let date = s
                .get("surgery_date")
                .or_else(|| s.get("date"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let desc = s
                .get("description")
                .or_else(|| s.get("surgery_type"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            records.push(serde_json::json!({
                "date": date,
                "weight": "",
                "observation": format!("[手術] {}", desc),
            }));
        }
    }

    // 按日期排序
    records.sort_by(|a, b| {
        let da = a.get("date").and_then(|v| v.as_str()).unwrap_or("");
        let db = b.get("date").and_then(|v| v.as_str()).unwrap_or("");
        da.cmp(db)
    });

    ctx.insert("records", &records);
    ctx
}

/// 將計畫批次醫療 JSON 資料轉為 Tera Context
fn build_project_medical_context(iacuc_no: &str, data: &serde_json::Value) -> tera::Context {
    let mut ctx = tera::Context::new();
    ctx.insert("iacuc_no", iacuc_no);

    let today = crate::time::now_taiwan().format("%Y-%m-%d").to_string();
    ctx.insert("export_date", &today);

    let mut animals: Vec<serde_json::Value> = Vec::new();
    if let Some(arr) = data.get("animals").and_then(|v| v.as_array()) {
        for animal_data in arr {
            let animal = animal_data
                .get("animal")
                .cloned()
                .unwrap_or(serde_json::json!({}));

            let vaccinations = animal_data
                .get("vaccinations")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            // 建構 records
            let sub_ctx = build_medical_context(animal_data);
            let records: Vec<serde_json::Value> = sub_ctx
                .get("records")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();

            animals.push(serde_json::json!({
                "animal": animal,
                "vaccinations": vaccinations,
                "records": records,
            }));
        }
    }

    ctx.insert("animals", &animals);
    ctx
}
