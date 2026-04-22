// 獸醫巡場報告 Service

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::Result;

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
        req: &CreateVetPatrolReportRequest,
        user_id: Uuid,
    ) -> Result<VetPatrolReport> {
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

        tx.commit().await?;
        Ok(report)
    }

    /// 更新巡場報告（條目全部替換）
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateVetPatrolReportRequest,
        user_id: Uuid,
    ) -> Result<VetPatrolReport> {
        let mut tx = pool.begin().await?;

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

        tx.commit().await?;
        Ok(report)
    }

    /// 刪除巡場報告（soft delete）
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE vet_patrol_reports SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }
}
