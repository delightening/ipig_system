// HR 加班管理

use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Timelike, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::{ActorContext, CurrentUser},
    models::{
        audit_diff::DataDiff, CompTimeBalance, CreateOvertimeRequest, OvertimeQuery,
        OvertimeRecord, OvertimeWithUser, PaginatedResponse, UpdateOvertimeRequest,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    Result,
};

use super::HrService;

/// 依加班類型回傳補休乘數（非加班費，本系統不計算加班費）。
/// A=平日(1.0), B=休息日(1.33), C=國定假日(1.66), D=天災(2.0)
pub(super) fn overtime_multiplier(overtime_type: &str) -> f64 {
    match overtime_type {
        "A" => 1.0,
        "B" => 1.33,
        "C" => 1.66,
        "D" => 2.0,
        _ => 1.0,
    }
}

// ============================================================
// 勞基法合規常數
// ============================================================

/// 勞基法 §30：每日標準工時上限 8 小時
pub const DAILY_REGULAR_HOURS: f64 = 8.0;

/// 勞基法 §30：每週標準工時上限 40 小時
pub const WEEKLY_REGULAR_HOURS: f64 = 40.0;

/// 勞基法 §32：每月加班時數上限 46 小時
pub const MONTHLY_OVERTIME_LIMIT: f64 = 46.0;

/// 勞基法 §32：特殊情況每月加班上限 54 小時（需勞資協議）。
/// 另規定每三個月上限 138 小時（46×3），若未來實作季度檢查再補常數。
pub const MONTHLY_OVERTIME_LIMIT_EXTENDED: f64 = 54.0;

/// 平日加班分段結果
#[derive(Debug, Clone, serde::Serialize)]
pub struct WeekdayOvertimeTiers {
    /// 前 2 小時
    pub tier1_hours: f64,
    /// 超過 2 小時
    pub tier2_hours: f64,
    /// 總時數
    pub total_hours: f64,
}

/// 勞基法 §24：平日加班分段計算。
/// - 前 2 小時為第一段
/// - 超過 2 小時為第二段
pub fn split_weekday_overtime(hours: f64) -> WeekdayOvertimeTiers {
    let total = (hours * 2.0).floor() / 2.0; // 四捨五入至 0.5
    let tier1 = total.min(2.0);
    let tier2 = (total - 2.0).max(0.0);
    WeekdayOvertimeTiers {
        tier1_hours: tier1,
        tier2_hours: tier2,
        total_hours: total,
    }
}

/// 加班類型代碼：A=平日加班（走加班費分段計算）。
pub const OVERTIME_TYPE_WEEKDAY: &str = "A";

/// 勞基法 §24：平日加班前 2 小時的加給係數。
pub const WEEKDAY_OT_TIER1_MULTIPLIER: f64 = 1.33;

/// 勞基法 §24：平日加班超過 2 小時的加給係數。
pub const WEEKDAY_OT_TIER2_MULTIPLIER: f64 = 1.66;

/// 將「下班後加班分鐘數」四捨五入至最近的 30 分鐘，回傳小時數。
///
/// 規則：≥15 分進位、<15 分捨去（例：25→30 分=0.5h、35→30 分=0.5h、
/// 55→60 分=1.0h、65→60 分=1.0h）。負值（早退）一律回 0。
pub fn round_overtime_minutes_to_half_hour(minutes: i64) -> f64 {
    if minutes <= 0 {
        return 0.0;
    }
    // round() 為「half away from zero」，故 15 分（0.5 單位）進位。
    let half_hour_units = (minutes as f64 / 30.0).round();
    half_hour_units * 0.5
}

/// 計算平日加班分段時數。`start_minutes` / `end_minutes` 為自午夜起算的分鐘數
/// （手動加班單填起訖時間；自動產生時 start = 班別下班、end = 打卡下班）。
/// 先取得加班分鐘（end − start），四捨五入至 30 分後再依 §24 分段。
pub fn weekday_overtime_tiers(start_minutes: i64, end_minutes: i64) -> WeekdayOvertimeTiers {
    let hours = round_overtime_minutes_to_half_hour(end_minutes - start_minutes);
    split_weekday_overtime(hours)
}

/// 平日加班加權係數時數 = tier1×1.33 + tier2×1.66。
/// 供薪資模組換算加班費（加班費 = 加權係數時數 × 時薪）。
pub fn weekday_overtime_weighted_hours(tiers: &WeekdayOvertimeTiers) -> f64 {
    tiers.tier1_hours * WEEKDAY_OT_TIER1_MULTIPLIER
        + tiers.tier2_hours * WEEKDAY_OT_TIER2_MULTIPLIER
}

/// 加班上限驗證結果
#[derive(Debug, Clone, serde::Serialize)]
pub struct OvertimeLimitCheck {
    /// 本月已累計加班時數
    pub monthly_total: f64,
    /// 本次申請時數
    pub requested_hours: f64,
    /// 合併後總時數
    pub projected_total: f64,
    /// 是否超過標準上限 (46hr)
    pub exceeds_standard_limit: bool,
    /// 是否超過特殊上限 (54hr)
    pub exceeds_extended_limit: bool,
    /// 警告訊息（空表示通過）
    pub warnings: Vec<String>,
}

/// 工時驗證結果
#[derive(Debug, Clone, serde::Serialize)]
pub struct WorkHoursValidation {
    /// 當日實際工時
    pub daily_hours: f64,
    /// 本週累計工時
    pub weekly_hours: f64,
    /// 是否超過每日標準 (8hr)
    pub exceeds_daily_limit: bool,
    /// 是否超過每週標準 (40hr)
    pub exceeds_weekly_limit: bool,
    /// 建議加班時數（超出 8 小時部分）
    pub suggested_overtime_hours: f64,
    /// 警告訊息
    pub warnings: Vec<String>,
}

