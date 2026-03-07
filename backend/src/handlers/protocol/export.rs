// 匯出 Handlers

use axum::{
    extract::{Path, State},
    http::header,
    response::IntoResponse,
    Extension,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    require_permission,
    services::{PdfService, ProtocolService},
    AppError, AppState, Result,
};

/// 匯出計畫書 PDF
#[utoipa::path(get, path = "/api/protocols/{id}/export-pdf", params(("id" = Uuid, Path, description = "專案 ID")), responses((status = 200, description = "PDF 檔案")), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn export_protocol_pdf(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    require_permission!(current_user, "aup.protocol.view_own");
    let protocol = ProtocolService::get_by_id(&state.db, id).await?;
    let has_view_all = current_user.has_permission("aup.protocol.view_all")
        || current_user.roles.iter().any(|r| ["IACUC_CHAIR", "IACUC_STAFF", "VET", "REVIEWER"].contains(&r.as_str()));
    let is_pi_or_coeditor: (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(SELECT 1 FROM user_protocols WHERE protocol_id = $1 AND user_id = $2 AND role_in_protocol IN ('PI', 'CLIENT', 'CO_EDITOR'))"#
    ).bind(id).bind(current_user.id).fetch_one(&state.db).await.unwrap_or((false,));
    let is_assigned_reviewer: (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(SELECT 1 FROM review_assignments WHERE protocol_id = $1 AND reviewer_id = $2)"#
    ).bind(id).bind(current_user.id).fetch_one(&state.db).await.unwrap_or((false,));
    let is_assigned_vet: (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(SELECT 1 FROM vet_review_assignments WHERE protocol_id = $1 AND vet_id = $2)"#
    ).bind(id).bind(current_user.id).fetch_one(&state.db).await.unwrap_or((false,));
    if !has_view_all && protocol.protocol.pi_user_id != current_user.id && !is_pi_or_coeditor.0 && !is_assigned_reviewer.0 && !is_assigned_vet.0 {
        return Err(AppError::Forbidden("You don't have permission to export this protocol".to_string()));
    }
    let pdf_bytes = PdfService::generate_protocol_pdf(&protocol)?;
    let filename = format!("{}_AUP計畫書.pdf", protocol.protocol.title);
    let encoded_filename = urlencoding::encode(&filename);
    Ok((
        [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (header::CONTENT_DISPOSITION, format!("attachment; filename*=UTF-8''{}", encoded_filename)),
        ],
        pdf_bytes,
    ))
}
