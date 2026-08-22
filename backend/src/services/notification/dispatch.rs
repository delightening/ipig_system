//! 員工通知 email 統一 chokepoint。
//!
//! 所有「寄給內部員工的通知 email」（in-scope，見計畫 / PROGRESS）一律經此分派：
//! 1. 收件人**請假中**（已核准、涵蓋台灣今日的假單）→ 跳過 email（`SkippedOnLeave`）。
//! 2. 否則依寄送時間窗（台灣週一至週五 09:00–17:00、排除國定假日）決定寄送時間：
//!    窗內 → 立即；窗外 → 排到下一個合法窗口（`Deferred`）。
//! 3. 一律入 outbox（worker 經 `EmailAdapter` 寄出），取得 durable + retry。
//!
//! 站內 in-app 通知不經此處（由各 notify_* 方法即時建立，不受時間窗 / 請假影響）。

use std::collections::HashSet;

use chrono::{Days, NaiveDate, Utc};
use uuid::Uuid;

use crate::middleware::ActorContext;
use crate::services::holiday;
use crate::services::outbox::OutboxService;
use crate::services::RenderedEmail;
use crate::{repositories, time, Result};

use super::send_window::{is_within_staff_window, next_window_open};
use super::NotificationService;

/// 計算下一個窗口時往後查假日的天數上限（對齊 send_window 的有界掃描）。
const HOLIDAY_LOOKAHEAD_DAYS: u64 = 14;

/// 一封待分派的員工通知 email。
pub struct StaffEmail {
    pub to_email: String,
    pub to_name: String,
    /// 收件人 user id；`Some` 時會做請假中判斷，`None`（如 IACUC 字串收件人）只套時間窗。
    pub recipient_user_id: Option<Uuid>,
    pub email: RenderedEmail,
}

/// 分派結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchOutcome {
    /// 在窗內，已排入 outbox 立即寄送。
    SentNow,
    /// 窗外，已排入 outbox 延後到下一個合法窗口。
    Deferred,
    /// 收件人請假中，未寄。
    SkippedOnLeave,
}

impl NotificationService {
    /// 分派一封員工通知 email（請假抑制 + 時間窗延後 + 入 outbox）。
    pub(crate) async fn dispatch_staff_email(
        &self,
        actor: &ActorContext,
        source: (&str, Uuid),
        msg: StaffEmail,
    ) -> Result<DispatchOutcome> {
        // 1. 先依時間窗決定「實際寄送日」（窗內＝現在；窗外＝下一個合法窗口）。
        let now = time::now_taiwan();
        let holidays = collect_lookahead_holidays(now.date_naive());
        let is_holiday = |d: NaiveDate| holidays.contains(&d);

        let (scheduled_taiwan, outcome) = if is_within_staff_window(now, &is_holiday) {
            (now, DispatchOutcome::SentNow)
        } else {
            let open = next_window_open(now, &is_holiday);
            tracing::info!(
                to = %msg.to_email,
                next_window = %open,
                "員工通知 email 在寄送時間窗外，延後到下一個窗口"
            );
            (open, DispatchOutcome::Deferred)
        };

        // 2. 請假判斷對齊「實際寄送日」：延後到下週一寄的信，若收件人那天才開始休假也應跳過
        //    （站內通知由呼叫端另行建立，不受此影響）。
        if let Some(uid) = msg.recipient_user_id {
            let send_date = scheduled_taiwan.date_naive();
            if repositories::hr::exists_approved_leave_covering_date(&self.db, uid, send_date)
                .await?
            {
                tracing::info!(user_id = %uid, to = %msg.to_email, "收件人寄送日請假中，跳過通知 email");
                return Ok(DispatchOutcome::SkippedOnLeave);
            }
        }

        // 3. 入 outbox（EmailAdapter payload schema: to / to_name / subject / plain_body / html_body）。
        let send_at = if outcome == DispatchOutcome::SentNow {
            Utc::now()
        } else {
            scheduled_taiwan.with_timezone(&Utc)
        };
        let payload = serde_json::json!({
            "to": msg.to_email,
            "to_name": msg.to_name,
            "subject": msg.email.subject,
            "plain_body": msg.email.plain_body,
            "html_body": msg.email.html_body,
        });
        OutboxService::enqueue(&self.db, actor, "email", payload, source, Some(send_at)).await?;
        Ok(outcome)
    }
}

/// 蒐集「今天起 14 天內」已暖機的國定假日（供下一個窗口掃描）；無全域服務時回空集合（fallback 僅週末）。
/// 純讀快取、不發 HTTP（見 HolidayService::collect_holidays）。
fn collect_lookahead_holidays(today: NaiveDate) -> HashSet<NaiveDate> {
    match holiday::global() {
        Some(svc) => {
            let end = today
                .checked_add_days(Days::new(HOLIDAY_LOOKAHEAD_DAYS))
                .unwrap_or(today);
            svc.collect_holidays(today, end)
        }
        None => HashSet::new(),
    }
}
