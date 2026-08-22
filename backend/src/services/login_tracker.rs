// Login Tracker Service
// 追蹤登入事件並檢測異常

use chrono::{Timelike, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::geoip::{GeoInfo, GeoIpService};
use super::security_notifier::{SecurityNotification, SecurityNotifier};
use crate::config::Config;
use crate::constants::{GUEST_DEMO_EMAIL, ROLE_ADMIN_LEGACY, ROLE_SYSTEM_ADMIN};
use crate::Result;

pub struct LoginTracker;

impl LoginTracker {
    /// 記錄成功登入
    pub async fn log_success(
        pool: &PgPool,
        user_id: Uuid,
        email: &str,
        ip: Option<&str>,
        user_agent: Option<&str>,
        geoip: &GeoIpService,
    ) -> Result<()> {
        let ua = parse_user_agent(user_agent);

        // 查詢 IP 地理位置
        let geo = ip
            .and_then(|ip_str| geoip.lookup(ip_str))
            .unwrap_or_default();

        // R49 follow-up：admin 在多項異常偵測中要特殊處理 — unusual_time 略過
        //（solo operator 工時不固定），new_device 仍警報（帳號最敏感）。
        let is_admin = is_admin_user(pool, user_id).await;

        // previous_login 必須在本次事件 INSERT **之前**查，否則查到的是自己這一筆。
        let (is_unusual_time, is_unusual_location, is_new_device, is_mass_login, previous_login) = tokio::join!(
            async { check_unusual_time() },
            async { check_unusual_location(pool, user_id, &geo).await },
            async { check_new_device(pool, user_id, user_agent, is_admin).await },
            async { check_mass_login(pool, user_id).await },
            async { find_previous_login(pool, user_id).await }
        );

        // 插入登入成功日誌（含地理位置資訊）
        sqlx::query(
            r#"
            INSERT INTO login_events (
                id, user_id, email, event_type, 
                ip_address, user_agent, device_type, browser, os,
                geo_country, geo_city, geo_timezone,
                is_unusual_time, is_unusual_location, is_new_device, is_mass_login,
                created_at
            ) VALUES (
                $1, $2, $3, 'login_success',
                $4::INET, $5, $6, $7, $8,
                $9, $10, $11,
                $12, $13, $14, $15, NOW()
            )
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(email)
        .bind(ip)
        .bind(user_agent)
        .bind(&ua.device_type)
        .bind(&ua.browser)
        .bind(&ua.os)
        .bind(&geo.country)
        .bind(&geo.city)
        .bind(&geo.timezone)
        .bind(is_unusual_time)
        .bind(is_unusual_location)
        .bind(is_new_device)
        .bind(is_mass_login)
        .execute(pool)
        .await?;

        // 如果有異常，建立個人警報
        let flags = AnomalyFlags {
            unusual_time: is_unusual_time,
            unusual_location: is_unusual_location,
            new_device: is_new_device,
            mass_login: is_mass_login,
        };
        if flags.any() {
            Self::create_login_alert(
                pool,
                &LoginAnomaly {
                    user_id,
                    email,
                    ip,
                    geo: &geo,
                    device: &ua,
                    previous_login: previous_login.as_ref(),
                    is_admin,
                    flags,
                },
            )
            .await?;
        }

        // 檢查全域多帳號大量登入 (疑似腳本)
        check_global_mass_login(pool).await?;

        Ok(())
    }

    /// 記錄失敗登入
    pub async fn log_failure(
        pool: &PgPool,
        config: &Config,
        email: &str,
        ip: Option<&str>,
        user_agent: Option<&str>,
        reason: &str,
        geoip: &GeoIpService,
    ) -> Result<()> {
        let device_info = parse_user_agent(user_agent);

        // 查詢 IP 地理位置
        let geo = ip
            .and_then(|ip_str| geoip.lookup(ip_str))
            .unwrap_or_default();

        // A5 資安/一致性修補：帳號鎖定計數的那筆 login_failure 已由 AuthService 的
        // advisory-lock 事務內原子寫入（insert_failure_event_in_tx）。此處不再重複
        // INSERT（否則每次失敗寫兩筆、鎖定門檻 5 實際被砍半成約 3），改為就地 UPDATE
        // 那筆最近的失敗紀錄，補上 GeoIP / device 等完整欄位。
        sqlx::query(
            r#"
            UPDATE login_events SET
                user_agent     = $2,
                device_type    = $3,
                browser        = $4,
                os             = $5,
                geo_country    = $6,
                geo_city       = $7,
                geo_timezone   = $8,
                failure_reason = $9
            WHERE id = (
                SELECT id FROM login_events
                WHERE email = $1
                  AND event_type = 'login_failure'
                  AND created_at > NOW() - INTERVAL '30 seconds'
                ORDER BY created_at DESC
                LIMIT 1
            )
            "#,
        )
        .bind(email)
        .bind(user_agent)
        .bind(&device_info.device_type)
        .bind(&device_info.browser)
        .bind(&device_info.os)
        .bind(&geo.country)
        .bind(&geo.city)
        .bind(&geo.timezone)
        .bind(reason)
        .execute(pool)
        .await?;

        // 檢查暴力破解
        Self::check_brute_force(pool, config, email, ip).await?;

        Ok(())
    }

    /// 記錄登出
    pub async fn log_logout(
        pool: &PgPool,
        user_id: Uuid,
        email: &str,
        ip: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO login_events (id, user_id, email, event_type, ip_address, created_at)
            VALUES ($1, $2, $3, 'logout', $4::INET, NOW())
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(email)
        .bind(ip)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 檢查暴力破解攻擊
    async fn check_brute_force(
        pool: &PgPool,
        config: &Config,
        email: &str,
        ip: Option<&str>,
    ) -> Result<()> {
        // Guest demo 帳號豁免 — 公開試用帳號難免有錯密嘗試，不應觸發 alert
        if email == GUEST_DEMO_EMAIL {
            return Ok(());
        }

        // 檢查過去 15 分鐘的失敗次數
        let (fail_count,): (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM login_events
            WHERE email = $1
              AND event_type = 'login_failure'
              AND created_at > NOW() - INTERVAL '15 minutes'
            "#,
        )
        .bind(email)
        .fetch_one(pool)
        .await?;

        if fail_count >= 5 {
            // R22-7: 去重 — 同一 email 在 30 分鐘內已有 open alert 則跳過
            let (recent_alert_count,): (i64,) = sqlx::query_as(
                r#"
                SELECT COUNT(*) FROM security_alerts
                WHERE alert_type = 'brute_force'
                  AND context_data->>'email' = $1
                  AND created_at > NOW() - INTERVAL '30 minutes'
                  AND status = 'open'
                "#,
            )
            .bind(email)
            .fetch_one(pool)
            .await?;

            if recent_alert_count == 0 {
                let alert_id = Uuid::new_v4();
                let description = format!(
                    "Email {} 在過去 15 分鐘內有 {} 次失敗登入嘗試",
                    email, fail_count
                );
                sqlx::query(
                    r#"
                    INSERT INTO security_alerts (
                        id, alert_type, severity, title, description,
                        context_data, created_at, updated_at, status
                    ) VALUES (
                        $1, 'brute_force', 'critical',
                        '偵測到可能的暴力破解攻擊',
                        $2, $3, NOW(), NOW(), 'open'
                    )
                    "#,
                )
                .bind(alert_id)
                .bind(&description)
                .bind(serde_json::json!({
                    "email": email,
                    "ip": ip,
                    "fail_count": fail_count
                }))
                .execute(pool)
                .await?;

                tracing::warn!("[R22-7] Brute force alert created for {email}");

                let notification = SecurityNotification {
                    alert_id,
                    alert_type: "brute_force".to_string(),
                    severity: "critical".to_string(),
                    title: "偵測到可能的暴力破解攻擊".to_string(),
                    description: Some(description),
                    context_data: Some(serde_json::json!({
                        "email": email,
                        "ip": ip,
                        "fail_count": fail_count,
                    })),
                    created_at: Utc::now(),
                };
                SecurityNotifier::dispatch(pool, config, &notification).await;
            }
        }

        Ok(())
    }

    /// 建立登入異常警報
    ///
    /// R49 follow-up 三階段降噪：
    /// - **A**：30 分鐘內同 user 已有 open 'unusual_login' alert → skip（對齊 brute_force pattern）
    /// - **B**：new_device 首次登入由 caller 已過濾（admin 例外，仍會傳 true）
    /// - **D**：admin AND 只有 unusual_time 觸發 → severity 降為 'info'（solo operator
    ///   工時不固定，但保留軌跡）；其他組合維持 'warning'
    async fn create_login_alert(pool: &PgPool, anomaly: &LoginAnomaly<'_>) -> Result<()> {
        let severity = anomaly.severity();

        if is_alert_deduplicated(pool, anomaly.user_id, severity).await? {
            return Ok(());
        }

        let title = format!("偵測到異常登入 ({})", anomaly.email);
        let description = format!(
            "帳號 {} 的登入觸發異常偵測：{}",
            anomaly.email,
            anomaly.reasons().join("、")
        );

        sqlx::query(
            r#"
            INSERT INTO security_alerts (
                id, alert_type, severity, title, description,
                user_id, context_data, created_at, updated_at, status
            ) VALUES (
                $1, 'unusual_login', $2,
                $3, $4, $5, $6, NOW(), NOW(), 'open'
            )
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(severity)
        .bind(&title)
        .bind(&description)
        .bind(anomaly.user_id)
        .bind(anomaly.context_data())
        .execute(pool)
        .await?;

        Ok(())
    }
}

/// 觸發異常登入警報的四個訊號。
#[derive(Clone, Copy)]
struct AnomalyFlags {
    unusual_time: bool,
    unusual_location: bool,
    new_device: bool,
    mass_login: bool,
}

impl AnomalyFlags {
    fn any(&self) -> bool {
        self.unusual_time || self.unusual_location || self.new_device || self.mass_login
    }
}

/// 異常登入警報的完整上下文。
///
/// 原本 `create_login_alert` 已是 9 個參數（靠 `#[allow(clippy::too_many_arguments)]` 壓住），
/// 這次要再帶 IP / 裝置 / 前次登入進去，改封裝成 struct 才不會繼續惡化。
struct LoginAnomaly<'a> {
    user_id: Uuid,
    email: &'a str,
    ip: Option<&'a str>,
    geo: &'a GeoInfo,
    device: &'a DeviceInfo,
    previous_login: Option<&'a PreviousLogin>,
    is_admin: bool,
    flags: AnomalyFlags,
}

impl LoginAnomaly<'_> {
    /// D: admin 且僅 unusual_time 觸發 → 降為 info（保留軌跡，不刺眼）。
    /// 多訊號同時觸發（time + location/device/mass）→ 真實風險，維持 warning。
    /// #395：severity 須在 dedup 前算好，dedup 才能 severity-aware。
    fn severity(&self) -> &'static str {
        let f = self.flags;
        let admin_unusual_time_only = self.is_admin
            && f.unusual_time
            && !f.unusual_location
            && !f.new_device
            && !f.mass_login;
        if admin_unusual_time_only {
            "info"
        } else {
            "warning"
        }
    }

    /// 給人看的觸發原因（組進 description）。
    fn reasons(&self) -> Vec<String> {
        let mut reasons = Vec::new();
        if self.flags.unusual_time {
            reasons.push("非工作時間登入".to_string());
        }
        if self.flags.unusual_location {
            // 如果有地理位置資訊，附加到原因中
            let location_desc = match (&self.geo.country, &self.geo.city) {
                (Some(country), Some(city)) => format!("來自新的地理位置（{} {}）", country, city),
                (Some(country), None) => format!("來自新的地理位置（{}）", country),
                _ => "來自新的 IP 位置".to_string(),
            };
            reasons.push(location_desc);
        }
        if self.flags.new_device {
            reasons.push("使用新裝置".to_string());
        }
        if self.flags.mass_login {
            reasons.push("同時大量登入".to_string());
        }
        reasons
    }

    /// 給機器看的觸發代碼（前端可據此分類，不必解析中文句子）。
    fn triggers(&self) -> Vec<&'static str> {
        let mut triggers = Vec::new();
        if self.flags.unusual_time {
            triggers.push("unusual_time");
        }
        if self.flags.unusual_location {
            triggers.push("unusual_location");
        }
        if self.flags.new_device {
            triggers.push("new_device");
        }
        if self.flags.mass_login {
            triggers.push("mass_login");
        }
        triggers
    }

    /// 警報詳情頁要顯示的來源資訊。
    ///
    /// 原本 unusual_login 警報**完全沒寫 context_data**（brute_force 有寫），
    /// 詳情頁只有一句「來自新的 IP 位置」卻查不到究竟是哪個 IP——資料明明就在
    /// `login_events`，只是沒帶過來，管理員無從判斷是本人換網路還是他人盜用。
    /// `previous_login_*` 是給人比對用的：同一個人換 4G 網路 vs 陌生位置，
    /// 對照前一次登入來源最容易一眼看出來。
    fn context_data(&self) -> serde_json::Value {
        let mut ctx = serde_json::Map::new();
        ctx.insert("email".to_string(), serde_json::json!(self.email));
        // ip 為 null 本身就是要看的訊號（反向代理沒帶 XFF），故不省略。
        ctx.insert("ip".to_string(), serde_json::json!(self.ip));
        ctx.insert("triggers".to_string(), serde_json::json!(self.triggers()));

        let optional_fields = [
            ("geo_country", self.geo.country.as_deref()),
            ("geo_city", self.geo.city.as_deref()),
            ("browser", self.device.browser.as_deref()),
            ("os", self.device.os.as_deref()),
            ("device_type", self.device.device_type.as_deref()),
        ];
        for (key, value) in optional_fields {
            if let Some(value) = value {
                ctx.insert(key.to_string(), serde_json::json!(value));
            }
        }

        if let Some(previous) = self.previous_login {
            ctx.insert(
                "previous_login_ip".to_string(),
                serde_json::json!(previous.ip),
            );
            ctx.insert(
                "previous_login_at".to_string(),
                serde_json::json!(previous.created_at),
            );
        }

        serde_json::Value::Object(ctx)
    }
}

