//! R30-3a Transactional Event Outbox.
//!
//! Decouples「業務 tx commit」from「外部訊息送達」(email / line / webhook):
//! - 業務 tx 內 `enqueue_tx` 寫一筆 PENDING row（< 1ms，無外部 I/O）
//! - 獨立 worker (`bin/outbox_worker.rs`) 後續 poll + send + retry + dead-letter
//!
//! 狀態機（每個 transition 唯一 caller，避免競態）：
//! - `enqueue_tx` → PENDING
//! - `claim_batch` (worker): PENDING/FAILED → SENDING（`FOR UPDATE SKIP LOCKED`）
//! - `mark_done` (worker): SENDING → DONE
//! - `mark_failed` (worker): SENDING → FAILED 或 DEAD（依 attempt_count）
//! - `reset_stuck` (cron): SENDING (>10min) → PENDING
//!
//! Design doc: `docs/design/r30-3-event-outbox.md`

mod adapter;
mod email_adapter;
#[cfg(test)]
mod tests;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{middleware::ActorContext, Result};

pub use adapter::{ChannelAdapter, ChannelRegistry, OutboxStatus};
pub use email_adapter::EmailAdapter;

/// `last_error` 寫入長度上限（DB 端 LEFT 截斷防無界增長）。
const MAX_LAST_ERROR_LEN: i32 = 2000;

/// 一筆 outbox 事件（worker 取出後傳給 ChannelAdapter::send）。
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OutboxEvent {
    pub id: Uuid,
    pub channel: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub attempt_count: i32,
    pub next_attempt_at: DateTime<Utc>,
    pub last_error: Option<String>,
    pub enqueued_by: Option<Uuid>,
    pub enqueued_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub done_at: Option<DateTime<Utc>>,
    pub source_entity: Option<String>,
    pub source_entity_id: Option<Uuid>,
}

pub struct OutboxService;

impl OutboxService {
    /// 在現有 tx 內排隊一筆事件。commit 失敗 → 整批 rollback（含 outbox row）。
    /// 回傳 outbox row id 供 caller 記到 audit / 後續查詢。
    pub async fn enqueue_tx(
        tx: &mut Transaction<'_, Postgres>,
        actor: &ActorContext,
        channel: &str,
        payload: serde_json::Value,
        source: (&str, Uuid),
    ) -> Result<Uuid> {
        let enqueued_by = actor.user_id();
        let id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO event_outbox
                (channel, payload, enqueued_by, source_entity, source_entity_id)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
        .bind(channel)
        .bind(&payload)
        .bind(enqueued_by)
        .bind(source.0)
        .bind(source.1)
        .fetch_one(&mut **tx)
        .await?;

        tracing::debug!(
            outbox_id = %id,
            channel,
            source_entity = source.0,
            source_entity_id = %source.1,
            "outbox: enqueued"
        );
        Ok(id)
    }

    /// 非交易版排隊（autocommit）。供「不在業務 tx 內」的寄送點使用
    /// （多數通知 / scheduler 寄信點）。`next_attempt_at` 可帶未來時間以延後寄送
    /// （員工通知時間窗外 → 排到下一個合法窗口）；`None` 表立即可取（DB default NOW）。
    pub async fn enqueue(
        pool: &PgPool,
        actor: &ActorContext,
        channel: &str,
        payload: serde_json::Value,
        source: (&str, Uuid),
        next_attempt_at: Option<DateTime<Utc>>,
    ) -> Result<Uuid> {
        let enqueued_by = actor.user_id();
        let id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO event_outbox
                (channel, payload, enqueued_by, source_entity, source_entity_id, next_attempt_at)
            VALUES ($1, $2, $3, $4, $5, COALESCE($6, NOW()))
            RETURNING id
            "#,
        )
        .bind(channel)
        .bind(&payload)
        .bind(enqueued_by)
        .bind(source.0)
        .bind(source.1)
        .bind(next_attempt_at)
        .fetch_one(pool)
        .await?;

