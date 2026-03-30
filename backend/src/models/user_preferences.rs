use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

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
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpsertPreferenceRequest {
    pub value: serde_json::Value,
}

/// 偏好設定回應
#[derive(Debug, Serialize, ToSchema)]
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
#[derive(Debug, Serialize, ToSchema)]
pub struct AllPreferencesResponse {
    pub preferences: Vec<PreferenceResponse>,
}

/// 預設的側邊欄順序
/// 使用前端 NavItem 的 title 屬性作為識別符
pub fn default_nav_order() -> serde_json::Value {
    serde_json::json!([
        "dashboard",
        "myProjects",
        "aupReview",
        "animalManagement",
        "人員管理",
        "ERP",
        "系統管理"
    ])
}

/// 預設的儀表板 Widget 設定 (react-grid-layout 格式)
/// i: Widget ID, x: 起始列(0-11), y: 起始行, w: 寬度(列數), h: 高度(行數)
/// 注意：此定義必須與前端 widgetConfig.ts 的 DEFAULT_DASHBOARD_LAYOUT 保持同步
pub fn default_dashboard_widgets() -> serde_json::Value {
    serde_json::json!([
        // 第一行: 今日日曆(4列) + 獸醫師評論(3列) + 正在用藥動物(2列)
        { "i": "calendar_widget", "x": 0, "y": 0, "w": 4, "h": 3, "visible": true, "minW": 2, "minH": 2 },
        { "i": "vet_comments", "x": 4, "y": 0, "w": 3, "h": 3, "visible": true, "minW": 2, "minH": 2 },
        { "i": "animals_on_medication", "x": 7, "y": 0, "w": 2, "h": 6, "visible": true, "minW": 2, "minH": 2 },
        // 第二行: 我的計畫(4列) + 請假餘額(3列)
        { "i": "my_projects", "x": 0, "y": 3, "w": 4, "h": 3, "visible": true, "minW": 2, "minH": 2 },
        { "i": "leave_balance", "x": 4, "y": 3, "w": 3, "h": 3, "visible": true, "minW": 2, "minH": 2 },
        // 第三行: 低庫存警示(2列) + 今日入庫(2列) + 最近單據(5列)
        { "i": "low_stock_alert", "x": 0, "y": 6, "w": 2, "h": 2, "visible": true, "minW": 2, "minH": 2 },
        { "i": "today_inbound", "x": 2, "y": 6, "w": 2, "h": 2, "visible": true, "minW": 2, "minH": 2 },
        { "i": "recent_documents", "x": 4, "y": 6, "w": 5, "h": 4, "visible": true, "minW": 2, "minH": 2 },
        // 第四行: 今日出庫(2列) + 待處理單據(2列)
        { "i": "today_outbound", "x": 0, "y": 8, "w": 2, "h": 2, "visible": true, "minW": 2, "minH": 2 },
        { "i": "pending_documents", "x": 2, "y": 8, "w": 2, "h": 2, "visible": true, "minW": 2, "minH": 2 },
        // 第五行: 近7天趨勢(4列) + 員工出勤表(5列)
        { "i": "weekly_trend", "x": 0, "y": 10, "w": 4, "h": 3, "visible": true, "minW": 2, "minH": 2, "options": { "days": 7 } },
        { "i": "staff_attendance", "x": 4, "y": 10, "w": 5, "h": 3, "visible": true, "minW": 2, "minH": 2 },
        // 第六行: Google 日曆事件(9列)
        { "i": "google_calendar_events", "x": 0, "y": 13, "w": 9, "h": 5, "visible": true, "minW": 2, "minH": 2 },
        // 即將到期假期 (預設隱藏)
        { "i": "upcoming_leaves", "x": 4, "y": 7, "w": 2, "h": 2, "visible": false, "minW": 2, "minH": 2 }
    ])
}

/// 實驗工作人員專用的儀表板預設佈局
/// 簡潔的佈局，只顯示與實驗動物相關的 widgets
pub fn default_dashboard_widgets_for_experiment_staff() -> serde_json::Value {
    serde_json::json!([
        // 第一行: 今日日曆(3列) + 請假餘額(3列) + 我的計畫(3列)
        { "i": "calendar_widget", "x": 0, "y": 0, "w": 3, "h": 4, "visible": true, "minW": 3, "minH": 3 },
        { "i": "leave_balance", "x": 3, "y": 0, "w": 3, "h": 4, "visible": true, "minW": 2, "minH": 2 },
        { "i": "my_projects", "x": 6, "y": 0, "w": 3, "h": 4, "visible": true, "minW": 3, "minH": 3 },
        // 第二行: 日曆事件(3列) + 正在用藥動物(3列) + 獸醫師評論(3列)
        { "i": "google_calendar_events", "x": 0, "y": 4, "w": 3, "h": 4, "visible": true, "minW": 3, "minH": 3 },
        { "i": "animals_on_medication", "x": 3, "y": 4, "w": 3, "h": 4, "visible": true, "minW": 2, "minH": 2 },
        { "i": "vet_comments", "x": 6, "y": 4, "w": 3, "h": 4, "visible": true, "minW": 3, "minH": 3 },
        // 隱藏的 widgets (可在設定中開啟)
        { "i": "low_stock_alert", "x": 0, "y": 8, "w": 2, "h": 1, "visible": false, "minW": 2, "minH": 1 },
        { "i": "pending_documents", "x": 2, "y": 8, "w": 2, "h": 1, "visible": false, "minW": 2, "minH": 1 },
        { "i": "today_inbound", "x": 4, "y": 8, "w": 2, "h": 1, "visible": false, "minW": 2, "minH": 1 },
        { "i": "today_outbound", "x": 6, "y": 8, "w": 2, "h": 1, "visible": false, "minW": 2, "minH": 1 },
        { "i": "weekly_trend", "x": 0, "y": 9, "w": 6, "h": 3, "visible": false, "minW": 4, "minH": 2, "options": { "days": 7 } },
        { "i": "recent_documents", "x": 6, "y": 9, "w": 6, "h": 3, "visible": false, "minW": 4, "minH": 2 },
        { "i": "staff_attendance", "x": 0, "y": 12, "w": 12, "h": 3, "visible": false, "minW": 6, "minH": 2 },
        { "i": "upcoming_leaves", "x": 0, "y": 15, "w": 3, "h": 1, "visible": false, "minW": 2, "minH": 1 }
    ])
}

/// 根據角色取得對應的儀表板預設佈局
pub fn get_dashboard_widgets_for_roles(roles: &[String]) -> serde_json::Value {
    // 如果只有 EXPERIMENT_STAFF 角色，使用簡潔佈局
    if roles.len() == 1 && roles.iter().any(|r| r == crate::constants::ROLE_EXPERIMENT_STAFF) {
        return default_dashboard_widgets_for_experiment_staff();
    }
    // 其他情況使用標準佈局
    default_dashboard_widgets()
}

