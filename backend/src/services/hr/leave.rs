// HR 請假管理

use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::{ActorContext, CurrentUser},
    models::{
        audit_diff::DataDiff, CreateLeaveRequest, LeaveQuery, LeaveRequest, LeaveRequestWithUser,
        PaginatedResponse, UpdateLeaveRequest,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    Result,
};

use super::HrService;

impl HrService {
    // ============================================
    // Leave
    // ============================================

    pub async fn list_leaves(
        pool: &PgPool,
        query: &LeaveQuery,
        _current_user: &CurrentUser,
    ) -> Result<PaginatedResponse<LeaveRequestWithUser>> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(500);
        let offset = (page - 1) * per_page;

        // 如果是待審核查詢，篩選所有 PENDING 狀態的請假
        let is_pending_approval = query.pending_approval.unwrap_or(false);

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM leave_requests
            WHERE ($1::uuid IS NULL OR user_id = $1)
              AND ($2::text IS NULL OR status::text = $2)
              AND ($3::text IS NULL OR leave_type::text = $3)
              AND ($4::date IS NULL OR start_date >= $4)
              AND ($5::date IS NULL OR end_date <= $5)
              AND ($6::bool = false OR status::text LIKE 'PENDING%')
            "#,
        )
        .bind(query.user_id)
        .bind(&query.status)
        .bind(&query.leave_type)
        .bind(query.from)
        .bind(query.to)
        .bind(is_pending_approval)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, LeaveRequestWithUser>(
            r#"
            SELECT 
                l.id, l.user_id, u.email as user_email, u.display_name as user_name,
                l.proxy_user_id, proxy.display_name as proxy_user_name,
                l.leave_type::text as leave_type, l.start_date, l.end_date, l.total_days, l.total_hours, l.reason,
                l.is_urgent, l.is_retroactive, l.status::text as status,
                l.current_approver_id, approver.display_name as current_approver_name,
                l.submitted_at, l.created_at
            FROM leave_requests l
            INNER JOIN users u ON l.user_id = u.id
            LEFT JOIN users proxy ON l.proxy_user_id = proxy.id
            LEFT JOIN users approver ON l.current_approver_id = approver.id
            WHERE ($1::uuid IS NULL OR l.user_id = $1)
              AND ($2::text IS NULL OR l.status::text = $2)
              AND ($3::text IS NULL OR l.leave_type::text = $3)
              AND ($4::date IS NULL OR l.start_date >= $4)
              AND ($5::date IS NULL OR l.end_date <= $5)
              AND ($6::bool = false OR l.status::text LIKE 'PENDING%')
            ORDER BY l.created_at DESC
            LIMIT $7 OFFSET $8
            "#,
        )
        .bind(query.user_id)
        .bind(&query.status)
        .bind(&query.leave_type)
        .bind(query.from)
        .bind(query.to)
        .bind(is_pending_approval)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    pub async fn get_leave(
        pool: &PgPool,
        id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<LeaveRequest> {
        let record = sqlx::query_as::<_, LeaveRequest>(
            r#"
            SELECT 
                id, user_id, proxy_user_id, leave_type::text as leave_type, start_date, end_date,
                start_time, end_time, total_days, total_hours, reason, supporting_documents,
                annual_leave_source_id, is_urgent, is_retroactive,
                status::text as status, current_approver_id, submitted_at, approved_at,
                rejected_at, cancelled_at, revoked_at, cancellation_reason, revocation_reason,
                created_at, updated_at
            FROM leave_requests WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        let has_view_all = current_user.has_permission("hr.leave.view_all");
        let is_owner = record.user_id == current_user.id;
        let is_approver = record.current_approver_id == Some(current_user.id);
        if !has_view_all && !is_owner && !is_approver {
            return Err(AppError::Forbidden("無權存取此請假紀錄".into()));
        }

        Ok(record)
    }

    /// 檢查時數是否為 0.5 的倍數
    fn is_half_hour_multiple(v: f64) -> bool {
        v >= 0.5 && (v * 2.0 - (v * 2.0).round()).abs() < 1e-9
    }

    pub async fn create_leave(
        pool: &PgPool,
        actor: &ActorContext,
        payload: &CreateLeaveRequest,
    ) -> Result<LeaveRequest> {
        let user = actor.require_user()?;
        let user_id = user.id;

        let effective_hours = payload.total_hours.unwrap_or(payload.total_days * 8.0);
        if !Self::is_half_hour_multiple(effective_hours) {
            return Err(AppError::BadRequest(
                "請假時數須為 0.5 小時的倍數（如 0.5、1、1.5、2...）".into(),
            ));
        }
        let total_days = payload.total_hours.map(|h| h / 8.0).unwrap_or(payload.total_days);
        let total_hours = Some(payload.total_hours.unwrap_or(payload.total_days * 8.0));

        let id = Uuid::new_v4();

        // 處理 supporting_documents 轉為 JSON
        let supporting_docs = payload
            .supporting_documents
            .as_ref()
            .map(|docs| serde_json::json!(docs))
            .unwrap_or_else(|| serde_json::json!([]));

        // 理由處理：特休假可以為空，其他假別需要檢查
        let reason = payload.reason.clone().unwrap_or_default();

        let mut tx = pool.begin().await?;

        let record = sqlx::query_as::<_, LeaveRequest>(
            r#"
            INSERT INTO leave_requests (
                id, user_id, proxy_user_id, leave_type, start_date, end_date, start_time, end_time,
                total_days, total_hours, reason, supporting_documents, is_urgent, is_retroactive, status
            ) VALUES ($1, $2, $3, $4::leave_type, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, 'DRAFT'::leave_status)
            RETURNING
                id, user_id, proxy_user_id, leave_type::text as leave_type, start_date, end_date,
                start_time, end_time, total_days, total_hours, reason, supporting_documents,
                annual_leave_source_id, is_urgent, is_retroactive,
                status::text as status, current_approver_id, submitted_at, approved_at,
                rejected_at, cancelled_at, revoked_at, cancellation_reason, revocation_reason,
                created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(payload.proxy_user_id)
        .bind(&payload.leave_type)
        .bind(payload.start_date)
        .bind(payload.end_date)
        .bind(payload.start_time)
        .bind(payload.end_time)
        .bind(total_days)
        .bind(total_hours)
        .bind(&reason)
        .bind(&supporting_docs)
        .bind(payload.is_urgent.unwrap_or(false))
        .bind(payload.is_retroactive.unwrap_or(false))
        .fetch_one(&mut *tx)
        .await?;

        let display = format!(
            "{} {}~{}",
            record.leave_type, record.start_date, record.end_date
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "HR",
                event_type: "LEAVE_CREATE",
                entity: Some(AuditEntity::new("leave_request", record.id, &display)),
                data_diff: Some(DataDiff::create_only(&record)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(record)
    }

    pub async fn update_leave(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        payload: &UpdateLeaveRequest,
    ) -> Result<LeaveRequest> {
        let _user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, LeaveRequest>(
            r#"
            SELECT
                id, user_id, proxy_user_id, leave_type::text as leave_type, start_date, end_date,
                start_time, end_time, total_days, total_hours, reason, supporting_documents,
                annual_leave_source_id, is_urgent, is_retroactive,
                status::text as status, current_approver_id, submitted_at, approved_at,
                rejected_at, cancelled_at, revoked_at, cancellation_reason, revocation_reason,
                created_at, updated_at
            FROM leave_requests WHERE id = $1 FOR UPDATE
            "#,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("請假申請不存在".into()))?;

        if before.status != "DRAFT" {
            return Err(AppError::BusinessRule(
                "僅草稿狀態的請假可更新".into(),
            ));
        }

        let after = sqlx::query_as::<_, LeaveRequest>(
            r#"
            UPDATE leave_requests
            SET start_date = COALESCE($2, start_date),
                end_date = COALESCE($3, end_date),
                start_time = COALESCE($4, start_time),
                end_time = COALESCE($5, end_time),
                total_days = COALESCE($6, total_days),
                total_hours = COALESCE($7, total_hours),
                reason = COALESCE($8, reason),
                proxy_user_id = COALESCE($9, proxy_user_id),
                updated_at = NOW()
            WHERE id = $1 AND status = 'DRAFT'::leave_status
            RETURNING
                id, user_id, proxy_user_id, leave_type::text as leave_type, start_date, end_date,
                start_time, end_time, total_days, total_hours, reason, supporting_documents,
                annual_leave_source_id, is_urgent, is_retroactive,
                status::text as status, current_approver_id, submitted_at, approved_at,
                rejected_at, cancelled_at, revoked_at, cancellation_reason, revocation_reason,
                created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(payload.start_date)
        .bind(payload.end_date)
        .bind(payload.start_time)
        .bind(payload.end_time)
        .bind(payload.total_days)
        .bind(payload.total_hours)
        .bind(&payload.reason)
        .bind(payload.proxy_user_id)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("{} {}~{}", after.leave_type, after.start_date, after.end_date);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "HR",
                event_type: "LEAVE_UPDATE",
                entity: Some(AuditEntity::new("leave_request", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(after)
    }

    pub async fn delete_leave(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<()> {
        let _user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, LeaveRequest>(
            r#"
            SELECT
                id, user_id, proxy_user_id, leave_type::text as leave_type, start_date, end_date,
                start_time, end_time, total_days, total_hours, reason, supporting_documents,
                annual_leave_source_id, is_urgent, is_retroactive,
                status::text as status, current_approver_id, submitted_at, approved_at,
                rejected_at, cancelled_at, revoked_at, cancellation_reason, revocation_reason,
                created_at, updated_at
            FROM leave_requests WHERE id = $1 AND status = 'DRAFT'::leave_status FOR UPDATE
            "#,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("請假申請不存在或非草稿狀態".into()))?;

        sqlx::query("DELETE FROM leave_requests WHERE id = $1 AND status = 'DRAFT'::leave_status")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        let display = format!(
            "{} {}~{}",
            before.leave_type, before.start_date, before.end_date
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "HR",
                event_type: "LEAVE_DELETE",
                entity: Some(AuditEntity::new("leave_request", before.id, &display)),
                data_diff: Some(DataDiff::delete_only(&before)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn submit_leave(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<LeaveRequest> {
        let _user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, LeaveRequest>(
            r#"
            SELECT
                id, user_id, proxy_user_id, leave_type::text as leave_type, start_date, end_date,
                start_time, end_time, total_days, total_hours, reason, supporting_documents,
                annual_leave_source_id, is_urgent, is_retroactive,
                status::text as status, current_approver_id, submitted_at, approved_at,
                rejected_at, cancelled_at, revoked_at, cancellation_reason, revocation_reason,
                created_at, updated_at
            FROM leave_requests WHERE id = $1 FOR UPDATE
            "#,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("請假申請不存在".into()))?;

        if before.status != "DRAFT" {
            return Err(AppError::BusinessRule(
                "僅草稿狀態的請假可送審".into(),
            ));
        }

        let after = sqlx::query_as::<_, LeaveRequest>(
            r#"
            UPDATE leave_requests
            SET status = 'PENDING_L1'::leave_status, submitted_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND status = 'DRAFT'::leave_status
            RETURNING
                id, user_id, proxy_user_id, leave_type::text as leave_type, start_date, end_date,
                start_time, end_time, total_days, total_hours, reason, supporting_documents,
                annual_leave_source_id, is_urgent, is_retroactive,
                status::text as status, current_approver_id, submitted_at, approved_at,
                rejected_at, cancelled_at, revoked_at, cancellation_reason, revocation_reason,
                created_at, updated_at
            "#,
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!(
            "{} {}~{}",
            after.leave_type, after.start_date, after.end_date
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "HR",
                event_type: "LEAVE_SUBMIT",
                entity: Some(AuditEntity::new("leave_request", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(after)
    }

    pub async fn approve_leave(
        pool: &PgPool,
        id: Uuid,
        approver_id: Uuid,
        comments: Option<&str>,
    ) -> Result<LeaveRequest> {
        let current: LeaveRequest = sqlx::query_as(
            r#"
            SELECT
                id, user_id, proxy_user_id, leave_type::text as leave_type, start_date, end_date,
                start_time, end_time, total_days, total_hours, reason, supporting_documents,
                annual_leave_source_id, is_urgent, is_retroactive,
                status::text as status, current_approver_id, submitted_at, approved_at,
                rejected_at, cancelled_at, revoked_at, cancellation_reason, revocation_reason,
                created_at, updated_at
            FROM leave_requests WHERE id = $1
            "#,
        )
            .bind(id)
            .fetch_one(pool)
            .await?;

        // Determine next status based on current status
        // PENDING_L1 (部門主管) → PENDING_HR (行政) → PENDING_GM (負責人) → APPROVED
        use crate::models::LeaveStatus;
        let next_status = match current.status.as_str() {
            s if s == LeaveStatus::PendingL1.as_str() => LeaveStatus::PendingHr.as_str(),
            s if s == LeaveStatus::PendingL2.as_str() => LeaveStatus::PendingHr.as_str(),
            s if s == LeaveStatus::PendingHr.as_str() => LeaveStatus::PendingGm.as_str(),
            s if s == LeaveStatus::PendingGm.as_str() => LeaveStatus::Approved.as_str(),
            _ => return Err(AppError::Validation("無法核准此狀態的請假".to_string())),
        };

        // 最終核准時，檢查並扣除假別餘額
        let is_final_approval = next_status == LeaveStatus::Approved.as_str();
        if is_final_approval {
            Self::deduct_leave_balance(pool, &current).await?;
        }

        sqlx::query(
            r#"
            INSERT INTO leave_approvals (id, leave_request_id, approver_id, approval_level, action, comments)
            VALUES ($1, $2, $3, $4, 'APPROVE', $5)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(approver_id)
        .bind(&current.status)
        .bind(comments)
        .execute(pool)
        .await?;

        let approved_at = if is_final_approval {
            Some(Utc::now())
        } else {
            None
        };

        // SEC-BIZ-5: 使用 WHERE status 條件防止 race condition（TOCTOU）
        // 若另一個請求已先修改狀態，此 UPDATE 不會匹配任何行 → 回傳衝突錯誤
        let update_result = sqlx::query(
            r#"
            UPDATE leave_requests
            SET status = $2::leave_status, approved_at = $3, current_approver_id = NULL, updated_at = NOW()
            WHERE id = $1 AND status = $4::leave_status
            "#,
        )
        .bind(id)
        .bind(next_status)
        .bind(approved_at)
        .bind(&current.status)
        .execute(pool)
        .await?;

        if update_result.rows_affected() == 0 {
            return Err(AppError::Conflict(
                "此請假狀態已被其他操作變更，請重新整理後再試".to_string(),
            ));
        }

        let record = sqlx::query_as::<_, LeaveRequest>(
            r#"
            SELECT
                id, user_id, proxy_user_id, leave_type::text as leave_type, start_date, end_date,
                start_time, end_time, total_days, total_hours, reason, supporting_documents,
                annual_leave_source_id, is_urgent, is_retroactive,
                status::text as status, current_approver_id, submitted_at, approved_at,
                rejected_at, cancelled_at, revoked_at, cancellation_reason, revocation_reason,
                created_at, updated_at
            FROM leave_requests WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn reject_leave(
        pool: &PgPool,
        id: Uuid,
        rejecter_id: Uuid,
        reason: &str,
    ) -> Result<LeaveRequest> {
        let current: LeaveRequest = sqlx::query_as(
            r#"
            SELECT 
                id, user_id, proxy_user_id, leave_type::text as leave_type, start_date, end_date,
                start_time, end_time, total_days, total_hours, reason, supporting_documents,
                annual_leave_source_id, is_urgent, is_retroactive,
                status::text as status, current_approver_id, submitted_at, approved_at,
                rejected_at, cancelled_at, revoked_at, cancellation_reason, revocation_reason,
                created_at, updated_at
            FROM leave_requests WHERE id = $1
            "#,
        )
            .bind(id)
            .fetch_one(pool)
            .await?;

        sqlx::query(
            r#"
            INSERT INTO leave_approvals (id, leave_request_id, approver_id, approval_level, action, comments)
            VALUES ($1, $2, $3, $4, 'REJECT', $5)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(rejecter_id)
        .bind(&current.status)
        .bind(reason)
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            UPDATE leave_requests
            SET status = 'REJECTED'::leave_status, rejected_at = NOW(), updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        let record = sqlx::query_as::<_, LeaveRequest>(
            r#"
            SELECT 
                id, user_id, proxy_user_id, leave_type::text as leave_type, start_date, end_date,
                start_time, end_time, total_days, total_hours, reason, supporting_documents,
                annual_leave_source_id, is_urgent, is_retroactive,
                status::text as status, current_approver_id, submitted_at, approved_at,
                rejected_at, cancelled_at, revoked_at, cancellation_reason, revocation_reason,
                created_at, updated_at
            FROM leave_requests WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// 計算有效請假時數（total_hours 優先，否則換算天數 × 8）
    pub(super) fn effective_hours(total_hours: Option<f64>, total_days: f64) -> f64 {
        total_hours.unwrap_or(total_days * 8.0)
    }

    pub async fn cancel_leave(
        pool: &PgPool,
        id: Uuid,
        _current_user: &CurrentUser,
        reason: Option<&str>,
    ) -> Result<LeaveRequest> {
        // 先查詢目前狀態，判斷是否需要回復餘額
        let current: LeaveRequest = sqlx::query_as(
            r#"
            SELECT
                id, user_id, proxy_user_id, leave_type::text as leave_type, start_date, end_date,
                start_time, end_time, total_days, total_hours, reason, supporting_documents,
                annual_leave_source_id, is_urgent, is_retroactive,
                status::text as status, current_approver_id, submitted_at, approved_at,
                rejected_at, cancelled_at, revoked_at, cancellation_reason, revocation_reason,
                created_at, updated_at
            FROM leave_requests WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        use crate::models::LeaveStatus;
        let was_approved = current.status == LeaveStatus::Approved.as_str();

        sqlx::query(
            r#"
            UPDATE leave_requests
            SET status = 'CANCELLED'::leave_status, cancelled_at = NOW(), cancellation_reason = $2, updated_at = NOW()
            WHERE id = $1 AND status IN ('DRAFT'::leave_status, 'PENDING_L1'::leave_status, 'PENDING_L2'::leave_status, 'PENDING_HR'::leave_status, 'PENDING_GM'::leave_status, 'APPROVED'::leave_status)
            "#,
        )
        .bind(id)
        .bind(reason)
        .execute(pool)
        .await?;

        // 已核准的請假取消時，回復餘額
        if was_approved {
            Self::restore_leave_balance(pool, &current).await?;
        }

        let record = sqlx::query_as::<_, LeaveRequest>(
            r#"
            SELECT
                id, user_id, proxy_user_id, leave_type::text as leave_type, start_date, end_date,
                start_time, end_time, total_days, total_hours, reason, supporting_documents,
                annual_leave_source_id, is_urgent, is_retroactive,
                status::text as status, current_approver_id, submitted_at, approved_at,
                rejected_at, cancelled_at, revoked_at, cancellation_reason, revocation_reason,
                created_at, updated_at
            FROM leave_requests WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    // ============================================
    // Balance deduction / restoration helpers
    // ============================================

    /// 扣除特休假餘額（FIFO：先到期先扣）並記錄 leave_balance_usage
    async fn deduct_annual_leave(
        pool: &PgPool,
        leave: &LeaveRequest,
    ) -> Result<()> {
        let mut remaining = leave.total_days;
        let entitlements: Vec<(Uuid, Decimal)> = sqlx::query_as(
            r#"
            SELECT id, (entitled_days - used_days) as available
            FROM annual_leave_entitlements
            WHERE user_id = $1 AND NOT is_expired AND (entitled_days - used_days) > 0
            ORDER BY expires_at ASC
            "#,
        )
        .bind(leave.user_id)
        .fetch_all(pool)
        .await?;

        let total_available: Decimal = entitlements.iter().map(|e| e.1).sum();
        if total_available < remaining {
            return Err(AppError::BusinessRule(format!(
                "特休假餘額不足：需要 {} 天，剩餘 {} 天",
                remaining, total_available
            )));
        }

        for (ent_id, available) in entitlements {
            if remaining <= Decimal::ZERO {
                break;
            }
            let deduct = remaining.min(available);
            Self::apply_annual_deduction(pool, leave.id, ent_id, deduct).await?;
            remaining -= deduct;
        }
        Ok(())
    }

    /// 執行單筆特休假扣除（UPDATE + INSERT usage）
    async fn apply_annual_deduction(
        pool: &PgPool,
        leave_id: Uuid,
        entitlement_id: Uuid,
        days: Decimal,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE annual_leave_entitlements SET used_days = used_days + $2, updated_at = NOW() WHERE id = $1",
        )
        .bind(entitlement_id)
        .bind(days)
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO leave_balance_usage
                (id, leave_request_id, source_type, annual_leave_entitlement_id, days_used, action)
            VALUES ($1, $2, 'annual', $3, $4, 'deduct')
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(leave_id)
        .bind(entitlement_id)
        .bind(days)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 扣除補休餘額（FIFO：先到期先扣）並記錄 leave_balance_usage
    async fn deduct_comp_time(
        pool: &PgPool,
        leave: &LeaveRequest,
    ) -> Result<()> {
        let hours = Self::effective_hours(
            leave.total_hours.map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            leave.total_days.to_string().parse::<f64>().unwrap_or(0.0),
        );
        let hours_dec = Decimal::from_f64_retain(hours)
            .ok_or_else(|| AppError::Internal("無法轉換補休時數".into()))?;

        let balances: Vec<(Uuid, Decimal)> = sqlx::query_as(
            r#"
            SELECT id, (original_hours - used_hours) as available
            FROM comp_time_balances
            WHERE user_id = $1 AND NOT is_expired AND (original_hours - used_hours) > 0
            ORDER BY expires_at ASC
            "#,
        )
        .bind(leave.user_id)
        .fetch_all(pool)
        .await?;

        let total_available: Decimal = balances.iter().map(|b| b.1).sum();
        if total_available < hours_dec {
            return Err(AppError::BusinessRule(format!(
                "補休餘額不足：需要 {} 小時，剩餘 {} 小時",
                hours_dec, total_available
            )));
        }

        let mut remaining = hours_dec;
        for (bal_id, available) in balances {
            if remaining <= Decimal::ZERO {
                break;
            }
            let deduct = remaining.min(available);
            Self::apply_comp_deduction(pool, leave.id, bal_id, deduct).await?;
            remaining -= deduct;
        }
        Ok(())
    }

    /// 執行單筆補休扣除（UPDATE + INSERT usage）
    async fn apply_comp_deduction(
        pool: &PgPool,
        leave_id: Uuid,
        balance_id: Uuid,
        hours: Decimal,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE comp_time_balances SET used_hours = used_hours + $2, updated_at = NOW() WHERE id = $1",
        )
        .bind(balance_id)
        .bind(hours)
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO leave_balance_usage
                (id, leave_request_id, source_type, comp_time_balance_id, hours_used, action)
            VALUES ($1, $2, 'comp_time', $3, $4, 'deduct')
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(leave_id)
        .bind(balance_id)
        .bind(hours)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 核准時依假別檢查並扣除餘額（僅 ANNUAL / COMPENSATORY 需要）
    async fn deduct_leave_balance(pool: &PgPool, leave: &LeaveRequest) -> Result<()> {
        match leave.leave_type.as_str() {
            "ANNUAL" => Self::deduct_annual_leave(pool, leave).await,
            "COMPENSATORY" => Self::deduct_comp_time(pool, leave).await,
            _ => Ok(()), // 其他假別無額度限制
        }
    }

    /// 取消/銷假時依假別回復餘額（僅 ANNUAL / COMPENSATORY 需要）
    async fn restore_leave_balance(pool: &PgPool, leave: &LeaveRequest) -> Result<()> {
        match leave.leave_type.as_str() {
            "ANNUAL" => Self::restore_annual_leave(pool, leave).await,
            "COMPENSATORY" => Self::restore_comp_time(pool, leave).await,
            _ => Ok(()),
        }
    }

    /// 回復特休假餘額：依 leave_balance_usage 的 deduct 紀錄逐筆還原
    async fn restore_annual_leave(pool: &PgPool, leave: &LeaveRequest) -> Result<()> {
        let usages: Vec<(Uuid, Uuid, Decimal)> = sqlx::query_as(
            r#"
            SELECT id, annual_leave_entitlement_id, days_used
            FROM leave_balance_usage
            WHERE leave_request_id = $1 AND source_type = 'annual' AND action = 'deduct'
            "#,
        )
        .bind(leave.id)
        .fetch_all(pool)
        .await?;

        for (usage_id, ent_id, days) in usages {
            sqlx::query(
                "UPDATE annual_leave_entitlements SET used_days = used_days - $2, updated_at = NOW() WHERE id = $1",
            )
            .bind(ent_id)
            .bind(days)
            .execute(pool)
            .await?;

            sqlx::query(
                r#"
                INSERT INTO leave_balance_usage
                    (id, leave_request_id, source_type, annual_leave_entitlement_id, days_used, action)
                VALUES ($1, $2, 'annual', $3, $4, 'restore')
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(leave.id)
            .bind(ent_id)
            .bind(days)
            .execute(pool)
            .await?;

            let _ = usage_id; // 僅用於未來稽核需求
        }
        Ok(())
    }

    /// 回復補休餘額：依 leave_balance_usage 的 deduct 紀錄逐筆還原
    async fn restore_comp_time(pool: &PgPool, leave: &LeaveRequest) -> Result<()> {
        let usages: Vec<(Uuid, Uuid, Decimal)> = sqlx::query_as(
            r#"
            SELECT id, comp_time_balance_id, hours_used
            FROM leave_balance_usage
            WHERE leave_request_id = $1 AND source_type = 'comp_time' AND action = 'deduct'
            "#,
        )
        .bind(leave.id)
        .fetch_all(pool)
        .await?;

        for (usage_id, bal_id, hours) in usages {
            sqlx::query(
                "UPDATE comp_time_balances SET used_hours = used_hours - $2, updated_at = NOW() WHERE id = $1",
            )
            .bind(bal_id)
            .bind(hours)
            .execute(pool)
            .await?;

            sqlx::query(
                r#"
                INSERT INTO leave_balance_usage
                    (id, leave_request_id, source_type, comp_time_balance_id, hours_used, action)
                VALUES ($1, $2, 'comp_time', $3, $4, 'restore')
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(leave.id)
            .bind(bal_id)
            .bind(hours)
            .execute(pool)
            .await?;

            let _ = usage_id;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::HrService;

    // --- is_half_hour_multiple ---

    #[test]
    fn test_is_half_hour_multiple_valid() {
        assert!(HrService::is_half_hour_multiple(0.5));
        assert!(HrService::is_half_hour_multiple(1.0));
        assert!(HrService::is_half_hour_multiple(1.5));
        assert!(HrService::is_half_hour_multiple(8.0));
        assert!(HrService::is_half_hour_multiple(0.5));
    }

    #[test]
    fn test_is_half_hour_multiple_invalid() {
        assert!(!HrService::is_half_hour_multiple(0.0));  // 小於 0.5
        assert!(!HrService::is_half_hour_multiple(0.3));
        assert!(!HrService::is_half_hour_multiple(1.1));
        assert!(!HrService::is_half_hour_multiple(2.3));
    }

    #[test]
    fn test_is_half_hour_multiple_boundary() {
        assert!(!HrService::is_half_hour_multiple(0.4));
        assert!(HrService::is_half_hour_multiple(0.5));
        assert!(!HrService::is_half_hour_multiple(0.6));
    }

    // --- effective_hours ---

    #[test]
    fn test_effective_hours_uses_total_hours_when_provided() {
        assert_eq!(HrService::effective_hours(Some(4.0), 1.0), 4.0);
        assert_eq!(HrService::effective_hours(Some(0.5), 3.0), 0.5);
    }

    #[test]
    fn test_effective_hours_converts_days_when_no_hours() {
        assert_eq!(HrService::effective_hours(None, 1.0), 8.0);
        assert_eq!(HrService::effective_hours(None, 0.5), 4.0);
        assert_eq!(HrService::effective_hours(None, 2.0), 16.0);
    }
}
