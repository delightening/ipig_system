use axum::{
    routing::post,
    Router,
};

use crate::{handlers, AppState};

/// 檔案上傳路由（套用較嚴格的 upload_rate_limit_middleware）
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/protocols/:id/attachments",
            post(handlers::upload_protocol_attachment),
        )
        .route("/animals/:id/photos", post(handlers::upload_animal_photo))
        .route(
            "/animals/:id/pathology/attachments",
            post(handlers::upload_pathology_report),
        )
        .route(
            "/animals/:id/sacrifice/photos",
            post(handlers::upload_sacrifice_photo),
        )
        .route(
            "/vet-recommendations/:record_type/:record_id/attachments",
            post(handlers::upload_vet_recommendation_attachment),
        )
        .route(
            "/observations/:id/attachments",
            post(handlers::upload_observation_attachment),
        )
        .route(
            "/hr/leaves/attachments",
            post(handlers::upload_leave_attachment),
        )
        .route(
            "/animals/import/basic",
            post(handlers::import_basic_data),
        )
        .route(
            "/animals/import/weights",
            post(handlers::import_weight_data),
        )
        .route(
            "/qau/sop/:id/upload",
            post(handlers::upload_sop_document),
        )
}
