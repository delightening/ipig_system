// 啟動配置警告 API
// 管理員登入後，前端可呼叫此 API 取得三項配置狀態（地理圍籬、管理員密碼、開發帳號）

use axum::{extract::State, Extension, Json};
use serde::Serialize;

use crate::{middleware::CurrentUser, AppState, Result};

/// 單一警告項目
#[derive(Debug, Serialize)]
pub struct ConfigWarningItem {
    /// "warn" | "ok" | "info"
    pub level: String,
    /// 項目標題
    pub title: String,
    /// 詳細說明（可為 null）
    pub detail: Option<String>,
}

/// 配置警告回應
#[derive(Debug, Serialize)]
pub struct ConfigWarningsResponse {
    pub warnings: Vec<ConfigWarningItem>,
    pub warn_count: usize,
}

/// GET /admin/config-warnings
/// 回傳啟動配置的三項檢查狀態
pub async fn get_config_warnings(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<ConfigWarningsResponse>> {
    let config = &state.config;
    let mut items: Vec<ConfigWarningItem> = Vec::new();
    let mut warn_count: usize = 0;

    // 1) 地理圍籬設定
    let has_ip = !config.allowed_clock_ip_ranges.is_empty();
    let has_gps = config.clock_office_latitude.is_some()
        && config.clock_office_longitude.is_some();

    if !has_ip && !has_gps {
        warn_count += 1;
        items.push(ConfigWarningItem {
            level: "warn".to_string(),
            title: "地理圍籬未設定".to_string(),
            detail: Some(
                "ALLOWED_CLOCK_IP_RANGES / CLOCK_OFFICE_LATITUDE / CLOCK_OFFICE_LONGITUDE 皆未設定，所有位置的打卡都會通過。請在 .env 設定辦公室 IP 或 GPS 座標。"
                    .to_string(),
            ),
        });
    } else if !has_ip {
        warn_count += 1;
        items.push(ConfigWarningItem {
            level: "warn".to_string(),
            title: "打卡 IP 白名單未設定".to_string(),
            detail: Some(
                "ALLOWED_CLOCK_IP_RANGES 未設定，僅依賴 GPS 驗證。建議同時設定 IP 範圍以提高安全性。"
                    .to_string(),
            ),
        });
    } else if !has_gps {
        warn_count += 1;
        items.push(ConfigWarningItem {
            level: "warn".to_string(),
            title: "打卡 GPS 座標未設定".to_string(),
            detail: Some(
                "CLOCK_OFFICE_LATITUDE / CLOCK_OFFICE_LONGITUDE 未設定，僅依賴 IP 驗證。建議同時設定 GPS 座標以支援行動裝置。"
                    .to_string(),
            ),
        });
    } else {
        items.push(ConfigWarningItem {
            level: "ok".to_string(),
            title: "地理圍籬設定正確（IP + GPS）".to_string(),
            detail: None,
        });
    }

    // 2) 管理員初始密碼
    match std::env::var("ADMIN_INITIAL_PASSWORD") {
        Err(_) => {
            warn_count += 1;
            items.push(ConfigWarningItem {
                level: "warn".to_string(),
                title: "ADMIN_INITIAL_PASSWORD 未設定".to_string(),
                detail: Some(
                    "管理員帳號將使用隨機產生的密碼，建議在 .env 中明確設定。".to_string(),
                ),
            });
        }
        Ok(pwd) if pwd == crate::constants::DEFAULT_INSECURE_PASSWORD || pwd.len() < 8 => {
            warn_count += 1;
            items.push(ConfigWarningItem {
                level: "warn".to_string(),
                title: "ADMIN_INITIAL_PASSWORD 使用預設值或過於簡短".to_string(),
                detail: Some("請在 .env 中設定一組高強度密碼（至少 8 字元）。".to_string()),
            });
        }
        _ => {
            items.push(ConfigWarningItem {
                level: "ok".to_string(),
                title: "ADMIN_INITIAL_PASSWORD 設定正確".to_string(),
                detail: None,
            });
        }
    }

    // 3) 開發帳號與測試密碼
    if config.seed_dev_users {
        match std::env::var("TEST_USER_PASSWORD") {
            Err(_) => {
                warn_count += 1;
                items.push(ConfigWarningItem {
                    level: "warn".to_string(),
                    title: "SEED_DEV_USERS=true 但 TEST_USER_PASSWORD 未設定".to_string(),
                    detail: Some(
                        "開發測試帳號將沒有可用密碼，請在 .env 中設定。".to_string(),
                    ),
                });
            }
            Ok(pwd) if pwd.len() < 8 => {
                warn_count += 1;
                items.push(ConfigWarningItem {
                    level: "warn".to_string(),
                    title: "TEST_USER_PASSWORD 長度不足 8 字元".to_string(),
                    detail: Some(
                        "請設定一組較安全的測試密碼（至少 8 字元）。".to_string(),
                    ),
                });
            }
            _ => {
                items.push(ConfigWarningItem {
                    level: "ok".to_string(),
                    title: "SEED_DEV_USERS 已啟用，TEST_USER_PASSWORD 設定正確".to_string(),
                    detail: None,
                });
            }
        }
    } else {
        items.push(ConfigWarningItem {
            level: "info".to_string(),
            title: "SEED_DEV_USERS 未啟用".to_string(),
            detail: Some("開發測試帳號不會被建立。".to_string()),
        });
    }

    Ok(Json(ConfigWarningsResponse {
        warnings: items,
        warn_count,
    }))
}
