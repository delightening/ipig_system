use sqlx::PgPool;
use uuid::Uuid;

use crate::models::NotificationSettings;
use crate::Result;

/// 查詢排程報表的建立者 ID
pub async fn find_scheduled_report_owner(
    pool: &PgPool,
    report_id: Uuid,
) -> Result<Option<Uuid>> {
    let owner: Option<(Uuid,)> = sqlx::query_as(
        "SELECT created_by FROM scheduled_reports WHERE id = $1",
    )
    .bind(report_id)
    .fetch_optional(pool)
    .await?;

    Ok(owner.map(|(id,)| id))
}

/// 查詢使用者的通知設定
pub async fn find_notification_settings_by_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<NotificationSettings>> {
    let settings = sqlx::query_as::<_, NotificationSettings>(
        "SELECT * FROM notification_settings WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(settings)
}
