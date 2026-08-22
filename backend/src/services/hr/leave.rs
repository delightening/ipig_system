// HR 請假管理

use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::{ActorContext, CurrentUser},
    models::{
        audit_diff::DataDiff, CreateLeaveRequest, CreateNotificationRequest, LeaveQuery,
        LeaveRequest, LeaveRequestWithUser, LeaveStatus, NotificationType, PaginatedResponse,
        UpdateLeaveRequest,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService, NotificationService,
    },
    Result,
};

use super::HrService;

/// `cancel_leave` 回傳：取消後的紀錄 + 通知派發所需資訊。
///
/// handler 不再自行讀 DB 推導 `was_approved` 或反查申請人名稱（符 CLAUDE.md §4：
/// handler 禁止 SQL / DB 讀取），一律由 service 於交易脈絡內取得後回傳。
pub struct CancelLeaveOutcome {
    pub record: LeaveRequest,
    pub was_approved: bool,
    /// 請假當事人 display_name（主管代取消時 ≠ 取消者）。
    pub applicant_name: String,
}

/// 一筆已回填（或將被回填，dry-run 時）的置頂待辦記錄，供
/// [`HrService::backfill_missing_leave_pins`] 的呼叫端（bin）列印。
#[derive(Debug, Clone)]
pub struct BackfilledLeavePin {
    pub leave_id: Uuid,
    pub applicant_name: String,
    /// 假單當時的狀態（`PENDING_PROXY` / `PENDING_L1` / `PENDING_DIRECTOR`）。
    pub status: String,
    pub recipient_role: &'static str,
    pub recipient_names: Vec<String>,
}

/// [`HrService::backfill_missing_leave_pins`] 的回填結果。
///
/// 所有跳過路徑都要有對應計數欄位：`created + already_pinned + missing_proxy +
/// no_recipients.len() + state_changed` 應等於候選假單總數。缺一個欄位就等於有一批
/// 假單被靜默略過、操作者無從對帳（2026-08-14 獨立審查抓到，當時三條 `continue`
/// 完全不計數）。
#[derive(Debug, Default)]
pub struct BackfillLeavePinsReport {
    /// 已建立（dry-run 時為「將建立」）的列。
    pub created: Vec<BackfilledLeavePin>,
    /// 該關全部收件人都已有置頂待辦、整筆略過的數量（冪等可重跑的依據）。
    pub already_pinned: i64,
    /// `PENDING_PROXY` 卻缺 `proxy_user_id` 的異常筆數（理論上不可能，防禦性記錄）。
    pub missing_proxy: i64,
    /// **算不出任何合法審核人**的假單 id。
    ///
    /// 這不是無害的略過——`current_stage_approvers_tx` 連 admin fallback 都空手，
    /// 代表這張單這一關「沒有任何人簽得下去」，正是本工具要偵測的病症本身。
    /// 必須讓操作者看見，不可只是 `continue`。
    pub no_recipients: Vec<Uuid>,
    /// 鎖定後重讀發現已離開待處理三關（被其他操作處理掉／轉關／刪除）的筆數。
    /// 屬預期的併發結果，非錯誤，但要能對帳。
    pub state_changed: i64,
}

impl HrService {
    // ============================================
    // Leave
    // ============================================

