//! 外部 PI 代簽授權（`protocol_pi_delegates`，migration 010）。
//!
//! 把「外部 PI 尚未開通系統帳號前，誰能代替他簽署/核准」從無聲借位
//! （`pi_user_id` 借用建立者/匯入者 id）變成顯式、需計劃負責人（SD）核准、
//! 留有核可證據的正式授權。不改動 `pi_user_id`/`pi_is_external` 語意——
//! 這是疊加在既有機制之外的一層，見 migration 010 開頭說明。
//!
//! # 核准/撤銷授權規則（決策已由使用者確認，不是實作細節猜測）
//!
//! - 核准**他人**為代理人：僅該計畫**現任 SD** 可核准。
//! - SD 想指定**自己**為代理人：SD 不可自己核准這個組合（自簽自證），須改由
//!   `IACUC_STAFF`（執行秘書）或 admin 核准——鏡像 `core.rs::validate_and_authorize_sd`
//!   「僅執秘/admin 可指派他人為 SD，其餘限本人」的規則，但方向相反：那邊是
//!   「指派他人需高權限、指派自己免審」，這裡凡「提出者與核准者同一人」的組合
//!   都要拉高一層核准權限。
//! - 撤銷：現任 SD，或 `IACUC_STAFF`/admin，皆可（撤銷是降低風險的動作，
//!   不需要像核准那樣嚴格區分本人/他人）。
//! - 一份計畫同時最多一筆生效中授權（`revoked_at IS NULL`），換人須先撤銷再
//!   重新核准，不做隱性覆蓋。

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::ProtocolService;
use crate::{
    middleware::{ActorContext, SYSTEM_USER_ID},
    models::{audit_diff::DataDiff, PiDelegateInfo, Protocol, ProtocolPiDelegate},
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    AppError, Result,
};

const EVT_AUTHORIZE: &str = "PROTOCOL_PI_DELEGATE_AUTHORIZE";
const EVT_REVOKE: &str = "PROTOCOL_PI_DELEGATE_REVOKE";

fn actor_id_or_system(actor: &ActorContext) -> Uuid {
    match actor {
        ActorContext::User(u) => u.id,
        ActorContext::System { .. } => SYSTEM_USER_ID,
        ActorContext::Anonymous => SYSTEM_USER_ID,
    }
}

/// 是否具「僅限 SD 自行核准自己時才需要」的高權限（`IACUC_STAFF` 或 admin）。
fn has_delegate_escalation_privilege(actor: &ActorContext) -> bool {
    match actor {
        ActorContext::User(u) => {
            u.is_admin()
                || u.roles
                    .iter()
                    .any(|r| r == crate::constants::ROLE_IACUC_STAFF)
        }
        ActorContext::System { .. } => true,
        ActorContext::Anonymous => false,
    }
}

