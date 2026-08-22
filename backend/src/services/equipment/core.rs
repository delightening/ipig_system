// 設備本體 CRUD、供應商、狀態日誌、履歷時間軸

use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    middleware::CurrentUser,
    models::{
        CreateEquipmentRequest, CreateEquipmentSupplierRequest, Equipment, EquipmentHistoryQuery,
        EquipmentQuery, EquipmentStatusLog, EquipmentSupplierWithPartner, EquipmentTimelineEntry,
        PaginatedResponse, TimelineRow, UpdateEquipmentRequest,
    },
    repositories, Result,
};

use super::{check_manage_permission, check_view_permission, EquipmentService};

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
        let rows =
            repositories::equipment::find_equipment_timeline(pool, equipment_id, per_page, offset)
                .await?;

        let data = rows.into_iter().map(build_timeline_entry).collect();
        Ok(PaginatedResponse::new(data, total, page, per_page))
    }
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