/// 加班單狀態的中文標籤（供錯誤訊息使用；前端另有一份 OVERTIME_STATUS_NAMES）。
pub(super) fn overtime_status_label(status: &str) -> &str {
    match status {
        "draft" => "草稿",
        "pending" | "pending_admin_staff" => "待行政審核",
        "pending_admin" => "待負責人審核",
        "approved" => "已核准",
        "rejected" => "已駁回",
        "voided" => "已作廢",
        other => other,
    }
}

/// 依加班類型回傳補休時數。
/// C/D 類型固定給 8 小時，其餘為 0。
pub(super) fn comp_time_hours_for_type(overtime_type: &str) -> f64 {
    match overtime_type {
        "C" | "D" => 8.0,
        _ => 0.0,
    }
}

/// 將開始與結束分鐘數換算為以 0.5 小時為單位的工作時數。
pub(super) fn calc_hours_from_minutes(start_minutes: i64, end_minutes: i64) -> f64 {
    let raw = (end_minutes - start_minutes) as f64 / 60.0;
    (raw * 2.0).floor() / 2.0
}

impl HrService {
    // ============================================
    // Overtime
    // ============================================

    pub async fn list_overtime(
        pool: &PgPool,
        query: &OvertimeQuery,
        current_user: &crate::middleware::CurrentUser,
    ) -> Result<PaginatedResponse<OvertimeWithUser>> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(500);
        let offset = (page - 1) * per_page;

