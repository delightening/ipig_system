// R53-9: 豬隻病歷週報 Handler
//
// 對應 R53-8 service。POST 而非 GET — filter 結構 nested arrays，避開
// query string complexity。
//
// Permission: `animal.record.view`（PI 角色擁有；GUEST 不帶）。
// 報表內容已對齊 R53-6 audit blacklist — 不包含 byproduct_sample。
//
// 後續 (R53-10/11/12)：Excel / PDF 匯出走 print-pdf 端點；frontend 報表
// 中心入口。本 handler 只回 JSON timeline。

use axum::{extract::State, http::header, response::Response, Extension, Json};

use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    middleware::{ActorContext, CurrentUser},
    require_permission,
    services::{
        access,
        audit::{ActivityLogEntry, AuditEntity},
        AnimalMedicalReportService, AuditService, ByproductMonthlyRow, ByproductSampleService,
        MedicalTimelineEvent, WeeklyMedicalReportFilter,
    },
    AppState, Result,
};

/// #447（Medium）：大量機敏病歷讀取留痕。週報三端點（JSON / xlsx / pdf）讀取後寫一筆
/// audit（記格式 + 筆數），補上「誰在何時拉了哪種跨計畫病歷彙整」的稽核軌跡。
async fn audit_weekly_read(
    state: &AppState,
    current_user: &CurrentUser,
    format: &str,
    count: usize,
) -> Result<()> {
    let actor = ActorContext::User(current_user.clone());
    let display = format!("豬隻病歷週報（{format}，{count} 筆）");
    // 純 audit 事件（無 entity 變更）→ log_activity_oneshot 的設計情境，免手動 tx 管理。
    AuditService::log_activity_oneshot(
        &state.db,
        &actor,
        ActivityLogEntry {
            event_category: "ANIMAL",
            event_type: "ANIMAL_MEDICAL_WEEKLY_REPORT_READ",
            entity: Some(AuditEntity::new(
                "animal_medical_report",
                Uuid::nil(),
                &display,
            )),
            data_diff: None,
            request_context: None,
        },
    )
    .await?;
    Ok(())
}

/// 計算週報資料邊界（IDOR 防護）：
/// view_all 角色回 `None`（不限）；否則回使用者可存取計畫集合（可能為空 = 看不到資料）。
/// 三個週報端點（JSON / xlsx / pdf）共用，避免無 view_all 的 PI / 委託人送空 filter
/// 即讀到全院跨計畫病歷。
async fn report_protocol_boundary(
    state: &AppState,
    current_user: &CurrentUser,
) -> Result<Option<Vec<Uuid>>> {
    if access::has_protocol_view_all(current_user) {
        return Ok(None);
    }
    let ids = access::accessible_protocol_ids(&state.db, current_user.id).await?;
    Ok(Some(ids))
}

/// POST /api/v1/reports/animal-medical/weekly
///
/// Body: WeeklyMedicalReportFilter（耳號 / 計畫案 / 時間區間，皆 optional，AND 邏輯）
pub async fn weekly_medical_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(filter): Json<WeeklyMedicalReportFilter>,
) -> Result<Json<Vec<MedicalTimelineEvent>>> {
    require_permission!(current_user, "animal.record.view");
    let boundary = report_protocol_boundary(&state, &current_user).await?;
    let events =
        AnimalMedicalReportService::weekly_report(&state.db, &filter, boundary.as_deref()).await?;
    audit_weekly_read(&state, &current_user, "JSON", events.len()).await?;
    Ok(Json(events))
}

/// POST /api/v1/reports/animal-medical/weekly/xlsx
pub async fn export_weekly_medical_report_xlsx(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(filter): Json<WeeklyMedicalReportFilter>,
) -> Result<Response> {
    require_permission!(current_user, "animal.record.view");
    let boundary = report_protocol_boundary(&state, &current_user).await?;
    let events =
        AnimalMedicalReportService::weekly_report(&state.db, &filter, boundary.as_deref()).await?;
    audit_weekly_read(&state, &current_user, "xlsx", events.len()).await?;
    let events_json = serde_json::to_value(&events)
        .map_err(|e| crate::AppError::Internal(format!("JSON serialize: {e}")))?;
    let (bytes, _) = state
        .pdf_service
        .render_weekly_medical_report_xlsx(&events_json)
        .await?;
    let resp = Response::builder()
        .header(
            header::CONTENT_TYPE,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"weekly_medical_report.xlsx\"",
        )
        .body(bytes.into())
        .map_err(|e| crate::AppError::Internal(format!("Response build: {e}")))?;
    Ok(resp)
}

/// R53-11: POST /api/v1/reports/animal-medical/weekly/pdf
pub async fn export_weekly_medical_report_pdf(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(filter): Json<WeeklyMedicalReportFilter>,
) -> Result<Response> {
    require_permission!(current_user, "animal.record.view");
    let boundary = report_protocol_boundary(&state, &current_user).await?;
    let events =
        AnimalMedicalReportService::weekly_report(&state.db, &filter, boundary.as_deref()).await?;
    audit_weekly_read(&state, &current_user, "PDF", events.len()).await?;
    let events_json = serde_json::to_value(&events)
        .map_err(|e| crate::AppError::Internal(format!("JSON serialize: {e}")))?;
    let date_range = match (filter.start_date, filter.end_date) {
        (Some(s), Some(e)) => format!("{} ~ {}", s, e),
        (Some(s), None) => format!("{} ~", s),
        (None, Some(e)) => format!("~ {}", e),
        _ => String::new(),
    };
    let (bytes, _) = state
        .pdf_service
        .render_weekly_medical_report_pdf(&events_json, &date_range)
        .await?;
    let resp = Response::builder()
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"weekly_medical_report.pdf\"",
        )
        .body(bytes.into())
        .map_err(|e| crate::AppError::Internal(format!("Response build: {e}")))?;
    Ok(resp)
}

/// R53-15: byproduct samples 月結報表 filter
#[derive(Debug, Deserialize)]
pub struct ByproductMonthlyFilter {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

/// POST /api/v1/reports/byproduct-monthly
pub async fn byproduct_monthly_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(filter): Json<ByproductMonthlyFilter>,
) -> Result<Json<Vec<ByproductMonthlyRow>>> {
    require_permission!(current_user, "animal.byproduct_sample.view");
    let rows =
        ByproductSampleService::list_monthly_report(&state.db, filter.start_date, filter.end_date)
            .await?;
    Ok(Json(rows))
}

/// POST /api/v1/reports/byproduct-monthly/xlsx
pub async fn export_byproduct_monthly_xlsx(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(filter): Json<ByproductMonthlyFilter>,
) -> Result<Response> {
    require_permission!(current_user, "animal.byproduct_sample.view");
    let rows =
        ByproductSampleService::list_monthly_report(&state.db, filter.start_date, filter.end_date)
            .await?;
    let rows_json = serde_json::to_value(&rows)
        .map_err(|e| crate::AppError::Internal(format!("JSON serialize: {e}")))?;
    let (bytes, _) = state
        .pdf_service
        .render_byproduct_monthly_xlsx(&rows_json)
        .await?;
    let resp = Response::builder()
        .header(
            header::CONTENT_TYPE,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"byproduct_monthly_report.xlsx\"",
        )
        .body(bytes.into())
        .map_err(|e| crate::AppError::Internal(format!("Response build: {e}")))?;
    Ok(resp)
}
