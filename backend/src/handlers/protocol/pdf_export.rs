// Gotenberg PDF 匯出 Handlers（審核結果、審查意見回覆表、AUP 計畫書）

use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Extension,
};
use base64::Engine;
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    require_permission,
    services::{access, pdf_service_client::DocxRenderFormat, FileService, ProtocolService},
    AppError, AppState, Result,
};

/// 建構 PDF 下載回應 — L2 加入可選的 X-PDF-Renderer header（降級提示用）
fn pdf_response(pdf_bytes: Vec<u8>, filename: &str, renderer: Option<String>) -> Result<Response> {
    crate::utils::http::file_response_with_renderer(
        pdf_bytes,
        "application/pdf",
        filename,
        false,
        renderer,
    )
    .map_err(AppError::Internal)
}

/// 驗證使用者對指定計畫的物件層存取權，通過則回傳已授權的 `Scoped<ProtocolId>` 證明。
async fn authorize_protocol_scope(
    state: &AppState,
    current_user: &CurrentUser,
    protocol_id: Uuid,
) -> Result<access::Scoped<access::ProtocolId>> {
    access::Scoped::<access::ProtocolId>::authorize(&state.db, current_user, protocol_id).await
}

/// 匯出審核結果 PDF (AD-04-01-10B)
#[utoipa::path(
    get,
    path = "/api/v1/protocols/{id}/export-review-result",
    params(("id" = Uuid, Path, description = "計畫 ID")),
    responses((status = 200, description = "PDF 檔案")),
    tag = "計畫書管理",
    security(("bearer" = []))
)]
pub async fn export_review_result(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    require_permission!(current_user, "aup.protocol.view_own");
    let scope = authorize_protocol_scope(&state, &current_user, id).await?;

    // R32-A8c：改走 docx → Word COM PDF（取代 Gotenberg HTML / Tera）。
    // 文件設計：每件 protocol 1 份彙整文件，召集人手寫簽名 + 勾選結論。
    // SQL 已下沉到 service 層（per CLAUDE.md「Handler 禁止寫 SQL」）。
    let data = ProtocolService::get_review_result_export_data(&state.db, scope).await?;
    let (pdf_bytes, renderer) = state
        .pdf_service
        .render_review_result_from_review_data(&data, DocxRenderFormat::Pdf)
        .await?;

    let iacuc = data
        .get("protocol")
        .and_then(|p| p.get("iacuc_no"))
        .and_then(|v| v.as_str())
        .or_else(|| {
            data.get("protocol")
                .and_then(|p| p.get("protocol_no"))
                .and_then(|v| v.as_str())
        })
        .unwrap_or("draft");
    let filename = format!("{}_審核結果.pdf", iacuc);
    pdf_response(pdf_bytes, &filename, renderer)
}

/// 匯出審查意見回覆表 PDF (AD-04-01-04C)
#[utoipa::path(
    get,
    path = "/api/v1/protocols/{id}/export-review-comments",
    params(("id" = Uuid, Path, description = "計畫 ID")),
    responses((status = 200, description = "PDF 檔案")),
    tag = "計畫書管理",
    security(("bearer" = []))
)]
pub async fn export_review_comments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    require_permission!(current_user, "aup.protocol.view_own");
    let scope = authorize_protocol_scope(&state, &current_user, id).await?;

    // R32-A8e：改走 docx → Word COM PDF（取代 Tera + Gotenberg HTML）。
    // 250 行 SQL/組裝邏輯下沉到 service 層。
    let data = ProtocolService::get_review_reply_export_data(&state.db, scope).await?;
    let (pdf_bytes, renderer) = state
        .pdf_service
        .render_review_reply_from_review_data(&data, DocxRenderFormat::Pdf)
        .await?;

    let no = data
        .get("application_no")
        .and_then(|v| v.as_str())
        .unwrap_or("draft");
    let filename = format!("{}_審查意見回覆表.pdf", no);
    pdf_response(pdf_bytes, &filename, renderer)
}

// R32-A8j: export_protocol_pdf_v2 + Tera HTML / Gotenberg / lopdf TOC 路徑已移除。
// 前端統一走 export_aup_v3（docx → Word COM PDF）。

