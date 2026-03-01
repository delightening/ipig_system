//! QAU (Quality Assurance Unit) 服務
//!
//! GLP 品質保證：提供研究狀態、審查進度、稽核摘要、動物實驗概覽的唯讀檢視

use chrono::Utc;
use serde::Serialize;
use sqlx::PgPool;

use crate::Result;

/// QAU 儀表板回應
#[derive(Debug, Serialize)]
pub struct QauDashboard {
    /// 計畫狀態分布
    pub protocol_status_summary: Vec<ProtocolStatusCount>,
    /// 審查進度（近期狀態變更數）
    pub review_progress: ReviewProgressSummary,
    /// 稽核摘要（依 entity_type 聚合）
    pub audit_summary: Vec<AuditEntityCount>,
    /// 動物實驗概覽
    pub animal_summary: AnimalSummary,
}

#[derive(Debug, Serialize)]
pub struct ProtocolStatusCount {
    pub status: String,
    pub display_name: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct ReviewProgressSummary {
    pub status_changes_last_7_days: i64,
    pub protocols_in_review: i64,
    pub protocols_pending_pi_response: i64,
}

#[derive(Debug, Serialize)]
pub struct AuditEntityCount {
    pub entity_type: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct AnimalSummary {
    pub total: i64,
    pub by_status: Vec<AnimalStatusCount>,
    pub in_experiment: i64,
    pub euthanized: i64,
    pub completed: i64,
}

#[derive(Debug, Serialize)]
pub struct AnimalStatusCount {
    pub status: String,
    pub display_name: String,
    pub count: i64,
}

pub struct QauService;

impl QauService {
    /// 取得 QAU 儀表板資料
    pub async fn get_dashboard(pool: &PgPool) -> Result<QauDashboard> {
        let protocol_status_summary = Self::get_protocol_status_summary(pool).await?;
        let review_progress = Self::get_review_progress(pool).await?;
        let audit_summary = Self::get_audit_summary(pool).await?;
        let animal_summary = Self::get_animal_summary(pool).await?;

        Ok(QauDashboard {
            protocol_status_summary,
            review_progress,
            audit_summary,
            animal_summary,
        })
    }

    async fn get_protocol_status_summary(pool: &PgPool) -> Result<Vec<ProtocolStatusCount>> {
        let rows: Vec<(String, i64)> = sqlx::query_as(
            r#"
            SELECT status::text, COUNT(*)::bigint
            FROM protocols
            WHERE status != 'DELETED'
            GROUP BY status
            ORDER BY count DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        let display_names: std::collections::HashMap<&str, &str> = [
            ("DRAFT", "草稿"),
            ("SUBMITTED", "已提交"),
            ("PRE_REVIEW", "行政預審"),
            ("PRE_REVIEW_REVISION_REQUIRED", "行政預審補件"),
            ("VET_REVIEW", "獸醫審查"),
            ("VET_REVISION_REQUIRED", "獸醫要求修訂"),
            ("UNDER_REVIEW", "審查中"),
            ("REVISION_REQUIRED", "需修訂"),
            ("RESUBMITTED", "已重送"),
            ("APPROVED", "已核准"),
            ("APPROVED_WITH_CONDITIONS", "附條件核准"),
            ("DEFERRED", "延後審議"),
            ("REJECTED", "已否決"),
            ("SUSPENDED", "已暫停"),
            ("CLOSED", "已結案"),
        ]
        .into_iter()
        .collect();

        Ok(rows
            .into_iter()
            .map(|(status, count)| ProtocolStatusCount {
                display_name: display_names
                    .get(status.as_str())
                    .copied()
                    .unwrap_or(&status)
                    .to_string(),
                status,
                count,
            })
            .collect())
    }

    async fn get_review_progress(pool: &PgPool) -> Result<ReviewProgressSummary> {
        let week_ago = Utc::now() - chrono::Duration::days(7);

        let status_changes: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)::bigint
            FROM protocol_status_history
            WHERE created_at >= $1
            "#,
        )
        .bind(week_ago)
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

        let in_review: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)::bigint
            FROM protocols
            WHERE status IN ('UNDER_REVIEW', 'VET_REVIEW', 'PRE_REVIEW', 'SUBMITTED', 'RESUBMITTED')
            AND status != 'DELETED'
            "#,
        )
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

        let pending_pi: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)::bigint
            FROM protocols
            WHERE status IN ('REVISION_REQUIRED', 'PRE_REVIEW_REVISION_REQUIRED', 'VET_REVISION_REQUIRED')
            AND status != 'DELETED'
            "#,
        )
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

        Ok(ReviewProgressSummary {
            status_changes_last_7_days: status_changes.0,
            protocols_in_review: in_review.0,
            protocols_pending_pi_response: pending_pi.0,
        })
    }

    async fn get_audit_summary(pool: &PgPool) -> Result<Vec<AuditEntityCount>> {
        let week_ago = Utc::now() - chrono::Duration::days(7);

        let rows: Vec<(Option<String>, i64)> = sqlx::query_as(
            r#"
            SELECT entity_type, COUNT(*)::bigint
            FROM user_activity_logs
            WHERE created_at >= $1
            AND entity_type IS NOT NULL
            GROUP BY entity_type
            ORDER BY count DESC
            LIMIT 15
            "#,
        )
        .bind(week_ago)
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(entity_type, count)| AuditEntityCount {
                entity_type: entity_type.unwrap_or_else(|| "unknown".to_string()),
                count,
            })
            .collect())
    }

    async fn get_animal_summary(pool: &PgPool) -> Result<AnimalSummary> {
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM animals")
            .fetch_one(pool)
            .await
            .unwrap_or((0,));

        let rows: Vec<(String, i64)> = sqlx::query_as(
            r#"
            SELECT status::text, COUNT(*)::bigint
            FROM animals
            GROUP BY status
            ORDER BY count DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        let display_names: std::collections::HashMap<&str, &str> = [
            ("unassigned", "未分配"),
            ("in_experiment", "實驗中"),
            ("completed", "實驗完成"),
            ("euthanized", "安樂死"),
            ("sudden_death", "猝死"),
            ("transferred", "已轉讓"),
        ]
        .into_iter()
        .collect();

        let by_status: Vec<AnimalStatusCount> = rows
            .into_iter()
            .map(|(status, count)| AnimalStatusCount {
                display_name: display_names
                    .get(status.as_str())
                    .copied()
                    .unwrap_or(&status)
                    .to_string(),
                status,
                count,
            })
            .collect();

        let in_experiment: i64 = by_status
            .iter()
            .find(|s| s.status == "in_experiment")
            .map(|s| s.count)
            .unwrap_or(0);
        let euthanized: i64 = by_status
            .iter()
            .find(|s| s.status == "euthanized")
            .map(|s| s.count)
            .unwrap_or(0);
        let completed: i64 = by_status
            .iter()
            .find(|s| s.status == "completed")
            .map(|s| s.count)
            .unwrap_or(0);

        Ok(AnimalSummary {
            total: total.0,
            by_status,
            in_experiment,
            euthanized,
            completed,
        })
    }
}
