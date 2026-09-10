use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::{handlers, AppState};

/// AUP 審查系統、審查人員、Co-editor、動物來源、修正申請路由
pub fn routes() -> Router<AppState> {
    Router::new()
        // 計畫書範本版本登記（院區層級，admin 管理）
        .route(
            "/protocol-template-versions",
            get(handlers::list_template_versions).post(handlers::create_template_version),
        )
        .route(
            "/protocol-template-versions/{id}",
            put(handlers::update_template_version).delete(handlers::delete_template_version),
        )
        .route(
            "/protocol-template-versions/{id}/set-current",
            post(handlers::set_current_template_version),
        )
        .route(
            "/protocol-template-versions/{id}/documents",
            get(handlers::list_template_version_documents)
                .post(handlers::upload_template_version_document),
        )
        .route(
            "/protocol-template-versions/{id}/documents/{doc_id}",
            delete(handlers::delete_template_version_document),
        )
        // 動物試驗申請須知版本登記（admin 管理；/active 供申請人讀取生效版本）
        .route(
            "/application-notices/active",
            get(handlers::get_active_notice),
        )
        .route(
            "/application-notices",
            get(handlers::list_notices).post(handlers::create_notice),
        )
        .route(
            "/application-notices/{id}/activate",
            post(handlers::activate_notice),
        )
        // Protocols
        .route(
            "/protocols",
            get(handlers::list_protocols).post(handlers::create_protocol),
        )
        .route(
            "/protocols/import-approved",
            post(handlers::import_approved_protocol),
        )
        // 計畫 PI / Study Director 可指派使用者下拉（門檻同匯入，非僅 admin）
        .route(
            "/protocols/assignable-users",
            get(handlers::list_assignable_users),
        )
        .route(
            "/protocols/{id}/finalize-import",
            post(handlers::finalize_import_protocol),
        )
        .route(
            "/protocols/{id}/import-reviews",
            post(handlers::record_import_reviews),
        )
        // R64-5c：刪除誤匯的匯入計劃（admin only）以重新匯入
        .route(
            "/protocols/{id}/imported",
            delete(handlers::delete_imported_protocol),
        )
        // Admin 軟刪除「已否決」計畫（設為 DELETED，從列表隱藏，保留資料）
        .route(
            "/protocols/{id}/soft-delete",
            post(handlers::soft_delete_protocol),
        )
        // 申請人（PI/SD）簽署當前生效申請須知（送審前置）
        .route(
            "/protocols/{id}/acknowledge-notice",
            post(handlers::acknowledge_notice),
        )
        // 須知簽署狀態（生效須知 + 是否已簽，供填表/送審顯示）
        .route(
            "/protocols/{id}/notice-acknowledgement",
            get(handlers::get_notice_acknowledgement_status),
        )
        // PI 帳號開通（建立者/SD/admin 建帳+relink，不寄信）
        .route(
            "/protocols/{id}/provision-pi",
            post(handlers::provision_pi_account),
        )
        // PI 代理授權（migration 010）：核准/撤銷生效中代理人。查詢走
        // `GET /protocols/{id}` 回應內的 `pi_delegate`/`is_pi_delegate`（已有 Scoped<ProtocolView>
        // 存取控管），不另開一支無額外授權檢查的唯讀端點。
        .route(
            "/protocols/{id}/pi-delegate",
            post(handlers::authorize_pi_delegate).delete(handlers::revoke_pi_delegate),
        )
        // PI 開通信 admin 核准寄送
        .route(
            "/pi-account-invites",
            get(handlers::list_pi_account_invites),
        )
        .route(
            "/pi-account-invites/{id}/approve-send",
            post(handlers::approve_send_pi_invite),
        )
        .route(
            "/protocols/{id}",
            get(handlers::get_protocol).put(handlers::update_protocol),
        )
        .route("/protocols/{id}/submit", post(handlers::submit_protocol))
        .route("/protocols/{id}/copy", post(handlers::copy_protocol))
        .route(
            "/protocols/{id}/status",
            post(handlers::change_protocol_status),
        )
        .route(
            "/protocols/{id}/versions",
            get(handlers::get_protocol_versions),
        )
        .route(
            "/protocols/{id}/activities",
            get(handlers::get_protocol_activities),
        )
        .route(
            "/protocols/{id}/animal-stats",
            get(handlers::get_protocol_animal_stats),
        )
        // R32-A8j: legacy v1 /export-pdf + v2 /export-pdf-v2 已刪除（前端統一走 v3 docx）
        // R32-A3 收尾：v3 (docxtpl + Gotenberg LibreOffice)
        .route(
            "/protocols/{id}/export-aup-v3",
            get(handlers::export_aup_v3),
        )
        .route(
            "/protocols/{id}/export-review-result",
            get(handlers::export_review_result),
        )
        .route(
            "/protocols/{id}/export-review-comments",
            get(handlers::export_review_comments),
        )
        // Review
        .route(
            "/reviews/assignments",
            get(handlers::list_review_assignments).post(handlers::assign_reviewer),
        )
        .route(
            "/reviews/comments",
            get(handlers::list_review_comments).post(handlers::create_review_comment),
        )
        .route(
            "/reviews/comments/{id}/resolve",
            post(handlers::resolve_review_comment),
        )
        .route(
            "/reviews/comments/reply",
            post(handlers::reply_review_comment),
        )
        // Draft Reply
        .route("/reviews/comments/draft", post(handlers::save_reply_draft))
        .route(
            "/reviews/comments/{id}/draft",
            get(handlers::get_reply_draft),
        )
        .route(
            "/reviews/comments/submit-draft",
            post(handlers::submit_reply_from_draft),
        )
        // Vet Review Form
        .route("/reviews/vet-form", post(handlers::save_vet_review_form))
        // AI Review & Validation (R20)
        .route(
            "/protocols/{id}/validate",
            post(handlers::validate_protocol),
        )
        .route(
            "/protocols/{id}/ai-review",
            post(handlers::ai_review_protocol),
        )
        .route(
            "/protocols/{id}/ai-review/latest",
            get(handlers::get_latest_ai_review),
        )
        .route(
            "/ai-review/remaining",
            get(handlers::get_ai_review_remaining),
        )
        .route(
            "/protocols/{id}/staff-review-assist",
            post(handlers::staff_review_assist),
        )
        .route(
            "/protocols/{id}/staff-review-assist/latest",
            get(handlers::get_latest_staff_review),
        )
        .route(
            "/protocols/{id}/staff-review-assist/batch-return",
            post(handlers::staff_batch_return),
        )
        // My Projects
        .route("/my-projects", get(handlers::get_my_protocols))
        // Animal Sources
        .route(
            "/animal-sources",
            get(handlers::list_animal_sources).post(handlers::create_animal_source),
        )
        .route(
            "/animal-sources/{id}",
            put(handlers::update_animal_source).delete(handlers::delete_animal_source),
        )
        .route(
            "/animal-sources/{id}/delete",
            post(handlers::delete_animal_source),
        )
        // Amendments (變更申請系統)
        .route(
            "/amendments",
            get(handlers::amendment::list_amendments).post(handlers::amendment::create_amendment),
        )
        .route(
            "/amendments/pending-count",
            get(handlers::amendment::get_pending_count),
        )
        // P6：補登歷史變更（建立 is_historical 草稿）。static path 優先於 :id
        .route(
            "/amendments/historical",
            post(handlers::amendment::create_historical_amendment),
        )
        .route(
            "/amendments/{id}",
            get(handlers::amendment::get_amendment).patch(handlers::amendment::update_amendment),
        )
        .route(
            "/amendments/{id}/submit",
            post(handlers::amendment::submit_amendment),
        )
        .route(
            "/amendments/{id}/classify",
            post(handlers::amendment::classify_amendment),
        )
        .route(
            "/amendments/{id}/start-review",
            post(handlers::amendment::start_amendment_review),
        )
        .route(
            "/amendments/{id}/decision",
            post(handlers::amendment::record_amendment_decision),
        )
        .route(
            "/amendments/{id}/status",
            post(handlers::amendment::change_amendment_status),
        )
        // R30-25b：標記 amendment 為 EFFECTIVE（GLP §58 正式生效）
        .route(
            "/amendments/{id}/effective",
            post(handlers::amendment::mark_amendment_effective),
        )
        // P6：完成補登歷史變更（DRAFT → EFFECTIVE）
        .route(
            "/amendments/{id}/finalize-historical",
            post(handlers::amendment::finalize_historical_amendment),
        )
        // P6-3：補登歷史變更審查文件（委員意見，支援院外委員）
        .route(
            "/amendments/{id}/historical-reviews",
            post(handlers::amendment::record_historical_amendment_reviews),
        )
        .route(
            "/amendments/{id}/versions",
            get(handlers::amendment::get_amendment_versions),
        )
        .route(
            "/amendments/{id}/history",
            get(handlers::amendment::get_amendment_history),
        )
        .route(
            "/amendments/{id}/assignments",
            get(handlers::amendment::get_amendment_assignments),
        )
        .route(
            "/protocols/{id}/amendments",
            get(handlers::amendment::list_protocol_amendments),
        )
}