        tracing::debug!(
            outbox_id = %id,
            channel,
            source_entity = source.0,
            source_entity_id = %source.1,
            scheduled = ?next_attempt_at,
            "outbox: enqueued (non-tx)"
        );
        Ok(id)
    }

    /// Worker 用：原子 claim 一批待處理事件並標記 SENDING。
    ///
    /// CTE + `FOR UPDATE SKIP LOCKED`：多 worker 互斥；單一 round trip。
    /// `attempt_count` 不在 claim 時遞增 — 只在 mark_failed 時遞增。
    pub async fn claim_batch(pool: &PgPool, limit: i32) -> Result<Vec<OutboxEvent>> {
        let events = sqlx::query_as::<_, OutboxEvent>(
            r#"
            WITH claimed AS (
                SELECT id
                FROM event_outbox
                WHERE status IN ($2, $3)
                  AND next_attempt_at <= NOW()
                ORDER BY next_attempt_at
                LIMIT $1
                FOR UPDATE SKIP LOCKED
            )
            UPDATE event_outbox o
            SET status = $4,
                started_at = NOW()
            FROM claimed
            WHERE o.id = claimed.id
            RETURNING o.*
            "#,
        )
        .bind(limit)
        .bind(OutboxStatus::Pending.as_str())
        .bind(OutboxStatus::Failed.as_str())
        .bind(OutboxStatus::Sending.as_str())
        .fetch_all(pool)
        .await?;
        Ok(events)
    }

    /// Worker 用：標記成功 → DONE。CAS guard `status = SENDING` 防被其他 caller 搶。
    pub async fn mark_done(pool: &PgPool, id: Uuid) -> Result<()> {
        let rows = sqlx::query(
            r#"
            UPDATE event_outbox
            SET status = $2,
                done_at = NOW(),
                last_error = NULL
            WHERE id = $1 AND status = $3
            "#,
        )
        .bind(id)
        .bind(OutboxStatus::Done.as_str())
        .bind(OutboxStatus::Sending.as_str())
        .execute(pool)
        .await?
        .rows_affected();

        if rows == 0 {
            tracing::warn!(outbox_id = %id, "outbox mark_done: row not in SENDING state");
        }
        Ok(())
    }

    /// Worker 用：標記失敗 + 算下次嘗試時間 + dead-letter 判斷。
    ///
    /// 流程：先 attempt_count = attempt_count + 1，再依新值算 next_attempt_at：
    /// - 1 → +10s, 2 → +1m, 3 → +10m, 4 → +1h, 5 → +6h（最後一次重試窗口）
    /// - 6（=已失敗 6 次）→ 狀態變 DEAD，不再排
    pub async fn mark_failed(pool: &PgPool, id: Uuid, error: &str) -> Result<()> {
        let mut tx = pool.begin().await?;
        let Some(prev_attempt) = load_locked_sending_attempt(&mut tx, id).await? else {
            tx.rollback().await?;
            return Ok(());
        };

        let new_attempt = prev_attempt + 1;
        let (new_status, next_at) = compute_next_attempt(new_attempt);

        write_failed_state_tx(&mut tx, id, new_status, new_attempt, next_at, error).await?;
        tx.commit().await?;

        log_failure(id, new_status, new_attempt, next_at, error);
        Ok(())
    }

    /// Cron 用：把卡 `SENDING` 超過 10 分鐘的 row 重設回 `PENDING`，讓 worker 重取。
    /// 場景：worker process crash / OOM / kill -9 而沒走 mark_done/failed 路徑。
    pub async fn reset_stuck(pool: &PgPool) -> Result<u64> {
        let rows = sqlx::query(
            r#"
            UPDATE event_outbox
            SET status = $1,
                started_at = NULL,
                last_error = LEFT(COALESCE(last_error, '') ||
                    CASE WHEN COALESCE(last_error,'')='' THEN '' ELSE ' | ' END ||
                    'reset from stuck SENDING at ' || TO_CHAR(NOW(), 'YYYY-MM-DD HH24:MI:SS'), $3)
            WHERE status = $2
              AND started_at < NOW() - INTERVAL '10 minutes'
            "#,
        )
        .bind(OutboxStatus::Pending.as_str())
        .bind(OutboxStatus::Sending.as_str())
        .bind(MAX_LAST_ERROR_LEN)
        .execute(pool)
        .await?
        .rows_affected();

        if rows > 0 {
            tracing::warn!(rows, "outbox: reset stuck SENDING events back to PENDING");
        }
        Ok(rows)
    }
}

