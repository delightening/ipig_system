// 設備維護管理 Service (實驗室 GLP)

use rand::Rng;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    middleware::CurrentUser,
    models::{
        AnnualPlanQuery, AnnualPlanWithEquipment, ApproveDisposalRequest, CalibrationCycle,
        CalibrationQuery, CalibrationWithEquipment, CreateCalibrationRequest,
        CreateDisposalRequest, CreateEquipmentRequest, CreateEquipmentSupplierRequest,
        CreateMaintenanceRequest, DisposalQuery, DisposalStatus, DisposalWithDetails, Equipment,
        EquipmentCalibration, EquipmentMaintenanceRecord, EquipmentQuery, EquipmentStatus,
        EquipmentStatusLog, EquipmentSupplierWithPartner, GenerateAnnualPlanRequest,
        MaintenanceQuery, MaintenanceRecordWithDetails, MaintenanceStatus, MaintenanceType,
        PaginatedResponse, UpdateCalibrationRequest, UpdateEquipmentRequest,
        UpdateMaintenanceRequest,
    },
    repositories, Result,
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
            INSERT INTO equipment (name, model, serial_number, location, notes, calibration_type, calibration_cycle, inspection_cycle)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(&payload.name)
        .bind(&payload.model)
        .bind(&payload.serial_number)
        .bind(&payload.location)
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
        let notes = payload.notes.as_ref().or(existing.notes.as_ref()).cloned();
        let is_active = payload.is_active.unwrap_or(existing.is_active);
        let new_status = payload.status.clone().unwrap_or(existing.status.clone());
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

        // 狀態變更時記錄 audit trail
        if new_status != existing.status {
            sqlx::query(
                r#"
                INSERT INTO equipment_status_logs (equipment_id, old_status, new_status, changed_by)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(id)
            .bind(&existing.status)
            .bind(&new_status)
            .bind(current_user.id)
            .execute(pool)
            .await?;
        }

        let record = sqlx::query_as::<_, Equipment>(
            r#"
            UPDATE equipment
            SET name = $2, model = $3, serial_number = $4, location = $5,
                notes = $6, is_active = $7, status = $8,
                calibration_type = $9, calibration_cycle = $10, inspection_cycle = $11,
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
        .bind(notes)
        .bind(is_active)
        .bind(&new_status)
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

    pub async fn list_equipment_suppliers(
        pool: &PgPool,
        equipment_id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<Vec<EquipmentSupplierWithPartner>> {
        check_view_permission(current_user)?;
        let data = sqlx::query_as::<_, EquipmentSupplierWithPartner>(
            r#"
            SELECT es.id, es.equipment_id, es.partner_id, p.name AS partner_name,
                   es.contact_person, es.contact_phone, es.contact_email, es.notes, es.created_at
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
                   ins.contact_person, ins.contact_phone, ins.contact_email, ins.notes, ins.created_at
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
                ec.report_number, ec.inspector, ec.created_at
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
                 partner_id, report_number, inspector, equipment_serial_number)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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

        let record = sqlx::query_as::<_, EquipmentCalibration>(
            r#"
            UPDATE equipment_calibrations
            SET calibration_type = $2, calibrated_at = $3, next_due_at = $4,
                result = $5, notes = $6, partner_id = $7,
                report_number = $8, inspector = $9, updated_at = NOW()
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
                   m.created_by, m.created_at
            FROM equipment_maintenance_records m
            INNER JOIN equipment e ON m.equipment_id = e.id
            LEFT JOIN partners p ON m.repair_partner_id = p.id
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

    pub async fn create_maintenance_record(
        pool: &PgPool,
        payload: &CreateMaintenanceRequest,
        current_user: &CurrentUser,
    ) -> Result<EquipmentMaintenanceRecord> {
        if !current_user.has_permission("equipment.maintenance.manage")
            && !current_user.has_permission("equipment.manage")
        {
            return Err(AppError::Forbidden("無權管理維修保養紀錄".into()));
        }
        payload.validate()?;

        // 驗證設備存在
        let equipment = repositories::equipment::find_equipment_by_id(pool, payload.equipment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("設備不存在".into()))?;

        // 維修類型自動變更設備狀態為「維修中」
        if payload.maintenance_type == MaintenanceType::Repair
            && equipment.status == EquipmentStatus::Active
        {
            sqlx::query(
                "INSERT INTO equipment_status_logs (equipment_id, old_status, new_status, changed_by, reason) VALUES ($1, $2, 'under_repair', $3, '建立維修紀錄，自動變更狀態')",
            )
            .bind(payload.equipment_id)
            .bind(&equipment.status)
            .bind(current_user.id)
            .execute(pool)
            .await?;

            sqlx::query("UPDATE equipment SET status = 'under_repair', is_active = false, updated_at = NOW() WHERE id = $1")
                .bind(payload.equipment_id)
                .execute(pool)
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
        .bind(current_user.id)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn update_maintenance_record(
        pool: &PgPool,
        id: Uuid,
        payload: &UpdateMaintenanceRequest,
        current_user: &CurrentUser,
    ) -> Result<EquipmentMaintenanceRecord> {
        if !current_user.has_permission("equipment.maintenance.manage")
            && !current_user.has_permission("equipment.manage")
        {
            return Err(AppError::Forbidden("無權管理維修保養紀錄".into()));
        }
        payload.validate()?;

        let existing = repositories::equipment::find_maintenance_record_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("維修保養紀錄不存在".into()))?;

        let new_status = payload.status.clone().unwrap_or(existing.status.clone());

        // 完修後自動恢復設備狀態為「啟用」
        if new_status == MaintenanceStatus::Completed && existing.status != MaintenanceStatus::Completed {
            sqlx::query(
                "INSERT INTO equipment_status_logs (equipment_id, old_status, new_status, changed_by, reason) VALUES ($1, (SELECT status FROM equipment WHERE id = $1), 'active', $2, '維修完成，自動恢復狀態')",
            )
            .bind(existing.equipment_id)
            .bind(current_user.id)
            .execute(pool)
            .await?;

            sqlx::query("UPDATE equipment SET status = 'active', is_active = true, updated_at = NOW() WHERE id = $1")
                .bind(existing.equipment_id)
                .execute(pool)
                .await?;
        }

        // 無法維修 → 發送通知給設備管理人與機構負責人
        if new_status == MaintenanceStatus::Unrepairable
            && existing.status != MaintenanceStatus::Unrepairable
        {
            let equip = repositories::equipment::find_equipment_by_id(pool, existing.equipment_id)
                .await?;
            if let Some(equip) = equip {
                let notification_svc =
                    crate::services::NotificationService::new(pool.clone());
                let problem = payload
                    .problem_description
                    .as_deref()
                    .or(existing.problem_description.as_deref())
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
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn delete_maintenance_record(
        pool: &PgPool,
        id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<()> {
        if !current_user.has_permission("equipment.maintenance.manage")
            && !current_user.has_permission("equipment.manage")
        {
            return Err(AppError::Forbidden("無權刪除維修保養紀錄".into()));
        }
        let result = sqlx::query("DELETE FROM equipment_maintenance_records WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("維修保養紀錄不存在".into()));
        }
        Ok(())
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

        // 預先產生所有隨機月份（避免持有 thread_rng 跨 await）
        let plans: Vec<_> = {
            let mut rng = rand::thread_rng();
            equipment_list
                .iter()
                .flat_map(|eq| {
                    let mut items = Vec::new();
                    if let (Some(cal_type), Some(cycle)) =
                        (&eq.calibration_type, &eq.calibration_cycle)
                    {
                        let months = pick_random_months(cycle, &mut rng);
                        items.push((eq.id, cal_type.clone(), cycle.clone(), months));
                    }
                    if let Some(cycle) = &eq.inspection_cycle {
                        let months = pick_random_months(cycle, &mut rng);
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
