//! ActorContext：標示「誰」觸發了資料變更，供 Service-driven audit 寫入使用。
//!
//! Service 層所有 mutation 函式都接 `&ActorContext` 參數，讓 audit log 的
//! `actor_user_id` 欄位能正確區分：
//! - HTTP request（帶 CurrentUser）
//! - Scheduler / bin 工具（系統自動執行）
//! - 尚未登入的操作（login 嘗試、CSP report）
//!
//! 對應 migration：`migrations/033_system_user.sql` 建立 SYSTEM user row
//! 讓 `user_activity_logs.actor_user_id` FK 約束有效。

use uuid::Uuid;

use crate::middleware::CurrentUser;

/// 保留給「系統觸發」操作的 audit actor_user_id。
/// 對應 migration 033 建立的 users row（is_active=false，無法登入）。
pub const SYSTEM_USER_ID: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000001");

/// 所有 service mutation 都接這個型別，標示操作來源。
#[derive(Debug, Clone)]
pub enum ActorContext {
    /// HTTP request 觸發，帶 CurrentUser
    User(CurrentUser),
    /// Scheduler / bin 工具 / migration 等系統自動執行。
    /// `reason` 限制為編譯時常數，避免任意動態字串污染 audit。
    System {
        reason: &'static str,
    },
    /// 尚未登入的可記錄事件（login 嘗試、CSP report、honeypot 命中等）。
    /// 匿名者**沒有專屬身份資料**；IP/UA 等請求脈絡透過 `RequestContext`
    /// 傳遞，操作內容（例如命中的蜜罐端點、違反的 CSP rule）透過
    /// `ActivityLogEntry::data_diff` 傳遞。
    Anonymous,
}

impl ActorContext {
    /// 取得 `user_activity_logs.actor_user_id` 欄位值。
    /// Anonymous 回 None（FK 欄位允許 NULL，代表匿名）。
    pub fn actor_user_id(&self) -> Option<Uuid> {
        match self {
            Self::User(u) => Some(u.id),
            Self::System { .. } => Some(SYSTEM_USER_ID),
            Self::Anonymous => None,
        }
    }

    /// 僅 User 變體有 CurrentUser（權限檢查用）。
    pub fn as_user(&self) -> Option<&CurrentUser> {
        match self {
            Self::User(u) => Some(u),
            _ => None,
        }
    }

    /// 要求必須是 User 變體，否則回 Forbidden。
    /// Service 層某些操作只能由真實登入使用者發起，用這個守門。
    pub fn require_user(&self) -> crate::Result<&CurrentUser> {
        self.as_user().ok_or_else(|| {
            crate::AppError::Forbidden(
                "此操作必須由已登入使用者觸發（System/Anonymous 不可）".into(),
            )
        })
    }

    /// audit 來源標籤（寫入 user_activity_logs.context_data 的 source 欄位）。
    pub fn source_label(&self) -> &'static str {
        match self {
            Self::User(_) => "user",
            Self::System { .. } => "system",
            Self::Anonymous => "anonymous",
        }
    }

    /// System reason（非 System 變體回 None）。
    pub fn system_reason(&self) -> Option<&'static str> {
        match self {
            Self::System { reason } => Some(*reason),
            _ => None,
        }
    }

    /// Impersonation 偵測：若 actor 為 User 且該 user 是被模擬的（SEC-11），
    /// 回傳真正操作的管理員 UUID；否則 None。
    pub fn impersonated_by(&self) -> Option<Uuid> {
        self.as_user().and_then(|u| u.impersonated_by)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_user(id: Uuid) -> CurrentUser {
        CurrentUser {
            id,
            email: "test@example.com".into(),
            roles: vec!["USER".into()],
            permissions: vec![],
            jti: "test-jti".into(),
            exp: 0,
            impersonated_by: None,
        }
    }

    #[test]
    fn user_actor_returns_user_id() {
        let uid = Uuid::new_v4();
        let actor = ActorContext::User(make_user(uid));
        assert_eq!(actor.actor_user_id(), Some(uid));
        assert_eq!(actor.source_label(), "user");
        assert!(actor.as_user().is_some());
    }

    #[test]
    fn system_actor_returns_system_uuid() {
        let actor = ActorContext::System {
            reason: "balance_expiration",
        };
        assert_eq!(actor.actor_user_id(), Some(SYSTEM_USER_ID));
        assert_eq!(actor.source_label(), "system");
        assert_eq!(actor.system_reason(), Some("balance_expiration"));
        assert!(actor.as_user().is_none());
    }

    #[test]
    fn anonymous_actor_returns_none_id() {
        let actor = ActorContext::Anonymous;
        assert_eq!(actor.actor_user_id(), None);
        assert_eq!(actor.source_label(), "anonymous");
        assert!(actor.as_user().is_none());
        // Anonymous 沒有 impersonation（也沒有專屬資料）
        assert!(actor.impersonated_by().is_none());
    }

    #[test]
    fn impersonation_detected_from_user_actor() {
        let uid = Uuid::new_v4();
        let admin_id = Uuid::new_v4();
        let mut user = make_user(uid);
        user.impersonated_by = Some(admin_id);
        let actor = ActorContext::User(user);
        assert_eq!(actor.impersonated_by(), Some(admin_id));
    }

    #[test]
    fn system_actor_never_impersonated() {
        let actor = ActorContext::System { reason: "cron" };
        assert_eq!(actor.impersonated_by(), None);
    }

    #[test]
    fn require_user_fails_on_system() {
        let actor = ActorContext::System { reason: "test" };
        let result = actor.require_user();
        assert!(result.is_err());
    }

    #[test]
    fn require_user_succeeds_on_user() {
        let uid = Uuid::new_v4();
        let actor = ActorContext::User(make_user(uid));
        let user = actor.require_user().expect("user actor should succeed");
        assert_eq!(user.id, uid);
    }

    #[test]
    fn system_user_id_is_constant() {
        // SYSTEM_USER_ID 與 migration 033 必須一致；改動會破壞 FK。
        assert_eq!(
            SYSTEM_USER_ID.to_string(),
            "00000000-0000-0000-0000-000000000001"
        );
    }
}
