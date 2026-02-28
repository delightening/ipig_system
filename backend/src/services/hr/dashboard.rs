// HR 儀表板行事曆

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::constants::TAIWAN_OFFSET_SECS;
use crate::{
    models::{DashboardCalendarData, TodayLeaveInfo},
    Result,
};

use super::HrService;

impl HrService {
    pub async fn get_dashboard_calendar(pool: &PgPool) -> Result<DashboardCalendarData> {
        let taiwan_tz = chrono::FixedOffset::east_opt(TAIWAN_OFFSET_SECS)
            .ok_or_else(|| crate::AppError::Internal("invalid timezone offset UTC+8".to_string()))?;
        let today = Utc::now().with_timezone(&taiwan_tz).date_naive();
        let upcoming_end = today + chrono::Duration::days(7);

        // 取得今日請假中的人
        #[allow(clippy::type_complexity)]
        let today_leaves_rows: Vec<(Uuid, String, String, chrono::NaiveDate, chrono::NaiveDate, Option<chrono::NaiveTime>, Option<chrono::NaiveTime>)> = sqlx::query_as(
            r#"
            SELECT l.user_id, u.display_name, l.leave_type::text, l.start_date, l.end_date, l.start_time, l.end_time
            FROM leave_requests l INNER JOIN users u ON l.user_id = u.id
            WHERE l.status::text = 'APPROVED' AND l.start_date <= $1 AND l.end_date >= $1
            ORDER BY u.display_name
            "#,
        ).bind(today).fetch_all(pool).await?;

        let today_leaves = Self::map_leave_info(today_leaves_rows);

        // 取得近期請假
        #[allow(clippy::type_complexity)]
        let upcoming_leaves_rows: Vec<(Uuid, String, String, chrono::NaiveDate, chrono::NaiveDate, Option<chrono::NaiveTime>, Option<chrono::NaiveTime>)> = sqlx::query_as(
            r#"
            SELECT l.user_id, u.display_name, l.leave_type::text, l.start_date, l.end_date, l.start_time, l.end_time
            FROM leave_requests l INNER JOIN users u ON l.user_id = u.id
            WHERE l.status::text = 'APPROVED' AND l.start_date > $1 AND l.start_date <= $2
            ORDER BY l.start_date, u.display_name LIMIT 10
            "#,
        ).bind(today).bind(upcoming_end).fetch_all(pool).await?;

        let upcoming_leaves = Self::map_leave_info(upcoming_leaves_rows);

        // Google Calendar 事件
        let today_events = match crate::services::CalendarService::get_config(pool).await {
            Ok(config) if config.is_configured => {
                let client = crate::services::google_calendar::GoogleCalendarClient::new(
                    &config.calendar_id,
                );
                client.fetch_events(today, today).await.unwrap_or_default()
            }
            _ => vec![],
        };

        Ok(DashboardCalendarData {
            today,
            today_leaves,
            today_events,
            upcoming_leaves,
        })
    }

    #[allow(clippy::type_complexity)]
    fn map_leave_info(
        rows: Vec<(
            Uuid,
            String,
            String,
            chrono::NaiveDate,
            chrono::NaiveDate,
            Option<chrono::NaiveTime>,
            Option<chrono::NaiveTime>,
        )>,
    ) -> Vec<TodayLeaveInfo> {
        rows.into_iter()
            .map(
                |(user_id, user_name, leave_type, start_date, end_date, start_time, end_time)| {
                    let leave_type_display = match leave_type.as_str() {
                        "ANNUAL" => "特休假",
                        "PERSONAL" => "事假",
                        "SICK" => "病假",
                        "COMPENSATORY" => "補休假",
                        "MARRIAGE" => "婚假",
                        "BEREAVEMENT" => "喪假",
                        "MATERNITY" => "產假",
                        "PATERNITY" => "陪產假",
                        "MENSTRUAL" => "生理假",
                        "OFFICIAL" => "公假",
                        _ => "請假",
                    };
                    let is_all_day = start_time.is_none() && end_time.is_none();
                    TodayLeaveInfo {
                        user_id,
                        user_name,
                        leave_type,
                        leave_type_display: leave_type_display.to_string(),
                        is_all_day,
                        start_date,
                        end_date,
                        start_time,
                        end_time,
                    }
                },
            )
            .collect()
    }
}
