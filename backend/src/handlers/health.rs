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
    /// R28-M5：Prometheus metrics recorder 狀態。
    /// "up" = 正常；"down" = `install_recorder()` 失敗，所有 metrics::counter!
    /// / gauge! / histogram! 呼叫變 NoopRecorder 靜默掉，無 ops 可觀測。
    /// degraded 由健檢回 503，方便 Prometheus / liveness probe 立即偵測。
    pub metrics: ComponentCheck,
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

    // R28-M5：Prometheus metrics recorder 健檢
    let metrics_up = state.metrics_handle.is_some();
    let metrics = ComponentCheck {
        status: if metrics_up { "up" } else { "down" },
    };

    let all_ok = db_result.is_ok() && pool_healthy && uploads_exists && metrics_up;

    if !all_ok {
        if db_result.is_err() {
            tracing::warn!("健康檢查失敗：資料庫連線異常（latency={}ms）", latency_ms);
        }
        if !pool_healthy {
            tracing::warn!(
                "健康檢查警告：連線池飽和 (active={}, idle={}, size={})",
                pool_active,
                pool_idle,
                pool_size
            );
        }
        if !uploads_exists {
            tracing::warn!("健康檢查警告：uploads 目錄不存在");
        }
        if !metrics_up {
            tracing::warn!(
                "健康檢查警告：Prometheus metrics recorder 未啟動，所有 metrics::* 呼叫被靜默"
            );
        }
    }

    // PR6 (方案 I, #238)：容器 / liveness 健檢只認 DB——DB up = 200（即使 metrics / disk /
    // pool degraded），DB down 才回 503。觀測層降級仍反映在 body.status="degraded" 與
    // checks 子物件（供 dashboard / 告警觀察），但不致 docker HEALTHCHECK 把容器標
    // unhealthy 而連帶拖垮 depends_on 的下游服務。
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
                metrics,
            },
        }),
    )
}

// ============================================================================
// GLP 匯出 pre-check：print-pdf 服務存活探測 + 失聯時自動 email 通知
// ============================================================================

/// In-process「print-pdf 服務未上線」email rate limit（30 分鐘最多 1 封）。
/// 重啟 backend 會 reset；避免使用者連 click 觸發 email storm。
static PDF_SERVICE_DOWN_LAST_ALERT: std::sync::Mutex<Option<std::time::Instant>> =
    std::sync::Mutex::new(None);
const PDF_SERVICE_DOWN_ALERT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1800);

/// 代理 print-pdf `/pdf-service-health`，frontend 用來決定 GLP 匯出是否可進行
/// （print-pdf 未上線 → 回 `glp_ready:false` → 前端 disable 匯出按鈕）。
/// 服務未上線時觸發 admin email 通知（rate-limited）。
pub async fn pdf_service_health(
    State(state): State<AppState>,
) -> (StatusCode, Json<serde_json::Value>) {
    let (body, ok) = match state.pdf_service.liveness().await {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!(error=%e, "pdf-service-health proxy failed");
            maybe_alert_pdf_service_down(
                state.clone(),
                Some(serde_json::json!({ "error": format!("unreachable: {e}") })),
                "pdf-service-health",
            );
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "glp_ready": false,
                    "error": format!("pdf-service unreachable: {e}"),
                })),
            );
        }
    };

    if !ok {
        maybe_alert_pdf_service_down(state.clone(), Some(body.clone()), "pdf-service-health");
    }

    let code = if ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (code, Json(body))
}