        // Handle comma-separated status values (e.g., "pending_admin_staff,pending_admin")
        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM overtime_records
            WHERE ($1::uuid IS NULL OR user_id = $1)
              AND ($2::text IS NULL OR status = ANY(string_to_array($2, ',')))
              AND ($3::date IS NULL OR overtime_date >= $3)
              AND ($4::date IS NULL OR overtime_date <= $4)
            "#,
        )
        .bind(query.user_id)
        .bind(&query.status)
        .bind(query.from)
        .bind(query.to)
        .fetch_one(pool)
        .await?;

        let mut data = sqlx::query_as::<_, OvertimeWithUser>(
            r#"
            SELECT
                o.id, o.user_id, u.email as user_email, u.display_name as user_name,
                o.overtime_date, o.start_time, o.end_time, o.hours,
                o.overtime_type, o.multiplier, o.comp_time_hours, o.comp_time_expires_at,
                o.status, o.reason, o.void_reason
            FROM overtime_records o
            INNER JOIN users u ON o.user_id = u.id
            WHERE ($1::uuid IS NULL OR o.user_id = $1)
              AND ($2::text IS NULL OR o.status = ANY(string_to_array($2, ',')))
              AND ($3::date IS NULL OR o.overtime_date >= $3)
              AND ($4::date IS NULL OR o.overtime_date <= $4)
            ORDER BY o.overtime_date DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(query.user_id)
        .bind(&query.status)
        .bind(query.from)
        .bind(query.to)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        // R72-2：逐列計算「當前使用者是否可核准」，與 approve_overtime handler 授權邏輯一致
        // （加班無部門主管階段）。
        let is_admin = current_user.is_admin();
        let is_admin_staff = current_user
            .roles
            .contains(&crate::constants::ROLE_ADMIN_STAFF.to_string());
        for row in &mut data {
            row.can_approve = if row.user_id == current_user.id {
                // 不可審核自己的申請。此規則由 approve_overtime 服務端強制執行，
                // 這裡只是讓 UI 不要顯示注定失敗的按鈕。
                // （2026-07 前這裡是唯一的防線、動作端沒擋，直接打 API 即可自審。）
                false
            } else {
                match row.status.as_str() {
                    "pending_admin_staff" => is_admin || is_admin_staff,
                    "pending_admin" => is_admin,
                    _ => false,
                }
            };
        }

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    pub async fn get_overtime(
        pool: &PgPool,
        id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<OvertimeWithUser> {
        let record = sqlx::query_as::<_, OvertimeWithUser>(
            r#"
            SELECT 
                o.id, o.user_id, u.email as user_email, u.display_name as user_name,
                o.overtime_date, o.start_time, o.end_time, o.hours,
                o.overtime_type, o.multiplier, o.comp_time_hours, o.comp_time_expires_at,
                o.status, o.reason, o.void_reason
            FROM overtime_records o
            INNER JOIN users u ON o.user_id = u.id
            WHERE o.id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        let has_view_all = current_user.has_permission("hr.overtime.view_all");
        let is_owner = record.user_id == current_user.id;
        if !has_view_all && !is_owner {
            return Err(AppError::Forbidden("無權存取此加班紀錄".into()));
        }

        Ok(record)
    }

    pub async fn create_overtime(
        pool: &PgPool,
        actor: &ActorContext,
        payload: &CreateOvertimeRequest,
    ) -> Result<OvertimeWithUser> {
        let user = actor.require_user()?;
        let user_id = user.id;

        // 將 NaiveTime 結合 overtime_date 轉換為 DateTime<Utc>
        let start_datetime =
            Utc.from_utc_datetime(&payload.overtime_date.and_time(payload.start_time));
        let end_datetime = Utc.from_utc_datetime(&payload.overtime_date.and_time(payload.end_time));

        // 計算時數。平日加班 (A) 採新規則：加班分鐘四捨五入至 30 分、依 §24 分段
        // （前 2h ×1.33、超過 ×1.66），並算出加權係數時數供薪資模組換算加班費。
        // 其餘類型 (B/C/D) 維持原以 0.5 小時捨入的時數，不分段。
        let start_minutes =
            payload.start_time.hour() as i64 * 60 + payload.start_time.minute() as i64;
        let end_minutes = payload.end_time.hour() as i64 * 60 + payload.end_time.minute() as i64;

        let is_weekday_ot = payload.overtime_type == OVERTIME_TYPE_WEEKDAY;
        let tiers = if is_weekday_ot {
            weekday_overtime_tiers(start_minutes, end_minutes)
        } else {
            WeekdayOvertimeTiers {
                tier1_hours: 0.0,
                tier2_hours: 0.0,
                total_hours: 0.0,
            }
        };
        let hours = if is_weekday_ot {
            tiers.total_hours
        } else {
            calc_hours_from_minutes(start_minutes, end_minutes)
        };
        let weighted_hours = if is_weekday_ot {
            weekday_overtime_weighted_hours(&tiers)
        } else {
            0.0
        };
        // 計費單位：平日(A) 按時數分段；值班(B/C/D) 按天（day_count 於 R77-2 接入）。
        let calc_unit = if is_weekday_ot { "hour" } else { "day" };

        let multiplier = overtime_multiplier(&payload.overtime_type);
        let comp_time_hours = comp_time_hours_for_type(&payload.overtime_type);
        let expires_at = payload.overtime_date + chrono::Duration::days(365);

        let id = Uuid::new_v4();
        let mut tx = pool.begin().await?;

        // R86-2：補登腳本中斷後重跑會把同一時段建成兩筆，兩筆各自核准就各授一份補休。
        // 這裡先查一次是為了給得出「撞到哪一筆」的訊息；真正的防線是 migration 142 的
        // partial unique index（並發重送時 check-then-insert 擋不住）。
        Self::ensure_no_duplicate_overtime(
            &mut tx,
            user_id,
            payload.overtime_date,
            start_datetime,
            end_datetime,
        )
        .await?;

        let record = sqlx::query_as::<_, OvertimeRecord>(
            r#"
            INSERT INTO overtime_records (
                id, user_id, overtime_date, start_time, end_time, hours,
                overtime_type, multiplier, comp_time_hours, comp_time_expires_at,
                calc_unit, tier1_hours, tier2_hours, weighted_hours,
                status, reason
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, 'draft', $15)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(payload.overtime_date)
        .bind(start_datetime)
        .bind(end_datetime)
        .bind(hours)
        .bind(&payload.overtime_type)
        .bind(multiplier)
        .bind(comp_time_hours)
        .bind(expires_at)
        .bind(calc_unit)
        .bind(tiers.tier1_hours)
        .bind(tiers.tier2_hours)
        .bind(weighted_hours)
        .bind(&payload.reason)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!(
            "{} {} ({}h)",
            record.overtime_date, record.overtime_type, record.hours
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "HR",
                event_type: "OVERTIME_CREATE",
                entity: Some(AuditEntity::new("overtime_record", record.id, &display)),
                data_diff: Some(DataDiff::create_only(&record)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Self::get_overtime_inner(pool, id).await
    }

    /// 同一人、同一天、同一起訖時間是否已有有效加班單（駁回／作廢的不算）。
    ///
    /// 對應 migration 142 的 `idx_overtime_records_no_duplicate`——條件必須與該索引一致，
    /// 否則會出現「應用層放行、DB 擋下」的 500。
    async fn ensure_no_duplicate_overtime(
        conn: &mut sqlx::PgConnection,
        user_id: Uuid,
        overtime_date: NaiveDate,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<()> {
        let existing: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT status FROM overtime_records
            WHERE user_id = $1
              AND overtime_date = $2
              AND start_time = $3
              AND end_time = $4
              AND status NOT IN ('rejected', 'voided')
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(overtime_date)
        .bind(start_time)
        .bind(end_time)
        .fetch_optional(conn)
        .await?;

        match existing {
            Some((status,)) => Err(AppError::Conflict(format!(
                "{} 這個時段已有一筆加班紀錄（{}），請勿重複建立",
                overtime_date,
                overtime_status_label(&status)
            ))),
            None => Ok(()),
        }
    }

    pub async fn update_overtime(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        payload: &UpdateOvertimeRequest,
    ) -> Result<OvertimeWithUser> {
        let user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, OvertimeRecord>(
            "SELECT * FROM overtime_records WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("加班紀錄不存在".into()))?;

        if before.user_id != user.id && !user.is_admin() {
            return Err(AppError::Forbidden("無權修改他人的加班紀錄".into()));
        }

        if before.status != "draft" {
            return Err(AppError::BusinessRule("僅草稿狀態的加班可更新".into()));
        }

        let after = sqlx::query_as::<_, OvertimeRecord>(
            r#"
            UPDATE overtime_records
            SET start_time = COALESCE($2, start_time),
                end_time = COALESCE($3, end_time),
                overtime_type = COALESCE($4, overtime_type),
                reason = COALESCE($5, reason),
                updated_at = NOW()
            WHERE id = $1 AND status = 'draft'
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(payload.start_time)
        .bind(payload.end_time)
        .bind(&payload.overtime_type)
        .bind(&payload.reason)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!(
            "{} {} ({}h)",
            after.overtime_date, after.overtime_type, after.hours
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "HR",
                event_type: "OVERTIME_UPDATE",
                entity: Some(AuditEntity::new("overtime_record", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Self::get_overtime_inner(pool, id).await
    }

    pub async fn delete_overtime(pool: &PgPool, actor: &ActorContext, id: Uuid) -> Result<()> {
        let user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, OvertimeRecord>(
            "SELECT * FROM overtime_records WHERE id = $1 AND status = 'draft' FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("加班紀錄不存在或非草稿狀態".into()))?;

        if before.user_id != user.id && !user.is_admin() {
            return Err(AppError::Forbidden("無權刪除他人的加班紀錄".into()));
        }

        sqlx::query("DELETE FROM overtime_records WHERE id = $1 AND status = 'draft'")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        let display = format!(
            "{} {} ({}h)",
            before.overtime_date, before.overtime_type, before.hours
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "HR",
                event_type: "OVERTIME_DELETE",
                entity: Some(AuditEntity::new("overtime_record", before.id, &display)),
                data_diff: Some(DataDiff::delete_only(&before)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn submit_overtime(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<OvertimeWithUser> {
        let user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, OvertimeRecord>(
            "SELECT * FROM overtime_records WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("加班紀錄不存在".into()))?;

        if before.user_id != user.id && !user.is_admin() {
            return Err(AppError::Forbidden("無權送審他人的加班紀錄".into()));
        }

        if before.status != "draft" {
            return Err(AppError::BusinessRule("僅草稿狀態的加班可送審".into()));
        }

        let after = sqlx::query_as::<_, OvertimeRecord>(
            r#"
            UPDATE overtime_records
            SET status = 'pending_admin_staff', submitted_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND status = 'draft'
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!(
            "{} {} ({}h)",
            after.overtime_date, after.overtime_type, after.hours
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "HR",
                event_type: "OVERTIME_SUBMIT",
                entity: Some(AuditEntity::new("overtime_record", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Self::get_overtime_inner(pool, id).await
    }

    /// 此人是否已在本單「核准」過任一關卡（SoD 判定基礎）。
    ///
    /// 對照 `leave.rs::has_prior_approval`。加班只有兩關且無代理確認關卡，
    /// 故不需排除 PENDING_PROXY 之類的中性動作。
    async fn has_prior_overtime_approval(
        conn: &mut sqlx::PgConnection,
        overtime_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool> {
        let exists: (bool,) = sqlx::query_as(
            r#"SELECT EXISTS(
                SELECT 1 FROM overtime_approvals
                WHERE overtime_record_id = $1 AND approver_id = $2 AND action = 'APPROVE'
            )"#,
        )
        .bind(overtime_id)
        .bind(user_id)
        .fetch_one(conn)
        .await?;
        Ok(exists.0)
    }

    /// 終審關是否還有「其他」合格核准者：在職、具 admin 權限、非申請人本人、
    /// 且尚未核准過本單任何關卡。
    ///
    /// 用途是判斷「SoD 能不能收緊」——有其他人可簽才擋；沒有就放行代批，
    /// 免得單一審批人組織把加班單卡死（對照 `leave.rs::director_has_eligible_approver`）。
    async fn final_stage_has_other_approver(
        conn: &mut sqlx::PgConnection,
        overtime_id: Uuid,
        applicant_id: Uuid,
        current_approver_id: Uuid,
    ) -> Result<bool> {
        let exists: (bool,) = sqlx::query_as(
            r#"SELECT EXISTS(
                SELECT 1 FROM users u
                JOIN user_roles ur ON ur.user_id = u.id
                JOIN roles r ON r.id = ur.role_id
                WHERE r.code IN ($1, $2)
                  AND u.is_active = true AND u.deleted_at IS NULL
                  AND u.id <> $3
                  AND u.id <> $4
                  AND NOT EXISTS (
                    SELECT 1 FROM overtime_approvals oa
                    WHERE oa.overtime_record_id = $5 AND oa.approver_id = u.id
                      AND oa.action = 'APPROVE'
                  )
            )"#,
        )
        .bind(crate::constants::ROLE_SYSTEM_ADMIN)
        .bind(crate::constants::ROLE_ADMIN_LEGACY)
        .bind(applicant_id)
        .bind(current_approver_id)
        .bind(overtime_id)
        .fetch_one(conn)
        .await?;
        Ok(exists.0)
    }

    pub async fn approve_overtime(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        approval_level: &str, // "admin_staff" or "admin"
    ) -> Result<OvertimeWithUser> {
        let user = actor.require_user()?;
        let approver_id = user.id;
        let mut tx = pool.begin().await?;

        // SELECT FOR UPDATE：行鎖 + before 快照
        let before: OvertimeRecord =
            sqlx::query_as("SELECT * FROM overtime_records WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("加班紀錄不存在".into()))?;

        // Determine next status based on current status and approval level
        let (expected_status, next_status, is_final) = match approval_level {
            "admin_staff" => ("pending_admin_staff", "pending_admin", false),
            "admin" => ("pending_admin", "approved", true),
            _ => return Err(AppError::Validation("無效的審核層級".to_string())),
        };

        // Verify current status matches expected
        if before.status != expected_status {
            return Err(AppError::Validation(format!(
                "目前狀態為 {}，無法進行 {} 層級審核",
                before.status, approval_level
            )));
        }

        // 不可審核自己的加班申請（任何關卡、任何角色皆不放寬，與請假一致：
        // services/hr/leave.rs 的 can_approve）。
        // 先前僅在列表的 can_approve 旗標上過濾（本檔 list 迴圈），動作端沒擋——
        // 直接呼叫本 API 即可核准自己的加班。
        if before.user_id == approver_id {
            return Err(AppError::Forbidden("不可審核自己的加班申請".to_string()));
        }

        // 職責分離（SoD）：核准過前一關者不得再核准終審關。
        // 僅在「確實沒有其他合格終審者」時放寬代批，否則單一審批人組織會卡死
        // ——同 leave.rs 終審關的處理。放寬時仍保證是真人簽核且非申請人本人。
        if is_final
            && Self::has_prior_overtime_approval(&mut tx, id, approver_id).await?
            && Self::final_stage_has_other_approver(&mut tx, id, before.user_id, approver_id)
                .await?
        {
            return Err(AppError::BusinessRule(
                "職責分離：您已核准本單前一關卡，終審請由其他負責人進行".to_string(),
            ));
        }

        // Update status
        let after: OvertimeRecord = sqlx::query_as(
            r#"
            UPDATE overtime_records
            SET status = $2, approved_by = $3, approved_at = NOW(), updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(next_status)
        .bind(approver_id)
        .fetch_one(&mut *tx)
        .await?;

        // Record approval in overtime_approvals table
        sqlx::query(
            r#"
            INSERT INTO overtime_approvals (id, overtime_record_id, approver_id, approval_level, action)
            VALUES ($1, $2, $3, $4, 'APPROVE')
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(approver_id)
        .bind(approval_level)
        .execute(&mut *tx)
        .await?;

        // Only credit comp_time after final approval (admin level)
        // 授予的補休餘額必須與 overtime 狀態變更在同一 tx 內（GLP 合規：
        // 補休餘額來源可追溯到特定加班紀錄與審核動作）
        if is_final {
            sqlx::query(
                r#"
                INSERT INTO comp_time_balances (
                    id, user_id, overtime_record_id, original_hours, earned_date, expires_at
                ) VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(after.user_id)
            .bind(after.id)
            .bind(after.comp_time_hours)
            .bind(after.overtime_date)
            .bind(after.comp_time_expires_at)
            .execute(&mut *tx)
            .await?;
        }

        // event_type 區分中途核准（行政）與最終核准（負責人 + 補休授予）
        let event_type = if is_final {
            "OVERTIME_APPROVE_FINAL"
        } else {
            "OVERTIME_APPROVE_INTERIM"
        };
        let display = format!(
            "{} {} ({}h)",
            after.overtime_date, after.overtime_type, after.hours
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "HR",
                event_type,
                entity: Some(AuditEntity::new("overtime_record", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Self::get_overtime_inner(pool, id).await
    }

    /// 作廢已核准的加班單（R86-2）。
    ///
    /// 存在理由：`delete_overtime` 只受理 draft，`reject_overtime` 只受理待審——
    /// 已核准的錯誤單（補登重跑造成的重複、時段填錯）此前完全沒有撤銷路徑，
    /// 而它已經授出補休餘額。
    ///
    /// 規則（2026-07-31 使用者裁定）：
    /// - ADMIN 單簽即可，但**理由必填**、且不得作廢自己的加班單。
    /// - 補休已被請假使用（`used_hours > 0`）或已折算加班費者一律擋下，
    ///   要求先處理掉已使用部分，避免造出負餘額或默默改到已核准的假單。
    /// - 原單保留（status='voided' + 作廢人/時間/理由），不刪除，稽核鏈完整。
    pub async fn void_overtime(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        reason: &str,
    ) -> Result<OvertimeWithUser> {
        let user = actor.require_user()?;
        let reason = Self::validate_void_request(user, reason)?;

        let mut tx = pool.begin().await?;
        let before = Self::load_voidable_overtime(&mut tx, id, user.id).await?;
        let balance = Self::take_unused_comp_time_balance(&mut tx, id).await?;

        let after: OvertimeRecord = sqlx::query_as(
            r#"
            UPDATE overtime_records
            SET status = 'voided', voided_by = $2, voided_at = NOW(),
                void_reason = $3, updated_at = NOW()
            WHERE id = $1 AND status = 'approved'
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(user.id)
        .bind(reason)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!(
            "{} {} ({}h)",
            after.overtime_date, after.overtime_type, after.hours
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "HR",
                event_type: "OVERTIME_VOID",
                entity: Some(AuditEntity::new("overtime_record", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        // 收回的補休餘額另記一筆——上面那筆 diff 只涵蓋 overtime_records，
        // 沒有這筆的話稽核者無法從日誌重建「究竟收回了幾小時、哪一批」。
        if let Some(balance) = balance {
            let display = format!("{} ({}h)", balance.earned_date, balance.original_hours);
            AuditService::log_activity_tx(
                &mut tx,
                actor,
                ActivityLogEntry {
                    event_category: "HR",
                    event_type: "COMP_TIME_REVOKE",
                    entity: Some(AuditEntity::new("comp_time_balance", balance.id, &display)),
                    data_diff: Some(DataDiff::delete_only(&balance)),
                    request_context: None,
                },
            )
            .await?;
        }

        tx.commit().await?;

        Self::get_overtime_inner(pool, id).await
    }

    /// 作廢的前置檢查（不需 DB）：權限 + 理由必填。回傳 trim 後的理由。
    fn validate_void_request<'a>(user: &CurrentUser, reason: &'a str) -> Result<&'a str> {
        if !user.is_admin() {
            return Err(AppError::Forbidden(
                "僅負責人可作廢已核准的加班單".to_string(),
            ));
        }
        let reason = reason.trim();
        if reason.is_empty() {
            return Err(AppError::Validation("作廢理由為必填".to_string()));
        }
        Ok(reason)
    }

    /// 取出並鎖定可作廢的加班單：必須存在、為已核准、且不是作廢者自己的單。
    async fn load_voidable_overtime(
        conn: &mut sqlx::PgConnection,
        id: Uuid,
        actor_id: Uuid,
    ) -> Result<OvertimeRecord> {
        let before: OvertimeRecord =
            sqlx::query_as("SELECT * FROM overtime_records WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_optional(conn)
                .await?
                .ok_or_else(|| AppError::NotFound("加班紀錄不存在".into()))?;

        if before.status != "approved" {
            return Err(AppError::BusinessRule(format!(
                "僅已核准的加班單可作廢（目前狀態：{}）",
                overtime_status_label(&before.status)
            )));
        }
        if before.user_id == actor_id {
            return Err(AppError::Forbidden(
                "不可作廢自己的加班單，請由其他負責人處理".to_string(),
            ));
        }
        Ok(before)
    }

    /// 收回該加班單授出的補休餘額，回傳被刪除的整列（供稽核記錄）。
    ///
    /// 餘額必須還沒被動用：已請假使用或已折算加班費就擋下，
    /// 否則作廢會造出負餘額或默默影響已核准的假單。
    async fn take_unused_comp_time_balance(
        conn: &mut sqlx::PgConnection,
        overtime_id: Uuid,
    ) -> Result<Option<CompTimeBalance>> {
        let balance: Option<CompTimeBalance> = sqlx::query_as(
            "SELECT * FROM comp_time_balances WHERE overtime_record_id = $1 FOR UPDATE",
        )
        .bind(overtime_id)
        .fetch_optional(&mut *conn)
        .await?;

        let Some(balance) = balance else {
            return Ok(None);
        };
        if balance.used_hours > Decimal::ZERO {
            return Err(AppError::BusinessRule(format!(
                "此加班授出的補休已使用 {} 小時，請先取消對應的補休假單再作廢",
                balance.used_hours
            )));
        }
        if balance.converted_to_pay {
            return Err(AppError::BusinessRule(
                "此加班的補休已折算加班費，無法作廢".to_string(),
            ));
        }

        // 餘額與加班單狀態必須同一 tx（同 approve_overtime 授予補休的理由）。
        sqlx::query("DELETE FROM comp_time_balances WHERE id = $1")
            .bind(balance.id)
            .execute(conn)
            .await?;
        Ok(Some(balance))
    }

    /// 內部用：不做權限檢查的 overtime 查詢（callers 已在 tx 內做過授權）
    async fn get_overtime_inner(pool: &PgPool, id: Uuid) -> Result<OvertimeWithUser> {
        let record = sqlx::query_as::<_, OvertimeWithUser>(
            r#"
            SELECT
                o.id, o.user_id, u.email as user_email, u.display_name as user_name,
                o.overtime_date, o.start_time, o.end_time, o.hours,
                o.overtime_type, o.multiplier, o.comp_time_hours, o.comp_time_expires_at,
                o.status, o.reason, o.void_reason
            FROM overtime_records o
            INNER JOIN users u ON o.user_id = u.id
            WHERE o.id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;
        Ok(record)
    }

    // ============================================================
    // 勞基法合規驗證
    // ============================================================

    /// 勞基法 §32：檢查本月加班上限。
    /// 查詢該員工當月已核准/待審加班時數，加上本次申請是否超限。
    pub async fn check_monthly_overtime_limit(
        pool: &PgPool,
        user_id: Uuid,
        overtime_date: NaiveDate,
        requested_hours: f64,
    ) -> Result<OvertimeLimitCheck> {
        let year = overtime_date.year();
        let month = overtime_date.month();

        let row: (f64,) = sqlx::query_as(
            r#"
            SELECT COALESCE(SUM(hours), 0)::float8
            FROM overtime_records
            WHERE user_id = $1
              AND EXTRACT(YEAR FROM overtime_date) = $2
              AND EXTRACT(MONTH FROM overtime_date) = $3
              AND status NOT IN ('rejected', 'cancelled')
            "#,
        )
        .bind(user_id)
        .bind(year as f64)
        .bind(month as f64)
        .fetch_one(pool)
        .await?;

        let monthly_total = row.0;
        let projected = monthly_total + requested_hours;
        let mut warnings = Vec::new();

        if projected > MONTHLY_OVERTIME_LIMIT {
            warnings.push(format!(
                "本月加班將達 {:.1} 小時，超過勞基法§32標準上限 {} 小時",
                projected, MONTHLY_OVERTIME_LIMIT
            ));
        }
        if projected > MONTHLY_OVERTIME_LIMIT_EXTENDED {
            warnings.push(format!(
                "本月加班將達 {:.1} 小時，超過勞基法§32特殊上限 {} 小時（需勞資協議）",
                projected, MONTHLY_OVERTIME_LIMIT_EXTENDED
            ));
        }

        Ok(OvertimeLimitCheck {
            monthly_total,
            requested_hours,
            projected_total: projected,
            exceeds_standard_limit: projected > MONTHLY_OVERTIME_LIMIT,
            exceeds_extended_limit: projected > MONTHLY_OVERTIME_LIMIT_EXTENDED,
            warnings,
        })
    }

    /// 勞基法 §30：驗證日/週工時。
    /// 計算指定日期的實際工時，以及該週的累計工時。
    pub async fn validate_work_hours(
        pool: &PgPool,
        user_id: Uuid,
        work_date: NaiveDate,
    ) -> Result<WorkHoursValidation> {
        use chrono::Datelike;

        // 查詢當日工時
        let daily: (f64,) = sqlx::query_as(
            r#"
            SELECT COALESCE(regular_hours, 0)::float8
            FROM attendance_records
            WHERE user_id = $1 AND work_date = $2
            "#,
        )
        .bind(user_id)
        .bind(work_date)
        .fetch_optional(pool)
        .await?
        .unwrap_or((0.0,));

        // 計算該週的週一到週日
        let weekday = work_date.weekday().num_days_from_monday(); // 0=Mon
        let week_start = work_date - chrono::Duration::days(weekday as i64);
        let week_end = week_start + chrono::Duration::days(6);

        // 查詢本週累計工時
        let weekly: (f64,) = sqlx::query_as(
            r#"
            SELECT COALESCE(SUM(regular_hours), 0)::float8
            FROM attendance_records
            WHERE user_id = $1 AND work_date BETWEEN $2 AND $3
            "#,
        )
        .bind(user_id)
        .bind(week_start)
        .bind(week_end)
        .fetch_one(pool)
        .await?;

        let daily_hours = daily.0;
        let weekly_hours = weekly.0;
        let mut warnings = Vec::new();

        let exceeds_daily = daily_hours > DAILY_REGULAR_HOURS;
        let exceeds_weekly = weekly_hours > WEEKLY_REGULAR_HOURS;
        let suggested_ot = (daily_hours - DAILY_REGULAR_HOURS).max(0.0);

        if exceeds_daily {
            warnings.push(format!(
                "當日工時 {:.1}hr 超過勞基法§30標準 {}hr，建議登錄 {:.1}hr 加班",
                daily_hours, DAILY_REGULAR_HOURS, suggested_ot
            ));
        }
        if exceeds_weekly {
            warnings.push(format!(
                "本週累計 {:.1}hr 超過勞基法§30標準 {}hr",
                weekly_hours, WEEKLY_REGULAR_HOURS
            ));
        }

        Ok(WorkHoursValidation {
            daily_hours,
            weekly_hours,
            exceeds_daily_limit: exceeds_daily,
            exceeds_weekly_limit: exceeds_weekly,
            suggested_overtime_hours: suggested_ot,
            warnings,
        })
    }

    /// 勞基法 §24：計算平日加班分段時數。
    pub fn calculate_weekday_overtime_tiers(hours: f64) -> WeekdayOvertimeTiers {
        split_weekday_overtime(hours)
    }

    pub async fn reject_overtime(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        reason: &str,
    ) -> Result<OvertimeWithUser> {
        let user = actor.require_user()?;
        let rejecter_id = user.id;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, OvertimeRecord>(
            "SELECT * FROM overtime_records WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("加班紀錄不存在".into()))?;

        // SEC-BIZ: 只有待審核（pending_*）的加班可駁回。
        // 缺少此守衛時，已核准的加班可被翻成 rejected 但補休餘額不會回收，
        // 造成幽靈補休 + 釋放月加班上限額度。
        if before.status != "pending_admin_staff" && before.status != "pending_admin" {
            return Err(AppError::Conflict(format!(
                "只有待審核的加班可駁回（目前狀態：{}）",
                before.status
            )));
        }

        // SEC-BIZ: 職權分離 — 不可駁回自己的加班（對齊 approve 端的自核守衛）
        if before.user_id == rejecter_id {
            return Err(AppError::BusinessRule("不可駁回自己的加班申請".into()));
        }

        let after = sqlx::query_as::<_, OvertimeRecord>(
            r#"
            UPDATE overtime_records
            SET status = 'rejected', rejected_by = $2, rejected_at = NOW(), rejection_reason = $3, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(rejecter_id)
        .bind(reason)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!(
            "{} {} ({}h)",
            after.overtime_date, after.overtime_type, after.hours
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "HR",
                event_type: "OVERTIME_REJECT",
                entity: Some(AuditEntity::new("overtime_record", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Self::get_overtime_inner(pool, id).await
    }
}

#[cfg(test)]
mod tests {
    use super::{
        calc_hours_from_minutes, comp_time_hours_for_type, overtime_multiplier,
        round_overtime_minutes_to_half_hour, split_weekday_overtime, weekday_overtime_tiers,
        weekday_overtime_weighted_hours,
    };

    // --- overtime_multiplier ---

    #[test]
    fn test_overtime_multiplier_known_types() {
        assert_eq!(overtime_multiplier("A"), 1.0);
        assert_eq!(overtime_multiplier("B"), 1.33);
        assert_eq!(overtime_multiplier("C"), 1.66);
        assert_eq!(overtime_multiplier("D"), 2.0);
    }

    #[test]
    fn test_overtime_multiplier_unknown_defaults_to_one() {
        assert_eq!(overtime_multiplier("X"), 1.0);
        assert_eq!(overtime_multiplier(""), 1.0);
    }

    // --- comp_time_hours_for_type ---

    #[test]
    fn test_comp_time_hours_c_d_get_eight() {
        assert_eq!(comp_time_hours_for_type("C"), 8.0);
        assert_eq!(comp_time_hours_for_type("D"), 8.0);
    }

    #[test]
    fn test_comp_time_hours_a_b_get_zero() {
        assert_eq!(comp_time_hours_for_type("A"), 0.0);
        assert_eq!(comp_time_hours_for_type("B"), 0.0);
        assert_eq!(comp_time_hours_for_type("X"), 0.0);
    }

    // --- calc_hours_from_minutes ---

    #[test]
    fn test_calc_hours_exact() {
        // 18:00 - 09:00 = 9h
        assert_eq!(calc_hours_from_minutes(540, 1080), 9.0);
    }

    #[test]
    fn test_calc_hours_half_hour_rounding() {
        // 62 分鐘 ≈ 1.0 小時（捨入至 0.5）
        assert_eq!(calc_hours_from_minutes(0, 62), 1.0);
        // 45 分鐘 → 0.5 小時
        assert_eq!(calc_hours_from_minutes(0, 45), 0.5);
        // 30 分鐘 → 0.5 小時
        assert_eq!(calc_hours_from_minutes(0, 30), 0.5);
    }

    #[test]
    fn test_calc_hours_one_and_half() {
        // 90 分鐘 = 1.5 小時
        assert_eq!(calc_hours_from_minutes(0, 90), 1.5);
    }

    // --- split_weekday_overtime (勞基法 §24) ---

    #[test]
    fn test_weekday_tiers_under_two_hours() {
        let t = split_weekday_overtime(1.5);
        assert_eq!(t.tier1_hours, 1.5);
        assert_eq!(t.tier2_hours, 0.0);
        assert_eq!(t.total_hours, 1.5);
    }

    #[test]
    fn test_weekday_tiers_exactly_two_hours() {
        let t = split_weekday_overtime(2.0);
        assert_eq!(t.tier1_hours, 2.0);
        assert_eq!(t.tier2_hours, 0.0);
    }

    #[test]
    fn test_weekday_tiers_over_two_hours() {
        let t = split_weekday_overtime(3.5);
        assert_eq!(t.tier1_hours, 2.0);
        assert_eq!(t.tier2_hours, 1.5);
        assert_eq!(t.total_hours, 3.5);
    }

    #[test]
    fn test_weekday_tiers_four_hours() {
        let t = split_weekday_overtime(4.0);
        assert_eq!(t.tier1_hours, 2.0);
        assert_eq!(t.tier2_hours, 2.0);
    }

    #[test]
    fn test_weekday_tiers_rounds_to_half() {
        // 2h20m = 2.33... → 捨入至 2.0
        let t = split_weekday_overtime(2.33);
        assert_eq!(t.total_hours, 2.0);
        assert_eq!(t.tier1_hours, 2.0);
        assert_eq!(t.tier2_hours, 0.0);
    }

    #[test]
    fn test_weekday_tiers_zero() {
        let t = split_weekday_overtime(0.0);
        assert_eq!(t.tier1_hours, 0.0);
        assert_eq!(t.tier2_hours, 0.0);
    }

    // --- round_overtime_minutes_to_half_hour (四捨五入至 30 分) ---

    #[test]
    fn test_round_ot_minutes_user_examples() {
        // 使用者明定：25→30、35→30、55→60、65→60
        assert_eq!(round_overtime_minutes_to_half_hour(25), 0.5);
        assert_eq!(round_overtime_minutes_to_half_hour(35), 0.5);
        assert_eq!(round_overtime_minutes_to_half_hour(55), 1.0);
        assert_eq!(round_overtime_minutes_to_half_hour(65), 1.0);
    }

    #[test]
    fn test_round_ot_minutes_boundaries() {
        assert_eq!(round_overtime_minutes_to_half_hour(0), 0.0);
        assert_eq!(round_overtime_minutes_to_half_hour(14), 0.0); // <15 捨去
        assert_eq!(round_overtime_minutes_to_half_hour(15), 0.5); // 15 進位
        assert_eq!(round_overtime_minutes_to_half_hour(45), 1.0); // 45 進位至 60
        assert_eq!(round_overtime_minutes_to_half_hour(120), 2.0);
    }

    #[test]
    fn test_round_ot_minutes_negative_is_zero() {
        // 早退（打卡早於班別下班）不算加班
        assert_eq!(round_overtime_minutes_to_half_hour(-10), 0.0);
    }

    // --- weekday_overtime_tiers (start, end；班別下班起算) ---

    #[test]
    fn test_tiers_standard_shift_clock_out_1900() {
        // 常規班 17:30 下班；打卡 19:00 → 90 分 → 1.5h
        let t = weekday_overtime_tiers(17 * 60 + 30, 19 * 60);
        assert_eq!(t.total_hours, 1.5);
        assert_eq!(t.tier1_hours, 1.5);
        assert_eq!(t.tier2_hours, 0.0);
    }

    #[test]
    fn test_tiers_early_shift_clock_out_1955() {
        // 早班 16:30 下班；打卡 19:55 → 205 分 → 四捨五入 210 分 → 3.5h
        let t = weekday_overtime_tiers(16 * 60 + 30, 19 * 60 + 55);
        assert_eq!(t.total_hours, 3.5);
        assert_eq!(t.tier1_hours, 2.0);
        assert_eq!(t.tier2_hours, 1.5);
    }

    #[test]
    fn test_tiers_clock_out_before_shift_end_is_zero() {
        // 早班 16:30，16:00 就打卡（早退）→ 加班 0
        let t = weekday_overtime_tiers(16 * 60 + 30, 16 * 60);
        assert_eq!(t.total_hours, 0.0);
        assert_eq!(t.tier1_hours, 0.0);
        assert_eq!(t.tier2_hours, 0.0);
    }

    // --- weekday_overtime_weighted_hours (加權係數) ---

    #[test]
    fn test_weighted_three_hours_equals_4_32() {
        // 常規班 17:30 下班，打卡 20:30 → 3h → 2×1.33 + 1×1.66 = 4.32
        let t = weekday_overtime_tiers(17 * 60 + 30, 17 * 60 + 30 + 180);
        let w = weekday_overtime_weighted_hours(&t);
        assert!((w - 4.32).abs() < 1e-9, "expected 4.32, got {w}");
    }

    #[test]
    fn test_weighted_under_two_hours() {
        // 1.5h 全在第一段 → 1.5×1.33 = 1.995
        let t = weekday_overtime_tiers(17 * 60 + 30, 17 * 60 + 30 + 90);
        let w = weekday_overtime_weighted_hours(&t);
        assert!((w - 1.995).abs() < 1e-9, "expected 1.995, got {w}");
    }

    #[test]
    fn test_weighted_zero_when_no_overtime() {
        let t = weekday_overtime_tiers(17 * 60 + 30, 17 * 60 + 30);
        assert_eq!(weekday_overtime_weighted_hours(&t), 0.0);
    }
}
