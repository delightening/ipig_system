// 獸醫巡場報告 Service

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::audit_diff::DataDiff,
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    AppError, Result,
};

// ── 報告主表 ──────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VetPatrolReport {
    pub id: Uuid,
    pub patrol_date: NaiveDate,
    pub week_start: Option<NaiveDate>,
    pub week_end: Option<NaiveDate>,
    pub status: String,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// R26-9: 巡場報告含觀察、建議、後續追蹤等醫療紀錄內容，為 GLP 研究資料本身，
// 需完整保留於 audit log。空 `redacted_fields()` 是主動決策。
impl crate::models::audit_diff::AuditRedact for VetPatrolReport {}

// ── Audit snapshot：包含 report + entries（GLP 醫療紀錄完整保留） ──────────────────────────────

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct VetPatrolEntrySnapshot {
    pub id: Uuid,
    pub category: String,
    pub animal_id: Option<Uuid>,
    pub observation: String,
    pub suggestion: String,
    pub follow_up: String,
    pub sort_order: i32,
}

impl crate::models::audit_diff::AuditRedact for VetPatrolEntrySnapshot {}

#[derive(Debug, Clone, Serialize)]
pub struct VetPatrolReportSnapshot {
    #[serde(flatten)]
    pub report: VetPatrolReport,
    pub entries: Vec<VetPatrolEntrySnapshot>,
}

impl crate::models::audit_diff::AuditRedact for VetPatrolReportSnapshot {}

async fn fetch_entry_snapshots(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    report_id: Uuid,
) -> Result<Vec<VetPatrolEntrySnapshot>> {
    let entries = sqlx::query_as::<_, VetPatrolEntrySnapshot>(
        r#"SELECT id, category, animal_id, observation, suggestion, follow_up, sort_order
           FROM vet_patrol_entries
           WHERE report_id = $1
           ORDER BY sort_order, created_at"#,
    )
    .bind(report_id)
    .fetch_all(&mut **tx)
    .await?;
    Ok(entries)
}

// ── 含耳號的條目（join animals） ──────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VetPatrolEntryWithAnimal {
    pub id: Uuid,
    pub report_id: Uuid,
    pub category: String,
    pub animal_id: Option<Uuid>,
    #[sqlx(default)]
    pub ear_tag: Option<String>,
    pub observation: String,
    pub suggestion: String,
    pub follow_up: String,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

// ── 報告 + 條目合併回應 ──────────────────────────────

#[derive(Debug, Serialize)]
pub struct VetPatrolReportWithEntries {
    #[serde(flatten)]
    pub report: VetPatrolReport,
    pub entries: Vec<VetPatrolEntryWithAnimal>,
}