/// 可重複呼叫的「print-pdf 服務未上線」email 觸發點。
/// rate-limited 30 分鐘；非同步 spawn 不阻塞 caller。
///
/// 觸發源：`pdf-service-health` endpoint（GLP 匯出 pre-check）偵測到 print-pdf
/// 未上線或回報未就緒時。`payload` 未提供時 spawn 內會自己 ping 取現況。
pub fn maybe_alert_pdf_service_down(
    state: AppState,
    payload: Option<serde_json::Value>,
    source: &'static str,
) {
    // poisoned mutex 不該發生（hold time 極短、無 panic），保險起見 recover
    let mut last = PDF_SERVICE_DOWN_LAST_ALERT
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let should_alert = match *last {
        None => true,
        Some(t) => t.elapsed() >= PDF_SERVICE_DOWN_ALERT_INTERVAL,
    };
    if !should_alert {
        return;
    }
    *last = Some(std::time::Instant::now());
    drop(last);

    // 非同步發 email（不阻塞 caller）
    tokio::spawn(async move {
        let body = match payload {
            Some(p) => p,
            None => {
                // 沒提供細節 → 自己 ping print-pdf 取現況
                match state.pdf_service.liveness().await {
                    Ok((b, _)) => b,
                    Err(e) => {
                        tracing::warn!(error=%e, source, "alert: ping pdf-service-health failed");
                        serde_json::json!({ "error": format!("ping failed: {e}") })
                    }
                }
            }
        };
        if let Err(e) = send_pdf_service_down_alert(&state, &body, source).await {
            tracing::error!(error=%e, source, "Failed to send pdf-service-down alert email");
        }
    });
}

/// 寫「print-pdf 服務未上線」警告 email 進 outbox。收件者：第一個 active admin 的 email。
async fn send_pdf_service_down_alert(
    state: &AppState,
    payload: &serde_json::Value,
    source: &str,
) -> Result<(), crate::error::AppError> {
    // 找第一個 active admin email
    let admin_email: Option<String> = sqlx::query_scalar(
        "SELECT email FROM users WHERE role = 'ADMIN' AND is_active = true AND deleted_at IS NULL ORDER BY created_at LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| crate::error::AppError::Internal(format!("query admin: {e}")))?;

    let Some(to) = admin_email else {
        tracing::warn!("no active admin found; daemon-down alert dropped");
        return Ok(());
    };

    // Gemini #531：print-pdf/FastAPI 非 2xx 錯誤體用 `detail` key，自構 payload 用 `error`，兩者都試。
    let detail = payload
        .get("error")
        .or_else(|| payload.get("detail"))
        .and_then(|v| v.as_str())
        .unwrap_or("(無細節)");

    let subject = "[iPig 告警] PDF 服務（print-pdf）未上線";
    let body = format!(
        "iPig 系統偵測到 print-pdf（PDF 渲染服務）未上線或回報未就緒。\n\n\
         影響：GLP 文件匯出（計畫書 / 審查回覆 / 審核結果 / 巡場報告 / 欄位狀態表）將無法產生 PDF。\n\
         前端已自動 disable 相關匯出按鈕。\n\n\
         時間：{ts}\n\
         觸發來源：{source}\n\
         細節：{detail}\n\n\
         本通知每 30 分鐘最多一封，避免 spam。\n\
         處置：(1) 確認 print-pdf container 仍 Running（docker compose ps）(2) 必要時 docker compose up -d print-pdf 重啟",
        ts = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
    );

    // 透過 outbox enqueue 一筆 email channel event
    let payload = serde_json::json!({
        "to": to,
        "subject": subject,
        "plain_body": body,
        "html_body": format!("<pre style=\"font-family: monospace;\">{}</pre>", body.replace('<', "&lt;")),
    });
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| crate::error::AppError::Internal(format!("tx: {e}")))?;
    crate::services::outbox::OutboxService::enqueue_tx(
        &mut tx,
        &crate::middleware::actor::ActorContext::System {
            reason: "daemon-health alert",
        },
        "email",
        payload,
        ("pdf_service_health", uuid::Uuid::new_v4()),
    )
    .await?;
    tx.commit()
        .await
        .map_err(|e| crate::error::AppError::Internal(format!("commit: {e}")))?;
    tracing::info!(to=%to, "daemon-down alert email queued");
    Ok(())
}
