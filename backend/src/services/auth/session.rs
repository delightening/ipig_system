use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, Header};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    config::Config,
    constants::{
        REFRESH_TOKEN_REUSE_RACE_WINDOW_SECS, REFRESH_TOKEN_REUSE_STALE_THRESHOLD_SECS,
        REFRESH_TOKEN_REVOKED_ADMIN_LOGOUT, REFRESH_TOKEN_REVOKED_IDLE_TIMEOUT,
        REFRESH_TOKEN_REVOKED_NORMAL_ROTATION, REFRESH_TOKEN_REVOKED_PASSWORD_CHANGED,
        REFRESH_TOKEN_REVOKED_REUSE_DETECTED, SEC_EVENT_REFRESH_TOKEN_REUSE,
        SESSION_ERROR_IDLE_TIMEOUT,
    },
    middleware::Claims,
    models::{LoginResponse, RefreshToken, User, UserResponse},
    AppError, Result,
};

use super::AuthService;

impl AuthService {
    /// 刷新 Token（含 R35-15 reuse detection）
    ///
    /// 流程：
    /// 1. SELECT 不加 revoked_at / expires_at filter，拿到 token row 後在 app 層判斷狀態
    /// 2. row 不存在 → 假 token，回 Invalid
    /// 3. row 存在但 revoked_at IS NOT NULL → **reuse detected**，
    ///    觸發整 family revoke + security_alert + 回 Invalid
    /// 4. row 存在但 expires_at <= NOW() → 過期，回 Invalid（無 alert）
    /// 5. row 存在且 active → 正常 rotation：
    ///    - 標記舊 token revoked_reason='normal_rotation'
    ///    - 在同 family_id 下發新 token
    pub async fn refresh_token(
        pool: &PgPool,
        config: &Config,
        refresh_token: &str,
        client_ip: &str,
        user_agent: Option<&str>,
    ) -> Result<LoginResponse> {
        let token_hash = Self::hash_token(refresh_token);

        // R35-15: 不加 filter — 讓 app 層分辨 active / revoked / expired / not-found
        let token_record =
            sqlx::query_as::<_, RefreshToken>("SELECT * FROM refresh_tokens WHERE token_hash = $1")
                .bind(&token_hash)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| AppError::Validation("Invalid refresh token".to_string()))?;

        // R35-15 / R46-1: reuse detection — 已撤銷 token 再次出現。
        // handle_refresh_token_reuse 入口會用 rotated_at 判定 race window，
        // 若為 race 則僅 tracing warn，不 revoke family / 不寫 alert。
        if token_record.revoked_at.is_some() {
            Self::handle_refresh_token_reuse(pool, &token_record, client_ip, user_agent).await?;
            return Err(AppError::Validation("Invalid refresh token".to_string()));
        }

        if token_record.expires_at <= Utc::now() {
            return Err(AppError::Validation("Invalid refresh token".to_string()));
        }

        // R41-1: 閒置 session 強制 revoke
        Self::reject_if_idle_timeout(pool, config, &token_record).await?;

        // 查詢用戶
        let user =
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 AND is_active = true")
                .bind(token_record.user_id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| AppError::Validation("User not found or inactive".to_string()))?;

        // R35-15 / R46-1/2: 標記舊 token revoked + 原因 + rotation metadata
        // （rotated_at / last_ip / last_user_agent 用於後續 reuse detection 啟發式）
        sqlx::query(
            "UPDATE refresh_tokens
             SET revoked_at = NOW(),
                 revoked_reason = $2,
                 rotated_at = NOW(),
                 last_ip = $3,
                 last_user_agent = $4
             WHERE id = $1",
        )
        .bind(token_record.id)
        .bind(REFRESH_TOKEN_REVOKED_NORMAL_ROTATION)
        .bind(client_ip)
        .bind(user_agent)
        .execute(pool)
        .await?;

        // 獲取角色和權限
        let (roles, permissions) = Self::get_user_roles_permissions(pool, user.id).await?;

        // 生成新的 tokens — R35-15: 新 token 加入既有 family，rotation 鏈一致
        let (access_token, expires_in) =
            Self::generate_access_token(config, &user, &roles, &permissions, None)?;
        let new_refresh_token = Self::generate_refresh_token_with_family(
            pool,
            user.id,
            config,
            Some(token_record.family_id),
        )
        .await?;

