// 請假行事曆（原生）
//
// 直接讀 `leave_requests`，不經 Google 同步佇列。兩者刻意分開：
// Google 同步只處理**已核准**的假單，且日曆授權是整本給的、無法逐筆過濾，
// 因此「顯示審核中」與「依觀看者決定可見範圍」這兩個需求它結構上做不到。

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use super::HrService;
use crate::{
    middleware::CurrentUser,
    models::{LeaveCalendarEntry, LeaveCalendarResponse, LeaveCalendarScope, LeaveStatus},
    Result,
};

/// 行事曆會顯示的狀態：已核准 + 所有「審核中」關卡。
///
/// 刻意不含 `DRAFT`（還沒送出，只有自己知道）、`REJECTED` / `CANCELLED` / `REVOKED`
/// （這幾天人會在，畫上去反而誤導排班）。
///
/// `PENDING_L2` / `PENDING_HR` / `PENDING_GM` 是**舊流程遺留**的 enum 值——現行
/// 簽核鏈是 `PENDING_PROXY → PENDING_L1 → PENDING_DIRECTOR`，沒有任何程式碼會把
/// 假單轉進那三個狀態，prod 也一筆都沒有。仍然列進來，是因為漏掉的後果是
/// **靜默**的：真有一筆舊資料落在那些狀態，它會從行事曆上消失而不報任何錯，
/// 排班的人不會發現有人那天其實不在。
///
/// 這也與既有慣例一致——`services/hr/leave.rs` 的代理人重疊檢查（`validate_proxy`）
/// 與狀態守衛都是把三個舊狀態一併列入。
const VISIBLE_STATUSES: [&str; 7] = [
    "APPROVED",
    "PENDING_PROXY",
    "PENDING_L1",
    "PENDING_L2",
    "PENDING_HR",
    "PENDING_GM",
    "PENDING_DIRECTOR",
];

/// 單次查詢的最大天數跨距。月視圖約 42 天、週視圖 7 天，
/// 一年份 366 天已遠超任何視圖所需；設限是避免有人手動改 query string 把整表撈走。
const MAX_RANGE_DAYS: i64 = 366;

struct CalendarRow {
    id: Uuid,
    user_id: Uuid,
    user_name: String,
    department_name: Option<String>,
    leave_type: String,
    status: String,
    start_date: NaiveDate,
    end_date: NaiveDate,
    start_time: Option<chrono::NaiveTime>,
    end_time: Option<chrono::NaiveTime>,
    total_days: rust_decimal::Decimal,
    total_hours: Option<rust_decimal::Decimal>,
    proxy_user_id: Option<Uuid>,
    proxy_user_name: Option<String>,
}

impl HrService {
    /// 觀看者是否為任一在職部門的主管。
    ///
    /// 「部門主管」是**資料關係**（`departments.manager_id`）而非角色——系統裡
    /// 沒有 DEPT_MANAGER 這個 role，所以看全場的資格只能這樣算，不能靠權限碼。
    async fn is_any_department_manager(pool: &PgPool, user_id: Uuid) -> Result<bool> {
        let exists: (bool,) = sqlx::query_as(
            r#"SELECT EXISTS(
                SELECT 1 FROM departments WHERE manager_id = $1 AND is_active = true
            )"#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;
        Ok(exists.0)
    }

