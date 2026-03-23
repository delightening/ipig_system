use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{Equipment, EquipmentCalibration, EquipmentDisposal, EquipmentMaintenanceRecord},
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