        // observability for cross-tab refresh race investigation —
        // 每次成功 rotation log 一行 INFO，便於 Loki / docker logs grep 觀察
        // 「同 family 在短時間內有幾次成功 rotation」「navigator.locks 是否真把
        // 跨 tab race 壓下來」等問題。每位 user 每 60 分鐘約 1 行，量極小。
        // 欄位名稱對齊 refresh_token_reuse_echo（line ~275），方便 LogQL 用
        // `token_id="<UUID>"` 追蹤完整生命週期：rotated → 後續若 echo / reuse。
        tracing::info!(
            event = "refresh_token_rotated",
            user_id = %user.id,
            family_id = %token_record.family_id,
            token_id = %token_record.id,
            client_ip = client_ip,
            "refresh token rotated successfully"
        );

        Ok(LoginResponse {
            access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
            user: UserResponse::from_user(&user, roles, permissions),
            must_change_password: user.must_change_password,
        })
    }

    /// R41-1: 閒置 session 檢查（refresh 流程內呼叫）。
    ///
    /// 對齊 NICS 附表十普級「存取控制 / 帳號管理」閒置鎖定要求。SPA 的 axios
    /// interceptor 在 access token 即將過期時觸發 refresh，故 refresh 滾動是
    /// 「使用者仍在用」的明確指標。`token_record.last_used_at` 距今超過
    /// `config.auth_idle_timeout_minutes` → 拒絕 + revoke。
    ///
    /// **已知限制**：access token TTL 大於 idle window 時（即 `jwt_expiration_seconds
    /// / 60 > auth_idle_timeout_minutes`），背景 polling 在 token 過期前不會觸發 refresh，
    /// 此函式只能在「下一次 refresh 嘗試」時才檢測到閒置（最遲於 access token 過期當下）。
    /// Config 啟動時的 sanity warning 於 `config.rs::Config::warn_if_idle_window_unusable` 發出。
    async fn reject_if_idle_timeout(
        pool: &PgPool,
        config: &Config,
        token_record: &RefreshToken,
    ) -> Result<()> {
        let idle_threshold = Duration::minutes(config.auth_idle_timeout_minutes);
        let elapsed = Utc::now() - token_record.last_used_at;
        if elapsed <= idle_threshold {
            return Ok(());
        }
        // 同步撤銷該 token，避免 attacker 拿到舊 token 後在閒置 window 後仍能用
        sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = NOW(), revoked_reason = $2 WHERE id = $1",
        )
        .bind(token_record.id)
        .bind(REFRESH_TOKEN_REVOKED_IDLE_TIMEOUT)
        .execute(pool)
        .await?;
        tracing::info!(
            user_id = %token_record.user_id,
            idle_minutes = elapsed.num_minutes(),
            threshold_minutes = config.auth_idle_timeout_minutes,
            "[R41-1] refresh rejected: session idle timeout"
        );
        Err(AppError::InvalidCredentials(
            SESSION_ERROR_IDLE_TIMEOUT.to_string(),
        ))
    }

    /// R35-15 / R46: refresh token reuse 處理。
    ///
    /// 三階段啟發式（由寬到嚴）：
    ///
    /// 1. **Race window**（R46-1）：`rotated_at` 距今 ≤ `REFRESH_TOKEN_REUSE_RACE_WINDOW_SECS`
    ///    視為併發 race（多分頁、行動斷網重試、雙擊），僅 `tracing::warn`，**不 revoke
    ///    family、不寫 alert**，caller 仍回 `AppError::Validation` 讓 client 重試（5 秒內
    ///    重試會用到 cookie 中已換的新 token）。
    ///
    /// 2. **IP/UA 比對**（R46-2）：超過 race window 後比對 reused request 的 IP/UA 與
    ///    token 上次 rotation 時的 IP/UA。完全相同 → severity 降為 `warning`（疑似
    ///    browser bug，token 未離開使用者裝置）；任一不同或缺資料 → 維持 `critical`
    ///    （疑似 token 外流，fail-safe）。
    ///
    /// 3. **真 reuse**：整 family revoke + 寫 security_alert，context_data 含 username /
    ///    last_login_ip / last_user_agent / reused_ip / reused_user_agent /
    ///    time_since_rotation_secs（R46-4/5）。
    ///
    /// 兩寫入包同一 transaction：security_alert insert 失敗時 family revoke 一併 rollback，
    /// 避免「整 family 撤了但無告警」靜默漏洞（fail-loud — 下次 reuse 時會重試一輪）。
    async fn handle_refresh_token_reuse(
        pool: &PgPool,
        token: &RefreshToken,
        client_ip: &str,
        user_agent: Option<&str>,
    ) -> Result<()> {
        // R46-1: race window — race_window 秒內的 reuse 視為併發，不觸發告警。
        // 用 abs() 對抗 app / DB 之間可能的 clock drift（負 elapsed 不該誤判為真 reuse）。
        //
        // 2026-05-20: rotated_at 可能為 NULL（family revoke 而非 normal rotation
        // 撤銷的 token，UPDATE 沒設 rotated_at — 見 below 已修正，但歷史資料仍
        // 存在 NULL）。fallback 到 revoked_at 作為 race anchor，確保 family-revoke
        // 場景的 race window 也有效。
        let race_anchor = token.rotated_at.or(token.revoked_at);
        if let Some(anchor) = race_anchor {
            let elapsed_secs = (Utc::now() - anchor).num_seconds();
            if elapsed_secs.abs() <= REFRESH_TOKEN_REUSE_RACE_WINDOW_SECS {
                tracing::warn!(
                    event = "refresh_token_reuse_race_window",
                    user_id = %token.user_id,
                    family_id = %token.family_id,
                    elapsed_secs = elapsed_secs,
                    client_ip = client_ip,
                    "refresh token reuse within race window — concurrent rotation, no family revoke"
                );
                return Ok(());
            }
        }

        // R46-2: severity 啟發式 — 比對 reused 與 last rotation 的 IP/UA
        let same_ip = token.last_ip.as_deref() == Some(client_ip);
        let same_ua = match (token.last_user_agent.as_deref(), user_agent) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        };
        // R46-3: stale-tab heuristic — pre-R46-2 legacy token 重用降為 warning。
        let secs_since_revoke = token.revoked_at.map(|r| (Utc::now() - r).num_seconds());
        let severity = classify_reuse_severity(
            same_ip,
            same_ua,
            token.last_ip.is_none() && token.last_user_agent.is_none(),
            secs_since_revoke,
        );

        // R46-4: 拿 username / display_name 讓告警可讀
        let user_info: Option<(String, String)> =
            sqlx::query_as("SELECT email, display_name FROM users WHERE id = $1")
                .bind(token.user_id)
                .fetch_optional(pool)
                .await?;
        let (username, display_name) =
            user_info.unwrap_or_else(|| ("(unknown)".to_string(), "(unknown)".to_string()));

        // .max(0) 對抗 clock drift — alert dialog 顯示負秒會混淆操作員
        let time_since_rotation_secs = token
            .rotated_at
            .map(|t| (Utc::now() - t).num_seconds().max(0));

        let mut tx = pool.begin().await?;

        // 2026-05-20: COALESCE 補 rotated_at — family revoke 場景之前漏設此欄，
        // 導致後續同 token 重用時 race window check 跳過（rotated_at=None），
        // 每次 echo 都被當作真實 reuse 寫 critical alert，灌爆 security_alerts。
        // 用 COALESCE 保留 normal rotation 已設的值，僅補 NULL 案例。
        let revoked = sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = NOW(),
                revoked_reason = $2,
                rotated_at = COALESCE(rotated_at, NOW())
            WHERE family_id = $1 AND revoked_at IS NULL
            "#,
        )
        .bind(token.family_id)
        .bind(REFRESH_TOKEN_REVOKED_REUSE_DETECTED)
        .execute(&mut *tx)
        .await?;

        // 2026-05-20: 0 row UPDATE = family 已死透（dead token echo）。停止寫 alert
        // 避免「stale browser cookie 反覆敲」的 case 灌爆 security_alerts。
        // 仍 debug log 留 trace 給 forensic 分析。
        if revoked.rows_affected() == 0 {
            tx.rollback().await?;
            tracing::debug!(
                event = "refresh_token_reuse_echo",
                user_id = %token.user_id,
                family_id = %token.family_id,
                token_id = %token.id,
                client_ip = client_ip,
                "stale token from dead family reused — silent echo (no new alert)"
            );
            return Ok(());
        }

        // R46-4/5: description 含 display_name + email；context_data 補完整上下文
        let description = format!(
            "User {} ({}) 的 refresh token 在已撤銷後再次使用，整 family ({}) 已撤銷",
            display_name, username, token.family_id
        );

        sqlx::query(
            r#"
            INSERT INTO security_alerts (
                alert_type, severity, title, description, user_id, context_data, status
            ) VALUES ($1, $2, $3, $4, $5, $6, 'open')
            "#,
        )
        .bind(SEC_EVENT_REFRESH_TOKEN_REUSE)
        .bind(severity)
        .bind("Refresh token 重用偵測 — 疑似 token 洩漏")
        .bind(description)
        .bind(token.user_id)
        .bind(serde_json::json!({
            "family_id": token.family_id,
            "reused_token_id": token.id,
            "revoked_count": revoked.rows_affected(),
            "username": username,
            "display_name": display_name,
            "last_login_ip": token.last_ip,
            "last_user_agent": token.last_user_agent,
            "reused_ip": client_ip,
            "reused_user_agent": user_agent,
            "time_since_rotation_secs": time_since_rotation_secs,
            "same_ip": same_ip,
            "same_user_agent": same_ua,
        }))
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // log 放 commit 後 — commit 失敗時不該記成功
        tracing::warn!(
            event = SEC_EVENT_REFRESH_TOKEN_REUSE,
            severity = severity,
            user_id = %token.user_id,
            family_id = %token.family_id,
            revoked_count = revoked.rows_affected(),
            same_ip = same_ip,
            same_user_agent = same_ua,
            time_since_rotation_secs = time_since_rotation_secs,
            "refresh token reuse detected — revoked entire family"
        );

        Ok(())
    }

    /// 模擬登入（管理員專用）
    pub async fn impersonate(
        pool: &PgPool,
        config: &Config,
        admin_user_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<LoginResponse> {
        // 查詢目標用戶
        let user =
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 AND is_active = true")
                .bind(target_user_id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| {
                    AppError::Validation("Target user not found or inactive".to_string())
                })?;

        // 獲取角色和權限
        let (roles, permissions) = Self::get_user_roles_permissions(pool, user.id).await?;

        // SEC-PRIV: 禁止模擬登入為其他管理員，防止管理員間的橫向提權
        let is_target_admin = roles.iter().any(|r| {
            r == crate::constants::ROLE_SYSTEM_ADMIN || r == crate::constants::ROLE_ADMIN_LEGACY
        });
        if is_target_admin {
            return Err(AppError::BusinessRule(
                "Cannot impersonate other administrators".to_string(),
            ));
        }

        // 生成 JWT（包含 impersonated_by 資訊，有效期受限為 30 分鐘）
        let (access_token, expires_in) =
            Self::generate_access_token(config, &user, &roles, &permissions, Some(admin_user_id))?;

        // MED-03: 模擬登入不為目標用戶建立 refresh token。
        // 原本做法會：1) 消耗目標用戶的 session 配額（SEC-28）；
        // 2) 產生目標用戶不知情的 session 記錄；
        // 3) 若管理員離開後，refresh token 可能被截取。
        // 模擬登入使用短效 access-only session（30 分鐘無法 refresh），
        // cookie builder 會跳過設定空的 refresh_token cookie。
        let refresh_token = String::new();

        // 審計日誌：記錄模擬登入行為（SEC-11）
        tracing::warn!(
            "[Security] 模擬登入 - 管理員 {} 模擬登入為用戶 {} ({})",
            admin_user_id,
            user.email,
            user.id
        );

        Ok(LoginResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
            user: UserResponse::from_user(&user, roles, permissions),
            must_change_password: user.must_change_password,
        })
    }

    /// 停止模擬登入，恢復管理員身分
    /// 查詢管理員帳號並簽發正常 token（不含 impersonated_by）
    pub async fn impersonate_restore(
        pool: &PgPool,
        config: &Config,
        admin_user_id: Uuid,
    ) -> Result<LoginResponse> {
        // 查詢管理員帳號
        let user =
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 AND is_active = true")
                .bind(admin_user_id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| {
                    AppError::Validation("Admin user not found or inactive".to_string())
                })?;

        // 獲取管理員的角色和權限
        let (roles, permissions) = Self::get_user_roles_permissions(pool, user.id).await?;

        // 生成正常的 JWT（不含 impersonated_by，使用正常有效期）
        let (access_token, expires_in) =
            Self::generate_access_token(config, &user, &roles, &permissions, None)?;

        // 生成 Refresh Token
        let refresh_token = Self::generate_refresh_token(pool, user.id, config).await?;

        Ok(LoginResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
            user: UserResponse::from_user(&user, roles, permissions),
            must_change_password: user.must_change_password,
        })
    }

    /// 生成 Access Token
    pub(super) fn generate_access_token(
        config: &Config,
        user: &User,
        roles: &[String],
        _permissions: &[String],
        impersonated_by: Option<Uuid>,
    ) -> Result<(String, i64)> {
        let now = Utc::now();
        // 模擬登入時限制有效期為 30 分鐘（SEC-11）
        // 否則使用 jwt_expiration_seconds（SEC-25，預設 15 分鐘）
        let expires_in = if impersonated_by.is_some() {
            1800 // 30 分鐘
        } else {
            config.jwt_expiration_seconds
        };
        let exp = now + Duration::seconds(expires_in);

        // Cookie 限制：Set-Cookie 的 name+value 須 ≤ 4096 字元。將 permissions 陣列
        // 嵌入 JWT 會使 cookie 超過瀏覽器硬限制（4096 bytes），導致 Set-Cookie 靜默丟棄、
        // 所有後續請求回傳 401。故一律省略 permissions，改由 auth_middleware 從資料庫
        // 動態載入並注入 CurrentUser。Admin 的 has_permission() 直接回傳 true，不需載入。
        let claims_permissions: Vec<String> = vec![];

        let claims = Claims {
            sub: user.id,
            email: user.email.clone(),
            roles: roles.to_vec(),
            permissions: claims_permissions,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            impersonated_by,
            // H3: 明確設定 issuer/audience 供驗證端校驗
            iss: "ipig-backend".to_string(),
            aud: "ipig-system".to_string(),
        };

        let token = encode(
            &Header::new(Algorithm::ES256),
            &claims,
            &config.jwt_keys.encoding,
        )
        .map_err(|e| AppError::Internal(format!("Failed to create token: {}", e)))?;

        Ok((token, expires_in))
    }

    /// 生成 Refresh Token（新登入 — 建新 family）
    pub(super) async fn generate_refresh_token(
        pool: &PgPool,
        user_id: Uuid,
        config: &Config,
    ) -> Result<String> {
        Self::generate_refresh_token_with_family(pool, user_id, config, None).await
    }

    /// R35-15: 生成 Refresh Token（rotation — 沿用既有 family；新登入 — 傳 None 建新 family）
    pub(super) async fn generate_refresh_token_with_family(
        pool: &PgPool,
        user_id: Uuid,
        config: &Config,
        family_id: Option<Uuid>,
    ) -> Result<String> {
        // SEC-34: CSPRNG 產生 256-bit 高熵 Refresh Token
        use rand::Rng;
        let mut token_bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut token_bytes);
        let token = hex::encode(token_bytes);
        let token_hash = Self::hash_token(&token);
        let expires_at = Utc::now() + Duration::days(config.jwt_refresh_expiration_days);
        let token_id = Uuid::new_v4();
        // R35-15: 新登入時 family_id = token_id（自成一家），rotation 時延續既有 family
        let family = family_id.unwrap_or(token_id);

        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, created_at, family_id)
            VALUES ($1, $2, $3, $4, NOW(), $5)
            "#,
        )
        .bind(token_id)
        .bind(user_id)
        .bind(&token_hash)
        .bind(expires_at)
        .bind(family)
        .execute(pool)
        .await?;

        Ok(token)
    }

    /// Hash token for storage（使用 SHA-256 密碼學雜湊）
    ///
    /// pub(crate)：R30-27c signature_bridge service 也需用同一 pattern hash mobile_token。
    pub(crate) fn hash_token(token: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// 撤銷用戶的所有有效 refresh tokens（密碼變更/重設/帳號停用後強制重新登入）
    /// BIZ-16: 提升為 pub 供 handler 層帳號停用時呼叫
    pub async fn revoke_all_user_tokens(pool: &PgPool, user_id: Uuid) -> Result<()> {
        Self::revoke_user_refresh_tokens(pool, user_id).await
    }

    /// Tx 版本的 [`revoke_all_user_tokens`]：供 force-logout 在同一 tx 內原子撤銷
    /// （session 停用 + `tokens_valid_after` + refresh 撤銷一次 commit），reason 記 ADMIN_LOGOUT。
    pub async fn revoke_all_user_tokens_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_id: Uuid,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = NOW(), revoked_reason = $2 WHERE user_id = $1 AND revoked_at IS NULL"
        )
        .bind(user_id)
        .bind(REFRESH_TOKEN_REVOKED_ADMIN_LOGOUT)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub(super) async fn revoke_user_refresh_tokens(pool: &PgPool, user_id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = NOW(), revoked_reason = $2 WHERE user_id = $1 AND revoked_at IS NULL"
        )
        .bind(user_id)
        .bind(REFRESH_TOKEN_REVOKED_PASSWORD_CHANGED)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Tx 版本：與 `revoke_user_refresh_tokens` 同邏輯，供 Service-driven 在同
    /// tx 內同時更新 password / revoke tokens / 寫 audit。
    pub(super) async fn revoke_user_refresh_tokens_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_id: Uuid,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = NOW(), revoked_reason = $2 WHERE user_id = $1 AND revoked_at IS NULL"
        )
        .bind(user_id)
        .bind(REFRESH_TOKEN_REVOKED_PASSWORD_CHANGED)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// 更新用戶密碼 hash（含 must_change_password 標記）。供 Service-driven 在同 tx 內使用。
    pub(super) async fn update_password_in_db_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_id: Uuid,
        password_hash: &str,
        must_change_password: bool,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE users SET password_hash = $1, must_change_password = $2, updated_at = NOW() WHERE id = $3"
        )
        .bind(password_hash)
        .bind(must_change_password)
        .bind(user_id)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }
}

