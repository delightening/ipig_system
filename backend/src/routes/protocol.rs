use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::{handlers, AppState};

/// AUP 審查系統、審查人員、Co-editor、動物來源、修正申請路由
pub fn routes() -> Router<AppState> {
    Router::new()
        // Protocols
        .route(
            "/protocols",
            get(handlers::list_protocols).post(handlers::create_protocol),
        )
        .route(
            "/protocols/:id",
            get(handlers::get_protocol).put(handlers::update_protocol),
        )
        .route("/protocols/:id/submit", post(handlers::submit_protocol))
        .route(
            "/protocols/:id/status",
            post(handlers::change_protocol_status),
        )
        .route(
            "/protocols/:id/versions",
            get(handlers::get_protocol_versions),
        )
        .route(
            "/protocols/:id/activities",
            get(handlers::get_protocol_activities),
        )
        .route(
            "/protocols/:id/animal-stats",
            get(handlers::get_protocol_animal_stats),
        )
        .route(
            "/protocols/:id/export-pdf",
            get(handlers::export_protocol_pdf),
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
            "/reviews/comments/:id/resolve",
            post(handlers::resolve_review_comment),
        )
        .route(
            "/reviews/comments/reply",
            post(handlers::reply_review_comment),
        )
        // Draft Reply
        .route("/reviews/comments/draft", post(handlers::save_reply_draft))
        .route(
            "/reviews/comments/:id/draft",
            get(handlers::get_reply_draft),
        )
        .route(
            "/reviews/comments/submit-draft",
            post(handlers::submit_reply_from_draft),
        )
        // Vet Review Form
        .route("/reviews/vet-form", post(handlers::save_vet_review_form))
        // Co-Editor Assignment
        .route(
            "/protocols/:id/co-editors",
            get(handlers::list_co_editors).post(handlers::assign_co_editor),
        )
        .route(
            "/protocols/:id/co-editors/:user_id",
            delete(handlers::remove_co_editor),
        )
        .route(
            "/protocols/:id/co-editors/:user_id/delete",
            post(handlers::remove_co_editor),
        )
        // My Projects
        .route("/my-projects", get(handlers::get_my_protocols))
        // Animal Sources
        .route(
            "/animal-sources",
            get(handlers::list_animal_sources).post(handlers::create_animal_source),
        )
        .route(
            "/animal-sources/:id",
            put(handlers::update_animal_source).delete(handlers::delete_animal_source),
        )
        .route(
            "/animal-sources/:id/delete",
            post(handlers::delete_animal_source),
        )
        // Amendments (變更申請系統)
        .route(
            "/amendments",
            get(handlers::amendment::list_amendments)
                .post(handlers::amendment::create_amendment),
        )
        .route(
            "/amendments/pending-count",
            get(handlers::amendment::get_pending_count),
        )
        .route(
            "/amendments/:id",
            get(handlers::amendment::get_amendment)
                .patch(handlers::amendment::update_amendment),
        )
        .route(
            "/amendments/:id/submit",
            post(handlers::amendment::submit_amendment),
        )
        .route(
            "/amendments/:id/classify",
            post(handlers::amendment::classify_amendment),
        )
        .route(
            "/amendments/:id/start-review",
            post(handlers::amendment::start_amendment_review),
        )
        .route(
            "/amendments/:id/decision",
            post(handlers::amendment::record_amendment_decision),
        )
        .route(
            "/amendments/:id/status",
            post(handlers::amendment::change_amendment_status),
        )
        .route(
            "/amendments/:id/versions",
            get(handlers::amendment::get_amendment_versions),
        )
        .route(
            "/amendments/:id/history",
            get(handlers::amendment::get_amendment_history),
        )
        .route(
            "/amendments/:id/assignments",
            get(handlers::amendment::get_amendment_assignments),
        )
        .route(
            "/protocols/:id/amendments",
            get(handlers::amendment::list_protocol_amendments),
        )
}
