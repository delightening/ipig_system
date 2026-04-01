use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{Equipment, EquipmentCalibration, EquipmentDisposal, EquipmentMaintenanceRecord, TimelineRow},
    AppError, Result,
};

pub async fn find_equipment_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Equipment>> {
    let record = sqlx::query_as::<_, Equipment>("SELECT * FROM equipment WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(record)
}

pub async fn find_equipment_calibration_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<EquipmentCalibration>> {
    let record = sqlx::query_as::<_, EquipmentCalibration>(
        "SELECT * FROM equipment_calibrations WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(record)
}

pub async fn find_maintenance_record_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<EquipmentMaintenanceRecord>> {
    let record = sqlx::query_as::<_, EquipmentMaintenanceRecord>(
        "SELECT * FROM equipment_maintenance_records WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(record)
}

pub async fn find_disposal_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<EquipmentDisposal>> {
    let record = sqlx::query_as::<_, EquipmentDisposal>(
        "SELECT * FROM equipment_disposals WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(record)
}

const TIMELINE_SQL: &str = r#"
WITH timeline AS (
    SELECT id, 'maintenance' AS event_type,
           reported_at::timestamptz AS occurred_at,
           maintenance_type::text AS sub_type,
           status::text AS sub_status,
           problem_description AS summary,
           notes,
           NULL::text AS actor_name
    FROM equipment_maintenance_records WHERE equipment_id = $1

    UNION ALL

    SELECT id, 'calibration' AS event_type,
           calibrated_at::timestamptz AS occurred_at,
           calibration_type::text AS sub_type,
           result AS sub_status,
           report_number AS summary,
           notes,
           inspector AS actor_name
    FROM equipment_calibrations WHERE equipment_id = $1

    UNION ALL

    SELECT sl.id, 'status_change' AS event_type,
           sl.created_at AS occurred_at,
           sl.old_status::text AS sub_type,
           sl.new_status::text AS sub_status,
           sl.reason AS summary,
           NULL AS notes,
           u.display_name AS actor_name
    FROM equipment_status_logs sl
    LEFT JOIN users u ON u.id = sl.changed_by
    WHERE sl.equipment_id = $1
)
"#;

pub async fn find_equipment_timeline(
    pool: &PgPool,
    equipment_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<TimelineRow>> {
    let sql = format!(
        "{TIMELINE_SQL} SELECT * FROM timeline ORDER BY occurred_at DESC LIMIT $2 OFFSET $3"
    );
    let rows = sqlx::query_as::<_, TimelineRow>(&sql)
        .bind(equipment_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(rows)
}

pub async fn count_equipment_timeline(
    pool: &PgPool,
    equipment_id: Uuid,
) -> Result<i64> {
    let sql = format!(
        "{TIMELINE_SQL} SELECT COUNT(*)::bigint AS cnt FROM timeline"
    );
    let (count,): (i64,) = sqlx::query_as(&sql)
        .bind(equipment_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(count)
}
