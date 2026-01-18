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

/// 預設的儀表板 Widget 設定 (react-grid-layout 格式)
/// i: Widget ID, x: 起始列(0-11), y: 起始行, w: 寬度(列數), h: 高度(行數)
pub fn default_dashboard_widgets() -> serde_json::Value {
    serde_json::json!([
        // 第一行: 今日日曆(4列) + 請假餘額(2列) + 我的計畫(3列) + 獸醫師評論(3列)
        { "i": "calendar_widget", "x": 0, "y": 0, "w": 4, "h": 3, "visible": true, "minW": 4, "minH": 2 },
        { "i": "leave_balance", "x": 4, "y": 0, "w": 2, "h": 1, "visible": true, "minW": 2, "minH": 1 },
        { "i": "my_projects", "x": 6, "y": 0, "w": 3, "h": 2, "visible": true, "minW": 3, "minH": 2 },
        { "i": "vet_comments", "x": 9, "y": 0, "w": 3, "h": 2, "visible": true, "minW": 3, "minH": 2 },
        // 第二行: 正在用藥動物 + 最近單據
        { "i": "animals_on_medication", "x": 4, "y": 1, "w": 2, "h": 2, "visible": true, "minW": 2, "minH": 2 },
        { "i": "recent_documents", "x": 6, "y": 2, "w": 6, "h": 3, "visible": true, "minW": 4, "minH": 2 },
        // 第三行: ERP 小 widgets
        { "i": "low_stock_alert", "x": 0, "y": 3, "w": 2, "h": 1, "visible": true, "minW": 2, "minH": 1 },
        { "i": "pending_documents", "x": 2, "y": 3, "w": 2, "h": 1, "visible": true, "minW": 2, "minH": 1 },
        { "i": "today_inbound", "x": 4, "y": 3, "w": 2, "h": 1, "visible": true, "minW": 2, "minH": 1 },
        { "i": "today_outbound", "x": 0, "y": 4, "w": 2, "h": 1, "visible": true, "minW": 2, "minH": 1 },
        // 第四行: 近7天趨勢
        { "i": "weekly_trend", "x": 0, "y": 5, "w": 6, "h": 3, "visible": true, "minW": 4, "minH": 2, "options": { "days": 7 } },
        // 第五行: 日曆事件 + 工作人員出勤表
        { "i": "google_calendar_events", "x": 0, "y": 8, "w": 4, "h": 3, "visible": true, "minW": 3, "minH": 2 },
        { "i": "staff_attendance", "x": 4, "y": 8, "w": 8, "h": 3, "visible": true, "minW": 6, "minH": 2 },
        // 即將到期假期 (預設隱藏)
        { "i": "upcoming_leaves", "x": 4, "y": 5, "w": 2, "h": 1, "visible": false, "minW": 2, "minH": 1 }
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
    if roles.len() == 1 && roles.iter().any(|r| r == "EXPERIMENT_STAFF") {
        return default_dashboard_widgets_for_experiment_staff();
    }
    // 其他情況使用標準佈局
    default_dashboard_widgets()
}