/// A: 30 分鐘 dedup — 但只被「同級或更高級」的既有 open alert 抑制（warning > info）。
/// #395：原本不分 severity，導致 admin 半夜正常登入寫的 info alert 會在 30 分鐘內
/// 壓掉後續真正高風險的 warning（新裝置+新地點＝帳號被盜訊號）。改為：新 warning
/// 只被既有 warning 抑制；新 info 才被 info/warning 抑制。
async fn is_alert_deduplicated(pool: &PgPool, user_id: Uuid, severity: &str) -> Result<bool> {
    let suppressing_severities: &[&str] = if severity == "warning" {
        &["warning"]
    } else {
        &["info", "warning"]
    };
    let suppressing: Vec<String> = suppressing_severities
        .iter()
        .map(|s| s.to_string())
        .collect();
    let (recent_alert_count,): (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM security_alerts
        WHERE alert_type = 'unusual_login'
          AND user_id = $1
          AND created_at > NOW() - INTERVAL '30 minutes'
          AND status = 'open'
          AND severity = ANY($2)
        "#,
    )
    .bind(user_id)
    .bind(&suppressing)
    .fetch_one(pool)
    .await?;

    if recent_alert_count > 0 {
        tracing::debug!(
            "[R49] unusual_login alert deduplicated for user {} (30min window, severity={})",
            user_id,
            severity
        );
        return Ok(true);
    }
    Ok(false)
}

