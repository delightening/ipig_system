//! 健康檢查端點 — 用於監控系統與資料庫連通性
//!
//! GET /api/health（公開端點，無需認證）

use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;

use crate::AppState;

/// 健康檢查回應結構
#[derive(Serialize)]
pub struct HealthResponse {
    /// 系統整體狀態：`"healthy"` 或 `"unhealthy"`
    pub status: &'static str,
    /// 應用程式版本（取自 Cargo.toml）
    pub version: &'static str,
    /// 各項子系統健康檢查結果
    pub checks: HealthChecks,
}

/// 各項子系統檢查結果
#[derive(Serialize)]
pub struct HealthChecks {
    pub database: DatabaseCheck,
}

/// 資料庫連通性檢查結果
#[derive(Serialize)]
pub struct DatabaseCheck {
    /// `"up"` 或 `"down"`
    pub status: &'static str,
    /// 查詢延遲（毫秒）
    pub latency_ms: u64,
}

/// 健康檢查 Handler
///
/// - DB 查詢成功 → 200 + `"healthy"`
/// - DB 查詢失敗 → 503 + `"unhealthy"`
pub async fn health_check(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    let start = std::time::Instant::now();

    // 測試資料庫連通性
    let db_result = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await;

    let latency_ms = start.elapsed().as_millis() as u64;

    match db_result {
        Ok(_) => (
            StatusCode::OK,
            Json(HealthResponse {
                status: "healthy",
                version: env!("CARGO_PKG_VERSION"),
                checks: HealthChecks {
                    database: DatabaseCheck {
                        status: "up",
                        latency_ms,
                    },
                },
            }),
        ),
        Err(e) => {
            tracing::warn!("健康檢查失敗：資料庫連線異常 - {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(HealthResponse {
                    status: "unhealthy",
                    version: env!("CARGO_PKG_VERSION"),
                    checks: HealthChecks {
                        database: DatabaseCheck {
                            status: "down",
                            latency_ms,
                        },
                    },
                }),
            )
        }
    }
}
