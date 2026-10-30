use axum::{routing::get, routing::post, Router};

use crate::{handlers, AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/legacy-import/batches",
            post(handlers::legacy_import::create_batch),
        )
        .route(
            "/legacy-import/batches/:batch_id/records",
            post(handlers::legacy_import::ingest_pig_records),
        )
        .route(
            "/legacy-import/batches/:batch_id/report",
            get(handlers::legacy_import::get_batch_report),
        )
}
