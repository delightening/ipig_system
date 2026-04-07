//! 健康檢查端點 — 用於監控系統與資料庫連通性
//!
//! GET /api/health（公開端點，無需認證）

use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::AppState;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
    // M14: 移除版本號（避免向攻擊者暴露 build 資訊）
    pub checks: HealthChecks,
}

#[derive(Serialize, ToSchema)]
pub struct HealthChecks {
    pub database: ComponentCheck,
    // M14: 移除 db_pool（size/idle/active 暴露連線池細節）
    pub disk: DiskCheck,
}

#[derive(Serialize, ToSchema)]
pub struct ComponentCheck {
    pub status: &'static str,
    // M14: 移除 latency_ms（可用於探測 DB 回應時序）
}

#[derive(Serialize, ToSchema)]
pub struct DiskCheck {
    pub status: &'static str,
}

#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "健康檢查正常", body = HealthResponse),
        (status = 503, description = "服務 degraded（DB 連線失敗）", body = HealthResponse)
    ),
    tag = "監控"
)]
pub async fn health_check(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    let start = std::time::Instant::now();

    let db_result = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await;
    let latency_ms = start.elapsed().as_millis() as u64;

    // M14: 不在回應中暴露 latency_ms（可用於時序探測），只記錄 log
    let database = match &db_result {
        Ok(_) => ComponentCheck { status: "up" },
        Err(_) => ComponentCheck { status: "down" },
    };

    // M14: 移除 pool 細節，只在內部用於判斷是否 degraded
    let pool_size = state.db.size();
    let pool_idle = state.db.num_idle() as u32;
    let pool_active = pool_size.saturating_sub(pool_idle);
    let pool_healthy = pool_idle > 0 || pool_size < state.db.options().get_max_connections();

    let uploads_path = std::path::Path::new("./uploads");
    let uploads_exists = uploads_path.exists() && uploads_path.is_dir();
    let disk = DiskCheck {
        status: if uploads_exists { "ok" } else { "missing" },
    };

    let all_ok = db_result.is_ok() && pool_healthy && uploads_exists;

    if !all_ok {
        if db_result.is_err() {
            tracing::warn!("健康檢查失敗：資料庫連線異常（latency={}ms）", latency_ms);
        }
        if !pool_healthy {
            tracing::warn!("健康檢查警告：連線池飽和 (active={}, idle={}, size={})", pool_active, pool_idle, pool_size);
        }
        if !uploads_exists {
            tracing::warn!("健康檢查警告：uploads 目錄不存在");
        }
    }

    let status_code = if db_result.is_ok() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(HealthResponse {
            status: if all_ok { "healthy" } else { "degraded" },
            // M14: version 與 db_pool 已移除，不對外暴露敏感系統資訊
            checks: HealthChecks {
                database,
                disk,
            },
        }),
    )
}