// security_alerts.severity 字面值（與其他 callsite — login_tracker / honeypot /
// rate_limiter / response_logger / audit_chain_verify — 共用 schema，
// 但目前 codebase 仍多處散落字面值；本檔先抽 const，全域統一留 backlog）
const ALERT_SEVERITY_WARNING: &str = "warning";
const ALERT_SEVERITY_CRITICAL: &str = "critical";

/// R46-3: refresh token reuse severity 分類（純函式，便於測試）。
///
/// - same IP + same UA：browser bug / 雙擊 → warning
/// - baseline 缺資料（pre-R46-2 legacy）+ stale reuse（> threshold）→ warning
/// - 其他：fail-safe critical
fn classify_reuse_severity(
    same_ip: bool,
    same_ua: bool,
    baseline_missing: bool,
    secs_since_revoke: Option<i64>,
) -> &'static str {
    let stale_reuse = secs_since_revoke
        .map(|s| s > REFRESH_TOKEN_REUSE_STALE_THRESHOLD_SECS)
        .unwrap_or(false);
    if (same_ip && same_ua) || (baseline_missing && stale_reuse) {
        ALERT_SEVERITY_WARNING
    } else {
        ALERT_SEVERITY_CRITICAL
    }
}

#[cfg(test)]
mod tests {
    use super::classify_reuse_severity;
    use crate::constants::REFRESH_TOKEN_REUSE_STALE_THRESHOLD_SECS;

