// 設備維護管理 Service (實驗室 GLP)

use rand::Rng;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    middleware::{ActorContext, CurrentUser},
    models::{
        audit_diff::DataDiff, AnnualPlanExecutionRow, AnnualPlanExecutionSummary,
        AnnualPlanQuery, AnnualPlanWithEquipment, ApproveDisposalRequest,
        ApproveIdleRequestRequest, CalibrationCycle, CalibrationQuery, CalibrationWithEquipment,
        CreateAnnualPlanRequest, CreateCalibrationRequest, CreateDisposalRequest,
        CreateEquipmentRequest, CreateEquipmentSupplierRequest, CreateIdleRequestRequest,
        CreateMaintenanceRequest, DisposalQuery, DisposalStatus, DisposalWithDetails,
        EquipmentDisposal, Equipment,
        EquipmentCalibration, EquipmentHistoryQuery, EquipmentMaintenanceRecord, EquipmentQuery,
        EquipmentStatus, EquipmentStatusLog, EquipmentSupplierWithPartner, EquipmentTimelineEntry,
        ExecutionSummaryQuery, GenerateAnnualPlanRequest, IdleRequestQuery, IdleRequestWithDetails,
        MaintenanceQuery, MaintenanceRecordWithDetails, MaintenanceStatus, MaintenanceType,
        MonthExecutionDetail, MonthExecutionStatus, PaginatedResponse, ReviewMaintenanceRequest,
        TimelineRow, UpdateAnnualPlanRequest, UpdateCalibrationRequest, UpdateEquipmentRequest,
        UpdateMaintenanceRequest,
    },
    repositories,
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    Result,
};

fn check_view_permission(current_user: &CurrentUser) -> Result<()> {
    if !current_user.has_permission("equipment.view")
        && !current_user.has_permission("equipment.manage")
    {
        return Err(AppError::Forbidden("無權查看設備".into()));
    }
    Ok(())
}

fn check_manage_permission(current_user: &CurrentUser) -> Result<()> {
    if !current_user.has_permission("equipment.manage") {
        return Err(AppError::Forbidden("無權管理設備".into()));
    }
    Ok(())
}

/// 驗證設備狀態轉換是否合法（GLP/ISO 合規）
pub fn validate_status_transition(
    from: &EquipmentStatus,
    to: &EquipmentStatus,
) -> Result<()> {
    if from == to {
        return Ok(());
    }
    let allowed = matches!(
        (from, to),
        (EquipmentStatus::Active, EquipmentStatus::UnderRepair)
            | (EquipmentStatus::Active, EquipmentStatus::Inactive)
            | (EquipmentStatus::Active, EquipmentStatus::Decommissioned)
            | (EquipmentStatus::Inactive, EquipmentStatus::Active)
            | (EquipmentStatus::UnderRepair, EquipmentStatus::Active)
            | (EquipmentStatus::UnderRepair, EquipmentStatus::Decommissioned)
            | (EquipmentStatus::Decommissioned, EquipmentStatus::Active)
    );
    if !allowed {
        return Err(AppError::BadRequest(format!(
            "不允許的狀態轉換：{:?} → {:?}",
            from, to
        )));
    }
    Ok(())
}

pub struct EquipmentService;

impl EquipmentService {
    // ========== Equipment CRUD ==========

