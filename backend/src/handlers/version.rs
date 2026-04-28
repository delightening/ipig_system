//! R30-22: 內部版本資訊端點
//!
//! GET /api/v1/admin/internal/version（admin-only，需 JWT + 系統管理員角色）
//!
//! 安全考量（對應 health.rs M14 註解）：本端點刻意**不**對外公開於 /api/health，
//! 避免向匿名攻擊者暴露 build 資訊（git_sha / build_time / migration version /
//! rust_version）。僅供已認證的系統管理員（IQ-PQ 驗證、版本對齊用）查詢。
//!
//! Build-time 注入：`GIT_SHA`、`BUILD_TIME`、`RUSTC_VERSION_RUNTIME` 皆透過
//! `option_env!` 讀取；Dockerfile 尚未注入時會 fallback 為 "unknown"，端點仍可
//! 運作（migration_max_version 仍然由 runtime 查 DB 取得）。
//!
//! TODO(R30-22-followup): 在 backend/Dockerfile 與 build script 注入：
//!   ARG GIT_SHA / BUILD_TIME；ENV GIT_SHA=$GIT_SHA BUILD_TIME=$BUILD_TIME
//!   並於 cargo build 前以 build.rs 讀取 env var 寫入 RUSTC_VERSION_RUNTIME。

use axum::{extract::State, Extension, Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{error::AppError, middleware::CurrentUser, AppState, Result};

#[derive(Serialize, ToSchema)]
pub struct VersionResponse {
    /// Build-time 注入的 git commit SHA（fallback "unknown"）
    pub git_sha: &'static str,
    /// Build-time 注入的 ISO 8601 build timestamp（fallback "unknown"）
    pub build_time: &'static str,
    /// Runtime 從 _sqlx_migrations 取得的最大 migration version
    pub migration_max_version: i64,
    /// Build 用的 rustc 版本（fallback "unknown"）
    pub rust_version: &'static str,
}

/// GET /admin/internal/version
///
/// Admin-only。回傳 build / migration / runtime 版本資訊，供 IQ-PQ 驗證、
/// staging vs prod 版本對齊、incident 排查使用。
#[utoipa::path(
    get,
    path = "/api/v1/admin/internal/version",
    responses(
        (status = 200, description = "版本資訊", body = VersionResponse),
        (status = 401, description = "未認證"),
        (status = 403, description = "非系統管理員")
    ),
    tag = "監控"
)]
pub async fn get_internal_version(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<VersionResponse>> {
    if !current_user.is_admin() {
        return Err(AppError::Forbidden(
            "僅系統管理員可查詢內部版本資訊".to_string(),
        ));
    }

    // R30-22: build-time env var，未注入時 fallback "unknown"，端點仍可工作。
    let git_sha = option_env!("GIT_SHA").unwrap_or("unknown");
    let build_time = option_env!("BUILD_TIME").unwrap_or("unknown");
    let rust_version = option_env!("RUSTC_VERSION_RUNTIME").unwrap_or("unknown");

    // Runtime 查詢：sqlx 將 migration version 存為 bigint。
    // 空表（理論上不會發生，啟動期 sqlx::migrate! 已跑）回 0。
    let migration_max_version: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version), 0)::bigint FROM _sqlx_migrations",
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(VersionResponse {
        git_sha,
        build_time,
        migration_max_version,
        rust_version,
    }))
}
