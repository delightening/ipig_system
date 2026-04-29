// 啟動配置完整性檢查模組

use crate::config::Config;
use crate::constants::DEFAULT_INSECURE_PASSWORD;
use super::is_production;

/// 在啟動時印出配置摘要框（永遠顯示），提示潛在的安全或設定問題
pub fn log_startup_config_check(config: &Config) {
    let mut items: Vec<String> = Vec::new();
    let mut warn_count: usize = 0;

    // 1) 地理圍籬設定
    let has_ip = !config.allowed_clock_ip_ranges.is_empty();
    let has_gps =
        config.clock_office_latitude.is_some() && config.clock_office_longitude.is_some();
    if !has_ip && !has_gps {
        warn_count += 1;
        items.push(
            "⚠️  地理圍籬未設定（ALLOWED_CLOCK_IP_RANGES / CLOCK_OFFICE_LATITUDE / CLOCK_OFFICE_LONGITUDE）\n     \
             → 所有位置的打卡都會通過！請在 .env 設定辦公室 IP 或 GPS 座標。".to_string()
        );
    } else if !has_ip {
        warn_count += 1;
        items.push(
            "⚠️  打卡 IP 白名單未設定（ALLOWED_CLOCK_IP_RANGES）\n     \
             → 僅依賴 GPS 驗證，建議同時設定 IP 範圍以提高安全性。"
                .to_string(),
        );
    } else if !has_gps {
        warn_count += 1;
        items.push(
            "⚠️  打卡 GPS 座標未設定（CLOCK_OFFICE_LATITUDE / CLOCK_OFFICE_LONGITUDE）\n     \
             → 僅依賴 IP 驗證，建議同時設定 GPS 座標以支援行動裝置。"
                .to_string(),
        );
    } else {
        items.push("✅ 地理圍籬設定正確（IP + GPS）".to_string());
    }

    // 2) 管理員初始密碼
    match &config.admin_initial_password {
        None => {
            warn_count += 1;
            items.push(
                "⚠️  ADMIN_INITIAL_PASSWORD 未設定\n     \
                 → 管理員帳號將使用隨機產生的密碼，建議在 .env 中明確設定。"
                    .to_string(),
            );
        }
        Some(pwd) if pwd == DEFAULT_INSECURE_PASSWORD || pwd.len() < 8 => {
            warn_count += 1;
            items.push(
                "⚠️  ADMIN_INITIAL_PASSWORD 使用預設值或過於簡短\n     \
                 → 請在 .env 中設定一組高強度密碼。"
                    .to_string(),
            );
        }
        _ => {
            items.push("✅ ADMIN_INITIAL_PASSWORD 設定正確".to_string());
        }
    }

    // 3) 開發帳號與測試密碼
    if config.seed_dev_users {
        match &config.test_user_password {
            None => {
                warn_count += 1;
                items.push(
                    "⚠️  SEED_DEV_USERS=true 但 TEST_USER_PASSWORD 未設定\n     \
                     → 開發測試帳號將沒有可用密碼，請在 .env 中設定。"
                        .to_string(),
                );
            }
            Some(pwd) if pwd.len() < 8 => {
                warn_count += 1;
                items.push(
                    "⚠️  TEST_USER_PASSWORD 長度不足 8 字元\n     \
                     → 請設定一組較安全的測試密碼。"
                        .to_string(),
                );
            }
            _ => {
                items.push(
                    "✅ SEED_DEV_USERS 已啟用，TEST_USER_PASSWORD 設定正確".to_string(),
                );
            }
        }
    } else {
        items.push(
            "ℹ️  SEED_DEV_USERS 未啟用（開發測試帳號不會被建立）".to_string(),
        );
    }

    // 輸出匯總框（永遠顯示）
    let numbered = items
        .iter()
        .enumerate()
        .map(|(i, w)| format!("  {}. {}", i + 1, w))
        .collect::<Vec<_>>()
        .join("\n");

    let header = if warn_count > 0 {
        format!("⚠️  啟動配置檢查：{} 項待處理", warn_count)
    } else {
        "✅ 啟動配置檢查：全部通過".to_string()
    };

    if warn_count > 0 {
        tracing::warn!(
            "\n╔════════════════════════════════════════════════════════════╗\n\
               ║  {}  ║\n\
               ╠════════════════════════════════════════════════════════════╣\n\
               {}\n\
               ╚════════════════════════════════════════════════════════════╝",
            header,
            numbered
        );

        // R30-23：production 環境硬性 fail-fast。dev / staging 仍 warn-only
        // 以便快速 iterate；production 啟動前必須清掉所有警告。
        // 對應 21 CFR §11.10(k) + GLP §1.4 production 啟動完整性要求。
        if is_production() {
            tracing::error!(
                "[R30-23] production 環境啟動前必須清掉所有 config 警告（{} 項）。\
                 退出（exit code 1）。設 APP_ENV=staging 或移除可避開此檢查。",
                warn_count
            );
            std::process::exit(1);
        }
    } else {
        tracing::info!(
            "\n╔════════════════════════════════════════════════════════════╗\n\
               ║  {}              ║\n\
               ╠════════════════════════════════════════════════════════════╣\n\
               {}\n\
               ╚════════════════════════════════════════════════════════════╝",
            header,
            numbered
        );
    }
}
