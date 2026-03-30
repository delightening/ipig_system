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
    services::{access, PdfService, ProtocolService},
    AppState, Result,
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
    access::require_protocol_view_access(
        &state.db, &current_user, id, protocol.protocol.pi_user_id,
    ).await?;
    let pdf_bytes = PdfService::generate_protocol_pdf(&protocol)?;
    let filename = format!("{}_AUP計畫書.pdf", protocol.protocol.title);
    Ok((
        [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (header::CONTENT_DISPOSITION, crate::utils::http::content_disposition_header(&filename)),
        ],
        pdf_bytes,
    ))
}