// ── 請求 ──────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateVetPatrolReportRequest {
    pub patrol_date: NaiveDate,
    pub week_start: Option<NaiveDate>,
    pub week_end: Option<NaiveDate>,
    pub entries: Vec<CreateVetPatrolEntryRequest>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVetPatrolReportRequest {
    pub patrol_date: Option<NaiveDate>,
    pub week_start: Option<NaiveDate>,
    pub week_end: Option<NaiveDate>,
    pub entries: Option<Vec<CreateVetPatrolEntryRequest>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVetPatrolEntryRequest {
    pub category: String,
    pub animal_id: Option<Uuid>,
    pub observation: String,
    pub suggestion: String,
    pub follow_up: String,
    pub sort_order: Option<i32>,
}

pub struct VetPatrolReportService;

impl VetPatrolReportService {
    /// 列出所有巡場報告（不含條目）
    pub async fn list(pool: &PgPool) -> Result<Vec<VetPatrolReport>> {
        let reports = sqlx::query_as::<_, VetPatrolReport>(
            r#"SELECT id, patrol_date, week_start, week_end, status,
                      created_by, updated_by, created_at, updated_at
               FROM vet_patrol_reports
               WHERE deleted_at IS NULL
               ORDER BY patrol_date DESC, created_at DESC"#,
        )
        .fetch_all(pool)
        .await?;
        Ok(reports)
    }

    /// 取得單一報告（含條目 + 耳號）
    pub async fn get(pool: &PgPool, id: Uuid) -> Result<Option<VetPatrolReportWithEntries>> {
        let report = sqlx::query_as::<_, VetPatrolReport>(
            r#"SELECT id, patrol_date, week_start, week_end, status,
                      created_by, updated_by, created_at, updated_at
               FROM vet_patrol_reports
               WHERE id = $1 AND deleted_at IS NULL"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        let Some(report) = report else {
            return Ok(None);
        };

        let entries = sqlx::query_as::<_, VetPatrolEntryWithAnimal>(
            r#"SELECT e.id, e.report_id, e.category, e.animal_id,
                      a.ear_tag,
                      e.observation, e.suggestion, e.follow_up,
                      e.sort_order, e.created_at
               FROM vet_patrol_entries e
               LEFT JOIN animals a ON a.id = e.animal_id
               WHERE e.report_id = $1
               ORDER BY e.sort_order, e.created_at"#,
        )
        .bind(report.id)
        .fetch_all(pool)
        .await?;

        Ok(Some(VetPatrolReportWithEntries { report, entries }))
    }

    /// 建立巡場報告（含條目）+ 自動同步到動物獸醫師建議
    pub async fn create(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateVetPatrolReportRequest,
    ) -> Result<VetPatrolReport> {
        let user_id = actor.require_user()?.id;
        let mut tx = pool.begin().await?;

        let report = sqlx::query_as::<_, VetPatrolReport>(
            r#"INSERT INTO vet_patrol_reports (patrol_date, week_start, week_end, created_by, updated_by)
               VALUES ($1, $2, $3, $4, $4)
               RETURNING id, patrol_date, week_start, week_end, status,
                         created_by, updated_by, created_at, updated_at"#,
        )
        .bind(req.patrol_date)
        .bind(req.week_start)
        .bind(req.week_end)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;

        for (i, entry) in req.entries.iter().enumerate() {
            sqlx::query(
                r#"INSERT INTO vet_patrol_entries
                       (report_id, category, animal_id, observation, suggestion, follow_up, sort_order)
                   VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            )
            .bind(report.id)
            .bind(&entry.category)
            .bind(entry.animal_id)
            .bind(&entry.observation)
            .bind(&entry.suggestion)
            .bind(&entry.follow_up)
            .bind(entry.sort_order.unwrap_or(i as i32))
            .execute(&mut *tx)
            .await?;

            // 自動同步到動物獸醫師建議
            if let Some(animal_id) = entry.animal_id {
                if !entry.observation.is_empty() || !entry.suggestion.is_empty() {
                    let treatment = if entry.suggestion.is_empty() {
                        entry.follow_up.clone()
                    } else {
                        entry.suggestion.clone()
                    };
                    sqlx::query(
                        r#"INSERT INTO animal_vet_advice_records
                               (animal_id, advice_date, observation, suggested_treatment, created_by, updated_by)
                           VALUES ($1, $2, $3, $4, $5, $5)"#,
                    )
                    .bind(animal_id)
                    .bind(req.patrol_date)
                    .bind(&entry.observation)
                    .bind(&treatment)
                    .bind(user_id)
                    .execute(&mut *tx)
                    .await?;
                }
            }
        }

        let entry_snapshots = fetch_entry_snapshots(&mut tx, report.id).await?;
        let snapshot = VetPatrolReportSnapshot {
            report: report.clone(),
            entries: entry_snapshots,
        };
        let display = format!("巡場報告 {}", report.patrol_date);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "VET_PATROL_REPORT_CREATED",
                entity: Some(AuditEntity::new("vet_patrol_reports", report.id, &display)),
                data_diff: Some(DataDiff::create_only(&snapshot)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(report)
    }

    /// 更新巡場報告（條目全部替換）
    pub async fn update(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateVetPatrolReportRequest,
    ) -> Result<VetPatrolReport> {
        let user_id = actor.require_user()?.id;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, VetPatrolReport>(
            r#"SELECT id, patrol_date, week_start, week_end, status,
                      created_by, updated_by, created_at, updated_at
               FROM vet_patrol_reports
               WHERE id = $1 AND deleted_at IS NULL
               FOR UPDATE"#,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到巡場報告".to_string()))?;
        let before_entries = fetch_entry_snapshots(&mut tx, id).await?;

        let report = sqlx::query_as::<_, VetPatrolReport>(
            r#"UPDATE vet_patrol_reports SET
                   patrol_date = COALESCE($2, patrol_date),
                   week_start  = COALESCE($3, week_start),
                   week_end    = COALESCE($4, week_end),
                   updated_by  = $5,
                   updated_at  = NOW()
               WHERE id = $1 AND deleted_at IS NULL
               RETURNING id, patrol_date, week_start, week_end, status,
                         created_by, updated_by, created_at, updated_at"#,
        )
        .bind(id)
        .bind(req.patrol_date)
        .bind(req.week_start)
        .bind(req.week_end)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;
        let after = report.clone();

        // 如果有傳入新的 entries，整批替換
        if let Some(entries) = &req.entries {
            sqlx::query("DELETE FROM vet_patrol_entries WHERE report_id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;

            for (i, entry) in entries.iter().enumerate() {
                sqlx::query(
                    r#"INSERT INTO vet_patrol_entries
                           (report_id, category, animal_id, observation, suggestion, follow_up, sort_order)
                       VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
                )
                .bind(id)
                .bind(&entry.category)
                .bind(entry.animal_id)
                .bind(&entry.observation)
                .bind(&entry.suggestion)
                .bind(&entry.follow_up)
                .bind(entry.sort_order.unwrap_or(i as i32))
                .execute(&mut *tx)
                .await?;
            }
        }

        let after_entries = fetch_entry_snapshots(&mut tx, id).await?;
        let before_snapshot = VetPatrolReportSnapshot {
            report: before,
            entries: before_entries,
        };
        let after_snapshot = VetPatrolReportSnapshot {
            report: after.clone(),
            entries: after_entries,
        };
        let display = format!("巡場報告 {}", after.patrol_date);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "VET_PATROL_REPORT_UPDATED",
                entity: Some(AuditEntity::new("vet_patrol_reports", id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before_snapshot), Some(&after_snapshot))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(report)
    }

    /// 刪除巡場報告（soft delete）
    pub async fn delete(pool: &PgPool, actor: &ActorContext, id: Uuid) -> Result<()> {
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, VetPatrolReport>(
            r#"SELECT id, patrol_date, week_start, week_end, status,
                      created_by, updated_by, created_at, updated_at
               FROM vet_patrol_reports
               WHERE id = $1 AND deleted_at IS NULL
               FOR UPDATE"#,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到巡場報告".to_string()))?;
        let before_entries = fetch_entry_snapshots(&mut tx, id).await?;

        sqlx::query(
            "UPDATE vet_patrol_reports SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;

        let snapshot = VetPatrolReportSnapshot {
            report: before.clone(),
            entries: before_entries,
        };
        let display = format!("巡場報告 {}", before.patrol_date);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "VET_PATROL_REPORT_DELETED",
                entity: Some(AuditEntity::new("vet_patrol_reports", id, &display)),
                data_diff: Some(DataDiff::delete_only(&snapshot)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
