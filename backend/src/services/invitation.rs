use chrono::{Duration, Utc};
use rand::RngCore;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    config::Config,
    models::{
        AcceptInvitationRequest, AcceptInvitationResponse, CreateInvitationRequest,
        CreateInvitationResponse, Invitation, InvitationListQuery, InvitationResponse,
        PaginatedResponse, PaginationParams, VerifyInvitationResponse,
        INVITATION_EXPIRY_DAYS, INVITATION_STATUS_ACCEPTED, INVITATION_STATUS_EXPIRED,
        INVITATION_STATUS_PENDING, INVITATION_STATUS_REVOKED,
    },
    services::{AuthService, EmailService},
    AppError, Result,
};

pub struct InvitationService;

impl InvitationService {
    /// 建立邀請
    pub async fn create(
        pool: &PgPool,
        config: &Config,
        req: &CreateInvitationRequest,
        invited_by: Uuid,
    ) -> Result<CreateInvitationResponse> {
        let email = req.email.trim().to_lowercase();

        // 1. 檢查 Email 是否已有帳號
        let user_exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 AND is_active = true)")
                .bind(&email)
                .fetch_one(pool)
                .await?;

        if user_exists {
            return Err(AppError::Conflict(
                "此 Email 已有帳號，請引導使用者至重設密碼頁面".to_string(),
            ));
        }

        // 2. 檢查是否已有 pending 邀請
        let existing: Option<Invitation> = sqlx::query_as(
            "SELECT * FROM invitations WHERE email = $1 AND status = $2",
        )
        .bind(&email)
        .bind(INVITATION_STATUS_PENDING)
        .fetch_optional(pool)
        .await?;

        if let Some(inv) = existing {
            return Err(AppError::Conflict(format!(
                "此 Email 已有待接受的邀請 (id: {})",
                inv.id
            )));
        }

        // 3. 產生 token 並建立邀請
        let token = generate_invitation_token();
        let expires_at = Utc::now() + Duration::days(INVITATION_EXPIRY_DAYS);
        let invite_link = format!("{}/invite/{}", config.app_url, token);