    pub async fn list_equipment(
        pool: &PgPool,
        query: &EquipmentQuery,
        current_user: &CurrentUser,
    ) -> Result<PaginatedResponse<Equipment>> {
        check_view_permission(current_user)?;

        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(100);
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM equipment
            WHERE ($1::text IS NULL OR name ILIKE '%' || $1 || '%' OR model ILIKE '%' || $1 || '%')
              AND ($2::bool IS NULL OR is_active = $2)
              AND ($3::equipment_status IS NULL OR status = $3)
            "#,
        )
        .bind(query.keyword.as_deref())
        .bind(query.is_active)
        .bind(&query.status)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, Equipment>(
            r#"
            SELECT * FROM equipment
            WHERE ($1::text IS NULL OR name ILIKE '%' || $1 || '%' OR model ILIKE '%' || $1 || '%')
              AND ($2::bool IS NULL OR is_active = $2)
              AND ($3::equipment_status IS NULL OR status = $3)
            ORDER BY name
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(query.keyword.as_deref())
        .bind(query.is_active)
        .bind(&query.status)
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
        check_view_permission(current_user)?;
        repositories::equipment::find_equipment_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("設備不存在".into()))
    }

    pub async fn create_equipment(
        pool: &PgPool,
        payload: &CreateEquipmentRequest,
        current_user: &CurrentUser,
    ) -> Result<Equipment> {
        check_manage_permission(current_user)?;
        payload.validate()?;

        let record = sqlx::query_as::<_, Equipment>(
            r#"
            INSERT INTO equipment
                (name, model, serial_number, location, department,
                 purchase_date, warranty_expiry, notes,
                 calibration_type, calibration_cycle, inspection_cycle)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(&payload.name)
        .bind(&payload.model)
        .bind(&payload.serial_number)
        .bind(&payload.location)
        .bind(&payload.department)
        .bind(payload.purchase_date)
        .bind(payload.warranty_expiry)
        .bind(&payload.notes)
        .bind(&payload.calibration_type)
        .bind(&payload.calibration_cycle)
        .bind(&payload.inspection_cycle)
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
        check_manage_permission(current_user)?;
        payload.validate()?;

        let existing = repositories::equipment::find_equipment_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("設備不存在".into()))?;

        let name = payload.name.as_deref().unwrap_or(&existing.name);
        let model = payload.model.as_ref().or(existing.model.as_ref()).cloned();
        let serial = payload
            .serial_number
            .as_ref()
            .or(existing.serial_number.as_ref())
            .cloned();
        let location = payload
            .location
            .as_ref()
            .or(existing.location.as_ref())
            .cloned();
        let department = payload
            .department
            .as_ref()
            .or(existing.department.as_ref())
            .cloned();
        let purchase_date = payload.purchase_date.or(existing.purchase_date);
        let warranty_expiry = payload.warranty_expiry.or(existing.warranty_expiry);
        let notes = payload.notes.as_ref().or(existing.notes.as_ref()).cloned();
        let cal_type = payload
            .calibration_type
            .as_ref()
            .or(existing.calibration_type.as_ref())
            .cloned();
        let cal_cycle = payload
            .calibration_cycle
            .as_ref()
            .or(existing.calibration_cycle.as_ref())
            .cloned();
        let insp_cycle = payload
            .inspection_cycle
            .as_ref()
            .or(existing.inspection_cycle.as_ref())
            .cloned();

        let record = sqlx::query_as::<_, Equipment>(
            r#"
            UPDATE equipment
            SET name = $2, model = $3, serial_number = $4, location = $5,
                department = $6, purchase_date = $7, warranty_expiry = $8,
                notes = $9,
                calibration_type = $10, calibration_cycle = $11, inspection_cycle = $12,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(model)
        .bind(serial)
        .bind(location)
        .bind(department)
        .bind(purchase_date)
        .bind(warranty_expiry)
        .bind(notes)
        .bind(cal_type)
        .bind(cal_cycle)
        .bind(insp_cycle)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn delete_equipment(
        pool: &PgPool,
        id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<()> {
        check_manage_permission(current_user)?;
        let result = sqlx::query("DELETE FROM equipment WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("設備不存在".into()));
        }
        Ok(())
    }

    // ========== Equipment Suppliers ==========

    pub async fn list_all_equipment_suppliers_summary(
        pool: &PgPool,
        current_user: &CurrentUser,
    ) -> Result<Vec<crate::models::EquipmentSupplierSummaryRow>> {
        check_view_permission(current_user)?;
        let data = sqlx::query_as::<_, crate::models::EquipmentSupplierSummaryRow>(
            r#"
            SELECT es.equipment_id, p.name AS partner_name
            FROM equipment_suppliers es
            INNER JOIN partners p ON es.partner_id = p.id
            ORDER BY es.equipment_id, p.name
            "#,
        )
        .fetch_all(pool)
        .await?;
        Ok(data)
    }

    pub async fn list_equipment_suppliers(
        pool: &PgPool,
        equipment_id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<Vec<EquipmentSupplierWithPartner>> {
        check_view_permission(current_user)?;
        let data = sqlx::query_as::<_, EquipmentSupplierWithPartner>(
            r#"
            SELECT es.id, es.equipment_id, es.partner_id, p.name AS partner_name,
                   es.contact_person, es.contact_phone, es.contact_email, es.notes,
                   p.phone AS partner_phone, p.phone_ext AS partner_phone_ext,
                   p.email AS partner_email, p.address AS partner_address,
                   es.created_at
            FROM equipment_suppliers es
            INNER JOIN partners p ON es.partner_id = p.id
            WHERE es.equipment_id = $1
            ORDER BY p.name
            "#,
        )
        .bind(equipment_id)
        .fetch_all(pool)
        .await?;
        Ok(data)
    }

    pub async fn add_equipment_supplier(
        pool: &PgPool,
        equipment_id: Uuid,
        payload: &CreateEquipmentSupplierRequest,
        current_user: &CurrentUser,
    ) -> Result<EquipmentSupplierWithPartner> {
        check_manage_permission(current_user)?;
        payload.validate()?;

        let record = sqlx::query_as::<_, EquipmentSupplierWithPartner>(
            r#"
            WITH ins AS (
                INSERT INTO equipment_suppliers (equipment_id, partner_id, contact_person, contact_phone, contact_email, notes)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING *
            )
            SELECT ins.id, ins.equipment_id, ins.partner_id, p.name AS partner_name,
                   ins.contact_person, ins.contact_phone, ins.contact_email, ins.notes,
                   p.phone AS partner_phone, p.phone_ext AS partner_phone_ext,
                   p.email AS partner_email, p.address AS partner_address,
                   ins.created_at
            FROM ins
            INNER JOIN partners p ON ins.partner_id = p.id
            "#,
        )
        .bind(equipment_id)
        .bind(payload.partner_id)
        .bind(&payload.contact_person)
        .bind(&payload.contact_phone)
        .bind(&payload.contact_email)
        .bind(&payload.notes)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn remove_equipment_supplier(
        pool: &PgPool,
        id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<()> {
        check_manage_permission(current_user)?;
        let result = sqlx::query("DELETE FROM equipment_suppliers WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("廠商關聯不存在".into()));
        }
        Ok(())
    }

    // ========== Status Logs ==========

    pub async fn list_status_logs(
        pool: &PgPool,
        equipment_id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<Vec<EquipmentStatusLog>> {
        check_view_permission(current_user)?;
        let data = sqlx::query_as::<_, EquipmentStatusLog>(
            r#"
            SELECT * FROM equipment_status_logs
            WHERE equipment_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(equipment_id)
        .fetch_all(pool)
        .await?;
        Ok(data)
    }

    // ========== Equipment Timeline (設備履歷) ==========

    pub async fn get_equipment_history(
        pool: &PgPool,
        equipment_id: Uuid,
        query: &EquipmentHistoryQuery,
        current_user: &CurrentUser,
    ) -> Result<PaginatedResponse<EquipmentTimelineEntry>> {
        check_view_permission(current_user)?;
        repositories::equipment::find_equipment_by_id(pool, equipment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("設備不存在".into()))?;

        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50);
        let offset = (page - 1) * per_page;

        let total = repositories::equipment::count_equipment_timeline(pool, equipment_id).await?;
        let rows = repositories::equipment::find_equipment_timeline(
            pool, equipment_id, per_page, offset,
        ).await?;

        let data = rows.into_iter().map(build_timeline_entry).collect();
        Ok(PaginatedResponse::new(data, total, page, per_page))
    }

    // ========== Calibrations (校正/確效/查核) ==========

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
        let result = payload.result.as_ref().or(existing.result.as_ref()).cloned();
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

    // ========== Maintenance Records (維修/保養) ==========

    pub async fn list_maintenance_records(
        pool: &PgPool,
        query: &MaintenanceQuery,
        current_user: &CurrentUser,
    ) -> Result<PaginatedResponse<MaintenanceRecordWithDetails>> {
        check_view_permission(current_user)?;

        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(100);
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM equipment_maintenance_records m
            WHERE ($1::uuid IS NULL OR m.equipment_id = $1)
              AND ($2::maintenance_type IS NULL OR m.maintenance_type = $2)
              AND ($3::maintenance_status IS NULL OR m.status = $3)
            "#,
        )
        .bind(query.equipment_id)
        .bind(&query.maintenance_type)
        .bind(&query.status)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, MaintenanceRecordWithDetails>(
            r#"
            SELECT m.id, m.equipment_id, e.name AS equipment_name,
                   m.maintenance_type, m.status, m.reported_at, m.completed_at,
                   m.problem_description, m.repair_content, m.repair_partner_id,
                   p.name AS repair_partner_name,
                   m.maintenance_items, m.performed_by, m.notes,
                   m.created_by,
                   m.reviewed_by, u2.display_name AS reviewer_name,
                   m.reviewed_at, m.review_notes,
                   m.created_at
            FROM equipment_maintenance_records m
            INNER JOIN equipment e ON m.equipment_id = e.id
            LEFT JOIN partners p ON m.repair_partner_id = p.id
            LEFT JOIN users u2 ON m.reviewed_by = u2.id
            WHERE ($1::uuid IS NULL OR m.equipment_id = $1)
              AND ($2::maintenance_type IS NULL OR m.maintenance_type = $2)
              AND ($3::maintenance_status IS NULL OR m.status = $3)
            ORDER BY m.reported_at DESC, m.created_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(query.equipment_id)
        .bind(&query.maintenance_type)
        .bind(&query.status)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    // ============================================
    // Transaction variants for cross-service atomicity (R26-3 Phase 2)
    // ============================================

    /// Transaction 版本：建立維修紀錄
    pub(super) async fn create_maintenance_record_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &ActorContext,
        payload: &CreateMaintenanceRequest,
    ) -> Result<EquipmentMaintenanceRecord> {
        let actor_id = actor
            .actor_user_id()
            .ok_or_else(|| AppError::Forbidden("Anonymous cannot create maintenance records".into()))?;
        payload.validate()?;

        let equipment = sqlx::query_as::<_, Equipment>(
            "SELECT * FROM equipment WHERE id = $1 FOR UPDATE",
        )
        .bind(payload.equipment_id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound("設備不存在".into()))?;

        if payload.maintenance_type == MaintenanceType::Repair
            && equipment.status == EquipmentStatus::Active
        {
            validate_status_transition(&equipment.status, &EquipmentStatus::UnderRepair)?;
            sqlx::query(
                "INSERT INTO equipment_status_logs (equipment_id, old_status, new_status, changed_by, reason) VALUES ($1, $2, 'under_repair', $3, '建立維修紀錄，自動變更狀態')",
            )
            .bind(payload.equipment_id)
            .bind(&equipment.status)
            .bind(actor_id)
            .execute(&mut **tx)
            .await?;

            sqlx::query("UPDATE equipment SET status = 'under_repair', is_active = false, updated_at = NOW() WHERE id = $1")
                .bind(payload.equipment_id)
                .execute(&mut **tx)
                .await?;
        }

        let record = sqlx::query_as::<_, EquipmentMaintenanceRecord>(
            r#"
            INSERT INTO equipment_maintenance_records
                (equipment_id, maintenance_type, reported_at, completed_at,
                 problem_description, repair_content, repair_partner_id,
                 maintenance_items, performed_by, notes, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(payload.equipment_id)
        .bind(&payload.maintenance_type)
        .bind(payload.reported_at)
        .bind(payload.completed_at)
        .bind(&payload.problem_description)
        .bind(&payload.repair_content)
        .bind(payload.repair_partner_id)
        .bind(&payload.maintenance_items)
        .bind(&payload.performed_by)
        .bind(&payload.notes)
        .bind(actor_id)
        .fetch_one(&mut **tx)
        .await?;

        let display = format!("{} {:?}", equipment.name, record.maintenance_type);
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "EQUIPMENT",
                event_type: "MAINTENANCE_CREATE",
                entity: Some(AuditEntity::new("maintenance_record", record.id, &display)),
                data_diff: Some(DataDiff::create_only(&record)),
                request_context: None,
            },
        )
        .await?;

        Ok(record)
    }

    /// Transaction 版本：更新維修紀錄
    pub(super) async fn update_maintenance_record_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &ActorContext,
        id: Uuid,
        payload: &UpdateMaintenanceRequest,
    ) -> Result<EquipmentMaintenanceRecord> {
        payload.validate()?;

        let existing = sqlx::query_as::<_, EquipmentMaintenanceRecord>(
            "SELECT * FROM equipment_maintenance_records WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound("維修保養紀錄不存在".into()))?;

        let mut new_status = payload.status.clone().unwrap_or(existing.status.clone());
        if new_status == MaintenanceStatus::Completed
            && existing.status != MaintenanceStatus::Completed
            && existing.status != MaintenanceStatus::PendingReview
        {
            new_status = MaintenanceStatus::PendingReview;
        }

        let completed_at = payload.completed_at.or(existing.completed_at);
        let problem_desc = payload
            .problem_description
            .as_ref()
            .or(existing.problem_description.as_ref())
            .cloned();
        let repair_content = payload
            .repair_content
            .as_ref()
            .or(existing.repair_content.as_ref())
            .cloned();
        let repair_partner = payload.repair_partner_id.or(existing.repair_partner_id);
        let maint_items = payload
            .maintenance_items
            .as_ref()
            .or(existing.maintenance_items.as_ref())
            .cloned();
        let performed = payload
            .performed_by
            .as_ref()
            .or(existing.performed_by.as_ref())
            .cloned();
        let notes = payload.notes.as_ref().or(existing.notes.as_ref()).cloned();

        let record = sqlx::query_as::<_, EquipmentMaintenanceRecord>(
            r#"
            UPDATE equipment_maintenance_records
            SET status = $2, completed_at = $3, problem_description = $4,
                repair_content = $5, repair_partner_id = $6,
                maintenance_items = $7, performed_by = $8, notes = $9,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&new_status)
        .bind(completed_at)
        .bind(problem_desc)
        .bind(repair_content)
        .bind(repair_partner)
        .bind(maint_items)
        .bind(performed)
        .bind(notes)
        .fetch_one(&mut **tx)
        .await?;

        let display = format!("maintenance {:?}", record.maintenance_type);
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "EQUIPMENT",
                event_type: "MAINTENANCE_UPDATE",
                entity: Some(AuditEntity::new("maintenance_record", record.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&existing), Some(&record))),
                request_context: None,
            },
        )
        .await?;

        Ok(record)
    }

    /// Transaction 版本：刪除維修紀錄
    pub(super) async fn delete_maintenance_record_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<()> {
        let before = sqlx::query_as::<_, EquipmentMaintenanceRecord>(
            "SELECT * FROM equipment_maintenance_records WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound("維修保養紀錄不存在".into()))?;

        sqlx::query("DELETE FROM equipment_maintenance_records WHERE id = $1")
            .bind(id)
            .execute(&mut **tx)
            .await?;

        let display = format!("maintenance {:?}", before.maintenance_type);
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "EQUIPMENT",
                event_type: "MAINTENANCE_DELETE",
                entity: Some(AuditEntity::new("maintenance_record", before.id, &display)),
                data_diff: Some(DataDiff::delete_only(&before)),
                request_context: None,
            },
        )
        .await?;

        Ok(())
    }

    pub async fn create_maintenance_record(
        pool: &PgPool,
        actor: &ActorContext,
        payload: &CreateMaintenanceRequest,
    ) -> Result<EquipmentMaintenanceRecord> {
        let current_user = actor.require_user()?;
        if !current_user.has_permission("equipment.maintenance.manage")
            && !current_user.has_permission("equipment.manage")
        {
            return Err(AppError::Forbidden("無權管理維修保養紀錄".into()));
        }

        let mut tx = pool.begin().await?;
        let record = Self::create_maintenance_record_tx(&mut tx, actor, payload).await?;
        tx.commit().await?;

        // P2-2: 報修自動通知維修人員（tx 外 fire-and-forget，避免拖延 commit）
        if payload.maintenance_type == MaintenanceType::Repair {
            if let Ok(Some(equipment)) = repositories::equipment::find_equipment_by_id(pool, payload.equipment_id).await {
                let notification_svc = crate::services::NotificationService::new(pool.clone());
                if let Err(e) = notification_svc
                    .send_equipment_repair_notification(
                        &equipment.name,
                        &current_user.email,
                        payload.problem_description.as_deref().unwrap_or("-"),
                    )
                    .await
                {
                    tracing::warn!("發送報修通知失敗: {e}");
                }
            }
        }

        Ok(record)
    }

    pub async fn update_maintenance_record(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        payload: &UpdateMaintenanceRequest,
    ) -> Result<EquipmentMaintenanceRecord> {
        let current_user = actor.require_user()?;
        if !current_user.has_permission("equipment.maintenance.manage")
            && !current_user.has_permission("equipment.manage")
        {
            return Err(AppError::Forbidden("無權管理維修保養紀錄".into()));
        }

        let mut tx = pool.begin().await?;
        let record = Self::update_maintenance_record_tx(&mut tx, actor, id, payload).await?;
        let existing_status = record.status.clone();
        tx.commit().await?;

        // 無法維修通知（tx 外 side effect）
        if existing_status == MaintenanceStatus::Unrepairable {
            if let Ok(Some(equip)) = repositories::equipment::find_equipment_by_id(pool, record.equipment_id).await {
                let notification_svc = crate::services::NotificationService::new(pool.clone());
                let problem = payload
                    .problem_description
                    .as_deref()
                    .unwrap_or("-");
                if let Err(e) = notification_svc
                    .send_equipment_unrepairable_notification(
                        &equip.name,
                        equip.serial_number.as_deref().unwrap_or("-"),
                        problem,
                    )
                    .await
                {
                    tracing::warn!("發送無法維修通知失敗: {e}");
                }
            }
        }

        Ok(record)
    }

    pub async fn delete_maintenance_record(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<()> {
        let current_user = actor.require_user()?;
        if !current_user.has_permission("equipment.maintenance.manage")
            && !current_user.has_permission("equipment.manage")
        {
            return Err(AppError::Forbidden("無權刪除維修保養紀錄".into()));
        }

        let mut tx = pool.begin().await?;
        Self::delete_maintenance_record_tx(&mut tx, actor, id).await?;
        tx.commit().await?;

        Ok(())
    }

    pub async fn review_maintenance_record(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        payload: &ReviewMaintenanceRequest,
    ) -> Result<EquipmentMaintenanceRecord> {
        let current_user = actor.require_user()?;
        if !current_user.has_permission("equipment.maintenance.review")
            && !current_user.has_permission("equipment.manage")
        {
            return Err(AppError::Forbidden("無權驗收維修保養紀錄".into()));
        }
        payload.validate()?;

        let mut tx = pool.begin().await?;

        let existing = sqlx::query_as::<_, EquipmentMaintenanceRecord>(
            "SELECT * FROM equipment_maintenance_records WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("維修保養紀錄不存在".into()))?;

        if existing.status != MaintenanceStatus::PendingReview {
            return Err(AppError::BadRequest("此紀錄非待驗收狀態".into()));
        }

        let new_status = if payload.approved {
            MaintenanceStatus::Completed
        } else {
            MaintenanceStatus::Pending
        };

        // 驗收通過 → 設備自動恢復啟用（tx 內執行 + 寫 EQUIPMENT_AUTO_RESTORE audit；
        // Gemini #166 HIGH 修正 tx 原子性 + Gemini #167 MEDIUM 補 audit）
        if payload.approved {
            auto_restore_equipment(&mut tx, actor, existing.equipment_id, current_user.id).await?;
        }

        let record = sqlx::query_as::<_, EquipmentMaintenanceRecord>(
            r#"
            UPDATE equipment_maintenance_records
            SET status = $2, reviewed_by = $3, reviewed_at = NOW(),
                review_notes = $4, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&new_status)
        .bind(current_user.id)
        .bind(&payload.review_notes)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!(
            "maintenance {:?} → {:?}",
            record.maintenance_type, new_status
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "EQUIPMENT",
                event_type: if payload.approved {
                    "MAINTENANCE_REVIEW_APPROVE"
                } else {
                    "MAINTENANCE_REVIEW_REJECT"
                },
                entity: Some(AuditEntity::new("maintenance_record", record.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&existing), Some(&record))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(record)
    }

    /// 為維修保養紀錄建立驗收簽章，與 record UPDATE 同 tx 原子。
    ///
    /// 流程（同一 tx 內）：
    ///   1. RBAC：`equipment.maintenance.review` / `equipment.manage`
    ///   2. SELECT FOR UPDATE 鎖 row + 狀態守衛（pending_review / 未簽過）
    ///   3. `SignatureService::sign_record_tx` 寫 electronic_signatures
    ///   4. UPDATE equipment_maintenance_records.reviewer_signature_id
    ///   5. audit log（service 層 log_activity_tx，21 CFR §11.10 audit trail）
    ///
    /// 任何步驟失敗 → 整個 tx rollback，不留簽章孤兒。
    #[allow(clippy::too_many_arguments)]
    pub async fn sign_maintenance_review_tx(
        pool: &PgPool,
        actor: &ActorContext,
        record_id: Uuid,
        sig_type: super::SignatureType,
        password: Option<&str>,
        handwriting_svg: Option<&str>,
        stroke_data: Option<&serde_json::Value>,
    ) -> Result<super::ElectronicSignature> {
        let current_user = actor.require_user()?;
        super::access::require_equipment_review(current_user)?;

        let mut tx = pool.begin().await?;

        let existing = sqlx::query_as::<_, EquipmentMaintenanceRecord>(
            "SELECT * FROM equipment_maintenance_records WHERE id = $1 FOR UPDATE",
        )
        .bind(record_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("維修保養紀錄不存在".into()))?;

        if existing.status != MaintenanceStatus::PendingReview {
            return Err(AppError::BadRequest("此紀錄非待驗收狀態，無法簽章".into()));
        }
        if existing.reviewer_signature_id.is_some() {
            return Err(AppError::Conflict("此紀錄已簽章，不得覆寫".into()));
        }

        let content = format!("maintenance_reviewer:{}", record_id);
        let signature = super::SignatureService::sign_record_tx(
            &mut tx,
            pool,
            "maintenance_reviewer",
            &record_id.to_string(),
            current_user.id,
            sig_type,
            &content,
            password,
            handwriting_svg,
            stroke_data,
        )
        .await?;

        let updated = sqlx::query_as::<_, EquipmentMaintenanceRecord>(
            "UPDATE equipment_maintenance_records \
             SET reviewer_signature_id = $1, updated_at = NOW() WHERE id = $2 \
             RETURNING *",
        )
        .bind(signature.id)
        .bind(record_id)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!(
            "maintenance_reviewer_signature:{:?}",
            updated.maintenance_type
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "EQUIPMENT",
                event_type: "MAINTENANCE_REVIEWER_SIGNATURE",
                entity: Some(AuditEntity::new("maintenance_record", record_id, &display)),
                data_diff: Some(DataDiff::compute(Some(&existing), Some(&updated))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(signature)
    }

    // ========== Disposal Records (報廢) ==========

    pub async fn list_disposals(
        pool: &PgPool,
        query: &DisposalQuery,
        current_user: &CurrentUser,
    ) -> Result<PaginatedResponse<DisposalWithDetails>> {
        check_view_permission(current_user)?;

        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(100);
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM equipment_disposals d
            WHERE ($1::uuid IS NULL OR d.equipment_id = $1)
              AND ($2::disposal_status IS NULL OR d.status = $2)
            "#,
        )
        .bind(query.equipment_id)
        .bind(&query.status)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, DisposalWithDetails>(
            r#"
            SELECT d.id, d.equipment_id, e.name AS equipment_name,
                   d.status, d.disposal_date, d.reason, d.disposal_method,
                   d.applied_by, u1.display_name AS applicant_name, d.applied_at,
                   d.approved_by, u2.display_name AS approver_name, d.approved_at,
                   d.rejection_reason, d.notes, d.created_at
            FROM equipment_disposals d
            INNER JOIN equipment e ON d.equipment_id = e.id
            INNER JOIN users u1 ON d.applied_by = u1.id
            LEFT JOIN users u2 ON d.approved_by = u2.id
            WHERE ($1::uuid IS NULL OR d.equipment_id = $1)
              AND ($2::disposal_status IS NULL OR d.status = $2)
            ORDER BY d.applied_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(query.equipment_id)
        .bind(&query.status)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    pub async fn create_disposal(
        pool: &PgPool,
        payload: &CreateDisposalRequest,
        current_user: &CurrentUser,
    ) -> Result<DisposalWithDetails> {
        check_manage_permission(current_user)?;
        payload.validate()?;

        // 驗證設備存在
        repositories::equipment::find_equipment_by_id(pool, payload.equipment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("設備不存在".into()))?;

        let record = sqlx::query_as::<_, DisposalWithDetails>(
            r#"
            WITH ins AS (
                INSERT INTO equipment_disposals (equipment_id, disposal_date, reason, disposal_method, applied_by, notes)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING *
            )
            SELECT ins.id, ins.equipment_id, e.name AS equipment_name,
                   ins.status, ins.disposal_date, ins.reason, ins.disposal_method,
                   ins.applied_by, u1.display_name AS applicant_name, ins.applied_at,
                   ins.approved_by, NULL::text AS approver_name, ins.approved_at,
                   ins.rejection_reason, ins.notes, ins.created_at
            FROM ins
            INNER JOIN equipment e ON ins.equipment_id = e.id
            INNER JOIN users u1 ON ins.applied_by = u1.id
            "#,
        )
        .bind(payload.equipment_id)
        .bind(payload.disposal_date)
        .bind(&payload.reason)
        .bind(&payload.disposal_method)
        .bind(current_user.id)
        .bind(&payload.notes)
        .fetch_one(pool)
        .await?;

        // 發送報廢申請通知
        let notification_svc = crate::services::NotificationService::new(pool.clone());
        if let Err(e) = notification_svc
            .send_equipment_disposal_notification(
                &record.equipment_name,
                &record.applicant_name,
                &payload.reason,
            )
            .await
        {
            tracing::warn!("發送報廢申請通知失敗: {e}");
        }

        Ok(record)
    }

    /// 為報廢申請建立申請人簽章，與 record UPDATE 同 tx 原子（21 CFR §11.10(e)(1)）。
    ///
    /// 流程（同一 tx 內）：
    ///   1. RBAC：`equipment.manage`（同 create_disposal）
    ///   2. SELECT FOR UPDATE 鎖 row + 狀態守衛（pending / 未簽過）+ 自簽檢查
    ///      （applied_by == current_user.id；申請人不能由他人代簽）
    ///   3. `SignatureService::sign_record_tx` 寫 electronic_signatures
    ///   4. UPDATE applicant_signature_id
    ///   5. audit log（log_activity_tx）
    pub async fn sign_disposal_applicant_tx(
        pool: &PgPool,
        actor: &ActorContext,
        disposal_id: Uuid,
        sig_type: super::SignatureType,
        password: Option<&str>,
        handwriting_svg: Option<&str>,
        stroke_data: Option<&serde_json::Value>,
    ) -> Result<super::ElectronicSignature> {
        let current_user = actor.require_user()?;
        super::access::require_equipment_manage(current_user)?;

        let mut tx = pool.begin().await?;

        let existing = sqlx::query_as::<_, EquipmentDisposal>(
            "SELECT * FROM equipment_disposals WHERE id = $1 FOR UPDATE",
        )
        .bind(disposal_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("報廢紀錄不存在".into()))?;

        if existing.status != DisposalStatus::Pending {
            return Err(AppError::BadRequest("此報廢申請非待處理狀態".into()));
        }
        if existing.applicant_signature_id.is_some() {
            return Err(AppError::Conflict("此報廢申請的申請人已簽章，不得覆寫".into()));
        }
        if existing.applied_by != current_user.id {
            return Err(AppError::Forbidden(
                "只有申請人本人可以簽章（不得代簽）".into(),
            ));
        }

        let content = format!("disposal_applicant:{}", disposal_id);
        let signature = super::SignatureService::sign_record_tx(
            &mut tx,
            pool,
            "disposal_applicant",
            &disposal_id.to_string(),
            current_user.id,
            sig_type,
            &content,
            password,
            handwriting_svg,
            stroke_data,
        )
        .await?;

        let updated = sqlx::query_as::<_, EquipmentDisposal>(
            "UPDATE equipment_disposals \
             SET applicant_signature_id = $1, updated_at = NOW() WHERE id = $2 \
             RETURNING *",
        )
        .bind(signature.id)
        .bind(disposal_id)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "EQUIPMENT",
                event_type: "DISPOSAL_APPLICANT_SIGNATURE",
                entity: Some(AuditEntity::new(
                    "equipment_disposal",
                    disposal_id,
                    "disposal_applicant_signature",
                )),
                data_diff: Some(DataDiff::compute(Some(&existing), Some(&updated))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(signature)
    }

    /// 為報廢申請建立核准人簽章，與 record UPDATE 同 tx 原子（21 CFR §11.10(e)(1)）。
    ///
    /// 流程（同一 tx 內）：
    ///   1. RBAC：`equipment.disposal.approve` 或 `equipment.manage`
    ///   2. SELECT FOR UPDATE 鎖 row + 狀態守衛（pending / 未簽過）+ 申請人不能自核
    ///      （applied_by != current_user.id；防止 self-approve 提權）
    ///   3. `SignatureService::sign_record_tx` 寫 electronic_signatures
    ///   4. UPDATE approver_signature_id
    ///   5. audit log（log_activity_tx）
    pub async fn sign_disposal_approver_tx(
        pool: &PgPool,
        actor: &ActorContext,
        disposal_id: Uuid,
        sig_type: super::SignatureType,
        password: Option<&str>,
        handwriting_svg: Option<&str>,
        stroke_data: Option<&serde_json::Value>,
    ) -> Result<super::ElectronicSignature> {
        let current_user = actor.require_user()?;
        super::access::require_equipment_disposal_approve(current_user)?;

        let mut tx = pool.begin().await?;

        let existing = sqlx::query_as::<_, EquipmentDisposal>(
            "SELECT * FROM equipment_disposals WHERE id = $1 FOR UPDATE",
        )
        .bind(disposal_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("報廢紀錄不存在".into()))?;

        if existing.status != DisposalStatus::Pending {
            return Err(AppError::BadRequest("此報廢申請非待處理狀態".into()));
        }
        if existing.approver_signature_id.is_some() {
            return Err(AppError::Conflict("此報廢申請的核准人已簽章，不得覆寫".into()));
        }
        if existing.applied_by == current_user.id {
            return Err(AppError::Forbidden(
                "申請人不得自核自簽（職權分離）".into(),
            ));
        }

        let content = format!("disposal_approver:{}", disposal_id);
        let signature = super::SignatureService::sign_record_tx(
            &mut tx,
            pool,
            "disposal_approver",
            &disposal_id.to_string(),
            current_user.id,
            sig_type,
            &content,
            password,
            handwriting_svg,
            stroke_data,
        )
        .await?;

        let updated = sqlx::query_as::<_, EquipmentDisposal>(
            "UPDATE equipment_disposals \
             SET approver_signature_id = $1, updated_at = NOW() WHERE id = $2 \
             RETURNING *",
        )
        .bind(signature.id)
        .bind(disposal_id)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "EQUIPMENT",
                event_type: "DISPOSAL_APPROVER_SIGNATURE",
                entity: Some(AuditEntity::new(
                    "equipment_disposal",
                    disposal_id,
                    "disposal_approver_signature",
                )),
                data_diff: Some(DataDiff::compute(Some(&existing), Some(&updated))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(signature)
    }

    pub async fn approve_disposal(
        pool: &PgPool,
        id: Uuid,
        payload: &ApproveDisposalRequest,
        current_user: &CurrentUser,
    ) -> Result<DisposalWithDetails> {
        if !current_user.has_permission("equipment.disposal.approve") {
            return Err(AppError::Forbidden("無權核准報廢申請".into()));
        }
        payload.validate()?;

        let existing = repositories::equipment::find_disposal_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("報廢紀錄不存在".into()))?;

        if existing.status != DisposalStatus::Pending {
            return Err(AppError::BadRequest("此報廢申請已處理".into()));
        }

        let new_status = if payload.approved {
            DisposalStatus::Approved
        } else {
            DisposalStatus::Rejected
        };

        sqlx::query(
            r#"
            UPDATE equipment_disposals
            SET status = $2, approved_by = $3, approved_at = NOW(),
                rejection_reason = $4, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(&new_status)
        .bind(current_user.id)
        .bind(&payload.rejection_reason)
        .execute(pool)
        .await?;

        // 核准後自動將設備狀態變為「報廢」
        if payload.approved {
            let equipment = repositories::equipment::find_equipment_by_id(pool, existing.equipment_id)
                .await?
                .ok_or_else(|| AppError::NotFound("設備不存在".into()))?;
            validate_status_transition(&equipment.status, &EquipmentStatus::Decommissioned)?;

            sqlx::query(
                "INSERT INTO equipment_status_logs (equipment_id, old_status, new_status, changed_by, reason) VALUES ($1, (SELECT status FROM equipment WHERE id = $1), 'decommissioned', $2, '報廢申請核准')",
            )
            .bind(existing.equipment_id)
            .bind(current_user.id)
            .execute(pool)
            .await?;

            sqlx::query("UPDATE equipment SET status = 'decommissioned', is_active = false, updated_at = NOW() WHERE id = $1")
                .bind(existing.equipment_id)
                .execute(pool)
                .await?;
        }

        // 重新查詢完整紀錄
        let record = sqlx::query_as::<_, DisposalWithDetails>(
            r#"
            SELECT d.id, d.equipment_id, e.name AS equipment_name,
                   d.status, d.disposal_date, d.reason, d.disposal_method,
                   d.applied_by, u1.display_name AS applicant_name, d.applied_at,
                   d.approved_by, u2.display_name AS approver_name, d.approved_at,
                   d.rejection_reason, d.notes, d.created_at
            FROM equipment_disposals d
            INNER JOIN equipment e ON d.equipment_id = e.id
            INNER JOIN users u1 ON d.applied_by = u1.id
            LEFT JOIN users u2 ON d.approved_by = u2.id
            WHERE d.id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// 管理員恢復已報廢設備（將 status 改回 active、is_active 改回 true）
    pub async fn restore_equipment(
        pool: &PgPool,
        disposal_id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<DisposalWithDetails> {
        if !current_user.has_permission("equipment.disposal.approve") {
            return Err(AppError::Forbidden("無權恢復報廢設備".into()));
        }

        let existing = repositories::equipment::find_disposal_by_id(pool, disposal_id)
            .await?
            .ok_or_else(|| AppError::NotFound("報廢紀錄不存在".into()))?;

        if existing.status != DisposalStatus::Approved {
            return Err(AppError::BadRequest("只能恢復已核准的報廢設備".into()));
        }

        // P2-1: 二次審批 — 恢復人不得與原核准人相同
        if let Some(approver) = existing.approved_by {
            if approver == current_user.id {
                return Err(AppError::BadRequest(
                    "報廢恢復需由原核准人以外的管理員執行（二次審批）".into(),
                ));
            }
        }

        validate_status_transition(&EquipmentStatus::Decommissioned, &EquipmentStatus::Active)?;

        // 記錄狀態變更日誌
        sqlx::query(
            "INSERT INTO equipment_status_logs (equipment_id, old_status, new_status, changed_by, reason) VALUES ($1, 'decommissioned', 'active', $2, '管理員恢復報廢設備（二次審批）')",
        )
        .bind(existing.equipment_id)
        .bind(current_user.id)
        .execute(pool)
        .await?;

        // 恢復設備狀態
        sqlx::query(
            "UPDATE equipment SET status = 'active', is_active = true, updated_at = NOW() WHERE id = $1",
        )
        .bind(existing.equipment_id)
        .execute(pool)
        .await?;

        // 將報廢紀錄狀態改為 rejected（表示已撤銷）
        sqlx::query(
            "UPDATE equipment_disposals SET status = 'rejected', rejection_reason = '管理員恢復設備', approved_by = $2, approved_at = NOW(), updated_at = NOW() WHERE id = $1",
        )
        .bind(disposal_id)
        .bind(current_user.id)
        .execute(pool)
        .await?;

        let record = sqlx::query_as::<_, DisposalWithDetails>(
            r#"
            SELECT d.id, d.equipment_id, e.name AS equipment_name,
                   d.status, d.disposal_date, d.reason, d.disposal_method,
                   d.applied_by, u1.display_name AS applicant_name, d.applied_at,
                   d.approved_by, u2.display_name AS approver_name, d.approved_at,
                   d.rejection_reason, d.notes, d.created_at
            FROM equipment_disposals d
            INNER JOIN equipment e ON d.equipment_id = e.id
            INNER JOIN users u1 ON d.applied_by = u1.id
            LEFT JOIN users u2 ON d.approved_by = u2.id
            WHERE d.id = $1
            "#,
        )
        .bind(disposal_id)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    // ========== Idle Requests (閒置審批) ==========

    pub async fn list_idle_requests(
        pool: &PgPool,
        query: &IdleRequestQuery,
        current_user: &CurrentUser,
    ) -> Result<PaginatedResponse<IdleRequestWithDetails>> {
        check_view_permission(current_user)?;

        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(100);
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM equipment_idle_requests ir
            WHERE ($1::uuid IS NULL OR ir.equipment_id = $1)
              AND ($2::disposal_status IS NULL OR ir.status = $2)
            "#,
        )
        .bind(query.equipment_id)
        .bind(&query.status)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, IdleRequestWithDetails>(
            r#"
            SELECT ir.id, ir.equipment_id, e.name AS equipment_name,
                   ir.request_type, ir.reason, ir.status,
                   ir.applied_by, u1.display_name AS applicant_name, ir.applied_at,
                   ir.approved_by, u2.display_name AS approver_name, ir.approved_at,
                   ir.rejection_reason, ir.notes, ir.created_at
            FROM equipment_idle_requests ir
            INNER JOIN equipment e ON ir.equipment_id = e.id
            INNER JOIN users u1 ON ir.applied_by = u1.id
            LEFT JOIN users u2 ON ir.approved_by = u2.id
            WHERE ($1::uuid IS NULL OR ir.equipment_id = $1)
              AND ($2::disposal_status IS NULL OR ir.status = $2)
            ORDER BY ir.created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(query.equipment_id)
        .bind(&query.status)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    pub async fn create_idle_request(
        pool: &PgPool,
        payload: &CreateIdleRequestRequest,
        current_user: &CurrentUser,
    ) -> Result<IdleRequestWithDetails> {
        check_manage_permission(current_user)?;
        payload.validate()?;

        if payload.request_type != "idle" && payload.request_type != "restore" {
            return Err(AppError::BadRequest(
                "request_type 必須為 'idle' 或 'restore'".into(),
            ));
        }

        let equipment = repositories::equipment::find_equipment_by_id(pool, payload.equipment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("設備不存在".into()))?;

        // 驗證狀態轉換合法性
        let target_status = if payload.request_type == "idle" {
            if equipment.status != EquipmentStatus::Active {
                return Err(AppError::BadRequest("只有啟用中的設備可以申請閒置".into()));
            }
            EquipmentStatus::Inactive
        } else {
            if equipment.status != EquipmentStatus::Inactive {
                return Err(AppError::BadRequest("只有閒置中的設備可以申請恢復".into()));
            }
            EquipmentStatus::Active
        };
        validate_status_transition(&equipment.status, &target_status)?;

        // 檢查是否已有待審批的申請
        let has_pending: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM equipment_idle_requests WHERE equipment_id = $1 AND status = 'pending')",
        )
        .bind(payload.equipment_id)
        .fetch_one(pool)
        .await?;

        if has_pending {
            return Err(AppError::BadRequest("該設備已有待審批的閒置/恢復申請".into()));
        }

        let record = sqlx::query_as::<_, IdleRequestWithDetails>(
            r#"
            WITH inserted AS (
                INSERT INTO equipment_idle_requests
                    (equipment_id, request_type, reason, applied_by, notes)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING *
            )
            SELECT i.id, i.equipment_id, e.name AS equipment_name,
                   i.request_type, i.reason, i.status,
                   i.applied_by, u1.display_name AS applicant_name, i.applied_at,
                   i.approved_by, NULL::text AS approver_name, i.approved_at,
                   i.rejection_reason, i.notes, i.created_at
            FROM inserted i
            INNER JOIN equipment e ON i.equipment_id = e.id
            INNER JOIN users u1 ON i.applied_by = u1.id
            "#,
        )
        .bind(payload.equipment_id)
        .bind(&payload.request_type)
        .bind(&payload.reason)
        .bind(current_user.id)
        .bind(&payload.notes)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn approve_idle_request(
        pool: &PgPool,
        id: Uuid,
        payload: &ApproveIdleRequestRequest,
        current_user: &CurrentUser,
    ) -> Result<IdleRequestWithDetails> {
        if !current_user.has_permission("equipment.idle.approve") {
            return Err(AppError::Forbidden("無權核准閒置申請".into()));
        }
        payload.validate()?;

        let existing = sqlx::query_as::<_, IdleRequestWithDetails>(
            r#"
            SELECT ir.id, ir.equipment_id, e.name AS equipment_name,
                   ir.request_type, ir.reason, ir.status,
                   ir.applied_by, u1.display_name AS applicant_name, ir.applied_at,
                   ir.approved_by, u2.display_name AS approver_name, ir.approved_at,
                   ir.rejection_reason, ir.notes, ir.created_at
            FROM equipment_idle_requests ir
            INNER JOIN equipment e ON ir.equipment_id = e.id
            INNER JOIN users u1 ON ir.applied_by = u1.id
            LEFT JOIN users u2 ON ir.approved_by = u2.id
            WHERE ir.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("閒置申請不存在".into()))?;

        if existing.status != DisposalStatus::Pending {
            return Err(AppError::BadRequest("此申請已處理".into()));
        }

        let new_status = if payload.approved {
            DisposalStatus::Approved
        } else {
            DisposalStatus::Rejected
        };

        sqlx::query(
            r#"
            UPDATE equipment_idle_requests
            SET status = $2, approved_by = $3, approved_at = NOW(),
                rejection_reason = $4, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(&new_status)
        .bind(current_user.id)
        .bind(&payload.rejection_reason)
        .execute(pool)
        .await?;

        if payload.approved {
            let equipment = repositories::equipment::find_equipment_by_id(pool, existing.equipment_id)
                .await?
                .ok_or_else(|| AppError::NotFound("設備不存在".into()))?;

            let (target_status, is_active, reason) = if existing.request_type == "idle" {
                (EquipmentStatus::Inactive, false, "閒置申請核准")
            } else {
                (EquipmentStatus::Active, true, "閒置恢復申請核准")
            };

            validate_status_transition(&equipment.status, &target_status)?;

            sqlx::query(
                "INSERT INTO equipment_status_logs (equipment_id, old_status, new_status, changed_by, reason) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(existing.equipment_id)
            .bind(&equipment.status)
            .bind(&target_status)
            .bind(current_user.id)
            .bind(reason)
            .execute(pool)
            .await?;

            sqlx::query(
                "UPDATE equipment SET status = $2, is_active = $3, updated_at = NOW() WHERE id = $1",
            )
            .bind(existing.equipment_id)
            .bind(&target_status)
            .bind(is_active)
            .execute(pool)
            .await?;
        }

        // P2-3: 通知申請人審批結果
        {
            let notification_svc = crate::services::NotificationService::new(pool.clone());
            let action = if payload.approved { "核准" } else { "駁回" };
            let type_label = if existing.request_type == "idle" { "閒置" } else { "恢復" };
            if let Err(e) = notification_svc
                .create_notification(crate::models::CreateNotificationRequest {
                    user_id: existing.applied_by,
                    notification_type: crate::models::NotificationType::SystemAlert,
                    title: format!("設備{}申請已{}", type_label, action),
                    content: Some(format!(
                        "您的設備「{}」{}申請已被{}。",
                        existing.equipment_name, type_label, action
                    )),
                    related_entity_type: Some("equipment".to_string()),
                    related_entity_id: None,
                })
                .await
            {
                tracing::warn!("發送閒置審批結果通知失敗: {e}");
            }
        }

        // 重新查詢完整紀錄
        let record = sqlx::query_as::<_, IdleRequestWithDetails>(
            r#"
            SELECT ir.id, ir.equipment_id, e.name AS equipment_name,
                   ir.request_type, ir.reason, ir.status,
                   ir.applied_by, u1.display_name AS applicant_name, ir.applied_at,
                   ir.approved_by, u2.display_name AS approver_name, ir.approved_at,
                   ir.rejection_reason, ir.notes, ir.created_at
            FROM equipment_idle_requests ir
            INNER JOIN equipment e ON ir.equipment_id = e.id
            INNER JOIN users u1 ON ir.applied_by = u1.id
            LEFT JOIN users u2 ON ir.approved_by = u2.id
            WHERE ir.id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    // ========== Annual Plan (年度計畫) ==========

    pub async fn list_annual_plans(
        pool: &PgPool,
        query: &AnnualPlanQuery,
        current_user: &CurrentUser,
    ) -> Result<Vec<AnnualPlanWithEquipment>> {
        check_view_permission(current_user)?;

        let data = sqlx::query_as::<_, AnnualPlanWithEquipment>(
            r#"
            SELECT ap.id, ap.year, ap.equipment_id, e.name AS equipment_name,
                   e.serial_number AS equipment_serial_number,
                   ap.calibration_type, ap.cycle,
                   ap.month_1, ap.month_2, ap.month_3, ap.month_4,
                   ap.month_5, ap.month_6, ap.month_7, ap.month_8,
                   ap.month_9, ap.month_10, ap.month_11, ap.month_12,
                   ap.generated_at
            FROM equipment_annual_plans ap
            INNER JOIN equipment e ON ap.equipment_id = e.id
            WHERE ap.year = $1
              AND ($2::uuid IS NULL OR ap.equipment_id = $2)
              AND ($3::calibration_type IS NULL OR ap.calibration_type = $3)
            ORDER BY e.name, ap.calibration_type
            "#,
        )
        .bind(query.year)
        .bind(query.equipment_id)
        .bind(&query.calibration_type)
        .fetch_all(pool)
        .await?;

        Ok(data)
    }

    pub async fn generate_annual_plan(
        pool: &PgPool,
        payload: &GenerateAnnualPlanRequest,
        current_user: &CurrentUser,
    ) -> Result<Vec<AnnualPlanWithEquipment>> {
        if !current_user.has_permission("equipment.plan.manage")
            && !current_user.has_permission("equipment.manage")
        {
            return Err(AppError::Forbidden("無權管理年度計畫".into()));
        }

        // 取得所有啟用的設備（有設定週期的）
        let equipment_list = sqlx::query_as::<_, Equipment>(
            "SELECT * FROM equipment WHERE status = 'active' AND (calibration_cycle IS NOT NULL OR inspection_cycle IS NOT NULL)",
        )
        .fetch_all(pool)
        .await?;

        // 查詢各設備最後一次校正月份，用於智慧月份推算
        #[derive(sqlx::FromRow)]
        struct LastCalMonth {
            equipment_id: Uuid,
            calibration_type: crate::models::CalibrationType,
            last_month: i32,
        }
        let eq_ids: Vec<Uuid> = equipment_list.iter().map(|e| e.id).collect();
        let last_cal_records = if eq_ids.is_empty() {
            vec![]
        } else {
            sqlx::query_as::<_, LastCalMonth>(
                r#"
                SELECT equipment_id, calibration_type,
                       EXTRACT(MONTH FROM MAX(calibrated_at))::int AS last_month
                FROM equipment_calibrations
                WHERE equipment_id = ANY($1)
                GROUP BY equipment_id, calibration_type
                "#,
            )
            .bind(&eq_ids)
            .fetch_all(pool)
            .await?
        };

        use std::collections::HashMap;
        let mut last_cal_map: HashMap<(Uuid, String), u32> = HashMap::new();
        for rec in last_cal_records {
            let type_key = format!("{:?}", rec.calibration_type).to_lowercase();
            last_cal_map.insert((rec.equipment_id, type_key), rec.last_month as u32);
        }

        // 預先產生所有月份（避免持有 thread_rng 跨 await）
        let plans: Vec<_> = {
            let mut rng = rand::thread_rng();
            equipment_list
                .iter()
                .flat_map(|eq| {
                    let mut items = Vec::new();
                    if let (Some(cal_type), Some(cycle)) =
                        (&eq.calibration_type, &eq.calibration_cycle)
                    {
                        let type_key = format!("{:?}", cal_type).to_lowercase();
                        let last_month = last_cal_map.get(&(eq.id, type_key)).copied();
                        let months = pick_smart_months(cycle, last_month, &mut rng);
                        items.push((eq.id, cal_type.clone(), cycle.clone(), months));
                    }
                    if let Some(cycle) = &eq.inspection_cycle {
                        let type_key = "inspection".to_string();
                        let last_month = last_cal_map.get(&(eq.id, type_key)).copied();
                        let months = pick_smart_months(cycle, last_month, &mut rng);
                        items.push((
                            eq.id,
                            crate::models::CalibrationType::Inspection,
                            cycle.clone(),
                            months,
                        ));
                    }
                    items
                })
                .collect()
        };

        for (eq_id, cal_type, cycle, months) in &plans {
            insert_annual_plan(pool, payload.year, *eq_id, cal_type, cycle, months).await?;
        }

        // 回傳產生的計畫
        let query = AnnualPlanQuery {
            year: payload.year,
            equipment_id: None,
            calibration_type: None,
        };
        Self::list_annual_plans(pool, &query, current_user).await
    }

    pub async fn create_annual_plan(
        pool: &PgPool,
        payload: &CreateAnnualPlanRequest,
        current_user: &CurrentUser,
    ) -> Result<AnnualPlanWithEquipment> {
        if !current_user.has_permission("equipment.plan.manage")
            && !current_user.has_permission("equipment.manage")
        {
            return Err(AppError::Forbidden("無權管理年度計畫".into()));
        }

        let months = [
            payload.month_1, payload.month_2, payload.month_3, payload.month_4,
            payload.month_5, payload.month_6, payload.month_7, payload.month_8,
            payload.month_9, payload.month_10, payload.month_11, payload.month_12,
        ];
        insert_annual_plan(
            pool,
            payload.year,
            payload.equipment_id,
            &payload.calibration_type,
            &payload.cycle,
            &months,
        )
        .await?;

        let plan = sqlx::query_as::<_, AnnualPlanWithEquipment>(
            r#"
            SELECT ap.id, ap.year, ap.equipment_id, e.name AS equipment_name,
                   e.serial_number AS equipment_serial_number,
                   ap.calibration_type, ap.cycle,
                   ap.month_1, ap.month_2, ap.month_3, ap.month_4,
                   ap.month_5, ap.month_6, ap.month_7, ap.month_8,
                   ap.month_9, ap.month_10, ap.month_11, ap.month_12,
                   ap.generated_at
            FROM equipment_annual_plans ap
            INNER JOIN equipment e ON ap.equipment_id = e.id
            WHERE ap.year = $1 AND ap.equipment_id = $2 AND ap.calibration_type = $3
            "#,
        )
        .bind(payload.year)
        .bind(payload.equipment_id)
        .bind(&payload.calibration_type)
        .fetch_one(pool)
        .await?;

        Ok(plan)
    }

    pub async fn update_annual_plan(
        pool: &PgPool,
        id: Uuid,
        payload: &UpdateAnnualPlanRequest,
        current_user: &CurrentUser,
    ) -> Result<AnnualPlanWithEquipment> {
        if !current_user.has_permission("equipment.plan.manage")
            && !current_user.has_permission("equipment.manage")
        {
            return Err(AppError::Forbidden("無權管理年度計畫".into()));
        }

        sqlx::query(
            r#"
            UPDATE equipment_annual_plans SET
                calibration_type = COALESCE($2::calibration_type, calibration_type),
                cycle = COALESCE($3::calibration_cycle, cycle),
                month_1 = $4, month_2 = $5, month_3 = $6, month_4 = $7,
                month_5 = $8, month_6 = $9, month_7 = $10, month_8 = $11,
                month_9 = $12, month_10 = $13, month_11 = $14, month_12 = $15,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(payload.calibration_type.as_ref())
        .bind(payload.cycle.as_ref())
        .bind(payload.month_1)
        .bind(payload.month_2)
        .bind(payload.month_3)
        .bind(payload.month_4)
        .bind(payload.month_5)
        .bind(payload.month_6)
        .bind(payload.month_7)
        .bind(payload.month_8)
        .bind(payload.month_9)
        .bind(payload.month_10)
        .bind(payload.month_11)
        .bind(payload.month_12)
        .execute(pool)
        .await?;

        let plan = sqlx::query_as::<_, AnnualPlanWithEquipment>(
            r#"
            SELECT ap.id, ap.year, ap.equipment_id, e.name AS equipment_name,
                   e.serial_number AS equipment_serial_number,
                   ap.calibration_type, ap.cycle,
                   ap.month_1, ap.month_2, ap.month_3, ap.month_4,
                   ap.month_5, ap.month_6, ap.month_7, ap.month_8,
                   ap.month_9, ap.month_10, ap.month_11, ap.month_12,
                   ap.generated_at
            FROM equipment_annual_plans ap
            INNER JOIN equipment e ON ap.equipment_id = e.id
            WHERE ap.id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(plan)
    }

    pub async fn delete_annual_plan(
        pool: &PgPool,
        id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<()> {
        if !current_user.has_permission("equipment.plan.manage")
            && !current_user.has_permission("equipment.manage")
        {
            return Err(AppError::Forbidden("無權管理年度計畫".into()));
        }

        let result = sqlx::query("DELETE FROM equipment_annual_plans WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("年度計畫項目不存在".into()));
        }

        Ok(())
    }

    pub async fn get_execution_summary(
        pool: &PgPool,
        query: &ExecutionSummaryQuery,
        current_user: &CurrentUser,
    ) -> Result<AnnualPlanExecutionSummary> {
        check_view_permission(current_user)?;

        // 查詢 1：取計畫列
        let plans = sqlx::query_as::<_, AnnualPlanWithEquipment>(
            r#"
            SELECT ap.id, ap.year, ap.equipment_id, e.name AS equipment_name,
                   e.serial_number AS equipment_serial_number,
                   ap.calibration_type, ap.cycle,
                   ap.month_1, ap.month_2, ap.month_3, ap.month_4,
                   ap.month_5, ap.month_6, ap.month_7, ap.month_8,
                   ap.month_9, ap.month_10, ap.month_11, ap.month_12,
                   ap.generated_at
            FROM equipment_annual_plans ap
            INNER JOIN equipment e ON ap.equipment_id = e.id
            WHERE ap.year = $1
              AND ($2::uuid IS NULL OR ap.equipment_id = $2)
              AND ($3::calibration_type IS NULL OR ap.calibration_type = $3)
            ORDER BY e.name, ap.calibration_type
            "#,
        )
        .bind(query.year)
        .bind(query.equipment_id)
        .bind(&query.calibration_type)
        .fetch_all(pool)
        .await?;

        // 查詢 2：取該年度實際校正記錄（每組 equipment+type+month 取最早一筆）
        #[derive(sqlx::FromRow)]
        struct CalRecord {
            equipment_id: Uuid,
            calibration_type: crate::models::CalibrationType,
            cal_month: i32,
            calibration_id: Uuid,
            calibrated_at: chrono::NaiveDate,
            result: Option<String>,
        }

        let cal_records = sqlx::query_as::<_, CalRecord>(
            r#"
            SELECT DISTINCT ON (ec.equipment_id, ec.calibration_type, EXTRACT(MONTH FROM ec.calibrated_at))
                ec.equipment_id,
                ec.calibration_type,
                EXTRACT(MONTH FROM ec.calibrated_at)::int AS cal_month,
                ec.id AS calibration_id,
                ec.calibrated_at,
                ec.result
            FROM equipment_calibrations ec
            WHERE EXTRACT(YEAR FROM ec.calibrated_at) = $1
              AND ($2::uuid IS NULL OR ec.equipment_id = $2)
              AND ($3::calibration_type IS NULL OR ec.calibration_type = $3)
            ORDER BY ec.equipment_id, ec.calibration_type, EXTRACT(MONTH FROM ec.calibrated_at), ec.calibrated_at ASC
            "#,
        )
        .bind(query.year)
        .bind(query.equipment_id)
        .bind(&query.calibration_type)
        .fetch_all(pool)
        .await?;

        // 建立 HashMap 加速查找
        use std::collections::HashMap;
        let mut cal_map: HashMap<(Uuid, String, i32), CalRecord> = HashMap::new();
        for rec in cal_records {
            let type_key = format!("{:?}", rec.calibration_type).to_lowercase();
            cal_map.insert((rec.equipment_id, type_key, rec.cal_month), rec);
        }

        let today = chrono::Local::now().date_naive();

        let mut rows: Vec<AnnualPlanExecutionRow> = Vec::new();
        let mut total_planned = 0i32;
        let mut total_completed = 0i32;
        let mut total_overdue = 0i32;

        for plan in &plans {
            let type_key = format!("{:?}", plan.calibration_type).to_lowercase();
            let plan_months = [
                plan.month_1, plan.month_2, plan.month_3, plan.month_4,
                plan.month_5, plan.month_6, plan.month_7, plan.month_8,
                plan.month_9, plan.month_10, plan.month_11, plan.month_12,
            ];

            let mut month_details: Vec<MonthExecutionDetail> = Vec::with_capacity(12);
            let mut planned_count = 0i32;
            let mut completed_count = 0i32;
            let mut overdue_count = 0i32;

            for m in 1i32..=12 {
                let planned = plan_months[(m - 1) as usize];
                let cal = cal_map.get(&(plan.equipment_id, type_key.clone(), m));

                let status = match (planned, cal) {
                    (false, _) => MonthExecutionStatus::Unplanned,
                    (true, Some(_)) => MonthExecutionStatus::Completed,
                    (true, None) => {
                        // 月末日期：m+1月1日前一天，12月用12/31
                        let month_end = if m < 12 {
                            chrono::NaiveDate::from_ymd_opt(query.year, (m + 1) as u32, 1)
                                .and_then(|d| d.pred_opt())
                                .unwrap_or(
                                    chrono::NaiveDate::from_ymd_opt(query.year, m as u32, 28)
                                        .expect("valid fallback date m/28"),
                                )
                        } else {
                            chrono::NaiveDate::from_ymd_opt(query.year, 12, 31)
                                .expect("valid date 12/31")
                        };
                        if month_end < today {
                            MonthExecutionStatus::Overdue
                        } else {
                            MonthExecutionStatus::PlannedPending
                        }
                    }
                };

                if planned {
                    planned_count += 1;
                }
                if status == MonthExecutionStatus::Completed {
                    completed_count += 1;
                }
                if status == MonthExecutionStatus::Overdue {
                    overdue_count += 1;
                }

                month_details.push(MonthExecutionDetail {
                    month: m,
                    planned,
                    status,
                    calibration_id: cal.map(|c| c.calibration_id),
                    calibrated_at: cal.map(|c| c.calibrated_at),
                    result: cal.and_then(|c| c.result.clone()),
                });
            }

            total_planned += planned_count;
            total_completed += completed_count;
            total_overdue += overdue_count;

            rows.push(AnnualPlanExecutionRow {
                plan_id: plan.id,
                year: plan.year,
                equipment_id: plan.equipment_id,
                equipment_name: plan.equipment_name.clone(),
                equipment_serial_number: plan.equipment_serial_number.clone(),
                calibration_type: plan.calibration_type.clone(),
                cycle: plan.cycle.clone(),
                months: month_details,
                planned_count,
                completed_count,
                overdue_count,
            });
        }

        let completion_rate = if total_planned > 0 {
            total_completed as f64 / total_planned as f64
        } else {
            0.0
        };

        Ok(AnnualPlanExecutionSummary {
            year: query.year,
            total_planned,
            total_completed,
            total_overdue,
            completion_rate,
            rows,
        })
    }
}

/// 設備自動恢復啟用（維修驗收通過時呼叫）。
///
/// 接受 `&mut Transaction`，讓 caller（`review_maintenance_record`）能將本函式
/// 的 status_log INSERT + equipment UPDATE **與同 tx 的 maintenance record 狀態
/// 更新 + audit** 一起原子落地。
///
/// Gemini PR #166 HIGH 指出：原 `pool`-based 版本在後續 maintenance record UPDATE
/// 失敗時，equipment status 已獨立 commit，產生資料不一致（設備已「恢復」但
/// 維修紀錄卻沒通過驗收）。本修復將函式簽名改為接 tx，由 caller 保證同 tx。
async fn auto_restore_equipment(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    actor: &ActorContext,
    equipment_id: Uuid,
    user_id: Uuid,
) -> Result<()> {
    // tx 內 SELECT FOR UPDATE：避免與其他 equipment status 變更併發衝突
    let before = sqlx::query_as::<_, Equipment>(
        "SELECT * FROM equipment WHERE id = $1 FOR UPDATE",
    )
    .bind(equipment_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| AppError::NotFound("設備不存在".into()))?;
    validate_status_transition(&before.status, &EquipmentStatus::Active)?;

    sqlx::query(
        "INSERT INTO equipment_status_logs (equipment_id, old_status, new_status, changed_by, reason) VALUES ($1, $2, 'active', $3, '維修驗收通過，自動恢復狀態')",
    )
    .bind(equipment_id)
    .bind(&before.status)
    .bind(user_id)
    .execute(&mut **tx)
    .await?;

    let after = sqlx::query_as::<_, Equipment>(
        "UPDATE equipment SET status = 'active', is_active = true, updated_at = NOW() WHERE id = $1 RETURNING *",
    )
    .bind(equipment_id)
    .fetch_one(&mut **tx)
    .await?;

    // Gemini PR #167 MEDIUM：R26 DoD-1 要求所有 mutation（含狀態轉換）同 tx 寫 audit。
    let display = format!("{} → active", after.name);
    AuditService::log_activity_tx(
        tx,
        actor,
        ActivityLogEntry {
            event_category: "EQUIPMENT",
            event_type: "EQUIPMENT_AUTO_RESTORE",
            entity: Some(AuditEntity::new("equipment", after.id, &display)),
            data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
            request_context: None,
        },
    )
    .await?;

    Ok(())
}

fn build_timeline_entry(row: TimelineRow) -> EquipmentTimelineEntry {
    let title = match row.event_type.as_str() {
        "maintenance" => format!(
            "{}  —  {}",
            match row.sub_type.as_deref() {
                Some("repair") => "維修",
                Some("maintenance") => "保養",
                _ => "維修/保養",
            },
            match row.sub_status.as_deref() {
                Some("pending") => "待處理",
                Some("in_progress") => "進行中",
                Some("completed") => "已完成",
                Some("unrepairable") => "無法維修",
                Some("pending_review") => "待驗收",
                _ => "—",
            }
        ),
        "calibration" => format!(
            "{}  —  {}",
            match row.sub_type.as_deref() {
                Some("calibration") => "校正",
                Some("validation") => "確效",
                Some("inspection") => "查核",
                _ => "校正/確效/查核",
            },
            row.sub_status.as_deref().unwrap_or("—")
        ),
        "status_change" => format!(
            "狀態變更：{} → {}",
            row.sub_type.as_deref().unwrap_or("?"),
            row.sub_status.as_deref().unwrap_or("?")
        ),
        _ => "未知事件".to_string(),
    };

    let detail = serde_json::json!({
        "summary": row.summary,
        "notes": row.notes,
        "actor_name": row.actor_name,
        "sub_type": row.sub_type,
        "sub_status": row.sub_status,
    });

    EquipmentTimelineEntry {
        id: row.id,
        event_type: row.event_type,
        occurred_at: row.occurred_at,
        title,
        subtitle: row.summary,
        detail,
    }
}

fn pick_smart_months(
    cycle: &CalibrationCycle,
    last_month: Option<u32>,
    rng: &mut impl Rng,
) -> [bool; 12] {
    let Some(lm) = last_month else {
        return pick_random_months(cycle, rng);
    };
    let lm = lm.clamp(1, 12) as usize - 1; // 0-indexed
    let mut months = [false; 12];
    match cycle {
        CalibrationCycle::Monthly => {
            months = [true; 12];
        }
        CalibrationCycle::Quarterly => {
            // 保持與歷史相同的季內偏移（0/1/2）
            let offset = lm % 3;
            for q in 0..4 {
                months[q * 3 + offset] = true;
            }
        }
        CalibrationCycle::SemiAnnual => {
            // 上半年取歷史月份，下半年加 6
            let first = lm % 6;
            let second = (first + 6) % 12;
            months[first] = true;
            months[second] = true;
        }
        CalibrationCycle::Annual => {
            months[lm] = true;
        }
    }
    months
}

fn pick_random_months(cycle: &CalibrationCycle, rng: &mut impl Rng) -> [bool; 12] {
    let mut months = [false; 12];
    match cycle {
        CalibrationCycle::Monthly => {
            months = [true; 12];
        }
        CalibrationCycle::Quarterly => {
            // 每季隨機選一個月
            for quarter in 0..4 {
                let start = quarter * 3;
                let pick = start + rng.gen_range(0..3);
                months[pick] = true;
            }
        }
        CalibrationCycle::SemiAnnual => {
            // 每半年隨機選一個月
            let first = rng.gen_range(0..6);
            let second = 6 + rng.gen_range(0..6);
            months[first] = true;
            months[second] = true;
        }
        CalibrationCycle::Annual => {
            // 一年隨機選一個月
            let pick = rng.gen_range(0..12);
            months[pick] = true;
        }
    }
    months
}

async fn insert_annual_plan(
    pool: &PgPool,
    year: i32,
    equipment_id: Uuid,
    calibration_type: &crate::models::CalibrationType,
    cycle: &CalibrationCycle,
    months: &[bool; 12],
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO equipment_annual_plans
            (year, equipment_id, calibration_type, cycle,
             month_1, month_2, month_3, month_4, month_5, month_6,
             month_7, month_8, month_9, month_10, month_11, month_12)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        ON CONFLICT (year, equipment_id, calibration_type)
        DO UPDATE SET
            cycle = EXCLUDED.cycle,
            month_1 = EXCLUDED.month_1, month_2 = EXCLUDED.month_2,
            month_3 = EXCLUDED.month_3, month_4 = EXCLUDED.month_4,
            month_5 = EXCLUDED.month_5, month_6 = EXCLUDED.month_6,
            month_7 = EXCLUDED.month_7, month_8 = EXCLUDED.month_8,
            month_9 = EXCLUDED.month_9, month_10 = EXCLUDED.month_10,
            month_11 = EXCLUDED.month_11, month_12 = EXCLUDED.month_12,
            generated_at = NOW(), updated_at = NOW()
        "#,
    )
    .bind(year)
    .bind(equipment_id)
    .bind(calibration_type)
    .bind(cycle)
    .bind(months[0])
    .bind(months[1])
    .bind(months[2])
    .bind(months[3])
    .bind(months[4])
    .bind(months[5])
    .bind(months[6])
    .bind(months[7])
    .bind(months[8])
    .bind(months[9])
    .bind(months[10])
    .bind(months[11])
    .execute(pool)
    .await?;

    Ok(())
}
