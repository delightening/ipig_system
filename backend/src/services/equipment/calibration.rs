// 校正 / 確效 / 查核（calibrations）

use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    middleware::CurrentUser,
    models::{
        CalibrationQuery, CalibrationWithEquipment, CreateCalibrationRequest, EquipmentCalibration,
        PaginatedResponse, UpdateCalibrationRequest,
    },
    repositories, Result,
};

use super::{check_manage_permission, check_view_permission, EquipmentService};

impl EquipmentService {
    pub async fn list_calibrations(
        pool: &PgPool,
        query: &CalibrationQuery,
        current_user: &CurrentUser,
    ) -> Result<PaginatedResponse<CalibrationWithEquipment>> {
        check_view_permission(current_user)?;

        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(100);
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM equipment_calibrations ec
            INNER JOIN equipment e ON ec.equipment_id = e.id
            WHERE ($1::uuid IS NULL OR ec.equipment_id = $1)
              AND ($2::calibration_type IS NULL OR ec.calibration_type = $2)
            "#,
        )
        .bind(query.equipment_id)
        .bind(&query.calibration_type)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, CalibrationWithEquipment>(
            r#"
            SELECT
                ec.id, ec.equipment_id, e.name AS equipment_name,
                e.serial_number AS equipment_serial_number,
                ec.calibration_type, ec.calibrated_at, ec.next_due_at,
                ec.result, ec.notes, ec.partner_id,
                p.name AS partner_name,
                ec.report_number, ec.inspector,
                ec.certificate_number, ec.performed_by,
                ec.acceptance_criteria, ec.measurement_uncertainty,
                ec.validation_phase, ec.protocol_number,
                ec.created_at
            FROM equipment_calibrations ec
            INNER JOIN equipment e ON ec.equipment_id = e.id
            LEFT JOIN partners p ON ec.partner_id = p.id
            WHERE ($1::uuid IS NULL OR ec.equipment_id = $1)
              AND ($2::calibration_type IS NULL OR ec.calibration_type = $2)
            ORDER BY ec.calibrated_at DESC, ec.created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(query.equipment_id)
        .bind(&query.calibration_type)
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
        check_view_permission(current_user)?;
        repositories::equipment::find_equipment_calibration_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("校準紀錄不存在".into()))
    }

    pub async fn create_calibration(
        pool: &PgPool,
        payload: &CreateCalibrationRequest,
        current_user: &CurrentUser,
    ) -> Result<EquipmentCalibration> {
        check_manage_permission(current_user)?;
        payload.validate()?;

        // 取得設備序號
        let equipment = repositories::equipment::find_equipment_by_id(pool, payload.equipment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("設備不存在".into()))?;

        let record = sqlx::query_as::<_, EquipmentCalibration>(
            r#"
            INSERT INTO equipment_calibrations
                (equipment_id, calibration_type, calibrated_at, next_due_at, result, notes,
                 partner_id, report_number, inspector, equipment_serial_number,
                 certificate_number, performed_by, acceptance_criteria, measurement_uncertainty,
                 validation_phase, protocol_number)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING *
            "#,
        )
        .bind(payload.equipment_id)
        .bind(&payload.calibration_type)
        .bind(payload.calibrated_at)
        .bind(payload.next_due_at)
        .bind(&payload.result)
        .bind(&payload.notes)
        .bind(payload.partner_id)
        .bind(&payload.report_number)
        .bind(&payload.inspector)
        .bind(&equipment.serial_number)
        .bind(&payload.certificate_number)
        .bind(&payload.performed_by)
        .bind(&payload.acceptance_criteria)
        .bind(&payload.measurement_uncertainty)
        .bind(&payload.validation_phase)
        .bind(&payload.protocol_number)
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
        check_manage_permission(current_user)?;
        payload.validate()?;

        let existing = repositories::equipment::find_equipment_calibration_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("校準紀錄不存在".into()))?;

        let cal_type = payload
            .calibration_type
            .as_ref()
            .unwrap_or(&existing.calibration_type);
        let calibrated_at = payload.calibrated_at.unwrap_or(existing.calibrated_at);
        let next_due_at = payload.next_due_at.or(existing.next_due_at);
        let result = payload
            .result
            .as_ref()
            .or(existing.result.as_ref())
            .cloned();
        let notes = payload.notes.as_ref().or(existing.notes.as_ref()).cloned();
        let partner_id = payload.partner_id.or(existing.partner_id);
        let report_number = payload
            .report_number
            .as_ref()
            .or(existing.report_number.as_ref())
            .cloned();
        let inspector = payload
            .inspector
            .as_ref()
            .or(existing.inspector.as_ref())
            .cloned();
        let certificate_number = payload
            .certificate_number
            .as_ref()
            .or(existing.certificate_number.as_ref())
            .cloned();
        let performed_by = payload
            .performed_by
            .as_ref()
            .or(existing.performed_by.as_ref())
            .cloned();
        let acceptance_criteria = payload
            .acceptance_criteria
            .as_ref()
            .or(existing.acceptance_criteria.as_ref())
            .cloned();
        let measurement_uncertainty = payload
            .measurement_uncertainty
            .as_ref()
            .or(existing.measurement_uncertainty.as_ref())
            .cloned();
        let validation_phase = payload
            .validation_phase
            .as_ref()
            .or(existing.validation_phase.as_ref())
            .cloned();
        let protocol_number = payload
            .protocol_number
            .as_ref()
            .or(existing.protocol_number.as_ref())
            .cloned();

        let record = sqlx::query_as::<_, EquipmentCalibration>(
            r#"
            UPDATE equipment_calibrations
            SET calibration_type = $2, calibrated_at = $3, next_due_at = $4,
                result = $5, notes = $6, partner_id = $7,
                report_number = $8, inspector = $9,
                certificate_number = $10, performed_by = $11,
                acceptance_criteria = $12, measurement_uncertainty = $13,
                validation_phase = $14, protocol_number = $15,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(cal_type)
        .bind(calibrated_at)
        .bind(next_due_at)
        .bind(result)
        .bind(notes)
        .bind(partner_id)
        .bind(report_number)
        .bind(inspector)
        .bind(certificate_number)
        .bind(performed_by)
        .bind(acceptance_criteria)
        .bind(measurement_uncertainty)
        .bind(validation_phase)
        .bind(protocol_number)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn delete_calibration(
        pool: &PgPool,
        id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<()> {
        check_manage_permission(current_user)?;
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
