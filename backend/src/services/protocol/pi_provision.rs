//! PI 帳號開通（補登歷史計畫的外部 PI）。
//!
//! 有權者（建立者/SD/admin）為計畫的外部 PI（`basic.pi`）開通系統帳號並 relink
//! `protocols.pi_user_id`，**但不寄信**；寄送「設定密碼」開通信須由 admin 核准
//! （見 `pi_account_invites` 與 admin 核准端點）。

use sqlx::PgPool;
use uuid::Uuid;

use super::ProtocolService;
use crate::{
    config::Config,
    middleware::{ActorContext, SYSTEM_USER_ID},
    models::{audit_diff::DataDiff, Protocol, User},
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService, AuthService, EmailService,
    },
    AppError, Result,
};

impl ProtocolService {
    /// 核准並寄送 PI 開通（設定密碼）信。R71-2：由 handler 下沉至此，補
    /// tx + FOR UPDATE 並發冪等 + `log_activity_tx` 稽核（寄開通信為敏感動作）。
    ///
    /// 做法 A（email-first，失敗 rollback）：token 與前置讀取於 tx 外完成，tx 內鎖列 +
    /// 標記 sent + 稽核，寄信成功才 commit；寄信失敗 → rollback（保留 pending 可重試）。
    /// `forgot_password` 走 pool 取連線，**必須在 tx 外**呼叫，否則持 tx 連線 + 行鎖時再向
    /// pool 取第二條連線會在 pool 耗盡時死結（gemini #726）。回傳收件 email。
    pub async fn approve_send_pi_invite(
        pool: &PgPool,
        actor: &ActorContext,
        config: &Config,
        invite_id: Uuid,
    ) -> Result<String> {
        let user = actor.require_user()?;

        // tx 外：取 email + 驗 pending；產 reset token（None 視為失敗 —— 不可寄信卻標 sent）
        let email = Self::fetch_pending_pi_invite_email(pool, invite_id).await?;
        let (_uid, token) = AuthService::forgot_password(pool, &email)
            .await?
            .ok_or_else(|| AppError::Internal("PI 帳號無法產生重設憑證，請稍後再試".into()))?;

        let mut tx = pool.begin().await?;
        Self::mark_pi_invite_sent_tx(&mut tx, actor, user.id, invite_id, &email).await?;

        // 寄信（SMTP，不取 DB 連線）；失敗 → ? 傳播 → tx rollback（保留 pending 可重試）。
        EmailService::send_password_reset_email(config, &email, &email, &token)
            .await
            .map_err(|e| {
                tracing::error!("寄送 PI 開通信失敗 {}: {}", email, e);
                AppError::Internal("寄送 PI 開通信失敗，請稍後再試".into())
            })?;

        tx.commit().await?;
        Ok(email)
    }

    /// tx 外非鎖讀取：回傳仍 pending 且帳號 active 未刪除的開通信收件 email。
    async fn fetch_pending_pi_invite_email(pool: &PgPool, invite_id: Uuid) -> Result<String> {
        let pre: Option<(String, String)> = sqlx::query_as(
            r#"SELECT i.email, i.status
               FROM pi_account_invites i
               JOIN users u ON i.pi_user_id = u.id
               WHERE i.id = $1 AND u.is_active = true AND u.deleted_at IS NULL"#,
        )
        .bind(invite_id)
        .fetch_optional(pool)
        .await?;
        let (email, status) = pre.ok_or_else(|| {
            AppError::NotFound("找不到可寄送的 PI 開通信（帳號可能已停用或刪除）".into())
        })?;
        if status != "pending" {
            return Err(AppError::BusinessRule(
                "此開通信已寄送或非待核准狀態".into(),
            ));
        }
        Ok(email)
    }