// ============================================================
// mark_failed helpers — 拆出讓主 fn ≤50 行（CLAUDE.md §2）
// ============================================================

/// SELECT FOR UPDATE 鎖 row + 守 status='SENDING'，回傳當前 attempt_count。
/// 不在 SENDING（已被別 caller 搶或不存在）→ 回 None，caller 應 rollback + skip。
async fn load_locked_sending_attempt(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<Option<i32>> {
    let row: Option<(i32, String)> =
        sqlx::query_as("SELECT attempt_count, status FROM event_outbox WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut **tx)
            .await?;

    match row {
        None => {
            tracing::warn!(outbox_id = %id, "outbox mark_failed: row not found");
            Ok(None)
        }
        Some((_, status)) if status != OutboxStatus::Sending.as_str() => {
            tracing::warn!(
                outbox_id = %id,
                status,
                "outbox mark_failed: row not in SENDING state, skipping"
            );
            Ok(None)
        }
        Some((attempt, _)) => Ok(Some(attempt)),
    }
}

/// 寫 FAILED / DEAD 狀態 + 截斷 last_error 至 MAX_LAST_ERROR_LEN，含 status='SENDING' CAS guard。
async fn write_failed_state_tx(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    new_status: OutboxStatus,
    new_attempt: i32,
    next_at: DateTime<Utc>,
    error: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE event_outbox
        SET status = $2,
            attempt_count = $3,
            next_attempt_at = $4,
            last_error = LEFT($5, $6)
        WHERE id = $1 AND status = $7
        "#,
    )
    .bind(id)
    .bind(new_status.as_str())
    .bind(new_attempt)
    .bind(next_at)
    .bind(error)
    .bind(MAX_LAST_ERROR_LEN)
    .bind(OutboxStatus::Sending.as_str())
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// mark_failed 後置 log（DEAD 走 error 等級，FAILED 走 warn）。
fn log_failure(
    id: Uuid,
    new_status: OutboxStatus,
    new_attempt: i32,
    next_at: DateTime<Utc>,
    error: &str,
) {
    if new_status == OutboxStatus::Dead {
        tracing::error!(
            outbox_id = %id,
            attempt_count = new_attempt,
            error,
            "outbox: event moved to DEAD after exhausting retries"
        );
    } else {
        tracing::warn!(
            outbox_id = %id,
            attempt_count = new_attempt,
            next_attempt_at = %next_at,
            error,
            "outbox: event marked FAILED, will retry"
        );
    }
}

/// Retry 表：依「失敗後 attempt_count 值」決定下次嘗試時間或 DEAD。
/// 詳見 design doc §「Retry 策略」。
fn compute_next_attempt(new_attempt_count: i32) -> (OutboxStatus, DateTime<Utc>) {
    let now = Utc::now();
    match new_attempt_count {
        1 => (OutboxStatus::Failed, now + Duration::seconds(10)),
        2 => (OutboxStatus::Failed, now + Duration::minutes(1)),
        3 => (OutboxStatus::Failed, now + Duration::minutes(10)),
        4 => (OutboxStatus::Failed, now + Duration::hours(1)),
        5 => (OutboxStatus::Failed, now + Duration::hours(6)),
        // 6 次失敗（已包含首次 + 5 retry）→ 終態 DEAD
        _ => (OutboxStatus::Dead, now),
    }
}

/// ActorContext 的 user_id 提取 helper（給 enqueue_tx 用）；
/// System / Anonymous 視為無人類發起者（NULL → DB FK 允許）。
trait ActorIdExt {
    fn user_id(&self) -> Option<Uuid>;
}

impl ActorIdExt for ActorContext {
    fn user_id(&self) -> Option<Uuid> {
        match self {
            ActorContext::User(u) => Some(u.id),
            ActorContext::System { .. } => Some(crate::SYSTEM_USER_ID),
            ActorContext::Anonymous => None,
        }
    }
}