/// R49 follow-up helper：檢查 user 是否具系統管理員角色（SYSTEM_ADMIN 或 legacy 'admin'）。
/// 用於 unusual_login 偵測中區分 admin 特殊待遇（B: new_device 仍警報、D: unusual_time 降級）。
async fn is_admin_user(pool: &PgPool, user_id: Uuid) -> bool {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM user_roles ur
            JOIN roles r ON r.id = ur.role_id
            WHERE ur.user_id = $1
              AND r.code IN ($2, $3)
              AND r.is_active = true
        )
        "#,
    )
    .bind(user_id)
    .bind(ROLE_SYSTEM_ADMIN)
    .bind(ROLE_ADMIN_LEGACY)
    .fetch_one(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::error!("is_admin_user query failed: {e}");
        false // DB 錯誤時保守處理：當作非 admin，走一般路徑
    })
}

// ============================================
// Helper Functions
// ============================================

struct DeviceInfo {
    device_type: Option<String>,
    browser: Option<String>,
    os: Option<String>,
}

/// 前一次成功登入的來源，供管理員在警報詳情比對「是不是同一個人換了網路」。
struct PreviousLogin {
    ip: Option<String>,
    created_at: chrono::DateTime<Utc>,
}

/// 查前一次成功登入。**必須在本次事件寫入前呼叫**，否則查到的是本次自己。
///
/// 這是警報上下文的補充資訊，查不到不影響登入紀錄本身，
/// 故 DB 錯誤時記 error 後降級為 None（不阻斷登入流程），與本檔其他 check_* 一致。
async fn find_previous_login(pool: &PgPool, user_id: Uuid) -> Option<PreviousLogin> {
    sqlx::query_as::<_, (Option<String>, chrono::DateTime<Utc>)>(
        r#"
        SELECT host(ip_address), created_at FROM login_events
        WHERE user_id = $1
          AND event_type = 'login_success'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::error!("find_previous_login query failed: {e}");
        None
    })
    .map(|(ip, created_at)| PreviousLogin { ip, created_at })
}