impl ProtocolService {
    /// 核准一位 PI 代理人。授權規則見檔案頂端說明。
    ///
    /// `expires_at`：授權自動失效時點，`None` = 不設期限（沿用原本行為）。
    /// 必須晚於現在。過期後所有授權判準一律不放行，但**不追溯**——見 migration 010
    /// 該欄的說明。
    pub async fn authorize_pi_delegate(
        pool: &PgPool,
        actor: &ActorContext,
        protocol_id: Uuid,
        delegate_user_id: Uuid,
        reason: Option<&str>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<ProtocolPiDelegate> {
        let actor_id = match actor {
            ActorContext::User(u) => u.id,
            _ => {
                return Err(AppError::Forbidden(
                    "核准 PI 代理人須由已登入使用者操作".into(),
                ))
            }
        };

        let mut tx = pool.begin().await?;

        let protocol =
            sqlx::query_as::<_, Protocol>("SELECT * FROM protocols WHERE id = $1 FOR UPDATE")
                .bind(protocol_id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("找不到計劃書".into()))?;

        if !protocol.pi_is_external {
            return Err(AppError::BusinessRule(
                "此計畫的 PI 已有系統帳號（非借位佔位），不適用代理人機制。".into(),
            ));
        }

        let Some(sd_id) = protocol.study_director_user_id else {
            return Err(AppError::BusinessRule(
                "本計畫尚未指派計劃負責人（Study Director），無法核准代理人。".into(),
            ));
        };

        let self_delegation = delegate_user_id == sd_id;
        let authorized = if self_delegation {
            has_delegate_escalation_privilege(actor)
        } else {
            actor_id == sd_id
        };
        if !authorized {
            return Err(AppError::Forbidden(if self_delegation {
                "計劃負責人不可自行核准自己為代理人，須由執行秘書或系統管理員核准。".into()
            } else {
                "僅本計畫的計劃負責人（Study Director）可核准代理人。".into()
            }));
        }

        // 資格：內部、啟用中帳號（比照 SD 資格門檻，見 core.rs::validate_and_authorize_sd）。
        //
        // 先鎖 users 那一列再讀 `is_active`（CodeRabbit #53）：不鎖的話「核准代理人」與
        // 「停用帳號」兩個交易可以交錯成「資格檢查通過 → 停用 commit → 才 INSERT 授權」，
        // 產出一筆掛在已停用帳號上的生效授權；那個帳號日後被重新啟用時，代理權就
        // **不經任何重新核准**自己回來了。
        //
        // 鎖法與序列化理由與 `core.rs::validate_and_authorize_sd`（SD 指派）逐字相同，
        // 那裡有兩種先後的完整推導；此處不重抄，只標明沿用同一套。鎖順序也一致
        // （protocols FOR UPDATE → users FOR SHARE），不會與 SD 指派互相死鎖。
        //
        // ⚠️ 鎖獨立成一句、不併進下面的 EXISTS：帶 join 的 EXISTS 子查詢加鎖定子句在
        // Postgres 有限制，且鎖到哪張表不明顯。
        let delegate_locked: Option<bool> =
            sqlx::query_scalar("SELECT is_active FROM users WHERE id = $1 FOR SHARE")
                .bind(delegate_user_id)
                .fetch_optional(&mut *tx)
                .await?;
        // 不存在就讓下面的資格查詢回同一句錯誤訊息，不另外分歧（同 core.rs 的處置）。
        let _ = delegate_locked;

        let delegate_valid: bool = sqlx::query_scalar(
            r#"SELECT EXISTS(
                 SELECT 1 FROM users
                 WHERE id = $1 AND is_active = true AND deleted_at IS NULL AND is_internal = true
               )"#,
        )
        .bind(delegate_user_id)
        .fetch_one(&mut *tx)
        .await?;
        if !delegate_valid {
            return Err(AppError::Validation(
                "代理人必須是啟用中的內部帳號。".into(),
            ));
        }

        // 期限必須是未來（migration 010 的 expires_at）。允許 NULL＝不設期限。
        if let Some(expires_at) = expires_at {
            if expires_at <= Utc::now() {
                return Err(AppError::Validation(
                    "代理授權的到期時間必須晚於現在。".into(),
                ));
            }
        }

        // 「一份計畫同時只有一位生效代理人」這條約束由部分唯一索引（`revoked_at IS NULL`）
        // 保證，而索引述詞不能用 `now()`，所以**已過期但未撤銷**的列仍然佔著那個位置。
        //
        // 已過期的授權在授權判準上已經一律不放行（各檢查點都帶
        // `expires_at IS NULL OR expires_at > now()`），讓它繼續卡住新的核准沒有意義，
        // 只會逼 SD 先手動撤銷一筆早就無效的紀錄。所以這裡自動把它關掉——
        // 手法與 SD 變更時的自動撤銷（`revoke_pi_delegate_for_sd_change_tx`）相同，
        // 一樣留下 `revoked_by` / `revoked_reason` 的可歸責痕跡，不是靜默刪除。
        let existing: Option<(Uuid, bool)> = sqlx::query_as(
            r#"SELECT id, (expires_at IS NOT NULL AND expires_at <= now()) AS is_expired
               FROM protocol_pi_delegates
               WHERE protocol_id = $1 AND revoked_at IS NULL"#,
        )
        .bind(protocol_id)
        .fetch_optional(&mut *tx)
        .await?;
        match existing {
            Some((_, false)) => {
                return Err(AppError::BusinessRule(
                    "本計畫已有生效中的代理人，請先撤銷再重新核准。".into(),
                ));
            }
            Some((expired_id, true)) => {
                sqlx::query(
                    r#"UPDATE protocol_pi_delegates
                       SET revoked_by = $1, revoked_at = NOW(),
                           revoked_reason = '授權已到期，因核准新代理人而自動關閉'
                       WHERE id = $2 AND revoked_at IS NULL"#,
                )
                .bind(actor_id)
                .bind(expired_id)
                .execute(&mut *tx)
                .await?;
            }
            None => {}
        }

        let delegate = sqlx::query_as::<_, ProtocolPiDelegate>(
            r#"INSERT INTO protocol_pi_delegates
                 (protocol_id, delegate_user_id, authorized_by, reason, expires_at)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING *"#,
        )
        .bind(protocol_id)
        .bind(delegate_user_id)
        .bind(actor_id)
        .bind(reason)
        .bind(expires_at)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type: EVT_AUTHORIZE,
                entity: Some(AuditEntity::new("protocol", protocol_id, &protocol.title)),
                data_diff: Some(DataDiff::create_only(&delegate)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(delegate)
    }

    /// 撤銷此計畫目前生效中的代理人（若有）。授權：現任 SD，或 `IACUC_STAFF`/admin。
    pub async fn revoke_pi_delegate(
        pool: &PgPool,
        actor: &ActorContext,
        protocol_id: Uuid,
        reason: Option<&str>,
    ) -> Result<()> {
        let mut tx = pool.begin().await?;

        let protocol =
            sqlx::query_as::<_, Protocol>("SELECT * FROM protocols WHERE id = $1 FOR UPDATE")
                .bind(protocol_id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("找不到計劃書".into()))?;

        let actor_id = actor_id_or_system(actor);
        let is_current_sd = protocol.study_director_user_id == Some(actor_id);
        if !is_current_sd && !has_delegate_escalation_privilege(actor) {
            return Err(AppError::Forbidden(
                "僅本計畫的計劃負責人、執行秘書或系統管理員可撤銷代理人。".into(),
            ));
        }

        let revoked: Option<Uuid> = sqlx::query_scalar(
            r#"UPDATE protocol_pi_delegates
               SET revoked_by = $1, revoked_at = NOW(), revoked_reason = $2
               WHERE protocol_id = $3 AND revoked_at IS NULL
               RETURNING id"#,
        )
        .bind(actor_id)
        .bind(reason)
        .bind(protocol_id)
        .fetch_optional(&mut *tx)
        .await?;

        if revoked.is_none() {
            return Err(AppError::NotFound("本計畫目前沒有生效中的代理人。".into()));
        }

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type: EVT_REVOKE,
                entity: Some(AuditEntity::new("protocol", protocol_id, &protocol.title)),
                data_diff: None,
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// SD 變更時（`core.rs::update`）同一 tx 內自動撤銷既有代理授權——SD 換人後，
    /// 舊 SD 的信任背書不延續給新 SD 承擔責任，新 SD 需重新核准。不重新鎖
    /// protocol（呼叫端已鎖）、不做授權檢查（SD 變更本身已通過授權），沒有生效中
    /// 代理人時安靜略過，不是錯誤。
    pub(crate) async fn revoke_pi_delegate_for_sd_change_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &ActorContext,
        protocol_id: Uuid,
        protocol_title: &str,
    ) -> Result<()> {
        let actor_id = actor_id_or_system(actor);
        let revoked: Option<Uuid> = sqlx::query_scalar(
            r#"UPDATE protocol_pi_delegates
               SET revoked_by = $1, revoked_at = NOW(),
                   revoked_reason = '計劃負責人變更，原授權自動撤銷'
               WHERE protocol_id = $2 AND revoked_at IS NULL
               RETURNING id"#,
        )
        .bind(actor_id)
        .bind(protocol_id)
        .fetch_optional(&mut **tx)
        .await?;

        if revoked.is_some() {
            AuditService::log_activity_tx(
                tx,
                actor,
                ActivityLogEntry {
                    event_category: "AUP",
                    event_type: EVT_REVOKE,
                    entity: Some(AuditEntity::new("protocol", protocol_id, protocol_title)),
                    data_diff: None,
                    request_context: None,
                },
            )
            .await?;
        }
        Ok(())
    }

    /// 此計畫目前生效中的代理授權（唯讀，供 `GET /protocols/{id}` 與代理人管理端點顯示）。
    ///
    /// ⚠️ 這裡用的是 runtime 的 `query_as::<_, PiDelegateInfo>`（非編譯期巨集，因為
    /// `COALESCE` 的 nullable 推斷會讓巨集判成 `Option<String>`）。**SELECT 少一欄不會
    /// 編譯失敗**，只會在解碼時回 `ColumnNotFound`，而本函式是 `GET /protocols/{id}`
    /// 的必經路徑——漏欄的後果是所有帶生效代理的計畫全部打不開。
    /// 往 `PiDelegateInfo` 加欄位時，這裡的投影要同步加。
    /// 防線是 `tests/api_protocol_pi_delegate.rs::active_pi_delegate_projects_every_field`。
    pub async fn active_pi_delegate(
        pool: &PgPool,
        protocol_id: Uuid,
    ) -> Result<Option<PiDelegateInfo>> {
        let info = sqlx::query_as::<_, PiDelegateInfo>(
            r#"SELECT d.id, d.delegate_user_id,
                      COALESCE(du.display_name, '') AS delegate_name,
                      d.authorized_by,
                      COALESCE(au.display_name, '') AS authorized_by_name,
                      d.authorized_at, d.expires_at, d.reason
               FROM protocol_pi_delegates d
               JOIN users du ON du.id = d.delegate_user_id
               JOIN users au ON au.id = d.authorized_by
               WHERE d.protocol_id = $1 AND d.revoked_at IS NULL
                 AND (d.expires_at IS NULL OR d.expires_at > now())"#,
        )
        .bind(protocol_id)
        .fetch_optional(pool)
        .await?;
        Ok(info)
    }
}