    #[test]
    fn matched_baseline_returns_warning() {
        assert_eq!(
            classify_reuse_severity(true, true, false, Some(10)),
            "warning"
        );
    }

    #[test]
    fn different_baseline_returns_critical() {
        assert_eq!(
            classify_reuse_severity(false, true, false, Some(10)),
            "critical"
        );
        assert_eq!(
            classify_reuse_severity(true, false, false, Some(10)),
            "critical"
        );
    }

    #[test]
    fn missing_baseline_recent_reuse_stays_critical() {
        assert_eq!(
            classify_reuse_severity(
                false,
                false,
                true,
                Some(REFRESH_TOKEN_REUSE_STALE_THRESHOLD_SECS - 1)
            ),
            "critical"
        );
    }

    #[test]
    fn missing_baseline_stale_reuse_downgrades_to_warning() {
        assert_eq!(
            classify_reuse_severity(
                false,
                false,
                true,
                Some(REFRESH_TOKEN_REUSE_STALE_THRESHOLD_SECS + 1)
            ),
            "warning"
        );
    }

    #[test]
    fn missing_baseline_no_revoke_time_stays_critical() {
        assert_eq!(
            classify_reuse_severity(false, false, true, None),
            "critical"
        );
    }

    #[test]
    fn partial_baseline_does_not_qualify_as_missing() {
        assert_eq!(
            classify_reuse_severity(
                false,
                false,
                false,
                Some(REFRESH_TOKEN_REUSE_STALE_THRESHOLD_SECS * 100)
            ),
            "critical"
        );
    }
}