fn parse_user_agent(ua: Option<&str>) -> DeviceInfo {
    let ua = match ua {
        Some(s) => s,
        None => {
            return DeviceInfo {
                device_type: None,
                browser: None,
                os: None,
            }
        }
    };

    // 簡單解析 (可以用更完整的 library 如 woothee)
    let device_type = if ua.contains("Mobile") || ua.contains("Android") {
        Some("mobile".to_string())
    } else if ua.contains("Tablet") || ua.contains("iPad") {
        Some("tablet".to_string())
    } else {
        Some("desktop".to_string())
    };

    let browser = if ua.contains("Chrome") && !ua.contains("Edge") {
        Some("Chrome".to_string())
    } else if ua.contains("Firefox") {
        Some("Firefox".to_string())
    } else if ua.contains("Safari") && !ua.contains("Chrome") {
        Some("Safari".to_string())
    } else if ua.contains("Edge") {
        Some("Edge".to_string())
    } else {
        None
    };

    let os = if ua.contains("Windows") {
        Some("Windows".to_string())
    } else if ua.contains("Mac OS") {
        Some("macOS".to_string())
    } else if ua.contains("Linux") {
        Some("Linux".to_string())
    } else if ua.contains("Android") {
        Some("Android".to_string())
    } else if ua.contains("iOS") || ua.contains("iPhone") {
        Some("iOS".to_string())
    } else {
        None
    };

    DeviceInfo {
        device_type,
        browser,
        os,
    }
}