/// 單張內嵌縮圖的大小上限（2MB）。超過者不內嵌，print-pdf 端僅顯示檔名。
const MAX_EMBED_PHOTO_BYTES: u64 = 2 * 1024 * 1024;

/// §8 試驗人員「職稱」空值預設（內部 staff 計畫，imported_at NULL）。
const PERSONNEL_POSITION_DEFAULT_STAFF: &str = "研究人員";
/// §8 試驗人員「職稱」空值預設（外部客戶匯入計畫，imported_at 非 NULL）。
const PERSONNEL_POSITION_DEFAULT_EXTERNAL: &str = "未填";

/// §3 物質照片：走訪 working_content.items.{test_items,control_items}[].photos[]，
/// 將圖片檔讀出後 base64 內嵌為 data URI（寫入 photo["datauri"]）。
/// best-effort：單張讀取失敗 / 過大 / 非圖片者略過，不影響整體匯出。
async fn inject_photo_datauris(working_content: &mut serde_json::Value) {
    let Some(items) = working_content.get_mut("items") else {
        return;
    };
    for key in ["test_items", "control_items"] {
        let Some(arr) = items.get_mut(key).and_then(|v| v.as_array_mut()) else {
            continue;
        };
        for item in arr.iter_mut() {
            let Some(photos) = item.get_mut("photos").and_then(|v| v.as_array_mut()) else {
                continue;
            };
            for photo in photos.iter_mut() {
                embed_photo_datauri(photo).await;
            }
        }
    }
}

/// §8 試驗人員「職稱」空值預設注入：走訪 working_content.personnel[]，將 position
/// 為空 / 缺漏者填入預設值——內部 staff 計畫填「研究人員」、外部客戶匯入計畫填「未填」。
/// 以 imported_at 非 NULL 判別外部客戶（與 access / amendment 既有匯入計劃判別一致），
/// 與前端 SectionPersonnel 顯示邏輯對齊（網頁 / PDF 同步）。已有 position 者不覆寫。
fn apply_personnel_position_defaults(working_content: &mut serde_json::Value, is_external: bool) {
    let default_position = if is_external {
        PERSONNEL_POSITION_DEFAULT_EXTERNAL
    } else {
        PERSONNEL_POSITION_DEFAULT_STAFF
    };
    let Some(personnel) = working_content
        .get_mut("personnel")
        .and_then(|v| v.as_array_mut())
    else {
        return;
    };
    for person in personnel.iter_mut() {
        let Some(obj) = person.as_object_mut() else {
            continue;
        };
        let is_empty = obj
            .get("position")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().is_empty())
            .unwrap_or(true);
        if is_empty {
            obj.insert(
                "position".to_string(),
                serde_json::Value::String(default_position.to_string()),
            );
        }
    }
}

/// 將單一 photo 物件的檔案讀出、base64 內嵌為 data URI。
/// 安全性依賴 `FileService::read`（canonicalize + 限制 uploads 根目錄，防 path traversal）；
/// MIME 以伺服器端推斷為準，僅內嵌 image/*。
async fn embed_photo_datauri(photo: &mut serde_json::Value) {
    let file_path = match photo.get("file_path").and_then(|v| v.as_str()) {
        Some(p) if !p.is_empty() => p.to_string(),
        _ => return,
    };
    // 前置大小過濾（metadata）：避免讀取超大檔
    if photo.get("file_size").and_then(|v| v.as_u64()).unwrap_or(0) > MAX_EMBED_PHOTO_BYTES {
        return;
    }
    let Ok((bytes, mime)) = FileService::read(&file_path).await else {
        return;
    };
    if !mime.starts_with("image/") || bytes.len() as u64 > MAX_EMBED_PHOTO_BYTES {
        return;
    }
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    if let Some(obj) = photo.as_object_mut() {
        obj.insert(
            "datauri".to_string(),
            serde_json::Value::String(format!("data:{mime};base64,{b64}")),
        );
    }
}

