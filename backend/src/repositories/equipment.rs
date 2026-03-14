use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{Equipment, EquipmentCalibration},
    AppError, Result,
};

/// 依 id 查詢設備（在 equipment.rs get + update 中出現 2 次）
pub async fn find_equipment_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Equipment>> {
    let record = sqlx::query_as::<_, Equipment>("SELECT * FROM equipment WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(record)
}

/// 依 id 查詢設備校準紀錄（在 equipment.rs get_calibration + update_calibration 中出現 2 次）
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