        let invitation: Invitation = sqlx::query_as(
            r#"
            INSERT INTO invitations (email, organization, invitation_token, invited_by, expires_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(&email)
        .bind(&req.organization)
        .bind(&token)
        .bind(invited_by)
        .bind(expires_at)
        .fetch_one(pool)
        .await?;

        // 4. 非同步發送 Email
        let config_clone = config.clone();
        let email_clone = email.clone();
        let link_clone = invite_link.clone();
        let expires_formatted = expires_at.format("%Y-%m-%d %H:%M").to_string();
        tokio::spawn(async move {
            if let Err(e) = EmailService::send_invitation_email(
                &config_clone,
                &email_clone,
                &link_clone,
                &expires_formatted,
            )
            .await
            {
                tracing::error!("Failed to send invitation email to {}: {}", email_clone, e);
            }
        });

        // 5. 查詢邀請者名稱
        let invited_by_name = get_user_display_name(pool, invited_by).await?;

        let response = InvitationResponse::from_invitation(
            &invitation,
            &invited_by_name,
            &config.app_url,
        );
        let link = response.invite_link.clone();

        Ok(CreateInvitationResponse {
            invitation: response,
            invite_link: link,
        })
    }

    /// 列出邀請
    pub async fn list(
        pool: &PgPool,
        config: &Config,
        query: &InvitationListQuery,
    ) -> Result<PaginatedResponse<InvitationResponse>> {
        let pagination = PaginationParams {
            page: query.page,
            per_page: query.per_page,
        };
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(20).clamp(1, 100);

        let (invitations, total) = if let Some(ref status) = query.status {
            let suffix = pagination.sql_suffix();
            let sql = format!(
                "SELECT * FROM invitations WHERE status = $1 ORDER BY created_at DESC{}",
                suffix
            );
            let items: Vec<Invitation> = sqlx::query_as(&sql)
                .bind(status)
                .fetch_all(pool)
                .await?;

            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM invitations WHERE status = $1")
                    .bind(status)
                    .fetch_one(pool)
                    .await?;

            (items, total)
        } else {
            let suffix = pagination.sql_suffix();
            let sql = format!(
                "SELECT * FROM invitations ORDER BY created_at DESC{}",
                suffix
            );
            let items: Vec<Invitation> = sqlx::query_as(&sql).fetch_all(pool).await?;

            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM invitations")
                .fetch_one(pool)
                .await?;

            (items, total)
        };

        // 批次查詢邀請者名稱
        let inviter_ids: Vec<Uuid> = invitations.iter().map(|i| i.invited_by).collect();
        let names = get_user_display_names(pool, &inviter_ids).await?;

        let data: Vec<InvitationResponse> = invitations
            .iter()
            .map(|inv| {
                let name = names
                    .iter()
                    .find(|(id, _)| *id == inv.invited_by)
                    .map(|(_, n)| n.as_str())
                    .unwrap_or("Unknown");
                InvitationResponse::from_invitation(inv, name, &config.app_url)
            })
            .collect();

        Ok(PaginatedResponse::new(data, total, page, per_page))
    }

    /// 撤銷邀請
    pub async fn revoke(pool: &PgPool, invitation_id: Uuid) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE invitations
            SET status = $1, updated_at = NOW()
            WHERE id = $2 AND status = $3
            "#,
        )
        .bind(INVITATION_STATUS_REVOKED)
        .bind(invitation_id)
        .bind(INVITATION_STATUS_PENDING)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(
                "邀請不存在或已非 pending 狀態".to_string(),
            ));
        }

        Ok(())
    }

    /// 重新發送邀請
    pub async fn resend(
        pool: &PgPool,
        config: &Config,
        invitation_id: Uuid,
    ) -> Result<InvitationResponse> {
        // 產生新 token 並重設過期時間
        let new_token = generate_invitation_token();
        let new_expires_at = Utc::now() + Duration::days(INVITATION_EXPIRY_DAYS);

        let invitation: Invitation = sqlx::query_as(
            r#"
            UPDATE invitations
            SET invitation_token = $1, expires_at = $2, updated_at = NOW()
            WHERE id = $3 AND status = $4
            RETURNING *
            "#,
        )
        .bind(&new_token)
        .bind(new_expires_at)
        .bind(invitation_id)
        .bind(INVITATION_STATUS_PENDING)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| {
            AppError::NotFound("邀請不存在或已非 pending 狀態".to_string())
        })?;

        // 非同步發送 Email
        let config_clone = config.clone();
        let email_clone = invitation.email.clone();
        let link = format!("{}/invite/{}", config.app_url, new_token);
        let link_clone = link.clone();
        let expires_formatted = new_expires_at.format("%Y-%m-%d %H:%M").to_string();
        tokio::spawn(async move {
            if let Err(e) = EmailService::send_invitation_email(
                &config_clone,
                &email_clone,
                &link_clone,
                &expires_formatted,
            )
            .await
            {
                tracing::error!(
                    "Failed to resend invitation email to {}: {}",
                    email_clone,
                    e
                );
            }
        });

        let invited_by_name = get_user_display_name(pool, invitation.invited_by).await?;
        Ok(InvitationResponse::from_invitation(
            &invitation,
            &invited_by_name,
            &config.app_url,
        ))
    }

    /// 驗證邀請 token
    pub async fn verify(pool: &PgPool, token: &str) -> Result<VerifyInvitationResponse> {
        let invitation: Option<Invitation> =
            sqlx::query_as("SELECT * FROM invitations WHERE invitation_token = $1")
                .bind(token)
                .fetch_optional(pool)
                .await?;

        let Some(inv) = invitation else {
            return Ok(VerifyInvitationResponse {
                valid: false,
                email: None,
                organization: None,
                reason: Some("not_found".to_string()),
            });
        };

        if inv.status == INVITATION_STATUS_ACCEPTED {
            return Ok(VerifyInvitationResponse {
                valid: false,
                email: None,
                organization: None,
                reason: Some("already_accepted".to_string()),
            });
        }

        if inv.status == INVITATION_STATUS_REVOKED {
            return Ok(VerifyInvitationResponse {
                valid: false,
                email: None,
                organization: None,
                reason: Some("revoked".to_string()),
            });
        }

        if inv.status == INVITATION_STATUS_EXPIRED || inv.expires_at < Utc::now() {
            return Ok(VerifyInvitationResponse {
                valid: false,
                email: None,
                organization: None,
                reason: Some("expired".to_string()),
            });
        }

        Ok(VerifyInvitationResponse {
            valid: true,
            email: Some(inv.email),
            organization: inv.organization,
            reason: None,
        })
    }

    /// 接受邀請並建立帳號
    pub async fn accept(
        pool: &PgPool,
        config: &Config,
        req: &AcceptInvitationRequest,
    ) -> Result<AcceptInvitationResponse> {
        // 驗證同意條款
        if !req.agree_terms {
            return Err(AppError::Validation("必須同意服務條款".to_string()));
        }

        // 1. 驗證 token
        let invitation: Invitation =
            sqlx::query_as("SELECT * FROM invitations WHERE invitation_token = $1")
                .bind(&req.invitation_token)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| AppError::NotFound("邀請連結無效".to_string()))?;

        if invitation.status != INVITATION_STATUS_PENDING {
            return Err(AppError::BadRequest("此邀請連結已使用或已失效".to_string()));
        }

        if invitation.expires_at < Utc::now() {
            // 順便更新狀態
            let _ = sqlx::query("UPDATE invitations SET status = $1, updated_at = NOW() WHERE id = $2")
                .bind(INVITATION_STATUS_EXPIRED)
                .bind(invitation.id)
                .execute(pool)
                .await;
            return Err(AppError::BadRequest("此邀請連結已過期".to_string()));
        }

        // 2. 檢查 Email 是否已有帳號（防止 race condition）
        let user_exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
                .bind(&invitation.email)
                .fetch_one(pool)
                .await?;

        if user_exists {
            return Err(AppError::Conflict("此 Email 已有帳號".to_string()));
        }

        // 3. Hash 密碼
        let password_hash = AuthService::hash_password(&req.password)?;

        // 4. 建立帳號（must_change_password = false，客戶已自行設定）
        let user_id = Uuid::new_v4();
        let user = sqlx::query_as::<_, crate::models::User>(
            r#"
            INSERT INTO users (
                id, email, password_hash, display_name, phone, organization,
                position, is_internal, is_active, must_change_password,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, false, true, false, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(&invitation.email)
        .bind(&password_hash)
        .bind(&req.display_name)
        .bind(&req.phone)
        .bind(&req.organization)
        .bind(&req.position)
        .fetch_one(pool)
        .await?;

        // 5. 自動分配 PI 角色
        let pi_role_id: Option<Uuid> =
            sqlx::query_scalar("SELECT id FROM roles WHERE code = 'PI' AND is_active = true")
                .fetch_optional(pool)
                .await?;

        if let Some(role_id) = pi_role_id {
            sqlx::query(
                "INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(user.id)
            .bind(role_id)
            .execute(pool)
            .await?;
        } else {
            tracing::warn!("PI role not found in roles table, skipping role assignment");
        }

        // 6. 更新邀請狀態
        sqlx::query(
            r#"
            UPDATE invitations
            SET status = $1, accepted_at = NOW(), created_user_id = $2, updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(INVITATION_STATUS_ACCEPTED)
        .bind(user.id)
        .bind(invitation.id)
        .execute(pool)
        .await?;

        // 7. 產生 JWT tokens（複用現有 login 邏輯）
        let login_response = AuthService::issue_login_tokens(pool, config, &user).await?;

        Ok(AcceptInvitationResponse {
            user: login_response.user,
            access_token: login_response.access_token,
            refresh_token: login_response.refresh_token,
        })
    }

    /// 過期未接受的邀請（排程用）
    pub async fn expire_stale(pool: &PgPool) -> Result<u64> {
        let result = sqlx::query(
            r#"
            UPDATE invitations
            SET status = $1, updated_at = NOW()
            WHERE status = $2 AND expires_at < NOW()
            "#,
        )
        .bind(INVITATION_STATUS_EXPIRED)
        .bind(INVITATION_STATUS_PENDING)
        .execute(pool)
        .await?;

        let count = result.rows_affected();
        if count > 0 {
            tracing::info!("[Invitation] Expired {} stale invitations", count);
        }

        Ok(count)
    }
}

impl InvitationResponse {
    fn from_invitation(inv: &Invitation, invited_by_name: &str, app_url: &str) -> Self {
        let invite_link = if inv.status == INVITATION_STATUS_PENDING {
            format!("{}/invite/{}", app_url, inv.invitation_token)
        } else {
            String::new()
        };

        Self {
            id: inv.id,
            email: inv.email.clone(),
            organization: inv.organization.clone(),
            invited_by: inv.invited_by,
            invited_by_name: invited_by_name.to_string(),
            status: inv.status.clone(),
            expires_at: inv.expires_at,
            accepted_at: inv.accepted_at,
            created_user_id: inv.created_user_id,
            invite_link,
            created_at: inv.created_at,
            updated_at: inv.updated_at,
        }
    }
}

/// 產生 64 字元 crypto-random token（base64url 編碼）
fn generate_invitation_token() -> String {
    let mut bytes = [0u8; 48]; // 48 bytes -> 64 chars in base64url
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    URL_SAFE_NO_PAD.encode(bytes)
}

/// 查詢單一使用者的顯示名稱
async fn get_user_display_name(pool: &PgPool, user_id: Uuid) -> Result<String> {
    let name: Option<String> =
        sqlx::query_scalar("SELECT display_name FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    Ok(name.unwrap_or_else(|| "Unknown".to_string()))
}

/// 批次查詢使用者的顯示名稱
async fn get_user_display_names(pool: &PgPool, user_ids: &[Uuid]) -> Result<Vec<(Uuid, String)>> {
    if user_ids.is_empty() {
        return Ok(vec![]);
    }

    let rows: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT id, display_name FROM users WHERE id = ANY($1)",
    )
    .bind(user_ids)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