    pub async fn list_leaves(
        pool: &PgPool,
        query: &LeaveQuery,
        current_user: &CurrentUser,
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

        let mut data = sqlx::query_as::<_, LeaveRequestWithUser>(
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

        // 逐列計算「當前使用者是否可審核」，與 approve/reject 服務層授權完全一致
        // （中央 can_user_approve_leave：兩關 + 職責分離 + admin 卡關代批）。
        // 僅對待審中(PENDING*)的列計算，其餘直接 false，避免多餘查詢。
        //
        // (列索引, 該關審核人 id) —— 姓名延後到迴圈結束後整頁一次撈。
        let mut stage_approver_ids: Vec<(usize, Vec<Uuid>)> = Vec::new();
        for (idx, row) in data.iter_mut().enumerate() {
            row.can_approve = if row.status.starts_with("PENDING") {
                Self::can_user_approve_leave(pool, row.id, &row.status, row.user_id, current_user)
                    .await?
            } else {
                false
            };
            // 代理確認關（PENDING_PROXY）：僅該假單指定的代理人本人可確認/退回。
            row.can_confirm_proxy = row.status == LeaveStatus::PendingProxy.as_str()
                && row.proxy_user_id == Some(current_user.id);

            // PENDING_PROXY 的 current_approver_name 已由上面的 SQL JOIN 帶出
            // （current_approver_id 送審時直接設成 proxy_user_id）。但 L1/DIRECTOR 這兩關
            // 沒有單一 approver_id 可 JOIN——審核人是動態算出的一批人（部門主管／負責人，
            // 缺人時 fallback 在職管理員），權威來源就是 current_stage_approvers（與
            // can_user_approve_leave 共用同一組查詢）。這裡補算，讓列表 tooltip
            // 「卡在誰手上」有資料可顯示，而非前端明明有 UI 卻永遠拿到 null。
            //
            // 姓名不在這裡查：先蒐集 id，迴圈後整頁一次撈齊（見下方），避免每列一次
            // roundtrip（per_page 上限 500）。
            if row.status == LeaveStatus::PendingL1.as_str()
                || row.status == LeaveStatus::PendingDirector.as_str()
            {
                let approver_ids =
                    Self::current_stage_approvers_known(pool, row.id, row.user_id, &row.status)
                        .await?;
                if !approver_ids.is_empty() {
                    stage_approver_ids.push((idx, approver_ids));
                }
            }
        }

        // 整頁一次撈齊審核人姓名（原本每列各一次 `WHERE id = ANY(...)`）。
        if !stage_approver_ids.is_empty() {
            let all_ids: Vec<Uuid> = stage_approver_ids
                .iter()
                .flat_map(|(_, ids)| ids.iter().copied())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            let name_by_id: std::collections::HashMap<Uuid, String> =
                sqlx::query_as::<_, (Uuid, String)>(
                    "SELECT id, display_name FROM users WHERE id = ANY($1)",
                )
                .bind(&all_ids)
                .fetch_all(pool)
                .await?
                .into_iter()
                .collect();

            for (idx, ids) in stage_approver_ids {
                // 維持原本 `ORDER BY display_name` 的呈現順序。查不到姓名者（帳號已硬刪）
                // 直接略過，不讓 tooltip 出現空字串。
                let mut names: Vec<String> = ids
                    .iter()
                    .filter_map(|id| name_by_id.get(id).cloned())
                    .collect();
                names.sort();
                if !names.is_empty() {
                    data[idx].current_approver_name = Some(names.join("、"));
                }
            }
        }

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

    // ============================================
    // 審核資格判定（兩關 + 職責分離 + admin 卡關代批）
    // ============================================

    /// 申請人所屬部門是否有「可審核 L1」的合法主管（在職、且非申請人本人）。
    /// 單位主管關（L1）目前合法審核人 —— 也是「這關該通知誰」的權威來源。
    ///
    /// `l1_has_eligible_approver`（自動跳關判斷）與通知 resolver
    /// `leave_current_stage_approvers`（該通知誰）都建立在這個查詢上，
    /// 刻意不各自維護一份條件：兩者一旦漂移，會出現「跳過這關但還是通知了
    /// 這關的人」或反過來「該通知的人沒收到」，且不會有任何錯誤訊息。
    async fn l1_eligible_managers(
        executor: impl sqlx::PgExecutor<'_>,
        applicant_id: Uuid,
    ) -> Result<Vec<Uuid>> {
        let rows: Vec<(Uuid,)> = sqlx::query_as(
            r#"SELECT d.manager_id FROM users u
                JOIN departments d ON u.department_id = d.id
                JOIN users m ON m.id = d.manager_id
                WHERE u.id = $1 AND m.is_active = true AND m.deleted_at IS NULL
                  AND d.manager_id <> $1"#,
        )
        .bind(applicant_id)
        .fetch_all(executor)
        .await?;
        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    async fn l1_has_eligible_approver(pool: &PgPool, applicant_id: Uuid) -> Result<bool> {
        Ok(!Self::l1_eligible_managers(pool, applicant_id)
            .await?
            .is_empty())
    }

    /// 指定使用者是否為申請人所屬部門的主管。
    async fn is_dept_manager_of(pool: &PgPool, applicant_id: Uuid, user_id: Uuid) -> Result<bool> {
        let exists: (bool,) = sqlx::query_as(
            r#"SELECT EXISTS(
                SELECT 1 FROM users u JOIN departments d ON u.department_id = d.id
                WHERE u.id = $1 AND d.manager_id = $2
            )"#,
        )
        .bind(applicant_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;
        Ok(exists.0)
    }

    /// 負責人（終審）關是否有合法審核人：在職 DIRECTOR、非申請人、且未批過本單前關（職責分離）。
    /// 負責人（終審）關目前合法審核人：在職 DIRECTOR、非申請人、且未批過本單
    /// 前關（職責分離）。同上，是「該通知誰」的權威來源，見 l1_eligible_managers
    /// 的說明——`director_has_eligible_approver` 建立在這上面，不重寫一份條件。
    async fn director_eligible_directors(
        executor: impl sqlx::PgExecutor<'_>,
        leave_id: Uuid,
        applicant_id: Uuid,
    ) -> Result<Vec<Uuid>> {
        let rows: Vec<(Uuid,)> = sqlx::query_as(
            r#"SELECT u.id FROM users u
                JOIN user_roles ur ON ur.user_id = u.id
                JOIN roles r ON r.id = ur.role_id
                WHERE r.code = $1 AND u.is_active = true AND u.deleted_at IS NULL
                  AND u.id <> $2
                  AND NOT EXISTS (
                    SELECT 1 FROM leave_approvals la
                    WHERE la.leave_request_id = $3 AND la.approver_id = u.id AND la.action = 'APPROVE'
                      AND la.approval_level <> 'PENDING_PROXY'
                  )"#,
        )
        .bind(crate::constants::ROLE_DIRECTOR)
        .bind(applicant_id)
        .bind(leave_id)
        .fetch_all(executor)
        .await?;
        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    async fn director_has_eligible_approver(
        pool: &PgPool,
        leave_id: Uuid,
        applicant_id: Uuid,
    ) -> Result<bool> {
        Ok(
            !Self::director_eligible_directors(pool, leave_id, applicant_id)
                .await?
                .is_empty(),
        )
    }

    /// 卡關代批的後備名單：在職管理員（SYSTEM_ADMIN / legacy admin），排除申請人本人。
    ///
    /// `can_user_approve_leave` 的第一條規則就是「不可審核自己的假單，任何角色皆不放寬」
    /// （見該函式開頭 `applicant_id == user.id` 檢查）——這裡若不排除申請人，當申請人
    /// 本身是管理員、且該關卡關時，會把申請人自己放進通知名單，點下去必定被那條規則
    /// 擋成 403，重新製造「被通知卻 403」的情境（CodeRabbit PR #67 review）。
    async fn admin_roster(
        executor: impl sqlx::PgExecutor<'_>,
        exclude_user_id: Uuid,
    ) -> Result<Vec<Uuid>> {
        let rows: Vec<(Uuid,)> = sqlx::query_as(
            r#"SELECT DISTINCT u.id FROM users u
                JOIN user_roles ur ON ur.user_id = u.id
                JOIN roles r ON r.id = ur.role_id
                WHERE r.code IN ($1, $2) AND u.is_active = true AND u.deleted_at IS NULL
                  AND u.id <> $3"#,
        )
        .bind(crate::constants::ROLE_SYSTEM_ADMIN)
        .bind(crate::constants::ROLE_ADMIN_LEGACY)
        .bind(exclude_user_id)
        .fetch_all(executor)
        .await?;
        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    /// 「這則假單現在這一關，該通知誰」的權威來源——與 `can_user_approve_leave`
    /// 的授權判準共用同一組查詢（`l1_eligible_managers` / `director_eligible_directors`
    /// / `admin_roster`），保證「被通知的人」與「真的點得下去核准鈕的人」永遠是同一批。
    ///
    /// 取代原本 `leave_submitted` 事件的角色廣播（`admin` + `DIRECTOR`，不分審核關卡）
    /// ——那會讓還輪不到的角色也收到「待處理」，點下去卻 403。供通知 resolver
    /// `leave_current_stage_approvers` 呼叫（`services/notification/resolvers.rs`）。
    ///
    /// 非審核中狀態（`DRAFT`、`PENDING_PROXY`、終態）回空——那些階段的「該通知誰」
    /// 由別的機制負責（代理確認見 `submit_leave`；終態的核准/駁回通知見
    /// handler 端既有的 `notify_leave_approved` 等）。
    pub async fn current_stage_approvers(pool: &PgPool, leave_id: Uuid) -> Result<Vec<Uuid>> {
        let row: Option<(String, Uuid)> =
            sqlx::query_as("SELECT status::text, user_id FROM leave_requests WHERE id = $1")
                .bind(leave_id)
                .fetch_optional(pool)
                .await?;
        let Some((status, applicant_id)) = row else {
            return Ok(vec![]);
        };
        Self::current_stage_approvers_known(pool, leave_id, applicant_id, &status).await
    }

    /// [`Self::current_stage_approvers`] 的「呼叫端已知 status / applicant_id」版本。
    ///
    /// 存在的理由只有一個：`list_leaves` 的迴圈裡這兩個值本來就在手上（`row.status` /
    /// `row.user_id`），走上面那支等於每列都白白重查一次 `leave_requests`。
    /// **判準本身不在這裡複製**——照樣呼叫同一組 `l1_eligible_managers` /
    /// `director_eligible_directors` / `admin_roster`，維持「該通知誰」與「誰真的
    /// 按得下核准鈕」單一權威來源（見 `l1_eligible_managers` 的說明）。
    ///
    /// ⚠️ 呼叫端有責任確保傳進來的 status / applicant_id 與 `leave_id` 是同一列且為最新
    /// ——本函式不再自行核對。需要「鎖定並重讀最新狀態」的場景請用
    /// [`Self::current_stage_approvers_tx`]。
    pub async fn current_stage_approvers_known(
        pool: &PgPool,
        leave_id: Uuid,
        applicant_id: Uuid,
        status: &str,
    ) -> Result<Vec<Uuid>> {
        match status {
            s if s == LeaveStatus::PendingL1.as_str() => {
                let managers = Self::l1_eligible_managers(pool, applicant_id).await?;
                if !managers.is_empty() {
                    return Ok(managers);
                }
                Self::admin_roster(pool, applicant_id).await
            }
            s if s == LeaveStatus::PendingDirector.as_str() => {
                let directors =
                    Self::director_eligible_directors(pool, leave_id, applicant_id).await?;
                if !directors.is_empty() {
                    return Ok(directors);
                }
                Self::admin_roster(pool, applicant_id).await
            }
            _ => Ok(vec![]),
        }
    }

    /// tx 內版本的 [`Self::current_stage_approvers`]——`approve_leave` 中途核准後，
    /// 在同一 tx 建立下一關待辦時使用，讀到的是本 tx 剛寫入、尚未 commit 的最新狀態。
    /// 邏輯與 pool 版完全一致（呼叫同一組 `l1_eligible_managers` /
    /// `director_eligible_directors` / `admin_roster`），只是換一個可重複借用的
    /// executor 型別，避免另外維護一份判準（CodeRabbit PR #67 review）。
    async fn current_stage_approvers_tx(
        tx: &mut Transaction<'_, Postgres>,
        leave_id: Uuid,
    ) -> Result<Vec<Uuid>> {
        let row: Option<(String, Uuid)> =
            sqlx::query_as("SELECT status::text, user_id FROM leave_requests WHERE id = $1")
                .bind(leave_id)
                .fetch_optional(&mut **tx)
                .await?;
        let Some((status, applicant_id)) = row else {
            return Ok(vec![]);
        };

        match status.as_str() {
            s if s == LeaveStatus::PendingL1.as_str() => {
                let managers = Self::l1_eligible_managers(&mut **tx, applicant_id).await?;
                if !managers.is_empty() {
                    return Ok(managers);
                }
                Self::admin_roster(&mut **tx, applicant_id).await
            }
            s if s == LeaveStatus::PendingDirector.as_str() => {
                let directors =
                    Self::director_eligible_directors(&mut **tx, leave_id, applicant_id).await?;
                if !directors.is_empty() {
                    return Ok(directors);
                }
                Self::admin_roster(&mut **tx, applicant_id).await
            }
            _ => Ok(vec![]),
        }
    }

    /// 一次性回填：PR #67 上線前送審／轉關的假單，當時置頂待辦機制尚未存在，
    /// 從未收到任何通知——`submit_leave`／`approve_leave` 的 tx 內建立邏輯只對「以後的
    /// 轉關」生效，不會回頭補這批舊資料（`reconcile_pinned_notifications` 也只單向
    /// 降級孤兒待辦，不會反向建立）。
    ///
    /// 對每張仍在 `PENDING_PROXY` / `PENDING_L1` / `PENDING_DIRECTOR` 的假單，依目前狀態
    /// 算出正確收件人（代理人，或 `current_stage_approvers` 算出的本關審核人），用與即時
    /// 流程完全相同的標題/內容格式與 `recipient_role` 標記補建置頂待辦。
    ///
    /// 冪等：已存在對應 `recipient_role` 的置頂列（`priority>0`）者略過，可重複執行。
    pub async fn backfill_missing_leave_pins(
        pool: &PgPool,
        dry_run: bool,
    ) -> Result<BackfillLeavePinsReport> {
        // 候選清單只決定「要檢查哪些 leave_id」，不假設這份快照撐到逐筆處理時還準確——
        // 真正的狀態判斷、收件人計算、冪等檢查全部在下面每筆各自的 tx 內用 FOR UPDATE
        // 重新鎖定/讀取。原本沒有鎖定、直接用候選清單當下捕捉到的 status/proxy_user_id，
        // 若假單在回填執行期間被使用者的正常操作改變階段（例如代理人剛好在這時候確認，
        // PENDING_PROXY → PENDING_L1），會用過期的 proxy_user_id 送一則「請確認代理」的
        // pin 給早就已經確認過的人（CodeRabbit review：merge risk high，要求
        // transaction-scoped locking + state revalidation，2026-08-14）。
        let candidate_ids: Vec<Uuid> = sqlx::query_scalar(
            r#"SELECT id FROM leave_requests
               WHERE status::text IN ('PENDING_PROXY', 'PENDING_L1', 'PENDING_DIRECTOR')
               ORDER BY submitted_at"#,
        )
        .fetch_all(pool)
        .await?;

        let mut report = BackfillLeavePinsReport::default();

        // (user_id, proxy_user_id, leave_type, start_date, end_date, status)
        type LockedLeaveRow = (Uuid, Option<Uuid>, String, NaiveDate, NaiveDate, String);

        for leave_id in candidate_ids {
            let mut tx = pool.begin().await?;

            let row: Option<LockedLeaveRow> = sqlx::query_as(
                r#"SELECT user_id, proxy_user_id, leave_type::text, start_date, end_date, status::text
                   FROM leave_requests WHERE id = $1 FOR UPDATE"#,
            )
            .bind(leave_id)
            .fetch_optional(&mut *tx)
            .await?;

            let Some((applicant_id, proxy_user_id, leave_type, start_date, end_date, status)) = row
            else {
                // 併發下已被刪除；leave_requests 無硬刪路徑，理論上不會發生。
                // 併入 state_changed：對操作者而言同樣是「這筆已不需處理」。
                report.state_changed += 1;
                continue;
            };

            if !matches!(
                status.as_str(),
                "PENDING_PROXY" | "PENDING_L1" | "PENDING_DIRECTOR"
            ) {
                // 鎖定後重讀發現已離開待處理三關（代理確認/退回、核准、駁回、取消……）——
                // 候選清單是稍早的快照，真實狀態已經變了，這正是上面要擋住的競態。
                report.state_changed += 1;
                continue;
            }

            let (role, recipients): (&'static str, Vec<Uuid>) =
                if status == LeaveStatus::PendingProxy.as_str() {
                    let Some(proxy_id) = proxy_user_id else {
                        // 不可能發生：validate_proxy 已保證非負責人自報必填代理人。
                        // 防禦性記錄而非 panic，回填作業不該因一筆異常資料整支中止。
                        report.missing_proxy += 1;
                        continue;
                    };
                    (crate::models::RECIPIENT_ROLE_PROXY, vec![proxy_id])
                } else {
                    (
                        crate::models::RECIPIENT_ROLE_APPROVER,
                        Self::current_stage_approvers_tx(&mut tx, leave_id).await?,
                    )
                };

            if recipients.is_empty() {
                // 連 admin fallback 都空手 ＝ 這張單這一關「沒有任何人簽得下去」。
                // 這正是本工具要偵測的病症本身，必須讓操作者看見，不可只是 continue。
                report.no_recipients.push(leave_id);
                continue;
            }

            // 冪等檢查逐收件人比對，不是整批比對：這一關可能有多位合法審核人
            // （如 director_eligible_directors 回傳多位 DIRECTOR、或 admin_roster
            // fallback 命中多位管理員）。先前用 (leave_id, role) 單一 EXISTS 判斷
            // 「已回填」，只要其中一位已經有 pin 就整批 continue——導致同一關卡
            // 其他還沒補到的人永遠補不到（Qodo review 抓到，2026-08-14）。
            let already_pinned_uids: Vec<Uuid> = sqlx::query_scalar(
                r#"SELECT user_id FROM notifications
                   WHERE related_entity_type = 'leave_request' AND related_entity_id = $1
                     AND recipient_role = $2 AND priority > 0"#,
            )
            .bind(leave_id)
            .bind(role)
            .fetch_all(&mut *tx)
            .await?;
            let recipients: Vec<Uuid> = recipients
                .into_iter()
                .filter(|uid| !already_pinned_uids.contains(uid))
                .collect();
            if recipients.is_empty() {
                report.already_pinned += 1;
                continue;
            }

            let applicant_name = sqlx::query_scalar::<_, String>(
                "SELECT display_name FROM users WHERE id = $1 AND is_active = true AND deleted_at IS NULL",
            )
            .bind(applicant_id)
            .fetch_optional(&mut *tx)
            .await?
            .unwrap_or_else(|| "申請人".to_string());

            let recipient_names: Vec<String> = sqlx::query_scalar(
                "SELECT display_name FROM users WHERE id = ANY($1) ORDER BY display_name",
            )
            .bind(&recipients)
            .fetch_all(&mut *tx)
            .await?;

            if !dry_run {
                // 與 submit_leave（代理人）／notify_leave_submitted＋approve_leave 中途核准
                // （審核人）的既有格式逐字一致——回填出來的通知要跟即時產生的看起來一樣，
                // 不是重新設計一份文案。
                let (title, content) = if role == crate::models::RECIPIENT_ROLE_PROXY {
                    let leave_type_display = crate::models::LeaveType::from_db_str(&leave_type)
                        .map_or(leave_type.as_str(), |t| t.display_name());
                    (
                        format!("[iPig] {} 指定您為職務代理人，待您確認", applicant_name),
                        format!(
                            "{} 的請假申請指定您為職務代理人，請確認或退回。\n\n假別：{}\n期間：{} ~ {}",
                            applicant_name, leave_type_display, start_date, end_date
                        ),
                    )
                } else {
                    (
                        format!("[iPig] 新請假申請 - {}", applicant_name),
                        format!(
                            "有新的請假申請待審核。\n\n申請人：{}\n假別：{}\n期間：{} ~ {}",
                            applicant_name, leave_type, start_date, end_date
                        ),
                    )
                };

                for uid in &recipients {
                    NotificationService::create_pinned_notification_tx_with_role(
                        &mut tx,
                        CreateNotificationRequest {
                            user_id: *uid,
                            notification_type: NotificationType::LeaveApproval,
                            title: title.clone(),
                            content: Some(content.clone()),
                            related_entity_type: Some("leave_request".to_string()),
                            related_entity_id: Some(leave_id),
                        },
                        role,
                    )
                    .await?;
                }
            }

            report.created.push(BackfilledLeavePin {
                leave_id,
                applicant_name,
                status,
                recipient_role: role,
                recipient_names,
            });

            tx.commit().await?;
        }

        Ok(report)
    }

    /// 職責分離：使用者是否已在本單「核准」過任一**審核關**（批過前關者不得再批後關）。
    /// 代理確認（approval_level='PENDING_PROXY'）不算審核，故排除——否則「單位主管兼代理人」
    /// 於確認代理後會被 SoD 擋住而無法審核 PENDING_L1，導致假單卡關。
    async fn has_prior_approval(pool: &PgPool, leave_id: Uuid, user_id: Uuid) -> Result<bool> {
        let exists: (bool,) = sqlx::query_as(
            r#"SELECT EXISTS(
                SELECT 1 FROM leave_approvals
                WHERE leave_request_id = $1 AND approver_id = $2 AND action = 'APPROVE'
                  AND approval_level <> 'PENDING_PROXY'
            )"#,
        )
        .bind(leave_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;
        Ok(exists.0)
    }

    /// 使用者是否為此關的「指定審核人」（非 admin 代批）：L1=單位主管、DIRECTOR=負責人。
    fn is_designated_for(status: &str, is_dept_manager: bool, is_director: bool) -> bool {
        match status {
            s if s == LeaveStatus::PendingL1.as_str() => is_dept_manager,
            s if s == LeaveStatus::PendingDirector.as_str() => is_director,
            _ => false,
        }
    }

    /// 中央授權判定：使用者能否審核（核准/駁回）此單當前關卡。
    /// 規則：不可批自己、批過前關者不得再批(SoD)、該關指定審核人可批、
    /// admin 僅在「該關無其他合法審核人（卡關）」時可代批。
    /// 例外（SoD 放寬）：終審（DIRECTOR）關卡關且無其他合法 DIRECTOR 時，
    /// admin 即使批過前關仍可代批，避免單一審批人組織下假單永久死鎖。
    pub async fn can_user_approve_leave(
        pool: &PgPool,
        leave_id: Uuid,
        status: &str,
        applicant_id: Uuid,
        user: &CurrentUser,
    ) -> Result<bool> {
        // 不可審核自己的假單（任何關卡、任何角色皆不放寬）
        if applicant_id == user.id {
            return Ok(false);
        }
        let is_admin = user.is_admin();
        let is_director = user
            .roles
            .iter()
            .any(|r| r == crate::constants::ROLE_DIRECTOR);
        // 職責分離（SoD）：是否已在本單「核准」過前一審核關（代理確認不算）。
        let has_prior = Self::has_prior_approval(pool, leave_id, user.id).await?;
        match status {
            s if s == LeaveStatus::PendingL1.as_str() => {
                // SoD：批過前關者不得再批本關（L1 不放寬）。
                if has_prior {
                    return Ok(false);
                }
                if Self::is_dept_manager_of(pool, applicant_id, user.id).await? {
                    return Ok(true);
                }
                Ok(is_admin && !Self::l1_has_eligible_approver(pool, applicant_id).await?)
            }
            s if s == LeaveStatus::PendingDirector.as_str() => {
                // 指定負責人（未批過前關）→ 正常簽核。
                if is_director && !has_prior {
                    return Ok(true);
                }
                // 終審關卡關代批：無其他合法 DIRECTOR 時，admin 可代批終審。
                // 此處刻意放寬 SoD——即使 admin 批過前關仍可代批。否則在單一審批人組織下，
                // admin 於前關用掉 SoD 額度後，終審關將無人可簽 → 假單永久死鎖。
                // director_has_eligible_approver 已排除「批過前關者」，故僅在真正無他人時才代批，
                // 且系統一定有 admin，終審恆有真人簽核。
                Ok(is_admin
                    && !Self::director_has_eligible_approver(pool, leave_id, applicant_id).await?)
            }
            _ => Ok(false),
        }
    }

    /// 使用者是否具「負責人」角色。申請人未必等於當前操作者（`hr.leave.manage` 可代人送審），
    /// 故查 DB 而非讀 `CurrentUser.roles`。
    async fn user_is_director(pool: &PgPool, user_id: Uuid) -> Result<bool> {
        let exists: (bool,) = sqlx::query_as(
            r#"SELECT EXISTS(
                SELECT 1 FROM users u
                JOIN user_roles ur ON ur.user_id = u.id
                JOIN roles r ON r.id = ur.role_id
                WHERE u.id = $1 AND r.code = $2
                  AND u.is_active = true AND u.deleted_at IS NULL
            )"#,
        )
        .bind(user_id)
        .bind(crate::constants::ROLE_DIRECTOR)
        .fetch_one(pool)
        .await?;
        Ok(exists.0)
    }

    /// 職務代理人驗證：必填、不可為申請人本人、須在職、且不可於同時段也在請假。
    /// 例外：負責人之上無人可代理其職務，代理人改為選填——未指定時其假單走報備制
    /// （見 `submit_leave`），不進代理確認關。
    async fn validate_proxy(
        pool: &PgPool,
        applicant_id: Uuid,
        proxy_id: Option<Uuid>,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<()> {
        let Some(proxy_id) = proxy_id else {
            if Self::user_is_director(pool, applicant_id).await? {
                return Ok(());
            }
            return Err(AppError::BadRequest("請假必須指定職務代理人".into()));
        };
        if proxy_id == applicant_id {
            return Err(AppError::BadRequest("職務代理人不可為申請人本人".into()));
        }
        let active: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1 AND is_active = true AND deleted_at IS NULL)",
        )
        .bind(proxy_id)
        .fetch_one(pool)
        .await?;
        if !active.0 {
            return Err(AppError::BadRequest("所選職務代理人無效或已停用".into()));
        }
        // 排除「同時段也在請假」的代理人（未終結/已核准且日期重疊）。
        let overlap: (bool,) = sqlx::query_as(
            r#"SELECT EXISTS(
                SELECT 1 FROM leave_requests
                WHERE user_id = $1
                  AND status IN ('PENDING_PROXY','PENDING_L1','PENDING_L2','PENDING_HR','PENDING_GM','PENDING_DIRECTOR','APPROVED')
                  AND start_date <= $3 AND end_date >= $2
            )"#,
        )
        .bind(proxy_id)
        .bind(start)
        .bind(end)
        .fetch_one(pool)
        .await?;
        if overlap.0 {
            return Err(AppError::BadRequest(
                "所選職務代理人於該期間也在請假，請改選他人".into(),
            ));
        }
        Ok(())
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

        // 職務代理人必選 + 驗證（不可自己、須在職、排除同時段也請假者）
        Self::validate_proxy(
            pool,
            user_id,
            payload.proxy_user_id,
            payload.start_date,
            payload.end_date,
        )
        .await?;

        let effective_hours = payload.total_hours.unwrap_or(payload.total_days * 8.0);
        if !Self::is_half_hour_multiple(effective_hours) {
            return Err(AppError::BadRequest(
                "請假時數須為 0.5 小時的倍數（如 0.5、1、1.5、2...）".into(),
            ));
        }
        let total_days = payload
            .total_hours
            .map(|h| h / 8.0)
            .unwrap_or(payload.total_days);
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
        let user = actor.require_user()?;

        // 同 create_leave：時數須為 0.5 的倍數。優先檢查 total_hours；
        // 若僅提供 total_days，換算為時數再檢查（避免 0.3 天 = 2.4 小時 的偷渡）
        if let Some(hours) = payload.total_hours {
            if !Self::is_half_hour_multiple(hours) {
                return Err(AppError::BadRequest(
                    "請假時數須為 0.5 小時的倍數（如 0.5、1、1.5、2...）".into(),
                ));
            }
        } else if let Some(days) = payload.total_days {
            if !Self::is_half_hour_multiple(days * 8.0) {
                return Err(AppError::BadRequest(
                    "請假天數換算為時數後須為 0.5 小時的倍數".into(),
                ));
            }
        }

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

        if before.user_id != user.id && !user.has_permission("hr.leave.manage") {
            return Err(AppError::Forbidden("無權修改他人的請假申請".into()));
        }

        if before.status != "DRAFT" {
            return Err(AppError::BusinessRule("僅草稿狀態的請假可更新".into()));
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

        let display = format!(
            "{} {}~{}",
            after.leave_type, after.start_date, after.end_date
        );
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

    pub async fn delete_leave(pool: &PgPool, actor: &ActorContext, id: Uuid) -> Result<()> {
        let user = actor.require_user()?;
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

        if before.user_id != user.id && !user.has_permission("hr.leave.manage") {
            return Err(AppError::Forbidden("無權刪除他人的請假申請".into()));
        }

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

    /// 以 FOR UPDATE 鎖定並載入單筆請假（供狀態轉移前讀取，DRY 共用）。
    async fn lock_leave_for_update(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: Uuid,
    ) -> Result<LeaveRequest> {
        sqlx::query_as::<_, LeaveRequest>(
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
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound("請假申請不存在".into()))
    }

    /// 寫入一筆審核歷程（leave_approvals）。approval_level 記錄當前關卡狀態、action 為 APPROVE/REJECT。
    async fn insert_leave_approval(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        leave_id: Uuid,
        approver_id: Uuid,
        approval_level: &str,
        action: &str,
        comments: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO leave_approvals (id, leave_request_id, approver_id, approval_level, action, comments)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(leave_id)
        .bind(approver_id)
        .bind(approval_level)
        .bind(action)
        .bind(comments)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn submit_leave(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<LeaveRequest> {
        let user = actor.require_user()?;
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

        if before.user_id != user.id && !user.has_permission("hr.leave.manage") {
            return Err(AppError::Forbidden("無權送審他人的請假申請".into()));
        }

        if before.status != "DRAFT" {
            return Err(AppError::BusinessRule("僅草稿狀態的請假可送審".into()));
        }

        // 送審時再驗證職務代理人（草稿可能未帶或代理人狀態已變）
        Self::validate_proxy(
            pool,
            before.user_id,
            before.proxy_user_id,
            before.start_date,
            before.end_date,
        )
        .await?;

        // 負責人本人請假走報備制：沒有人可代理其職務，終審關也只剩他自己
        // （`can_user_approve_leave` 首條即禁止批自己的單，admin 代批同樣被擋）→
        // 送出即核准並扣餘額，否則假單會永久卡在終審關。
        const SELF_REPORT_COMMENT: &str = "負責人報備制：送出即核准";
        let director_self_report =
            before.proxy_user_id.is_none() && Self::user_is_director(pool, before.user_id).await?;

        let after = if director_self_report {
            Self::deduct_leave_balance(&mut tx, &before).await?;
            Self::insert_leave_approval(
                &mut tx,
                id,
                before.user_id,
                LeaveStatus::PendingDirector.as_str(),
                "APPROVE",
                Some(SELF_REPORT_COMMENT),
            )
            .await?;
            sqlx::query_as::<_, LeaveRequest>(
                r#"
                UPDATE leave_requests
                SET status = 'APPROVED'::leave_status, current_approver_id = NULL,
                    submitted_at = NOW(), approved_at = NOW(), updated_at = NOW()
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
            .await?
        } else {
            // 審核鏈第一關為「代理人確認」（非負責人時 validate_proxy 已保證 proxy 非空）。
            // current_approver_id 設為代理人，供「待我確認」清單反查。
            sqlx::query_as::<_, LeaveRequest>(
                r#"
                UPDATE leave_requests
                SET status = 'PENDING_PROXY'::leave_status, current_approver_id = proxy_user_id,
                    submitted_at = NOW(), updated_at = NOW()
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
            .await?
        };

        // 代理人確認是審核鏈第一關：送審後代理人要做出動作（確認或退回），
        // 屬「待處理」而非單純告知，故走置頂待辦而非一般通知。
        //
        // 建立放在這個 tx 內（而非 handler 端 best-effort），對齊
        // `create_pinned_notification_tx` 文件要求的「建立與解除都在業務狀態轉換自己的
        // tx 內」——只讓解除端 tx-scoped、建立端留在 commit 後 best-effort，
        // 會在「commit → 併發解除掃不到尚未建立的列 → 才建立」之間留下孤兒視窗
        // （見該函式文件；巡場報告已示範同一模式）。
        //
        // `director_self_report` 分支不進這裡：那個分支送出即核准（已是終態），
        // 沒有代理人需要確認。非該分支時 `proxy_user_id` 保證為 Some——
        // 上方 `validate_proxy` 已強制「非負責人自報則必填代理人」，
        // 唯一允許 None 的路徑（負責人本人）已被 `director_self_report` 分走。
        if !director_self_report {
            // `validate_proxy` 已保證這個分支下必為 Some，但不用 `expect()`：
            // 不變條件寫在另一個函式裡，日後那裡放寬就會變成 panic in prod。
            // 轉成 Internal 錯誤 → 整個 tx rollback，假單不會半途落地。
            let Some(proxy_id) = after.proxy_user_id else {
                return Err(AppError::Internal(
                    "送審流程要求職務代理人，但假單未帶 proxy_user_id".to_string(),
                ));
            };
            let applicant_name = crate::repositories::user::find_active_user_display_name_by_id_tx(
                &mut tx,
                after.user_id,
            )
            .await?
            .unwrap_or_else(|| "申請人".to_string());
            let leave_type_display = crate::models::LeaveType::from_db_str(&after.leave_type)
                .map_or(after.leave_type.as_str(), |t| t.display_name());
            crate::services::NotificationService::create_pinned_notification_tx_with_role(
                &mut tx,
                crate::models::CreateNotificationRequest {
                    user_id: proxy_id,
                    notification_type: crate::models::NotificationType::LeaveApproval,
                    title: format!("[iPig] {} 指定您為職務代理人，待您確認", applicant_name),
                    content: Some(format!(
                        "{} 的請假申請指定您為職務代理人，請確認或退回。\n\n假別：{}\n期間：{} ~ {}",
                        applicant_name, leave_type_display, after.start_date, after.end_date
                    )),
                    related_entity_type: Some("leave_request".to_string()),
                    related_entity_id: Some(after.id),
                },
                crate::models::RECIPIENT_ROLE_PROXY,
            )
            .await?;
        }

        let display = format!(
            "{} {}~{}",
            after.leave_type, after.start_date, after.end_date
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "HR",
                // 報備制與一般送審分開記事件，稽核查詢時可直接篩出負責人自核的假單。
                event_type: if director_self_report {
                    "LEAVE_DIRECTOR_SELF_APPROVE"
                } else {
                    "LEAVE_SUBMIT"
                },
                entity: Some(AuditEntity::new("leave_request", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(after)
    }

    /// 代理人確認：PENDING_PROXY →（單位主管有主管）PENDING_L1，否則跳關 → PENDING_DIRECTOR。
    /// 僅該假單指定的職務代理人本人可確認。
    pub async fn proxy_confirm_leave(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<LeaveRequest> {
        let user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        // 資源不存在時亦回 Forbidden（與存在但非代理人一致），避免以 404/403 差異
        // 探測特定假單是否存在（existence oracle 防護）。
        let before = match Self::lock_leave_for_update(&mut tx, id).await {
            Ok(b) => b,
            Err(AppError::NotFound(_)) => {
                return Err(AppError::Forbidden("僅指定的職務代理人可確認此請假".into()))
            }
            Err(e) => return Err(e),
        };

        if before.proxy_user_id != Some(user.id) {
            return Err(AppError::Forbidden("僅指定的職務代理人可確認此請假".into()));
        }
        if before.status != LeaveStatus::PendingProxy.as_str() {
            return Err(AppError::BusinessRule(
                "僅待代理確認狀態的請假可由代理人確認".into(),
            ));
        }

        // 卡關自動跳關：申請人部門無合法單位主管時，直接進「待負責人簽核」關。
        let next_status = if Self::l1_has_eligible_approver(pool, before.user_id).await? {
            LeaveStatus::PendingL1.as_str()
        } else {
            LeaveStatus::PendingDirector.as_str()
        };

        // 代理確認記入審核歷程（action 沿用 APPROVE，approval_level 標記 PENDING_PROXY 以資區分）。
        Self::insert_leave_approval(
            &mut tx,
            id,
            user.id,
            LeaveStatus::PendingProxy.as_str(),
            "APPROVE",
            None,
        )
        .await?;

        let after = sqlx::query_as::<_, LeaveRequest>(
            r#"
            UPDATE leave_requests
            SET status = $2::leave_status, current_approver_id = NULL, updated_at = NOW()
            WHERE id = $1 AND status = 'PENDING_PROXY'::leave_status
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
        .bind(next_status)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| {
            AppError::Conflict("此請假狀態已被其他操作變更，請重新整理後再試".to_string())
        })?;

        // 代理人已做出動作（確認），置頂待辦解除。放在 log_activity_tx 之前
        // ——與 audit HMAC chain 的 global advisory lock 排序一致（該鎖持有到 tx 結束，
        // notifications 的寫入要排在它取得之前，見 crud.rs 的說明）。
        crate::services::NotificationService::resolve_pinned_notifications_tx(
            &mut tx,
            "leave_request",
            after.id,
        )
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
                event_type: "LEAVE_PROXY_CONFIRM",
                entity: Some(AuditEntity::new("leave_request", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(after)
    }

    /// 代理人退回：PENDING_PROXY → DRAFT（供申請人重新指定代理人）。保留原 proxy_user_id 與歷程。
    pub async fn proxy_reject_leave(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        reason: Option<&str>,
    ) -> Result<LeaveRequest> {
        let user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        // existence oracle 防護：不存在時亦回 Forbidden（與非代理人一致）。
        let before = match Self::lock_leave_for_update(&mut tx, id).await {
            Ok(b) => b,
            Err(AppError::NotFound(_)) => {
                return Err(AppError::Forbidden("僅指定的職務代理人可退回此請假".into()))
            }
            Err(e) => return Err(e),
        };

        if before.proxy_user_id != Some(user.id) {
            return Err(AppError::Forbidden("僅指定的職務代理人可退回此請假".into()));
        }
        if before.status != LeaveStatus::PendingProxy.as_str() {
            return Err(AppError::BusinessRule(
                "僅待代理確認狀態的請假可由代理人退回".into(),
            ));
        }

        Self::insert_leave_approval(
            &mut tx,
            id,
            user.id,
            LeaveStatus::PendingProxy.as_str(),
            "REJECT",
            reason,
        )
        .await?;

        // 退回草稿：清 current_approver_id 與 submitted_at，保留 proxy_user_id 供申請人參考。
        let after = sqlx::query_as::<_, LeaveRequest>(
            r#"
            UPDATE leave_requests
            SET status = 'DRAFT'::leave_status, current_approver_id = NULL,
                submitted_at = NULL, updated_at = NOW()
            WHERE id = $1 AND status = 'PENDING_PROXY'::leave_status
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
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| {
            AppError::Conflict("此請假狀態已被其他操作變更，請重新整理後再試".to_string())
        })?;

        // 代理人已做出動作（退回），置頂待辦解除——退回同樣是「代理人不再需要做這件事」，
        // 不是只有確認才算數。申請人重新指定代理人並再次送審時會建立一則新的待辦。
        crate::services::NotificationService::resolve_pinned_notifications_tx(
            &mut tx,
            "leave_request",
            after.id,
        )
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
                event_type: "LEAVE_PROXY_REJECT",
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
        actor: &ActorContext,
        id: Uuid,
        comments: Option<&str>,
    ) -> Result<LeaveRequest> {
        let user = actor.require_user()?;
        let approver_id = user.id;
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
        .fetch_one(&mut *tx)
        .await?;

        // 授權判定（中央邏輯，含「不可批自己」「職責分離」「卡關 admin 代批」）。
        if !Self::can_user_approve_leave(pool, before.id, &before.status, before.user_id, user)
            .await?
        {
            return Err(AppError::Forbidden(
                "您無權審核此請假，或已審核過本單前一關".into(),
            ));
        }

        // 是否為 admin 代批（非該關指定審核人）→ 稽核事件區分
        let is_director = user
            .roles
            .iter()
            .any(|r| r == crate::constants::ROLE_DIRECTOR);
        let is_dept_manager = Self::is_dept_manager_of(pool, before.user_id, approver_id).await?;
        let is_override = user.is_admin()
            && !Self::is_designated_for(&before.status, is_dept_manager, is_director);

        // 兩關流程：待單位主管(L1) → 待負責人(DIRECTOR，終審) → 已核准
        let next_status = match before.status.as_str() {
            s if s == LeaveStatus::PendingL1.as_str() => LeaveStatus::PendingDirector.as_str(),
            s if s == LeaveStatus::PendingDirector.as_str() => LeaveStatus::Approved.as_str(),
            _ => return Err(AppError::Validation("無法核准此狀態的請假".to_string())),
        };

        // 最終核准時，檢查並扣除假別餘額（同一 tx 內，餘額異動與狀態變更原子化）
        let is_final_approval = next_status == LeaveStatus::Approved.as_str();
        if is_final_approval {
            Self::deduct_leave_balance(&mut tx, &before).await?;
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
        .bind(&before.status)
        .bind(comments)
        .execute(&mut *tx)
        .await?;

        let approved_at = if is_final_approval {
            Some(Utc::now())
        } else {
            None
        };

        // SEC-BIZ-5: 使用 WHERE status 條件防止 race condition（TOCTOU）
        // 若另一個請求已先修改狀態，此 UPDATE 不會匹配任何行 → 回傳衝突錯誤
        let after_opt = sqlx::query_as::<_, LeaveRequest>(
            r#"
            UPDATE leave_requests
            SET status = $2::leave_status, approved_at = $3, current_approver_id = NULL, updated_at = NOW()
            WHERE id = $1 AND status = $4::leave_status
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
        .bind(next_status)
        .bind(approved_at)
        .bind(&before.status)
        .fetch_optional(&mut *tx)
        .await?;

        let after = after_opt.ok_or_else(|| {
            AppError::Conflict("此請假狀態已被其他操作變更，請重新整理後再試".to_string())
        })?;

        // 這一關的審核人已做出動作（核准），該關的置頂待辦解除——無論是中途核准
        // （單位主管過關，換負責人接手）還是最終核准，這一步都要做。
        crate::services::NotificationService::resolve_pinned_notifications_tx(
            &mut tx,
            "leave_request",
            after.id,
        )
        .await?;

        // 中途核准（單位主管 → 負責人）：下一關待辦的建立放進同一個 tx，與上面的
        // 解除原子化。CodeRabbit PR #67 review：改前是 commit 後 tokio::spawn
        // best-effort 建立，若失敗（暫時性 DB 錯誤、或程序在 commit 與任務執行之間
        // 關閉），單位主管的待辦已解除、負責人的待辦卻從未建立——假單靜默卡在終審關，
        // 沒有任何人的「待處理」清單會顯示它，也沒有錯誤訊息。
        if !is_final_approval {
            let next_approvers = Self::current_stage_approvers_tx(&mut tx, after.id).await?;
            let applicant_name = sqlx::query_scalar::<_, String>(
                "SELECT display_name FROM users WHERE id = $1 AND is_active = true AND deleted_at IS NULL",
            )
            .bind(after.user_id)
            .fetch_optional(&mut *tx)
            .await?
            .unwrap_or_else(|| "申請人".to_string());
            let title = format!("[iPig] 新請假申請 - {}", applicant_name);
            let content = format!(
                "有新的請假申請待審核。\n\n申請人：{}\n假別：{}\n期間：{} ~ {}",
                applicant_name, after.leave_type, after.start_date, after.end_date
            );
            for uid in next_approvers {
                NotificationService::create_pinned_notification_tx_with_role(
                    &mut tx,
                    CreateNotificationRequest {
                        user_id: uid,
                        notification_type: NotificationType::LeaveApproval,
                        title: title.clone(),
                        content: Some(content.clone()),
                        related_entity_type: Some("leave_request".to_string()),
                        related_entity_id: Some(after.id),
                    },
                    crate::models::RECIPIENT_ROLE_APPROVER,
                )
                .await?;
            }
        }

        // event_type 區分中途/最終核准 + admin 代批（override），方便稽核查詢
        let event_type = match (is_final_approval, is_override) {
            (true, true) => "LEAVE_APPROVE_FINAL_OVERRIDE",
            (true, false) => "LEAVE_APPROVE_FINAL",
            (false, true) => "LEAVE_APPROVE_INTERIM_OVERRIDE",
            (false, false) => "LEAVE_APPROVE_INTERIM",
        };
        let display = format!(
            "{} {}~{}",
            after.leave_type, after.start_date, after.end_date
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "HR",
                event_type,
                entity: Some(AuditEntity::new("leave_request", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(after)
    }

    pub async fn reject_leave(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        reason: &str,
    ) -> Result<LeaveRequest> {
        let user = actor.require_user()?;
        let rejecter_id = user.id;
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
        .fetch_one(&mut *tx)
        .await?;

        // SEC-BIZ: 只有待審核（PENDING*）的請假可被駁回。
        // 缺少此守衛時，已核准的請假可被翻成 REJECTED 但餘額不會回補（reject 不走 restore_leave_balance），
        // 造成員工特休/補休默默損失，且終態（CANCELLED/REVOKED）也會被竄改。
        if !before.status.starts_with("PENDING") {
            return Err(AppError::Conflict(format!(
                "只有待審核的請假可駁回（目前狀態：{}）",
                before.status
            )));
        }

        // 授權判定：駁回與核准同一資格（該關合法審核人，含 SoD 與 admin 卡關代批）。
        if !Self::can_user_approve_leave(pool, before.id, &before.status, before.user_id, user)
            .await?
        {
            return Err(AppError::Forbidden(
                "您無權駁回此請假，或已審核過本單前一關".into(),
            ));
        }

        sqlx::query(
            r#"
            INSERT INTO leave_approvals (id, leave_request_id, approver_id, approval_level, action, comments)
            VALUES ($1, $2, $3, $4, 'REJECT', $5)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(rejecter_id)
        .bind(&before.status)
        .bind(reason)
        .execute(&mut *tx)
        .await?;

        let after = sqlx::query_as::<_, LeaveRequest>(
            r#"
            UPDATE leave_requests
            SET status = 'REJECTED'::leave_status, rejected_at = NOW(), updated_at = NOW()
            WHERE id = $1
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

        // 駁回是終態，這一關（也是本單唯一還可能存在的一則）置頂待辦解除，
        // 不再建立新的——沒有下一關了。
        crate::services::NotificationService::resolve_pinned_notifications_tx(
            &mut tx,
            "leave_request",
            after.id,
        )
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
                event_type: "LEAVE_REJECT",
                entity: Some(AuditEntity::new("leave_request", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(after)
    }

    /// 計算有效請假時數（total_hours 優先，否則換算天數 × 8）
    #[cfg(test)]
    pub(super) fn effective_hours(total_hours: Option<f64>, total_days: f64) -> f64 {
        total_hours.unwrap_or(total_days * 8.0)
    }

    pub async fn cancel_leave(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        reason: Option<&str>,
    ) -> Result<CancelLeaveOutcome> {
        let current = actor.require_user()?;
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
        .ok_or_else(|| AppError::NotFound("找不到請假紀錄".into()))?;

        // SEC-IDOR: 只有本人或具 hr.leave.manage 權限者可取消（與 update/delete_leave 一致）
        if before.user_id != current.id && !current.has_permission("hr.leave.manage") {
            return Err(AppError::Forbidden("無權取消他人請假".into()));
        }

        let was_approved = before.status == LeaveStatus::Approved.as_str();

        let after = sqlx::query_as::<_, LeaveRequest>(
            r#"
            UPDATE leave_requests
            SET status = 'CANCELLED'::leave_status, current_approver_id = NULL,
                cancelled_at = NOW(), cancellation_reason = $2, updated_at = NOW()
            WHERE id = $1 AND status IN ('DRAFT'::leave_status, 'PENDING_PROXY'::leave_status, 'PENDING_L1'::leave_status, 'PENDING_L2'::leave_status, 'PENDING_HR'::leave_status, 'PENDING_GM'::leave_status, 'PENDING_DIRECTOR'::leave_status, 'APPROVED'::leave_status)
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
        .bind(reason)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| {
            AppError::BusinessRule(
                "請假狀態不允許取消（可能已經是已取消 / 駁回 / 撤銷）".to_string(),
            )
        })?;

        // 已核准的請假取消時，回復餘額（同一 tx 內，原子化）
        if was_approved {
            Self::restore_leave_balance(&mut tx, &before).await?;
        }

        // 取消可以發生在 PENDING_PROXY 關卡（代理人還沒確認就被申請人取消）——
        // 這種情況下代理人的「待您確認」置頂待辦要跟著解除，否則代理人會對著一件
        // 已經不存在的請假案永遠卡著一則待辦、且無法自行清除。
        // 其餘狀態下呼叫本函式是 no-op（沒有置頂列可解）。
        crate::services::NotificationService::resolve_pinned_notifications_tx(
            &mut tx,
            "leave_request",
            after.id,
        )
        .await?;

        // 已核准狀態取消要特別標記（可能牽涉薪資/考勤結算）
        let event_type = if was_approved {
            "LEAVE_CANCEL_RETROACTIVE"
        } else {
            "LEAVE_CANCEL"
        };
        let display = format!(
            "{} {}~{}",
            after.leave_type, after.start_date, after.end_date
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "HR",
                event_type,
                entity: Some(AuditEntity::new("leave_request", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        // 通知須以「請假當事人」為主體（主管代取消時 ≠ 取消者）。
        let applicant_name =
            crate::repositories::user::find_user_display_name_by_id(pool, after.user_id)
                .await?
                .unwrap_or_else(|| "申請人".to_string());

        Ok(CancelLeaveOutcome {
            record: after,
            was_approved,
            applicant_name,
        })
    }

    // ============================================
    // Balance deduction / restoration helpers
    // ============================================

    /// 扣除特休假餘額（FIFO：先到期先扣）並記錄 leave_balance_usage
    async fn deduct_annual_leave(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        leave: &LeaveRequest,
    ) -> Result<()> {
        let mut remaining = leave.total_days;
        let entitlements: Vec<(Uuid, Decimal)> = sqlx::query_as(
            r#"
            SELECT id, (entitled_days - used_days) as available
            FROM annual_leave_entitlements
            WHERE user_id = $1 AND NOT is_expired AND (entitled_days - used_days) > 0
            ORDER BY expires_at ASC
            FOR UPDATE
            "#,
        )
        .bind(leave.user_id)
        .fetch_all(&mut **tx)
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
            Self::apply_annual_deduction(tx, leave.id, ent_id, deduct).await?;
            remaining -= deduct;
        }
        Ok(())
    }

    /// 執行單筆特休假扣除（UPDATE + INSERT usage）
    async fn apply_annual_deduction(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        leave_id: Uuid,
        entitlement_id: Uuid,
        days: Decimal,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE annual_leave_entitlements SET used_days = used_days + $2, updated_at = NOW() WHERE id = $1",
        )
        .bind(entitlement_id)
        .bind(days)
        .execute(&mut **tx)
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
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// 扣除補休餘額（FIFO：先到期先扣）並記錄 leave_balance_usage
    async fn deduct_comp_time(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        leave: &LeaveRequest,
    ) -> Result<()> {
        let hours_dec = leave
            .total_hours
            .unwrap_or_else(|| leave.total_days * Decimal::from(8));

        let balances: Vec<(Uuid, Decimal)> = sqlx::query_as(
            r#"
            SELECT id, (original_hours - used_hours) as available
            FROM comp_time_balances
            WHERE user_id = $1 AND NOT is_expired AND (original_hours - used_hours) > 0
            ORDER BY expires_at ASC
            FOR UPDATE
            "#,
        )
        .bind(leave.user_id)
        .fetch_all(&mut **tx)
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
            Self::apply_comp_deduction(tx, leave.id, bal_id, deduct).await?;
            remaining -= deduct;
        }
        Ok(())
    }

    /// 執行單筆補休扣除（UPDATE + INSERT usage）
    async fn apply_comp_deduction(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        leave_id: Uuid,
        balance_id: Uuid,
        hours: Decimal,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE comp_time_balances SET used_hours = used_hours + $2, updated_at = NOW() WHERE id = $1",
        )
        .bind(balance_id)
        .bind(hours)
        .execute(&mut **tx)
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
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// 核准時依假別檢查並扣除餘額（僅 ANNUAL / COMPENSATORY 需要）
    async fn deduct_leave_balance(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        leave: &LeaveRequest,
    ) -> Result<()> {
        match leave.leave_type.as_str() {
            "ANNUAL" => Self::deduct_annual_leave(tx, leave).await,
            "COMPENSATORY" => Self::deduct_comp_time(tx, leave).await,
            _ => Ok(()), // 其他假別無額度限制
        }
    }

    /// 取消/銷假時依假別回復餘額（僅 ANNUAL / COMPENSATORY 需要）
    async fn restore_leave_balance(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        leave: &LeaveRequest,
    ) -> Result<()> {
        match leave.leave_type.as_str() {
            "ANNUAL" => Self::restore_annual_leave(tx, leave).await,
            "COMPENSATORY" => Self::restore_comp_time(tx, leave).await,
            _ => Ok(()),
        }
    }

    /// 回復特休假餘額：依 leave_balance_usage 的 deduct 紀錄逐筆還原
    async fn restore_annual_leave(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        leave: &LeaveRequest,
    ) -> Result<()> {
        let usages: Vec<(Uuid, Uuid, Decimal)> = sqlx::query_as(
            r#"
            SELECT id, annual_leave_entitlement_id, days_used
            FROM leave_balance_usage
            WHERE leave_request_id = $1 AND source_type = 'annual' AND action = 'deduct'
            "#,
        )
        .bind(leave.id)
        .fetch_all(&mut **tx)
        .await?;

        for (usage_id, ent_id, days) in usages {
            sqlx::query(
                "UPDATE annual_leave_entitlements SET used_days = used_days - $2, updated_at = NOW() WHERE id = $1",
            )
            .bind(ent_id)
            .bind(days)
            .execute(&mut **tx)
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
            .execute(&mut **tx)
            .await?;

            let _ = usage_id; // 僅用於未來稽核需求
        }
        Ok(())
    }

    /// 回復補休餘額：依 leave_balance_usage 的 deduct 紀錄逐筆還原
    async fn restore_comp_time(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        leave: &LeaveRequest,
    ) -> Result<()> {
        let usages: Vec<(Uuid, Uuid, Decimal)> = sqlx::query_as(
            r#"
            SELECT id, comp_time_balance_id, hours_used
            FROM leave_balance_usage
            WHERE leave_request_id = $1 AND source_type = 'comp_time' AND action = 'deduct'
            "#,
        )
        .bind(leave.id)
        .fetch_all(&mut **tx)
        .await?;

        for (usage_id, bal_id, hours) in usages {
            sqlx::query(
                "UPDATE comp_time_balances SET used_hours = used_hours - $2, updated_at = NOW() WHERE id = $1",
            )
            .bind(bal_id)
            .bind(hours)
            .execute(&mut **tx)
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
            .execute(&mut **tx)
            .await?;

            let _ = usage_id;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::HrService;

    // --- is_designated_for（兩關指定審核人判定） ---

    #[test]
    fn test_designated_l1_is_dept_manager() {
        // 待部門主管關：只有部門主管是指定審核人
        assert!(HrService::is_designated_for("PENDING_L1", true, false));
        assert!(!HrService::is_designated_for("PENDING_L1", false, true));
        assert!(!HrService::is_designated_for("PENDING_L1", false, false));
    }

    #[test]
    fn test_designated_director_is_director_role() {
        // 待負責人關：只有 DIRECTOR 是指定審核人（單位主管身分不算）
        assert!(HrService::is_designated_for(
            "PENDING_DIRECTOR",
            false,
            true
        ));
        assert!(!HrService::is_designated_for(
            "PENDING_DIRECTOR",
            true,
            false
        ));
        assert!(!HrService::is_designated_for(
            "PENDING_DIRECTOR",
            false,
            false
        ));
    }

    #[test]
    fn test_designated_other_status_false() {
        // 非審核關卡（草稿/已核准/已移除的 GM 關）無指定審核人
        assert!(!HrService::is_designated_for("DRAFT", true, true));
        assert!(!HrService::is_designated_for("PENDING_GM", true, true));
        assert!(!HrService::is_designated_for("APPROVED", true, true));
    }

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
        assert!(!HrService::is_half_hour_multiple(0.0)); // 小於 0.5
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