fn check_unusual_time() -> bool {
    // 台灣時區 UTC+8
    let taiwan_hour = (Utc::now().hour() + 8) % 24;
    is_unusual_hour(taiwan_hour)
}

/// 判斷台灣時區某小時是否為「異常時間」（正常工作時間 07:00–18:00 之外）。
/// 抽為純函式以便單元測試（不依賴 `Utc::now()`）。
/// 07:00 起點：原 08:00–18:00 過窄，把員工 07:xx 早班誤標為異常（2026-07-04 調整）。
/// 18:00 後主要為 admin 系統維護（unusual_time-only 已降為 info，見 create_login_alert）。
fn is_unusual_hour(taiwan_hour: u32) -> bool {
    !(7..18).contains(&taiwan_hour)
}

async fn check_new_device(
    pool: &PgPool,
    user_id: Uuid,
    user_agent: Option<&str>,
    is_admin: bool,
) -> bool {
    let ua = match user_agent {
        Some(s) => s,
        None => return false,
    };

    // R49 follow-up B: 首次登入（過去無任何 login_success event）→ 非異常，跳過。
    // **admin 例外**：admin 帳號最敏感，首次出現新裝置仍視為新裝置告警（避免漏掉帳號被盜場景）。
    if !is_admin {
        let prior_login_count = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*) FROM login_events
            WHERE user_id = $1
              AND event_type = 'login_success'
            "#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map(|r| r.0)
        .unwrap_or(0);
        if prior_login_count == 0 {
            return false;
        }
    }

    // 檢查過去 30 天是否用過這個 user agent
    let count = sqlx::query_as::<_, (i64,)>(
        r#"
        SELECT COUNT(*) FROM login_events
        WHERE user_id = $1
          AND user_agent = $2
          AND event_type = 'login_success'
          AND created_at > NOW() - INTERVAL '30 days'
        "#,
    )
    .bind(user_id)
    .bind(ua)
    .fetch_one(pool)
    .await
    .map(|r| r.0)
    .unwrap_or_else(|e| {
        tracing::error!("check_new_device query failed: {e}");
        0 // DB 錯誤時保守處理：視為新裝置
    });

    count == 0
}

