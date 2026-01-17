use axum::{
    extract::{Path, State},
    Extension, Json,
};

use crate::error::AppError;
use crate::middleware::CurrentUser;
use crate::models::user_preferences::{
    AllPreferencesResponse, PreferenceResponse, UpsertPreferenceRequest, UserPreference,
    default_nav_order, default_dashboard_widgets,
};
use crate::AppState;

/// 取得單一偏好設定
/// GET /me/preferences/:key
pub async fn get_preference(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(key): Path<String>,
) -> Result<Json<PreferenceResponse>, AppError> {
    let preference = sqlx::query_as::<_, UserPreference>(
        r#"
        SELECT id, user_id, preference_key, preference_value, created_at, updated_at
        FROM user_preferences
        WHERE user_id = $1 AND preference_key = $2
        "#
    )
    .bind(current_user.id)
    .bind(&key)
    .fetch_optional(&state.db)
    .await?;

    match preference {
        Some(pref) => Ok(Json(PreferenceResponse::from(pref))),
        None => {
            // 回傳預設值
            let default_value = get_default_value(&key);
            Ok(Json(PreferenceResponse {
                key,
                value: default_value,
                updated_at: None,
            }))
        }
    }
}

/// 更新偏好設定 (upsert)
/// PUT /me/preferences/:key
pub async fn upsert_preference(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(key): Path<String>,
    Json(payload): Json<UpsertPreferenceRequest>,
) -> Result<Json<PreferenceResponse>, AppError> {
    let preference = sqlx::query_as::<_, UserPreference>(
        r#"
        INSERT INTO user_preferences (user_id, preference_key, preference_value)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, preference_key)
        DO UPDATE SET preference_value = EXCLUDED.preference_value, updated_at = CURRENT_TIMESTAMP
        RETURNING id, user_id, preference_key, preference_value, created_at, updated_at
        "#
    )
    .bind(current_user.id)
    .bind(&key)
    .bind(&payload.value)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(PreferenceResponse::from(preference)))
}

/// 取得所有偏好設定
/// GET /me/preferences
pub async fn get_all_preferences(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<AllPreferencesResponse>, AppError> {
    let preferences = sqlx::query_as::<_, UserPreference>(
        r#"
        SELECT id, user_id, preference_key, preference_value, created_at, updated_at
        FROM user_preferences
        WHERE user_id = $1
        ORDER BY preference_key
        "#
    )
    .bind(current_user.id)
    .fetch_all(&state.db)
    .await?;

    let response = AllPreferencesResponse {
        preferences: preferences.into_iter().map(PreferenceResponse::from).collect::<Vec<_>>(),
    };

    Ok(Json(response))
}

/// 刪除偏好設定（重置為預設）
/// DELETE /me/preferences/:key
pub async fn delete_preference(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(key): Path<String>,
) -> Result<Json<PreferenceResponse>, AppError> {
    sqlx::query(
        r#"
        DELETE FROM user_preferences
        WHERE user_id = $1 AND preference_key = $2
        "#
    )
    .bind(current_user.id)
    .bind(&key)
    .execute(&state.db)
    .await?;

    // 回傳預設值
    let default_value = get_default_value(&key);
    Ok(Json(PreferenceResponse {
        key,
        value: default_value,
        updated_at: None,
    }))
}

/// 取得特定 key 的預設值
fn get_default_value(key: &str) -> serde_json::Value {
    match key {
        "nav_order" => default_nav_order(),
        "dashboard_widgets" => default_dashboard_widgets(),
        _ => serde_json::Value::Null,
    }
}