    /// 請假行事曆資料。
    ///
    /// 可見範圍：
    /// - 管理員 / `hr.leave.view_all` / 任一部門主管 → 全場
    /// - 其餘 → 自己所屬部門 + 上層部門 + 下層部門
    ///
    /// 另外**不受部門限制**地一律看得到：自己請的假、以及自己被指派為代理人的假。
    /// 後者是刻意的——你要替人頂班，本來就該知道他哪天不在、請什麼假。
    pub async fn get_leave_calendar(
        pool: &PgPool,
        current_user: &CurrentUser,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<LeaveCalendarResponse> {
        if end < start {
            return Err(crate::AppError::BadRequest(
                "結束日期不得早於開始日期".into(),
            ));
        }
        if (end - start).num_days() > MAX_RANGE_DAYS {
            return Err(crate::AppError::BadRequest(format!(
                "查詢區間不得超過 {MAX_RANGE_DAYS} 天"
            )));
        }

        let viewer_id = current_user.id;
        let see_all = current_user.is_admin()
            || current_user.has_permission("hr.leave.view_all")
            || Self::is_any_department_manager(pool, viewer_id).await?;

        let rows: Vec<CalendarRow> = sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                String,
                Option<String>,
                String,
                String,
                NaiveDate,
                NaiveDate,
                Option<chrono::NaiveTime>,
                Option<chrono::NaiveTime>,
                rust_decimal::Decimal,
                Option<rust_decimal::Decimal>,
                Option<Uuid>,
                Option<String>,
            ),
        >(
            r#"
            WITH viewer AS (
                SELECT department_id FROM users WHERE id = $1
            ),
            visible_depts AS (
                -- 自己的部門 + 上一層 + 下一層。實習部門掛在試驗部下，
                -- 兩邊是同一個實際工作團隊，所以上下都要看得到。
                SELECT d.id
                FROM departments d, viewer v
                WHERE v.department_id IS NOT NULL
                  -- 與 is_any_department_manager 共用同一個「有效部門」定義。
                  -- 少了這個條件，已停用的部門仍會擴大可見範圍。
                  AND d.is_active = true
                  AND (
                    d.id = v.department_id
                    OR d.id = (SELECT parent_id FROM departments WHERE id = v.department_id)
                    OR d.parent_id = v.department_id
                  )
            )
            SELECT
                l.id,
                l.user_id,
                u.display_name,
                d.name,
                l.leave_type::text,
                l.status::text,
                l.start_date,
                l.end_date,
                l.start_time,
                l.end_time,
                l.total_days,
                l.total_hours,
                l.proxy_user_id,
                p.display_name
            FROM leave_requests l
            INNER JOIN users u ON u.id = l.user_id
            LEFT JOIN departments d ON d.id = u.department_id
            LEFT JOIN users p ON p.id = l.proxy_user_id
            WHERE l.status::text = ANY($2)
              AND l.start_date <= $4
              AND l.end_date >= $3
              AND (
                $5
                OR l.user_id = $1
                OR l.proxy_user_id = $1
                OR u.department_id IN (SELECT id FROM visible_depts)
              )
            ORDER BY l.start_date, u.display_name
            "#,
        )
        .bind(viewer_id)
        .bind(VISIBLE_STATUSES.as_slice())
        .bind(start)
        .bind(end)
        .bind(see_all)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(
            |(
                id,
                user_id,
                user_name,
                department_name,
                leave_type,
                status,
                start_date,
                end_date,
                start_time,
                end_time,
                total_days,
                total_hours,
                proxy_user_id,
                proxy_user_name,
            )| CalendarRow {
                id,
                user_id,
                user_name,
                department_name,
                leave_type,
                status,
                start_date,
                end_date,
                start_time,
                end_time,
                total_days,
                total_hours,
                proxy_user_id,
                proxy_user_name,
            },
        )
        .collect();

        let entries = rows
            .into_iter()
            .map(|r| {
                let is_all_day = r.start_time.is_none() && r.end_time.is_none();
                LeaveCalendarEntry {
                    leave_type_display: crate::constants::get_leave_type_display(&r.leave_type)
                        .to_string(),
                    pending_stage_display: pending_stage_display(&r.status).map(str::to_string),
                    // 代理人「已確認」的判準是假單已離開 PENDING_PROXY 關。
                    // 沒有 proxy_confirmed_at 欄位，確認事實記在 leave_approvals；
                    // 而狀態要前進就必須經過代理人確認，故兩者等價且省一次 join。
                    proxy_confirmed: r.status != LeaveStatus::PendingProxy.as_str(),
                    is_mine: r.user_id == viewer_id,
                    is_my_proxy_duty: r.proxy_user_id == Some(viewer_id),
                    id: r.id,
                    user_id: r.user_id,
                    user_name: r.user_name,
                    department_name: r.department_name,
                    leave_type: r.leave_type,
                    status: r.status,
                    start_date: r.start_date,
                    end_date: r.end_date,
                    start_time: r.start_time,
                    end_time: r.end_time,
                    is_all_day,
                    is_partial_day: is_partial_day(r.total_days),
                    total_days: r.total_days,
                    total_hours: r.total_hours,
                    proxy_user_id: r.proxy_user_id,
                    proxy_user_name: r.proxy_user_name,
                }
            })
            .collect();

        Ok(LeaveCalendarResponse {
            entries,
            scope: if see_all {
                LeaveCalendarScope::All
            } else {
                LeaveCalendarScope::Department
            },
        })
    }
}

