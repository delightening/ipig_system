//! R30-3a Outbox Worker — independent binary that polls `event_outbox` and
//! ships events through registered ChannelAdapters.
//!
//! Lifecycle:
//! 1. Load Config + open DB pool + AuditService HMAC key (lib re-uses these)
//! 2. Build ChannelRegistry with all v1 adapters (currently: email)
//! 3. Loop: every 5s call `claim_batch` then process the batch concurrently
//!    via `for_each_concurrent` (CONCURRENCY = 10), marking each row done/failed
//! 4. Every minute call `OutboxService::reset_stuck` to recover crashed workers
//! 5. SIGINT / SIGTERM → CancellationToken cancelled → in-flight batch finishes
//!    → process exits cleanly
//!
//! Deploy: independent container `Dockerfile.outbox-worker`. Multiple replicas
//! are safe — `claim_batch` uses `FOR UPDATE SKIP LOCKED`.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use erp_backend::{
    config::Config,
    services::{AuditService, ChannelRegistry, EmailAdapter, OutboxService},
};
use futures::stream::StreamExt;
use sqlx::postgres::PgPoolOptions;
use tokio_util::sync::CancellationToken;

const POLL_INTERVAL: Duration = Duration::from_secs(5);
const STUCK_RESET_INTERVAL: Duration = Duration::from_secs(60);
const BATCH_LIMIT: i32 = 10;
const CONCURRENCY: usize = 10;

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();
    tracing::info!("outbox_worker: starting");

    let (pool, registry, cancel) = init_runtime().await?;
    tokio::spawn(stuck_reset_loop(pool.clone(), cancel.clone()));
    run_loop(pool, registry, cancel).await;

    tracing::info!("outbox_worker: graceful shutdown complete");
    Ok(())
}

fn init_logging() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
}

/// 載入 Config + 建 pool + 註冊 adapters + signal handlers。
async fn init_runtime() -> Result<(sqlx::PgPool, Arc<ChannelRegistry>, CancellationToken)> {
    let config = Config::from_env().context("loading config from env")?;
    // pool >= CONCURRENCY + reset_stuck cron + mark_failed tx 同時 — 預留 5 個緩衝
    let pool = PgPoolOptions::new()
        .max_connections(15)
        .connect(&config.database_url)
        .await
        .context("connecting to database")?;
    AuditService::init_hmac_key(config.audit_hmac_key.clone());

    let registry =
        Arc::new(ChannelRegistry::new().register(EmailAdapter::new(pool.clone(), config.clone())));
    tracing::info!(
        registered_channels = ?registry.registered_channels(),
        "outbox_worker: ChannelRegistry ready"
    );

    let cancel = CancellationToken::new();
    install_signal_handlers(cancel.clone());
    Ok((pool, registry, cancel))
}

/// 主 polling loop：每 POLL_INTERVAL 取一批 + 並行送。
async fn run_loop(pool: sqlx::PgPool, registry: Arc<ChannelRegistry>, cancel: CancellationToken) {
    let mut ticker = tokio::time::interval(POLL_INTERVAL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                tracing::info!("outbox_worker: shutdown signal received, exiting loop");
                break;
            }
            _ = ticker.tick() => {}
        }
        if let Err(e) = process_batch(&pool, registry.clone()).await {
            tracing::error!(error = %e, "outbox_worker: batch processing failed (non-fatal)");
        }
    }
}

async fn process_batch(pool: &sqlx::PgPool, registry: Arc<ChannelRegistry>) -> Result<()> {
    let batch = OutboxService::claim_batch(pool, BATCH_LIMIT)
        .await
        .context("claim_batch")?;

    if batch.is_empty() {
        return Ok(());
    }

    tracing::debug!(count = batch.len(), "outbox_worker: claimed batch");

    futures::stream::iter(batch)
        .for_each_concurrent(CONCURRENCY, |event| {
            let pool = pool.clone();
            let registry = registry.clone();
            async move {
                let event_id = event.id;
                let result = registry.send(&event).await;
                let mark = match result {
                    Ok(_) => {
                        tracing::info!(
                            outbox_id = %event_id,
                            channel = event.channel,
                            "outbox: event sent"
                        );
                        OutboxService::mark_done(&pool, event_id).await
                    }
                    Err(e) => OutboxService::mark_failed(&pool, event_id, &e.to_string()).await,
                };
                if let Err(e) = mark {
                    tracing::error!(
                        outbox_id = %event_id,
                        error = %e,
                        "outbox: mark_done/failed write itself failed; reset_stuck will recover"
                    );
                }
            }
        })
        .await;

    Ok(())
}

async fn stuck_reset_loop(pool: sqlx::PgPool, cancel: CancellationToken) {
    let mut ticker = tokio::time::interval(STUCK_RESET_INTERVAL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            _ = ticker.tick() => {}
        }
        match OutboxService::reset_stuck(&pool).await {
            Ok(0) => {}
            Ok(n) => tracing::warn!(rows = n, "outbox: reset_stuck recovered events"),
            Err(e) => tracing::error!(error = %e, "outbox: reset_stuck failed"),
        }
    }
}

#[cfg(unix)]
fn install_signal_handlers(cancel: CancellationToken) {
    use tokio::signal::unix::{signal, SignalKind};
    tokio::spawn(async move {
        let mut sigint = signal(SignalKind::interrupt()).expect("install SIGINT handler");
        let mut sigterm = signal(SignalKind::terminate()).expect("install SIGTERM handler");
        tokio::select! {
            _ = sigint.recv() => tracing::info!("outbox_worker: SIGINT received"),
            _ = sigterm.recv() => tracing::info!("outbox_worker: SIGTERM received"),
        }
        cancel.cancel();
    });
}

#[cfg(not(unix))]
fn install_signal_handlers(cancel: CancellationToken) {
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            tracing::info!("outbox_worker: ctrl_c received");
        }
        cancel.cancel();
    });
}