/// R32-A3 收尾 / A8j：AUP 計畫書匯出（print-pdf / WeasyPrint）。
///
/// `format=pdf`（預設）回 PDF；`format=html` 回送進 WeasyPrint 前的同一份 HTML，
/// 供「計畫內容」分頁預覽 iframe。docx 已移除（print-pdf 僅產 PDF，見 R74-1）。
#[utoipa::path(
    get,
    path = "/api/v1/protocols/{id}/export-aup-v3",
    params(
        ("id" = Uuid, Path, description = "計畫 ID"),
        ("format" = Option<String>, Query, description = "pdf | html（預設 pdf；html 供前端預覽）")
    ),
    responses((status = 200, description = "AUP 計畫書檔案")),
    tag = "計畫書管理",
    security(("bearer" = []))
)]
pub async fn export_aup_v3(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Result<Response> {
    require_permission!(current_user, "aup.protocol.view_own");
    let scope = authorize_protocol_scope(&state, &current_user, id).await?;

    let format_str = params.get("format").map(String::as_str).unwrap_or("pdf");
    if !matches!(format_str, "pdf" | "html") {
        return Err(crate::AppError::BadRequest(format!(
            "Invalid format: {format_str} (expected 'pdf' or 'html')"
        )));
    }

    let protocol = ProtocolService::get_by_id(&state.db, scope.id()).await?;
    let mut working_content = protocol
        .protocol
        .working_content
        .clone()
        .unwrap_or(serde_json::json!({}));

    // §3 物質照片：將圖片檔讀出、base64 內嵌為 data URI，供 print-pdf 渲染縮圖
    // （print-pdf 容器無法存取 uploads，故由 backend 此處解析注入）。
    inject_photo_datauris(&mut working_content).await;

    // §8 人員職稱空值預設：內部 staff →「研究人員」、外部客戶匯入計畫 →「未填」。
    let is_external = protocol.protocol.imported_at.is_some();
    apply_personnel_position_defaults(&mut working_content, is_external);

    // format=html：回傳與匯出 PDF 同源的 HTML，供「計畫內容」分頁預覽 iframe。
    if format_str == "html" {
        let html = state.pdf_service.render_aup_html(&working_content).await?;
        return Ok(axum::response::Html(html).into_response());
    }

    // 僅 PDF：print-pdf/WeasyPrint 只產 PDF，docx 路徑已移除（見 R74-1）。
    let (bytes, renderer) = state
        .pdf_service
        .render_aup_from_working_content(&working_content, DocxRenderFormat::Pdf)
        .await?;

    let filename = format!("{}_AUP計畫書.pdf", protocol.protocol.title);
    crate::utils::http::file_response_with_renderer(
        bytes,
        "application/pdf",
        &filename,
        false,
        renderer,
    )
    .map_err(crate::AppError::Internal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn position_at(wc: &serde_json::Value, idx: usize) -> Option<&str> {
        wc.get("personnel")?
            .as_array()?
            .get(idx)?
            .get("position")?
            .as_str()
    }

    #[test]
    fn staff_empty_position_defaults_to_researcher() {
        let mut wc = json!({
            "personnel": [
                { "name": "甲", "position": "" },        // 空字串
                { "name": "乙", "position": "   " },      // 純空白
                { "name": "丙" },                          // 缺漏 position
                { "name": "丁", "position": "教授" },     // 已有值，不覆寫
            ]
        });
        apply_personnel_position_defaults(&mut wc, false);
        assert_eq!(position_at(&wc, 0), Some(PERSONNEL_POSITION_DEFAULT_STAFF));
        assert_eq!(position_at(&wc, 1), Some(PERSONNEL_POSITION_DEFAULT_STAFF));
        assert_eq!(position_at(&wc, 2), Some(PERSONNEL_POSITION_DEFAULT_STAFF));
        assert_eq!(position_at(&wc, 3), Some("教授"));
    }

    #[test]
    fn external_empty_position_defaults_to_unfilled() {
        let mut wc = json!({ "personnel": [ { "name": "甲", "position": "" } ] });
        apply_personnel_position_defaults(&mut wc, true);
        assert_eq!(
            position_at(&wc, 0),
            Some(PERSONNEL_POSITION_DEFAULT_EXTERNAL)
        );
    }

    #[test]
    fn missing_personnel_key_is_noop() {
        let mut wc = json!({ "basic": { "study_title": "X" } });
        apply_personnel_position_defaults(&mut wc, false);
        assert!(wc.get("personnel").is_none());
    }
}
