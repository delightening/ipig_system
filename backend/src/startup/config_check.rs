// 啟動配置完整性檢查模組

use crate::config::Config;
use crate::constants::DEFAULT_INSECURE_PASSWORD;

/// 在啟動時印出配置摘要框（永遠顯示），提示潛在的安全或設定問題。
///
/// **R30-23**：回傳 `warn_count`，呼叫端依 `is_production()` 決定是否 fail-fast。
/// 把 `process::exit` 邏輯移到 `main.rs` 與 `run_db_self_test` 一致，
/// 並讓本函式在整合測試中可重用。
///
/// **Low-6 (#283) 範圍說明**：本檢查涵蓋「軟性警告項」（地理圍籬 / admin 密碼 /
/// AUDIT_HMAC_KEY / CSRF_SECRET / SEED_DEV_USERS / TEST_USER_PASSWORD）。真正高危的硬性
/// secret（JWT 私/公鑰）已在 `Config::from_env` 以 `.context()?` 強制把關（缺失即啟動失敗），
/// 不會走到這裡。CSRF_SECRET 未設時 config 會回退為 JWT 派生（R82-6），故列為軟性警告 →
/// prod 由 main.rs `is_production()` gate fail-fast。X-Internal-Token / SMTP 等 prod-required
/// 但目前不 fail 的項，若要納入須在此 `items` / `warn_count` 補列。
pub fn log_startup_config_check(config: &Config) -> usize {
    let mut items: Vec<String> = Vec::new();
    let mut warn_count: usize = 0;

    // 1) 地理圍籬設定
    let has_ip = !config.allowed_clock_ip_ranges.is_empty();
    let has_gps = config.clock_office_latitude.is_some() && config.clock_office_longitude.is_some();
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

    // 3) AUDIT HMAC key（R63-B9）
    if config.audit_hmac_key.is_none() {
        warn_count += 1;
        items.push(
            "⚠️  AUDIT_HMAC_KEY 未設定（需 ≥44 字元）\n     \
             → 稽核日誌 HMAC chain 將無法運作，GLP 合規要求此金鑰。"
                .to_string(),
        );
    } else {
        items.push("✅ AUDIT_HMAC_KEY 設定正確".to_string());
    }

    // 3b) CSRF_SECRET 獨立性 + 強度（R82-6 / gemini security-high）：未顯式設定時 config 會
    // 回退為 JWT 私鑰 SHA256 派生，JWT 私鑰外洩即可離線推導 CSRF secret（破壞金鑰隔離）。
    // 另外「設了但空 / 過短」也危險（含 CSRF_SECRET_FILE 指向空檔 → Some("")）。三種情形都
    // 記 warn，main.rs 於 is_production() 時 fail-fast（dev/CI/test 仍可走派生 fallback）。
    // 強度門檻 ≥44 字元對齊 AUDIT_HMAC_KEY。
    match crate::config::read_secret("CSRF_SECRET") {
        Some(ref s) if s.len() >= 44 => {
            items.push("✅ CSRF_SECRET 獨立設定正確".to_string());
        }
        Some(s) => {
            warn_count += 1;
            items.push(format!(
                "⚠️  CSRF_SECRET 已設定但強度不足（長度 {}，需 ≥44 字元）\n     \
                 → 空 / 過短金鑰使 CSRF HMAC 防護失效。\n     \
                 → prod 請設 ≥44 字元的獨立 CSRF_SECRET。",
                s.len()
            ));
        }
        None => {
            warn_count += 1;
            items.push(
                "⚠️  CSRF_SECRET 未設定（正回退為 JWT 私鑰派生）\n     \
                 → 破壞金鑰隔離：JWT 私鑰外洩即可離線推導 CSRF secret。\n     \
                 → prod 請設獨立 CSRF_SECRET（或 CSRF_SECRET_FILE），建議 ≥44 字元。"
                    .to_string(),
            );
        }
    }

    // 4) SEED_DEV_USERS 在非測試環境（R63-B3）
    if config.seed_dev_users && !config.is_ci && !config.cookie_secure {
        warn_count += 1;
        items.push(
            "⚠️  SEED_DEV_USERS=true 在非 CI、非 HTTPS 環境\n     \
             → 開發帳號會被建立在可能是 production 的環境中，請確認是否正確。"
                .to_string(),
        );
    }

    // 5) 開發帳號與測試密碼
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
                items.push("✅ SEED_DEV_USERS 已啟用，TEST_USER_PASSWORD 設定正確".to_string());
            }
        }
    } else {
        items.push("ℹ️  SEED_DEV_USERS 未啟用（開發測試帳號不會被建立）".to_string());
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
    warn_count
}