    /// tx 內：鎖列 + 權威重驗 pending（並發冪等）→ 標記 sent → in-tx 稽核。
    async fn mark_pi_invite_sent_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &ActorContext,
        reviewer_id: Uuid,
        invite_id: Uuid,
        email: &str,
    ) -> Result<()> {
        let locked: Option<String> =
            sqlx::query_scalar("SELECT status FROM pi_account_invites WHERE id = $1 FOR UPDATE")
                .bind(invite_id)
                .fetch_optional(&mut **tx)
                .await?;
        match locked.as_deref() {
            Some("pending") => {}
            Some(_) => {
                return Err(AppError::BusinessRule(
                    "此開通信已寄送或非待核准狀態".into(),
                ))
            }
            None => return Err(AppError::NotFound("找不到可寄送的 PI 開通信".into())),
        }

        sqlx::query(
            "UPDATE pi_account_invites SET status = 'sent', approved_by = $1, sent_at = NOW(), updated_at = NOW() WHERE id = $2 AND status = 'pending'",
        )
        .bind(reviewer_id)
        .bind(invite_id)
        .execute(&mut **tx)
        .await?;

        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "PROTOCOL",
                event_type: "PI_INVITE_SEND",
                entity: Some(AuditEntity::new("pi_account_invite", invite_id, email)),
                data_diff: None,
                request_context: None,
            },
        )
        .await?;
        Ok(())
    }

    /// 開通計畫外部 PI 帳號 + relink；建立待核准開通信（status=pending，不寄信）。
    /// 回傳 (pi_user_id, email, created_new_account)。
    pub async fn provision_pi_account(
        pool: &PgPool,
        actor: &ActorContext,
        protocol_id: Uuid,
    ) -> Result<(Uuid, String, bool)> {
        let provisioned_by = match actor {
            ActorContext::User(u) => u.id,
            ActorContext::System { .. } => SYSTEM_USER_ID,
            ActorContext::Anonymous => {
                return Err(AppError::Forbidden(
                    "開通 PI 帳號須由已登入使用者或系統觸發".into(),
                ));
            }
        };

        // 隨機密碼 Argon2 雜湊放在 tx / FOR UPDATE 鎖之前計算（CPU 密集 ~數百 ms），
        // 避免在持有 protocol 行鎖期間執行而拉長鎖持有時間（gemini review）。
        // relink 路徑用不到此值，屬可接受的小額浪費。
        let random_pw = AuthService::hash_password(&Uuid::new_v4().to_string())?;

        let mut tx = pool.begin().await?;

        let before: Protocol =
            sqlx::query_as::<_, Protocol>("SELECT * FROM protocols WHERE id = $1 FOR UPDATE")
                .bind(protocol_id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Protocol not found".to_string()))?;

        // 取研究資料 basic.pi（客人）name/email
        let (email, display_name) = {
            let pi = before
                .working_content
                .as_ref()
                .and_then(|w| w.get("basic"))
                .and_then(|b| b.get("pi"));
            let email = pi
                .and_then(|p| p.get("email"))
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| AppError::BusinessRule("此計畫的 PI 無 email，無法開通帳號".into()))?
                .to_string();
            let display_name = pi
                .and_then(|p| p.get("name"))
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or(email.as_str())
                .to_string();
            (email, display_name)
        };

        // email 已存在使用者 → relink 既有；否則建新帳號（隨機密碼、待設密碼、外部）。
        // 排除軟刪除帳號；同時帶出 is_internal 與「是否具非 PI 高權角色」以做 relink 守衛。
        let existing: Option<(Uuid, bool, bool)> = sqlx::query_as(
            r#"
            SELECT u.id, u.is_internal,
                   EXISTS(
                       SELECT 1 FROM user_roles ur
                       JOIN roles r ON ur.role_id = r.id
                       WHERE ur.user_id = u.id AND r.code <> 'PI'
                   ) AS has_privileged_role
            FROM users u
            WHERE LOWER(u.email) = LOWER($1) AND u.deleted_at IS NULL
            "#,
        )
        .bind(&email)
        .fetch_optional(&mut *tx)
        .await?;

        let (pi_user_id, created_new_account) = match existing {
            Some((uid, is_internal, has_privileged_role)) => {
                // H3：不可把外部 PI 掛到內部或具管理權限的既有帳號
                // （否則 admin 核准後會對該帳號寄「設定密碼」連結，且計畫 PI 被誤掛高權帳號）。
                if is_internal || has_privileged_role {
                    return Err(AppError::BusinessRule(
                        "此 Email 已對應內部或具管理權限的帳號，不可作為外部 PI 開通；\
                         請改用其他 Email 或聯絡系統管理員"
                            .into(),
                    ));
                }
                (uid, false)
            }
            None => {
                // RETURNING * 直接取回 User，供下方 USER_CREATE 稽核使用
                let new_user: User = sqlx::query_as::<_, User>(
                    r#"INSERT INTO users (id, email, password_hash, display_name,
                            is_internal, is_active, must_change_password, created_at, updated_at)
                       VALUES ($1, $2, $3, $4, false, true, true, NOW(), NOW())
                       RETURNING *"#,
                )
                .bind(Uuid::new_v4())
                .bind(&email)
                .bind(&random_pw)
                .bind(&display_name)
                .fetch_one(&mut *tx)
                .await?;

                // H2：補 USER_CREATE 稽核。此路徑未走 UserService::create，
                // 否則新建的可登入帳號在 user 稽核軸上會完全隱形。
                AuditService::log_activity_tx(
                    &mut tx,
                    actor,
                    ActivityLogEntry {
                        event_category: "ADMIN",
                        event_type: "USER_CREATE",
                        entity: Some(AuditEntity::new("user", new_user.id, &display_name)),
                        data_diff: Some(DataDiff::create_only(&new_user)),
                        request_context: None,
                    },
                )
                .await?;

                (new_user.id, true)
            }
        };

        // 確保 PI 角色
        let pi_role_id: Option<Uuid> =
            sqlx::query_scalar("SELECT id FROM roles WHERE code = 'PI' AND is_active = true")
                .fetch_optional(&mut *tx)
                .await?;
        if let Some(role) = pi_role_id {
            sqlx::query(
                "INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(pi_user_id)
            .bind(role)
            .execute(&mut *tx)
            .await?;
        }

        // relink pi_user_id（若與現值不同）
        if before.pi_user_id != pi_user_id {
            sqlx::query("UPDATE protocols SET pi_user_id = $1, updated_at = NOW() WHERE id = $2")
                .bind(pi_user_id)
                .bind(protocol_id)
                .execute(&mut *tx)
                .await?;
        }

        let after: Protocol =
            sqlx::query_as::<_, Protocol>("SELECT * FROM protocols WHERE id = $1")
                .bind(protocol_id)
                .fetch_one(&mut *tx)
                .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type: "PROTOCOL_PI_PROVISIONED",
                entity: Some(AuditEntity::new("protocol", before.id, &before.title)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        // 待核准開通信（覆蓋同計畫舊 pending）
        sqlx::query("DELETE FROM pi_account_invites WHERE protocol_id = $1 AND status = 'pending'")
            .bind(protocol_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            r#"INSERT INTO pi_account_invites (protocol_id, pi_user_id, email, status, provisioned_by, provisioned_at)
               VALUES ($1, $2, $3, 'pending', $4, NOW())"#,
        )
        .bind(protocol_id)
        .bind(pi_user_id)
        .bind(&email)
        .bind(provisioned_by)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok((pi_user_id, email, created_new_account))
    }
}