/// 是否為不足整日的假。
///
/// 判準用 `total_days` 而非 `start_time`/`end_time`：請假表單只收時數、不收時段，
/// 實測 prod 全部 19 張假單的起訖時間皆為 NULL，其中 3 張其實是半天假
/// （例如 `total_days = 0.56` / `total_hours = 4.50`）。若沿用起訖時間判斷，
/// 這 3 張會與整天假長得一模一樣。
fn is_partial_day(total_days: rust_decimal::Decimal) -> bool {
    total_days.fract() != rust_decimal::Decimal::ZERO
}

/// 審核中的假單目前卡在哪一關；已核准回 `None`。
///
/// 三個舊流程遺留狀態（L2 / HR / GM）一併對應——理由見 `VISIBLE_STATUSES`：
/// 它們不會再被寫入，但若真有舊資料，標不出關卡的半透明色塊比漏掉更難察覺。
fn pending_stage_display(status: &str) -> Option<&'static str> {
    match status {
        s if s == LeaveStatus::PendingProxy.as_str() => {
            Some(LeaveStatus::PendingProxy.display_name())
        }
        s if s == LeaveStatus::PendingL1.as_str() => Some(LeaveStatus::PendingL1.display_name()),
        s if s == LeaveStatus::PendingL2.as_str() => Some(LeaveStatus::PendingL2.display_name()),
        s if s == LeaveStatus::PendingHr.as_str() => Some(LeaveStatus::PendingHr.display_name()),
        s if s == LeaveStatus::PendingGm.as_str() => Some(LeaveStatus::PendingGm.display_name()),
        s if s == LeaveStatus::PendingDirector.as_str() => {
            Some(LeaveStatus::PendingDirector.display_name())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_stage_only_for_in_flight_statuses() {
        assert_eq!(pending_stage_display("PENDING_PROXY"), Some("待代理確認"));
        assert_eq!(pending_stage_display("PENDING_L1"), Some("待單位主管審核"));
        assert_eq!(
            pending_stage_display("PENDING_DIRECTOR"),
            Some("待負責人簽核")
        );
        assert_eq!(pending_stage_display("APPROVED"), None);
    }

    /// 半天假的判準是 `total_days` 有小數，不是「有沒有起訖時間」。
    ///
    /// 用 prod 實測值當案例：`0.56` 天（4.5 小時）必須被判為半天假。
    /// 若哪天有人把判準改回看 `start_time`，這個測試會紅——而 UI 上的症狀
    /// 只是「半天假看起來像整天假」，不會有任何錯誤訊息。
    #[test]
    fn partial_day_is_derived_from_total_days_not_times() {
        use rust_decimal::Decimal;
        use std::str::FromStr;

        let case = |s: &str| is_partial_day(Decimal::from_str(s).expect("parse decimal"));
        assert!(case("0.56"), "prod 實際存在的 4.5 小時假應判為半天");
        assert!(case("0.50"), "半天");
        assert!(case("1.50"), "一天半也含半天成分");
        assert!(!case("1.00"), "整天不是半天假");
        assert!(!case("3.00"), "連續整天不是半天假");
    }

    /// 行事曆的狀態清單必須與「卡在哪一關」的對應表同步：
    /// 每個非 APPROVED 的可見狀態都要能翻出關卡名稱，否則前端會拿到
    /// 一個標不出關卡的半透明色塊，使用者看不出它在等誰。
    #[test]
    fn every_visible_pending_status_has_a_stage_label() {
        for status in VISIBLE_STATUSES {
            if status == LeaveStatus::Approved.as_str() {
                continue;
            }
            assert!(
                pending_stage_display(status).is_some(),
                "可見狀態 {status} 缺少關卡名稱"
            );
        }
    }
}