/// 檢查異常地理位置：該使用者過去 30 天從未在此國家登入過 → 標記為異常。
///
/// GeoIP 無法解析（多半是 `geoip/GeoLite2-City.mmdb` 未安裝，服務走降級模式）時
/// **不判定**地點異常。原本的降級方案是退回「IP 完全比對」，但台灣家用寬頻與行動
/// 網路每次撥接都換 IP，等於每個使用者換個網路就被標成「新地點」——警報全是狼來了，
/// 真訊號反而被埋掉；且該路徑產生的描述只寫得出「來自新的 IP 位置」，沒有任何可
/// 判讀的資訊。與其留一個必定誤報的偵測，不如在資料庫到位前明確不判定。
/// 安裝 .mmdb（`scripts/update_geoip.sh`）後自動恢復國家層級比對。
async fn check_unusual_location(pool: &PgPool, user_id: Uuid, geo: &GeoInfo) -> bool {
    let Some(country) = geo.country.as_ref() else {
        return false;
    };

    let count = sqlx::query_as::<_, (i64,)>(
        r#"
        SELECT COUNT(*) FROM login_events
        WHERE user_id = $1
          AND geo_country = $2
          AND event_type = 'login_success'
          AND created_at > NOW() - INTERVAL '30 days'
        "#,
    )
    .bind(user_id)
    .bind(country)
    .fetch_one(pool)
    .await
    .map(|r| r.0)
    .unwrap_or_else(|e| {
        tracing::error!("check_unusual_location country query failed: {e}");
        0
    });

    count == 0
}

