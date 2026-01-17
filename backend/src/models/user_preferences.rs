use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 使用者偏好設定
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserPreference {
    pub id: Uuid,
    pub user_id: Uuid,
    pub preference_key: String,
    pub preference_value: serde_json::Value,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// 建立或更新偏好設定的請求
#[derive(Debug, Deserialize)]
pub struct UpsertPreferenceRequest {
    pub value: serde_json::Value,
}

/// 偏好設定回應
#[derive(Debug, Serialize)]
pub struct PreferenceResponse {
    pub key: String,
    pub value: serde_json::Value,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<UserPreference> for PreferenceResponse {
    fn from(pref: UserPreference) -> Self {
        Self {
            key: pref.preference_key,
            value: pref.preference_value,
            updated_at: pref.updated_at,
        }
    }
}

/// 所有偏好設定回應
#[derive(Debug, Serialize)]
pub struct AllPreferencesResponse {
    pub preferences: Vec<PreferenceResponse>,
}

/// 預設的側邊欄順序
pub fn default_nav_order() -> serde_json::Value {
    serde_json::json!([
        "儀表板",
        "我的計劃",
        "AUP 審查系統",
        "人員管理",
        "實驗動物管理",
        "採購管理",
        "銷售管理",
        "倉儲作業",
        "報表中心",
        "基礎資料",
        "系統管理"
    ])
}

/// 預設的儀表板 Widget 設定
pub fn default_dashboard_widgets() -> serde_json::Value {
    serde_json::json!([
        { "id": "leave_balance", "visible": true, "order": 0 },
        { "id": "my_projects", "visible": true, "order": 1 },
        { "id": "animals_on_medication", "visible": true, "order": 2 },
        { "id": "vet_comments", "visible": true, "order": 3 },
        { "id": "low_stock_alert", "visible": true, "order": 4 },
        { "id": "pending_documents", "visible": true, "order": 5 },
        { "id": "today_inbound", "visible": true, "order": 6 },
        { "id": "today_outbound", "visible": true, "order": 7 },
        { "id": "weekly_trend", "visible": true, "order": 8 },
        { "id": "recent_documents", "visible": true, "order": 9 },
        { "id": "upcoming_leaves", "visible": true, "order": 10 },
        { "id": "staff_attendance", "visible": false, "order": 11 }
    ])
}
