// 設備與校準紀錄 Service (實驗室 GLP)

use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    middleware::CurrentUser,
    models::{
        CalibrationQuery, CalibrationWithEquipment, CreateCalibrationRequest,
        CreateEquipmentRequest, Equipment, EquipmentCalibration, EquipmentQuery,
        PaginatedResponse, UpdateCalibrationRequest, UpdateEquipmentRequest,
    },
    Result,
};

pub struct EquipmentService;

impl EquipmentService {
    pub async fn list_equipment(
        pool: &PgPool,
        query: &EquipmentQuery,
        current_user: &CurrentUser,
    ) -> Result<PaginatedResponse<Equipment>> {
        if !current_user.has_permission("equipment.view") && !current_user.has_permission("equipment.manage") {
            return Err(AppError::Forbidden("無權查看設備".into()));
        }

        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(100);
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM equipment
            WHERE ($1::text IS NULL OR name ILIKE '%' || $1 || '%' OR model ILIKE '%' || $1 || '%')
              AND ($2::bool IS NULL OR is_active = $2)
            "#,
        )
        .bind(query.keyword.as_deref())
        .bind(query.is_active)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, Equipment>(
            r#"
            SELECT * FROM equipment
            WHERE ($1::text IS NULL OR name ILIKE '%' || $1 || '%' OR model ILIKE '%' || $1 || '%')
              AND ($2::bool IS NULL OR is_active = $2)
            ORDER BY name
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(query.keyword.as_deref())
        .bind(query.is_active)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    pub async fn get_equipment(
        pool: &PgPool,
        id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<Equipment> {
        if !current_user.has_permission("equipment.view") && !current_user.has_permission("equipment.manage") {
            return Err(AppError::Forbidden("無權查看設備".into()));
        }

        let record = sqlx::query_as::<_, Equipment>("SELECT * FROM equipment WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("設備不存在".into()))?;

        Ok(record)
    }

    pub async fn create_equipment(
        pool: &PgPool,
        payload: &CreateEquipmentRequest,
        current_user: &CurrentUser,
    ) -> Result<Equipment> {
        if !current_user.has_permission("equipment.manage") {
            return Err(AppError::Forbidden("無權新增設備".into()));
        }
        payload.validate()?;

        let record = sqlx::query_as::<_, Equipment>(
            r#"
            INSERT INTO equipment (name, model, serial_number, location, notes)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(&payload.name)
        .bind(&payload.model)
        .bind(&payload.serial_number)
        .bind(&payload.location)
        .bind(&payload.notes)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn update_equipment(
        pool: &PgPool,
        id: Uuid,
        payload: &UpdateEquipmentRequest,
        current_user: &CurrentUser,
    ) -> Result<Equipment> {
        if !current_user.has_permission("equipment.manage") {
            return Err(AppError::Forbidden("無權編輯設備".into()));
        }
        payload.validate()?;

        let existing = sqlx::query_as::<_, Equipment>("SELECT * FROM equipment WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("設備不存在".into()))?;

        let name = payload.name.as_deref().unwrap_or(&existing.name);
        let model = payload.model.as_ref().or(existing.model.as_ref()).cloned();
        let serial_number = payload.serial_number.as_ref().or(existing.serial_number.as_ref()).cloned();
        let location = payload.location.as_ref().or(existing.location.as_ref()).cloned();
        let notes = payload.notes.as_ref().or(existing.notes.as_ref()).cloned();
        let is_active = payload.is_active.unwrap_or(existing.is_active);

        let record = sqlx::query_as::<_, Equipment>(
            r#"
            UPDATE equipment
            SET name = $2, model = $3, serial_number = $4, location = $5, notes = $6, is_active = $7, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(model)
        .bind(serial_number)
        .bind(location)
        .bind(notes)
        .bind(is_active)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn delete_equipment(
        pool: &PgPool,
        id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<()> {
        if !current_user.has_permission("equipment.manage") {
            return Err(AppError::Forbidden("無權刪除設備".into()));
        }

        let result = sqlx::query("DELETE FROM equipment WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("設備不存在".into()));
        }

        Ok(())
    }

    // ========== Calibrations ==========

    pub async fn list_calibrations(
        pool: &PgPool,
        query: &CalibrationQuery,
        current_user: &CurrentUser,
    ) -> Result<PaginatedResponse<CalibrationWithEquipment>> {
        if !current_user.has_permission("equipment.view") && !current_user.has_permission("equipment.manage") {
            return Err(AppError::Forbidden("無權查看校準紀錄".into()));
        }

        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(100);
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM equipment_calibrations ec
            INNER JOIN equipment e ON ec.equipment_id = e.id
            WHERE ($1::uuid IS NULL OR ec.equipment_id = $1)
            "#,
        )
        .bind(query.equipment_id)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, CalibrationWithEquipment>(
            r#"
            SELECT
                ec.id, ec.equipment_id, e.name as equipment_name,
                ec.calibrated_at, ec.next_due_at, ec.result, ec.notes, ec.created_at
            FROM equipment_calibrations ec
            INNER JOIN equipment e ON ec.equipment_id = e.id
            WHERE ($1::uuid IS NULL OR ec.equipment_id = $1)
            ORDER BY ec.calibrated_at DESC, ec.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(query.equipment_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    pub async fn get_calibration(
        pool: &PgPool,
        id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<EquipmentCalibration> {
        if !current_user.has_permission("equipment.view") && !current_user.has_permission("equipment.manage") {
            return Err(AppError::Forbidden("無權查看校準紀錄".into()));
        }

        let record = sqlx::query_as::<_, EquipmentCalibration>(
            "SELECT * FROM equipment_calibrations WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("校準紀錄不存在".into()))?;

        Ok(record)
    }

    pub async fn create_calibration(
        pool: &PgPool,
        payload: &CreateCalibrationRequest,
        current_user: &CurrentUser,
    ) -> Result<EquipmentCalibration> {
        if !current_user.has_permission("equipment.manage") {
            return Err(AppError::Forbidden("無權新增校準紀錄".into()));
        }
        payload.validate()?;

        let record = sqlx::query_as::<_, EquipmentCalibration>(
            r#"
            INSERT INTO equipment_calibrations (equipment_id, calibrated_at, next_due_at, result, notes)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(payload.equipment_id)
        .bind(payload.calibrated_at)
        .bind(payload.next_due_at)
        .bind(&payload.result)
        .bind(&payload.notes)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn update_calibration(
        pool: &PgPool,
        id: Uuid,
        payload: &UpdateCalibrationRequest,
        current_user: &CurrentUser,
    ) -> Result<EquipmentCalibration> {
        if !current_user.has_permission("equipment.manage") {
            return Err(AppError::Forbidden("無權編輯校準紀錄".into()));
        }
        payload.validate()?;

        let existing = sqlx::query_as::<_, EquipmentCalibration>(
            "SELECT * FROM equipment_calibrations WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("校準紀錄不存在".into()))?;

        let calibrated_at = payload.calibrated_at.unwrap_or(existing.calibrated_at);
        let next_due_at = payload.next_due_at.or(existing.next_due_at);
        let result = payload.result.as_ref().or(existing.result.as_ref()).cloned();
        let notes = payload.notes.as_ref().or(existing.notes.as_ref()).cloned();

        let record = sqlx::query_as::<_, EquipmentCalibration>(
            r#"
            UPDATE equipment_calibrations
            SET calibrated_at = $2, next_due_at = $3, result = $4, notes = $5, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(calibrated_at)
        .bind(next_due_at)
        .bind(result)
        .bind(notes)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn delete_calibration(
        pool: &PgPool,
        id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<()> {
        if !current_user.has_permission("equipment.manage") {
            return Err(AppError::Forbidden("無權刪除校準紀錄".into()));
        }

        let result = sqlx::query("DELETE FROM equipment_calibrations WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("校準紀錄不存在".into()));
        }

        Ok(())
    }
}
