use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    config::Config,
    models::{
        AcceptInvitationRequest, AcceptInvitationResponse, CreateInvitationRequest,
        CreateInvitationResponse, Invitation, InvitationAvailableRole, InvitationListQuery,
        InvitationResponse, InvitationRoleSummary, PaginatedResponse, PaginationParams,
        VerifyInvitationResponse, INVITATION_EXPIRY_DAYS, INVITATION_STATUS_ACCEPTED,
        INVITATION_STATUS_EXPIRED, INVITATION_STATUS_PENDING, INVITATION_STATUS_REVOKED,
    },
    services::{access, AuthService, EmailService},
    AppError, Result,
};

/// 列表過濾片段：隱藏「已被同 email 的已接受邀請取代」的過期紀錄。
///
/// 一個 email 若已存在 `accepted` 邀請（代表帳號已建立），其先前未接受而轉為
/// `expired` 的舊邀請在管理列表中自動隱藏，避免同一人同時出現在「已過期」與
/// 「已接受」造成混淆。
///
/// `$1` / `$2` 由呼叫端**依序綁定** `INVITATION_STATUS_EXPIRED` /
/// `INVITATION_STATUS_ACCEPTED`（不在 SQL 內硬編魔術字串）；故此片段必須擺在
/// WHERE 條件最前、且呼叫端的 bind 順序須與之一致。狀態若再有篩選，接於其後以
/// `$3` 綁定。
const HIDE_SUPERSEDED_EXPIRED: &str = "NOT (i.status = $1 AND EXISTS (\
    SELECT 1 FROM invitations a WHERE a.email = i.email AND a.status = $2))";

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

        // 1. 檢查 Email 是否已有帳號（軟刪 user 不視為佔用 email）
        let user_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 AND is_active = true AND deleted_at IS NULL)",
        )
        .bind(&email)
        .fetch_one(pool)
        .await?;

        if user_exists {
            return Err(AppError::Conflict(
                "此 Email 已有帳號，請引導使用者至重設密碼頁面".to_string(),
            ));
        }

        // 2. 檢查是否已有 pending 邀請
        let existing: Option<Invitation> =
            sqlx::query_as("SELECT * FROM invitations WHERE email = $1 AND status = $2")
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

        // 3. 驗證 role_ids 全部存在且 active（先去重避免 invitation_roles PK 衝突）
        let mut unique_role_ids: Vec<Uuid> = req.role_ids.clone();
        unique_role_ids.sort();
        unique_role_ids.dedup();

        let valid_role_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM roles WHERE id = ANY($1) AND is_active = true",
        )
        .bind(&unique_role_ids)
        .fetch_one(pool)
        .await?;

        if valid_role_count != unique_role_ids.len() as i64 {
            return Err(AppError::Validation("部分角色不存在或已停用".to_string()));
        }

        // 3b. SEC-PRIV (CSO-r3 #2): 邀請建立端須與 UserService 角色指派授權一致，
        //     否則持有 invitation.create 的非管理員（如 IACUC_STAFF）可邀請並建立
        //     legacy `admin` 帳號，旁路繞過 user.rs 的提權守衛。
        access::require_authority_to_assign_roles(pool, Some(invited_by), &unique_role_ids).await?;

        // 4. 在交易中產生 token / 寫入邀請 / 寫入 roles
        let token = generate_invitation_token();
        let expires_at = Utc::now() + Duration::days(INVITATION_EXPIRY_DAYS);

        let mut tx = pool.begin().await?;

        let invitation: Invitation = sqlx::query_as(
            r#"
            INSERT INTO invitations (
                email, organization, display_name, phone, position,
                invitation_token, invited_by, expires_at, is_internal
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(&email)
        .bind(&req.organization)
        .bind(&req.display_name)
        .bind(req.phone.as_deref())
        .bind(req.position.as_deref())
        .bind(&token)
        .bind(invited_by)
        .bind(expires_at)
        .bind(req.is_internal)
        .fetch_one(&mut *tx)
        .await?;

        for role_id in &unique_role_ids {
            sqlx::query("INSERT INTO invitation_roles (invitation_id, role_id) VALUES ($1, $2)")
                .bind(invitation.id)
                .bind(role_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        // 5. 非同步發送 Email
        spawn_send_invitation_email(config, &email, &token, expires_at);

        // 6. 查詢邀請者名稱與 role 摘要
        let invited_by_name = get_user_display_name(pool, invited_by).await?;
        let roles = list_invitation_roles(pool, invitation.id).await?;

        let response = InvitationResponse::from_invitation(
            &invitation,
            &invited_by_name,
            &config.app_url,
            roles,
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
            // 片段 $1/$2 綁 EXPIRED/ACCEPTED，狀態篩選接於其後綁 $3。
            let sql = format!(
                "SELECT * FROM invitations i WHERE {HIDE_SUPERSEDED_EXPIRED} AND i.status = $3 \
                 ORDER BY i.created_at DESC{suffix}"
            );
            let items: Vec<Invitation> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
                .bind(INVITATION_STATUS_EXPIRED)
                .bind(INVITATION_STATUS_ACCEPTED)
                .bind(status)
                .fetch_all(pool)
                .await?;

            let total: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
                "SELECT COUNT(*) FROM invitations i WHERE {HIDE_SUPERSEDED_EXPIRED} AND i.status = $3"
            )))
            .bind(INVITATION_STATUS_EXPIRED)
            .bind(INVITATION_STATUS_ACCEPTED)
            .bind(status)
            .fetch_one(pool)
            .await?;

            (items, total)
        } else {
            let suffix = pagination.sql_suffix();
            let sql = format!(
                "SELECT * FROM invitations i WHERE {HIDE_SUPERSEDED_EXPIRED} \
                 ORDER BY i.created_at DESC{suffix}"
            );
            let items: Vec<Invitation> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
                .bind(INVITATION_STATUS_EXPIRED)
                .bind(INVITATION_STATUS_ACCEPTED)
                .fetch_all(pool)
                .await?;

            let total: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
                "SELECT COUNT(*) FROM invitations i WHERE {HIDE_SUPERSEDED_EXPIRED}"
            )))
            .bind(INVITATION_STATUS_EXPIRED)
            .bind(INVITATION_STATUS_ACCEPTED)
            .fetch_one(pool)
            .await?;

            (items, total)
        };

        // 批次查詢邀請者名稱
        let inviter_ids: Vec<Uuid> = invitations.iter().map(|i| i.invited_by).collect();
        let names = get_user_display_names(pool, &inviter_ids).await?;

        // 批次查詢 invitation_id -> roles，建 HashMap O(1) 查找
        let invitation_ids: Vec<Uuid> = invitations.iter().map(|i| i.id).collect();
        let roles_by_inv = list_invitation_roles_batch(pool, &invitation_ids).await?;

        let mut roles_map: HashMap<Uuid, Vec<InvitationRoleSummary>> = HashMap::new();
        for (iid, role) in roles_by_inv {
            roles_map.entry(iid).or_default().push(role);
        }

        let names_map: HashMap<Uuid, String> = names.into_iter().collect();

        let data: Vec<InvitationResponse> = invitations
            .iter()
            .map(|inv| {
                let name = names_map
                    .get(&inv.invited_by)
                    .map(String::as_str)
                    .unwrap_or("Unknown");
                let roles = roles_map.remove(&inv.id).unwrap_or_default();
                InvitationResponse::from_invitation(inv, name, &config.app_url, roles)
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

    /// 重新發送邀請（待接受或已過期皆可；已過期者重發後回到待接受狀態）
    pub async fn resend(
        pool: &PgPool,
        config: &Config,
        invitation_id: Uuid,
    ) -> Result<InvitationResponse> {
        // 1. 取邀請並驗證狀態：僅 pending / expired 可重發
        let existing: Invitation = sqlx::query_as("SELECT * FROM invitations WHERE id = $1")
            .bind(invitation_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("邀請不存在".to_string()))?;

        if existing.status != INVITATION_STATUS_PENDING
            && existing.status != INVITATION_STATUS_EXPIRED
        {
            return Err(AppError::BadRequest(
                "僅待接受或已過期的邀請可重新發送".to_string(),
            ));
        }

        // 2. 此 Email 已有啟用帳號則不重發（與 create 一致，避免無效邀請）
        let user_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 AND is_active = true AND deleted_at IS NULL)",
        )
        .bind(&existing.email)
        .fetch_one(pool)
        .await?;
        if user_exists {
            return Err(AppError::Conflict(
                "此 Email 已有帳號，無需重新發送邀請".to_string(),
            ));
        }

        // 3. 換新 token / 展延效期 / 狀態回到 pending
        let new_token = generate_invitation_token();
        let new_expires_at = Utc::now() + Duration::days(INVITATION_EXPIRY_DAYS);
        let invitation: Invitation = sqlx::query_as(
            r#"
            UPDATE invitations
            SET invitation_token = $1, expires_at = $2, status = $3, updated_at = NOW()
            WHERE id = $4
            RETURNING *
            "#,
        )
        .bind(&new_token)
        .bind(new_expires_at)
        .bind(INVITATION_STATUS_PENDING)
        .bind(invitation_id)
        .fetch_one(pool)
        .await?;

        spawn_send_invitation_email(config, &invitation.email, &new_token, new_expires_at);

        let invited_by_name = get_user_display_name(pool, invitation.invited_by).await?;
        let roles = list_invitation_roles(pool, invitation.id).await?;
        Ok(InvitationResponse::from_invitation(
            &invitation,
            &invited_by_name,
            &config.app_url,
            roles,
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
                display_name: None,
                phone: None,
                position: None,
                roles: vec![],
                reason: Some("not_found".to_string()),
            });
        };

        if inv.status == INVITATION_STATUS_ACCEPTED {
            return Ok(invalid(Some("already_accepted")));
        }
        if inv.status == INVITATION_STATUS_REVOKED {
            return Ok(invalid(Some("revoked")));
        }
        if inv.status == INVITATION_STATUS_EXPIRED || inv.expires_at < Utc::now() {
            return Ok(invalid(Some("expired")));
        }

        let roles = list_invitation_roles(pool, inv.id).await?;

        Ok(VerifyInvitationResponse {
            valid: true,
            email: Some(inv.email),
            organization: inv.organization,
            display_name: inv.display_name,
            phone: inv.phone,
            position: inv.position,
            roles,
            reason: None,
        })
    }

    /// 接受邀請並建立帳號
    pub async fn accept(
        pool: &PgPool,
        config: &Config,
        req: &AcceptInvitationRequest,
    ) -> Result<AcceptInvitationResponse> {
        if !req.agree_terms {
            return Err(AppError::Validation("必須同意服務條款".to_string()));
        }

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
            let _ =
                sqlx::query("UPDATE invitations SET status = $1, updated_at = NOW() WHERE id = $2")
                    .bind(INVITATION_STATUS_EXPIRED)
                    .bind(invitation.id)
                    .execute(pool)
                    .await;
            return Err(AppError::BadRequest("此邀請連結已過期".to_string()));
        }

        let user_exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
                .bind(&invitation.email)
                .fetch_one(pool)
                .await?;

        if user_exists {
            return Err(AppError::Conflict("此 Email 已有帳號".to_string()));
        }

        // 取角色：以 invitation_roles 為準。若空（既有舊邀請）fallback 到 PI。
        let mut role_ids: Vec<Uuid> =
            sqlx::query_scalar("SELECT role_id FROM invitation_roles WHERE invitation_id = $1")
                .bind(invitation.id)
                .fetch_all(pool)
                .await?;

        if role_ids.is_empty() {
            let pi_role_id: Option<Uuid> =
                sqlx::query_scalar("SELECT id FROM roles WHERE code = 'PI' AND is_active = true")
                    .fetch_optional(pool)
                    .await?;
            if let Some(pi) = pi_role_id {
                role_ids.push(pi);
            } else {
                tracing::warn!(
                    "Invitation {} has no roles and PI role missing — user will be created with no roles",
                    invitation.id
                );
            }
        }

        // SEC-PRIV (CSO-r3 #2) 深度防禦：接受端再次以「原邀請人」為 actor 驗證角色指派
        // 授權。create 端已擋下管理員層級角色，此處兜底防範直接寫入 invitation_roles
        // 或邀請人於建立後被降級的情況（管理員層級角色 fail-closed）。
        access::require_authority_to_assign_roles(pool, Some(invitation.invited_by), &role_ids)
            .await?;

        // A2 資安修補：邀請接受路徑補上全站密碼強度檢查（10 字元 + 大小寫 + 數字 + 弱密黑名單），
        // 與 reset_password / 自助改密一致，堵住外部 PI 自助建帳這個全站最弱的密碼閘門（GLP/Part 11）。
        AuthService::validate_password_strength(&req.password)?;
        let password_hash = AuthService::hash_password(&req.password)?;

        let mut tx = pool.begin().await?;

        let user_id = Uuid::new_v4();
        let user = sqlx::query_as::<_, crate::models::User>(
            r#"
            INSERT INTO users (
                id, email, password_hash, display_name, phone, organization,
                position, is_internal, is_active, must_change_password,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, false, NOW(), NOW())
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
        // 取邀請單上由邀請人判定的值。
        //
        // ⚠️ 這裡原本是硬編的 `false`——**不論受邀者的角色是什麼**，走邀請進來的
        // 人一律被記成外部人員。實測造成 9 位在職同仁的 is_internal 與角色矛盾，
        // 其中一位具 EXPERIMENT_STAFF 卻因此無法被加入任何部門，連帶讓他的
        // 假單找不到單位主管簽核。
        //
        // 不改用「依角色推導」是因為那同樣會猜錯：REVIEWER / VET / IACUC_CHAIR
        // 在角色表標為內部，實際擔任者卻多為外聘人員。這件事只能由邀請人判斷。
        .bind(invitation.is_internal)
        .fetch_one(&mut *tx)
        .await?;

        for role_id in &role_ids {
            sqlx::query(
                "INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(user.id)
            .bind(role_id)
            .execute(&mut *tx)
            .await?;
        }

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
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

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

    /// 給邀請建立 UI 用的 lightweight 角色列表（避免要求 dev.role.view 權限）
    pub async fn list_available_roles(pool: &PgPool) -> Result<Vec<InvitationAvailableRole>> {
        let rows: Vec<InvitationAvailableRole> = sqlx::query_as(
            "SELECT id, code, name, is_internal FROM roles WHERE is_active = true ORDER BY is_internal, name",
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}

impl InvitationResponse {
    fn from_invitation(
        inv: &Invitation,
        invited_by_name: &str,
        app_url: &str,
        roles: Vec<InvitationRoleSummary>,
    ) -> Self {
        let invite_link = if inv.status == INVITATION_STATUS_PENDING {
            format!("{}/invite/{}", app_url, inv.invitation_token)
        } else {
            String::new()
        };

        Self {
            id: inv.id,
            email: inv.email.clone(),
            organization: inv.organization.clone(),
            display_name: inv.display_name.clone(),
            phone: inv.phone.clone(),
            position: inv.position.clone(),
            invited_by: inv.invited_by,
            invited_by_name: invited_by_name.to_string(),
            is_internal: inv.is_internal,
            status: inv.status.clone(),
            expires_at: inv.expires_at,
            accepted_at: inv.accepted_at,
            created_user_id: inv.created_user_id,
            invite_link,
            roles,
            created_at: inv.created_at,
            updated_at: inv.updated_at,
        }
    }
}

fn invalid(reason: Option<&str>) -> VerifyInvitationResponse {
    VerifyInvitationResponse {
        valid: false,
        email: None,
        organization: None,
        display_name: None,
        phone: None,
        position: None,
        roles: vec![],
        reason: reason.map(String::from),
    }
}

/// 非同步寄送邀請 Email：寄送失敗只記 log，不阻斷主流程。
fn spawn_send_invitation_email(
    config: &Config,
    email: &str,
    token: &str,
    expires_at: DateTime<Utc>,
) {
    let config = config.clone();
    let email = email.to_string();
    let link = format!("{}/invite/{}", config.app_url, token);
    let expires_formatted = expires_at.format("%Y-%m-%d %H:%M").to_string();
    tokio::spawn(async move {
        if let Err(e) =
            EmailService::send_invitation_email(&config, &email, &link, &expires_formatted).await
        {
            tracing::error!("Failed to send invitation email to {}: {}", email, e);
        }
    });
}

/// 產生 64 字元 crypto-random token（base64url 編碼）
fn generate_invitation_token() -> String {
    let mut bytes = [0u8; 48];
    rand::rng().fill_bytes(&mut bytes);
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    URL_SAFE_NO_PAD.encode(bytes)
}

/// 查詢單一使用者的顯示名稱
async fn get_user_display_name(pool: &PgPool, user_id: Uuid) -> Result<String> {
    let name: Option<String> = sqlx::query_scalar("SELECT display_name FROM users WHERE id = $1")
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

    let rows: Vec<(Uuid, String)> =
        sqlx::query_as("SELECT id, display_name FROM users WHERE id = ANY($1)")
            .bind(user_ids)
            .fetch_all(pool)
            .await?;

    Ok(rows)
}

/// 查詢單一邀請的角色摘要
async fn list_invitation_roles(
    pool: &PgPool,
    invitation_id: Uuid,
) -> Result<Vec<InvitationRoleSummary>> {
    let rows: Vec<InvitationRoleSummary> = sqlx::query_as(
        r#"
        SELECT r.id, r.code, r.name
        FROM invitation_roles ir
        JOIN roles r ON r.id = ir.role_id
        WHERE ir.invitation_id = $1
        ORDER BY r.name
        "#,
    )
    .bind(invitation_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// 批次查詢多個邀請的角色摘要：回傳 (invitation_id, role) 配對
async fn list_invitation_roles_batch(
    pool: &PgPool,
    invitation_ids: &[Uuid],
) -> Result<Vec<(Uuid, InvitationRoleSummary)>> {
    if invitation_ids.is_empty() {
        return Ok(vec![]);
    }
    let rows: Vec<(Uuid, Uuid, String, String)> = sqlx::query_as(
        r#"
        SELECT ir.invitation_id, r.id, r.code, r.name
        FROM invitation_roles ir
        JOIN roles r ON r.id = ir.role_id
        WHERE ir.invitation_id = ANY($1)
        ORDER BY r.name
        "#,
    )
    .bind(invitation_ids)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(iid, id, code, name)| (iid, InvitationRoleSummary { id, code, name }))
        .collect())
}