async fn check_mass_login(pool: &PgPool, user_id: Uuid) -> bool {
    // 檢查過去 15 分鐘內成功登入次數
    let count = sqlx::query_as::<_, (i64,)>(
        r#"
        SELECT COUNT(*) FROM login_events
        WHERE user_id = $1
          AND event_type = 'login_success'
          AND created_at > NOW() - INTERVAL '15 minutes'
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map(|r| r.0)
    .unwrap_or_else(|e| {
        tracing::error!("check_mass_login query failed: {e}");
        0
    });

    // 如果連同本次（尚未寫入前的查詢）已有 4 次以上，則本次標記為大量登入
    count >= 4
}

async fn check_global_mass_login(pool: &PgPool) -> Result<()> {
    // 檢查過去 5 分鐘內，成功登入的不同帳號數量
    let (count,): (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(DISTINCT user_id) FROM login_events
        WHERE event_type = 'login_success'
          AND created_at > NOW() - INTERVAL '5 minutes'
        "#,
    )
    .fetch_one(pool)
    .await?;

    // 如果 5 分鐘內超過 10 個不同帳號登入，觸發全域警報
    if count >= 10 {
        create_global_mass_login_alert(pool, count).await?;
    }

    Ok(())
}

async fn create_global_mass_login_alert(pool: &PgPool, account_count: i64) -> Result<()> {
    // 檢查是否最近 10 分鐘內已經發過相同的全域警報 (避免洗版)
    let (recent_alert_count,): (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM security_alerts
        WHERE alert_type = 'global_mass_login'
          AND created_at > NOW() - INTERVAL '10 minutes'
          AND status = 'open'
        "#,
    )
    .fetch_one(pool)
    .await?;

    if recent_alert_count > 0 {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO security_alerts (
            id, alert_type, severity, title, description,
            context_data, created_at, updated_at, status
        ) VALUES (
            $1, 'global_mass_login', 'critical',
            '偵測到疑似腳本之多帳號大量登入',
            $2, $3, NOW(), NOW(), 'open'
        )
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(format!(
        "系統偵測到全域在過去 5 分鐘內有 {} 個不同帳號成功登入，疑似為自動化腳本行為。",
        account_count
    ))
    .bind(serde_json::json!({
        "account_count": account_count,
        "time_window_minutes": 5
    }))
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use uuid::Uuid;

    use super::{is_unusual_hour, AnomalyFlags, DeviceInfo, GeoInfo, LoginAnomaly, PreviousLogin};

    #[test]
    fn work_hours_07_to_17_are_normal() {
        // 正常工作時間 07:00–17:59（台灣時區小時 7..=17）不應標為異常
        for h in 7..18 {
            assert!(!is_unusual_hour(h), "台灣 {h} 點應為正常工作時間");
        }
    }

    #[test]
    fn off_hours_are_unusual() {
        // 07:00 前與 18:00 後為異常時間
        for h in [0, 1, 5, 6, 18, 19, 22, 23] {
            assert!(is_unusual_hour(h), "台灣 {h} 點應為異常時間");
        }
    }

    fn device() -> DeviceInfo {
        DeviceInfo {
            device_type: Some("desktop".to_string()),
            browser: Some("Chrome".to_string()),
            os: Some("Windows".to_string()),
        }
    }

    fn anomaly<'a>(
        geo: &'a GeoInfo,
        device: &'a DeviceInfo,
        previous_login: Option<&'a PreviousLogin>,
        is_admin: bool,
        flags: AnomalyFlags,
    ) -> LoginAnomaly<'a> {
        LoginAnomaly {
            user_id: Uuid::nil(),
            email: "vet@example.com",
            ip: Some("203.0.113.9"),
            geo,
            device,
            previous_login,
            is_admin,
            flags,
        }
    }

    const NEW_DEVICE_ONLY: AnomalyFlags = AnomalyFlags {
        unusual_time: false,
        unusual_location: false,
        new_device: true,
        mass_login: false,
    };

    /// 本 PR 的核心：警報要帶得出「是哪個 IP」，否則管理員無從判斷是本人換網路
    /// 還是他人盜用。previous_login_* 是拿來對照的另一半。
    #[test]
    fn context_data_carries_source_ip_and_previous_login() {
        let geo = GeoInfo::default();
        let device = device();
        let previous = PreviousLogin {
            ip: Some("198.51.100.4".to_string()),
            created_at: chrono::Utc
                .with_ymd_and_hms(2026, 7, 29, 1, 2, 3)
                .single()
                .expect("fixed timestamp"),
        };
        let context =
            anomaly(&geo, &device, Some(&previous), false, NEW_DEVICE_ONLY).context_data();

        assert_eq!(context["ip"], "203.0.113.9");
        assert_eq!(context["email"], "vet@example.com");
        assert_eq!(context["triggers"][0], "new_device");
        assert_eq!(context["browser"], "Chrome");
        assert_eq!(context["os"], "Windows");
        assert_eq!(context["previous_login_ip"], "198.51.100.4");
        assert!(
            context.get("previous_login_at").is_some(),
            "前次登入時間要一併帶上，只有 IP 無法判斷是多久以前的事"
        );
    }

    /// GeoIP 未安裝 .mmdb 時 geo 全空——這些欄位要整個省略，
    /// 而不是塞一堆 null 讓詳情頁看起來像壞掉。
    #[test]
    fn context_data_omits_absent_geo_fields() {
        let geo = GeoInfo::default();
        let device = device();
        let context = anomaly(&geo, &device, None, false, NEW_DEVICE_ONLY).context_data();

        assert!(context.get("geo_country").is_none());
        assert!(context.get("geo_city").is_none());
        assert!(context.get("previous_login_ip").is_none());
        // ip 為 null 本身是要看的訊號（反向代理沒帶 XFF），故不省略
        assert!(context.get("ip").is_some());
    }

    /// #395 的行為不能被這次重構弄丟：admin 只有「非工作時間」觸發才降 info，
    /// 一旦同時有其他訊號（新裝置＝可能被盜）就必須維持 warning。
    #[test]
    fn admin_unusual_time_alone_is_info_but_other_signals_stay_warning() {
        let geo = GeoInfo::default();
        let device = device();
        let time_only = AnomalyFlags {
            unusual_time: true,
            unusual_location: false,
            new_device: false,
            mass_login: false,
        };
        let time_and_device = AnomalyFlags {
            new_device: true,
            ..time_only
        };

        assert_eq!(
            anomaly(&geo, &device, None, true, time_only).severity(),
            "info"
        );
        assert_eq!(
            anomaly(&geo, &device, None, true, time_and_device).severity(),
            "warning"
        );
        assert_eq!(
            anomaly(&geo, &device, None, false, time_only).severity(),
            "warning",
            "非 admin 的非工作時間登入不降級"
        );
    }
}
