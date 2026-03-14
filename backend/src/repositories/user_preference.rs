use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user_preferences::UserPreference;
use crate::Result;

/// 依 user_id 和 key 查詢單筆偏好
pub async fn find_preference_by_user_and_key(
    pool: &PgPool,
    user_id: Uuid,
    key: &str,
) -> Result<Option<UserPreference>> {
    let preference = sqlx::query_as::<_, UserPreference>(
        r#"
        SELECT id, user_id, preference_key, preference_value, created_at, updated_at
        FROM user_preferences
        WHERE user_id = $1 AND preference_key = $2
        "#,
    )
    .bind(user_id)
    .bind(key)
    .fetch_optional(pool)
    .await?;

    Ok(preference)
}

/// Upsert 偏好設定（新增或更新）
pub async fn upsert_preference(
    pool: &PgPool,
    user_id: Uuid,
    key: &str,
    value: &serde_json::Value,
) -> Result<UserPreference> {
    let preference = sqlx::query_as::<_, UserPreference>(
        r#"
        INSERT INTO user_preferences (user_id, preference_key, preference_value)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, preference_key)
        DO UPDATE SET preference_value = EXCLUDED.preference_value, updated_at = CURRENT_TIMESTAMP
        RETURNING id, user_id, preference_key, preference_value, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(key)
    .bind(value)
    .fetch_one(pool)
    .await?;

    Ok(preference)
}

/// 查詢使用者所有偏好設定
pub async fn list_preferences_by_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<UserPreference>> {
    let preferences = sqlx::query_as::<_, UserPreference>(
        r#"
        SELECT id, user_id, preference_key, preference_value, created_at, updated_at
        FROM user_preferences
        WHERE user_id = $1
        ORDER BY preference_key
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(preferences)
}

/// 刪除使用者特定偏好
pub async fn delete_preference_by_user_and_key(
    pool: &PgPool,
    user_id: Uuid,
    key: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        DELETE FROM user_preferences
        WHERE user_id = $1 AND preference_key = $2
        "#,
    )
    .bind(user_id)
    .bind(key)
    .execute(pool)
    .await?;

    Ok(())
}
