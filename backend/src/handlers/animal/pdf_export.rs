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
    middleware::{ActorContext, CurrentUser},
    models::ExportRequest,
    require_permission,
    services::{
        access,
        audit::{ActivityLogEntry, AuditEntity},
        pdf_service_client::{DocxRenderFormat, XlsxRenderFormat},
        AnimalBloodTestService, AnimalMedicalService, AnimalService, AuditService, FileService,
    },
    AppError, AppState, Result,
};

// PenInfo struct removed — was only used by export_pen_report
// (deleted along with pen_inspection.html / Gotenberg HTML→PDF path).
// Frontend now uses /animals/vet-patrol/export-v3 → Excel COM daemon.

/// 匯出單隻動物病歷 PDF（Gotenberg 版）
pub async fn export_animal_medical_pdf(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<ExportRequest>,
) -> Result<Response> {
    require_permission!(current_user, "animal.export.medical");
    // C3: 驗證使用者為該動物所屬計畫的成員，防止跨計畫 PDF 匯出 IDOR。
    // 匯出＝整本病歷帶走，曝險大於線上逐筆讀取，故維持「限自己計畫」（不放寬給 view_all 跨計畫）。
    access::require_animal_access(&state.db, &current_user, animal_id).await?;

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
            // R32-A8a + coderabbit PR #332：source_name 已在 service 層 JOIN
            let (pdf_bytes, renderer) = state
                .pdf_service
                .render_medical_record_from_animal_data(&data, DocxRenderFormat::Pdf)
                .await?;
            let filename = format!("medical_record_{}.pdf", animal_id);
            crate::utils::http::file_response_with_renderer(
                pdf_bytes,
                "application/pdf",
                &filename,
                false,
                renderer,
            )
            .map_err(AppError::Internal)
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

/// R32-A8b：匯出單一手術紀錄 PDF（v3 docx → Word COM）。
///
/// `format` query：`docx` / `pdf`（預設 pdf）。route：
/// `GET /api/v1/surgeries/:id/export-pdf-v3?format=pdf`
#[utoipa::path(
    get,
    path = "/api/v1/surgeries/{id}/export-pdf-v3",
    params(
        ("id" = Uuid, Path, description = "手術紀錄 ID"),
        ("format" = Option<String>, Query, description = "docx / pdf（預設 pdf）"),
    ),
    responses((status = 200, description = "手術紀錄檔案")),
    tag = "動物管理",
    security(("bearer" = []))
)]
pub async fn export_surgery_pdf_v3(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(surgery_id): Path<Uuid>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Result<Response> {
    require_permission!(current_user, "animal.export.surgery");

    let surgery = crate::services::AnimalSurgeryService::get_by_id(&state.db, surgery_id).await?;
    // C3：匯出限該動物所屬計畫成員，防跨計畫匯出 IDOR（不放寬給 view_all）
    let scope = access::Scoped::<access::AnimalWrite>::authorize(
        &state.db,
        &current_user,
        surgery.animal_id,
    )
    .await?;

    let format = match params.get("format").map(String::as_str) {
        Some("docx") => DocxRenderFormat::Docx,
        Some("pdf") | None => DocxRenderFormat::Pdf,
        Some(other) => {
            return Err(AppError::BadRequest(format!(
                "Invalid format: {other} (expected 'docx' or 'pdf')"
            )));
        }
    };

    // SQL 都在 service 層
    let data = crate::services::AnimalSurgeryService::get_surgery_export_data(
        &state.db, scope, surgery_id,
    )
    .await?;
    let (bytes, renderer) = state
        .pdf_service
        .render_surgery_from_surgery_data(&data, format)
        .await?;

    let actor = ActorContext::User(current_user.clone());
    let display = format!("手術 {} ({})", surgery_id, surgery.surgery_date);
    if let Err(e) = AuditService::log_activity_oneshot(
        &state.db,
        &actor,
        ActivityLogEntry {
            event_category: "ANIMAL",
            event_type: "EXPORT_SURGERY",
            entity: Some(AuditEntity::new("animal_surgeries", surgery_id, &display)),
            data_diff: None,
            request_context: None,
        },
    )
    .await
    {
        tracing::error!("寫入 user_activity_logs 失敗 (EXPORT_SURGERY): {e}");
    }

    let filename = format!(
        "surgery_{}_{}.{}",
        surgery.animal_id,
        surgery.surgery_date,
        format.extension()
    );
    crate::utils::http::file_response_with_renderer(
        bytes,
        format.mime_type(),
        &filename,
        false,
        renderer,
    )
    .map_err(AppError::Internal)
}

/// 匯出計畫批次病歷 PDF（Gotenberg 版）
pub async fn export_project_medical_pdf(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(iacuc_no): Path<String>,
    Json(req): Json<ExportRequest>,
) -> Result<Response> {
    require_permission!(current_user, "animal.export.medical");
    // C3: 驗證使用者是否為該 IACUC 計畫的成員，防止跨計畫批次 PDF 匯出 IDOR
    access::require_iacuc_protocol_access(&state.db, &current_user, &iacuc_no).await?;

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
            // R32-A8d + coderabbit PR #332：source_name batch JOIN 已在 service 層
            let (pdf_bytes, renderer) = state
                .pdf_service
                .render_project_medical_from_project_data(&data)
                .await?;
            let filename = format!("project_medical_{}.pdf", iacuc_no);
            crate::utils::http::file_response_with_renderer(
                pdf_bytes,
                "application/pdf",
                &filename,
                false,
                renderer,
            )
            .map_err(AppError::Internal)
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

// export_pen_report handler removed — frontend has been routed to
// export_vet_patrol_v3 (Excel COM daemon path) since R32-A3b. The
// pen_inspection.html + Gotenberg HTML→PDF chain is no longer used.

/// 匯出獸醫巡場報告 PDF
///
/// R39: 從 legacy `vet_patrol_report.html` + `state.gotenberg.html_to_pdf`
/// 切換到 `state.pdf_service.render_vet_patrol_report_from_report_data`
/// （docx → Word COM daemon → PDF），與 R38 主路徑一致。
/// 同時加 `?inline=1` query 支援預覽 vs 下載 disposition 切換（對齊 R35-4 / vet_patrol_v3）。
pub async fn export_vet_patrol_report_pdf(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Response> {
    // SEC-IDOR + R75-5: 巡場為全場內部福利文件（表無 protocol 欄、無法逐筆 scope）。
    // 限內部監督/實驗人員（view_all）或 SD；排除外部 CLIENT/PI 亦持有的 animal.record.view。
    access::require_vet_patrol_view(&current_user)?;
    use crate::services::VetPatrolReportService;

    let report_with_entries = VetPatrolReportService::get(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("巡場報告不存在".into()))?;

    let report = &report_with_entries.report;
    let entries = &report_with_entries.entries;
    let entry_photos = &report_with_entries.entry_photos;

    // 查詢建立者姓名
    let vet_name: String = if let Some(uid) = report.created_by {
        sqlx::query_scalar::<_, String>("SELECT display_name FROM users WHERE id = $1")
            .bind(uid)
            .fetch_optional(&state.db)
            .await?
            .unwrap_or_default()
    } else {
        String::new()
    };

    // 按類別分組並合併文字
    let category_order = [
        ("pig_condition", "豬隻狀況"),
        ("epidemic_prevention", "防疫及消毒計畫"),
        ("case_record", "病歷紀錄"),
        ("other", "其他"),
    ];

    // R39: 組裝 entry → photos 對映；每筆觀察讀檔 + base64 編 data URL
    //
    // 並發讀檔（futures::join_all）— 序列讀 N 張照片時 PDF 產生時間 = N × disk I/O；
    // 並發後接近 max(disk I/O)。Tokio 預設 multi-thread runtime 自然分散 blocking work。
    use base64::Engine as _;
    use futures::future::join_all;

    let entry_photo_futures = entry_photos.iter().map(|photo| async move {
        let result = FileService::read(&photo.file_path).await;
        (photo, result)
    });
    let entry_photo_results = join_all(entry_photo_futures).await;

    let mut entry_photo_map: std::collections::HashMap<Uuid, Vec<serde_json::Value>> =
        std::collections::HashMap::new();
    for (photo, result) in entry_photo_results {
        match result {
            Ok((data, _)) => {
                let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
                let data_url = format!("data:{};base64,{}", photo.mime_type, b64);
                entry_photo_map
                    .entry(photo.entry_id)
                    .or_default()
                    .push(serde_json::json!({
                        "src": data_url,
                        "caption": photo.caption,
                    }));
            }
            Err(e) => {
                tracing::warn!(
                    "vet_patrol PDF：entry photo {} 讀取失敗（{e}），略過",
                    photo.file_path
                );
            }
        }
    }

    // R39-D2：以「同隻豬一組」為單位的照片群組（給第 2 頁 photo_groups 範本用）
    let mut photo_groups: Vec<serde_json::Value> = Vec::new();

    let mut categories: Vec<serde_json::Value> = Vec::new();
    for (key, label) in &category_order {
        let cat_entries: Vec<_> = entries.iter().filter(|e| e.category == *key).collect();

        let mut entry_rows: Vec<serde_json::Value> = Vec::new();
        let mut cat_photos: Vec<serde_json::Value> = Vec::new();

        for e in &cat_entries {
            let combined_tag = if !e.ear_tags.is_empty() {
                e.ear_tags
                    .iter()
                    .map(|t| format!("#{t}"))
                    .collect::<Vec<_>>()
                    .join("、")
            } else if let Some(t) = e.ear_tag.as_deref() {
                format!("#{t}")
            } else {
                String::new()
            };
            let prefix = if combined_tag.is_empty() {
                String::new()
            } else {
                format!("{combined_tag} ")
            };
            let obs_text = if e.observation.is_empty() {
                String::new()
            } else {
                format!("{}{}", prefix, e.observation)
            };
            let sug_text = e.suggestion.clone();
            let fup_text = e.follow_up.clone();
            entry_rows.push(serde_json::json!({
                "observation": obs_text,
                "suggestion": sug_text,
                "follow_up": fup_text,
            }));
            if let Some(ps) = entry_photo_map.get(&e.id) {
                let tag_prefix = combined_tag.clone();
                let mut group_srcs: Vec<serde_json::Value> = Vec::new();
                // 每張照片自己的說明（使用者在前端 CaptionInput 填的），與 srcs 同序。
                // 這是修「PDF 圖說只有耳號」的關鍵：group 只有一個 caption 欄位，
                // 範本便把同組每張照片的圖說都印成同一行耳號，使用者填的說明整個被丟掉
                // （它只進了 categories[].photos，而範本明講不使用那條路徑）。
                let mut group_photos: Vec<serde_json::Value> = Vec::new();
                for p in ps.iter() {
                    let mut p_with_label = p.clone();
                    let original = p
                        .get("caption")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if let Some(obj) = p_with_label.as_object_mut() {
                        let new_caption = match (tag_prefix.as_str(), original.as_str()) {
                            ("", "") => String::new(),
                            (tag, "") => tag.to_string(),
                            ("", cap) => cap.to_string(),
                            (tag, cap) => format!("{tag} {cap}"),
                        };
                        obj.insert("caption".into(), serde_json::Value::String(new_caption));
                    }
                    if let Some(src) = p.get("src").cloned() {
                        group_srcs.push(src.clone());
                        // caption 只放使用者填的說明，不含耳號——耳號由範本統一印在說明上一行，
                        // 避免同組每張都重覆一次耳號。
                        group_photos.push(serde_json::json!({
                            "src": src,
                            "caption": original,
                        }));
                    }
                    cat_photos.push(p_with_label);
                }
                if !group_srcs.is_empty() {
                    photo_groups.push(serde_json::json!({
                        "caption": tag_prefix,
                        "description": e.observation,
                        // srcs 保留：舊版 print-pdf 仍只認得它（兩個服務非原子部署）。
                        "srcs": group_srcs,
                        "photos": group_photos,
                    }));
                }
            }
        }

        // 模板（vet_patrol_report.html）主表每類別只有單一「觀察內容 / 建議」格，
        // 讀 category 層級的 observation / suggestion（非逐筆 entries），對齊官方 GLP
        // 母表 AD-02-02-01 單格版面。故將各筆 entry 文字以換行合併回 category 層級，
        // 否則主表會空白（PDF 印不出網頁上顯示的逐筆內容）。
        let join_nonempty = |field: &str| -> String {
            entry_rows
                .iter()
                .filter_map(|r| r.get(field).and_then(|v| v.as_str()))
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        };
        let cat_observation = join_nonempty("observation");
        let cat_suggestion = join_nonempty("suggestion");
        let cat_follow_up = join_nonempty("follow_up");
        categories.push(serde_json::json!({
            "label": label,
            "entries": entry_rows,
            "observation": cat_observation,
            "suggestion": cat_suggestion,
            "follow_up": cat_follow_up,
            "photos": cat_photos,
        }));
    }

    // Report-level 整體環境照（送 payload.photos 對齊範本根層 `{%p for pair in photos | batch(2) %}`）
    let report_photo_futures = report_with_entries.photos.iter().map(|photo| async move {
        let result = FileService::read(&photo.file_path).await;
        (photo, result)
    });
    let report_photo_results = join_all(report_photo_futures).await;

    let mut root_photos: Vec<serde_json::Value> = Vec::new();
    for (photo, result) in report_photo_results {
        match result {
            Ok((data, _)) => {
                let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
                let data_url = format!("data:{};base64,{}", photo.mime_type, b64);
                // R39-D2：整體環境照每張當作獨立 group（caption 用原 caption 或「整體環境」）
                let group_caption = if photo.caption.is_empty() {
                    "整體環境".to_string()
                } else {
                    photo.caption.clone()
                };
                photo_groups.push(serde_json::json!({
                    "caption": group_caption,
                    "srcs": [data_url.clone()],
                }));
                root_photos.push(serde_json::json!({
                    "src": data_url,
                    "caption": photo.caption,
                }));
            }
            Err(e) => {
                tracing::warn!(
                    "vet_patrol PDF：report photo {} 讀取失敗（{e}），略過",
                    photo.file_path
                );
            }
        }
    }

    let patrol_date_display = report.patrol_date.format("%Y年%m月%d日").to_string();
    let payload = serde_json::json!({
        "vet_name": vet_name,
        "companion": report.accompanying_personnel.as_deref().unwrap_or(""),
        "patrol_date": report.patrol_date.format("%Y-%m-%d").to_string(),
        "patrol_date_display": patrol_date_display,
        "categories": categories,
        "photos": root_photos,
        "photo_groups": photo_groups,
    });

    let (pdf_bytes, renderer) = state
        .pdf_service
        .render_vet_patrol_report_from_report_data(&payload, DocxRenderFormat::Pdf)
        .await?;

    let filename = format!(
        "試驗豬場巡場報告_{}.pdf",
        report.patrol_date.format("%Y%m%d")
    );
    let inline = matches!(params.get("inline").map(String::as_str), Some("1"));
    crate::utils::http::file_response_with_renderer(
        pdf_bytes,
        "application/pdf",
        &filename,
        inline,
        renderer,
    )
    .map_err(AppError::Internal)
}

/// 匯出單隻動物血液檢查紀錄 PDF（透過 FastAPI pdf-service）
///
/// 流程：Rust → pdf-service (Jinja2 render) → Gotenberg → PDF bytes
pub async fn export_blood_test_pdf(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Response> {
    require_permission!(current_user, "animal.export.medical");
    // 匯出限自己計畫成員，防跨計畫匯出 IDOR（不放寬給 view_all）
    let scope =
        access::Scoped::<access::AnimalWrite>::authorize(&state.db, &current_user, animal_id)
            .await?;

    let animal = AnimalService::get_by_id(&state.db, animal_id).await?;
    let items = AnimalBloodTestService::list_blood_test_export_rows(&state.db, scope).await?;
    let today = crate::time::now_taiwan().format("%Y-%m-%d").to_string();
    let exporter_name =
        crate::repositories::user::find_user_display_name_by_id(&state.db, current_user.id)
            .await?
            .unwrap_or_else(|| current_user.email.clone());
    let payload = serde_json::json!({
        "animal_ear_tag": animal.ear_tag,
        "animal_iacuc_no": animal.iacuc_no,
        "export_date": today,
        "exporter_name": exporter_name,
        "items": items,
    });

    // R32-A8h：改走 docx → Word COM PDF（取代 legacy blood_test.html Jinja2 路徑）。
    let (pdf_bytes, renderer) = state
        .pdf_service
        .render_blood_test_from_blood_test_data(&payload, DocxRenderFormat::Pdf)
        .await?;

    let iacuc = animal.iacuc_no.as_deref().unwrap_or("unassigned");
    let actor = ActorContext::User(current_user.clone());
    let export_display = format!("[{}] {}", iacuc, animal.ear_tag);
    log_blood_test_export(&state.db, &actor, animal_id, &export_display).await;

    let filename = format!("blood_test_{}_{}.pdf", iacuc, animal.ear_tag);
    crate::utils::http::file_response_with_renderer(
        pdf_bytes,
        "application/pdf",
        &filename,
        false,
        renderer,
    )
    .map_err(AppError::Internal)
}

/// R32-A8h：blood_test PDF 匯出 audit log helper（抽出避免 handler 超過 50 行）。
async fn log_blood_test_export(
    db: &sqlx::PgPool,
    actor: &ActorContext,
    animal_id: Uuid,
    export_display: &str,
) {
    if let Err(e) = AuditService::log_activity_oneshot(
        db,
        actor,
        ActivityLogEntry {
            event_category: "ANIMAL",
            event_type: "EXPORT_BLOOD_TEST",
            entity: Some(AuditEntity::new("animal", animal_id, export_display)),
            data_diff: None,
            request_context: None,
        },
    )
    .await
    {
        tracing::error!("寫入 user_activity_logs 失敗 (EXPORT_BLOOD_TEST): {}", e);
    }
}

/// R32-A3b 收尾：欄位巡視 v3（xlsx 範本 + Gotenberg LibreOffice）匯出。
///
/// 與 [`export_vet_patrol_report_pdf`] 並存（v2 走 legacy HTML，v3 走 xlsx），
/// 等 R32-A7 砍舊路徑後 v2 移除。
///
/// `format=xlsx` 回 .xlsx；`format=pdf` 經 LibreOffice 轉 PDF。
/// Query：inspector_name / patrol_date（YYYY-MM-DD）/ period（AM|PM）。
#[utoipa::path(
    get,
    path = "/api/v1/animals/vet-patrol/export-v3",
    params(
        ("format" = Option<String>, Query, description = "xlsx | pdf（預設 xlsx）"),
        ("inspector_name" = Option<String>, Query, description = "巡視人姓名"),
        ("patrol_date" = Option<String>, Query, description = "巡視日期 YYYY-MM-DD"),
        ("period" = Option<String>, Query, description = "AM | PM"),
    ),
    responses((status = 200, description = "巡視報告檔案")),
    tag = "動物管理",
    security(("bearer" = []))
)]
pub async fn export_vet_patrol_v3(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Result<Response> {
    // 對齊既有 export_pen_report 的權限與 active filter
    require_permission!(current_user, "animal.animal.view_all");

    let format = match params.get("format").map(String::as_str) {
        Some("pdf") => XlsxRenderFormat::Pdf,
        Some("xlsx") | None => XlsxRenderFormat::Xlsx,
        Some(other) => {
            return Err(AppError::BadRequest(format!(
                "Invalid format: {other} (expected 'xlsx' or 'pdf')"
            )));
        }
    };
    let inspector_name = params.get("inspector_name").cloned().unwrap_or_default();
    let patrol_date = params.get("patrol_date").cloned().unwrap_or_default();
    if !patrol_date.is_empty() {
        // 嚴格 YYYY-MM-DD（chrono 同時驗合法日期）— 避免 `/`、`..`、CRLF 等
        // 進入下載 filename 與下游 service。
        chrono::NaiveDate::parse_from_str(&patrol_date, "%Y-%m-%d").map_err(|_| {
            AppError::BadRequest(format!(
                "Invalid patrol_date: {patrol_date} (expected YYYY-MM-DD)"
            ))
        })?;
    }
    let period = params.get("period").cloned().unwrap_or_default();
    if !matches!(period.as_str(), "" | "AM" | "PM") {
        return Err(AppError::BadRequest(format!(
            "Invalid period: {period} (expected '' | 'AM' | 'PM')"
        )));
    }

    // SQL 查詢已下沉到 service 層（per CLAUDE.md「Handler 禁止寫 SQL」）
    let animals_json =
        crate::services::VetPatrolReportService::list_animals_for_patrol(&state.db).await?;

    let (bytes, renderer) = state
        .pdf_service
        .render_vet_patrol_from_animals(
            &animals_json,
            &inspector_name,
            &patrol_date,
            &period,
            format,
        )
        .await?;

    let filename = format!(
        "欄位狀態表_{}.{}",
        if patrol_date.is_empty() {
            "report"
        } else {
            patrol_date.as_str()
        },
        format.extension()
    );
    // 與 R35-4 warehouse PDF 對齊：?inline=1 → Content-Disposition: inline，
    // 瀏覽器 PDF viewer 用 filename 當分頁標題（取代 blob: URL UUID）。
    // 預設 attachment（下載），AnimalPenReport 預覽呼叫時帶 inline=1。
    let inline = matches!(params.get("inline").map(String::as_str), Some("1"));
    crate::utils::http::file_response_with_renderer(
        bytes,
        format.mime_type(),
        &filename,
        inline,
        renderer,
    )
    .map_err(AppError::Internal)
}
