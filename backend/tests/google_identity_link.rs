//! R121：Google 登入「找帳號／自動綁定」的資料庫層整合測試。
//!
//! `AuthService::resolve_google_user` 決定「哪個 Google 帳號可以登入成哪個系統帳號」，
//! 是帳號接管風險最集中的地方，但它全部都在 SQL 上，`--lib` 單元測試碰不到。
//! 這裡逐條釘住使用者 2026-09-23 裁定的規則（見 `docs/TODO.md` R121）：
//! 只開放既有帳號、email 相同（不分大小寫）且已驗證才自動綁定、之後以 `sub` 為準。

mod common;

use erp_backend::models::User;
use erp_backend::services::audit::RequestContext;
use erp_backend::services::google_oidc::{
    GoogleAuthResult, GoogleLoginFailure, VerifiedGoogleIdentity,
};
use erp_backend::services::AuthService;
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

/// 每支測試各自產生不會撞號的 email 與 Google sub（測試庫不清理，跨次執行共用）。
fn unique(label: &str) -> String {
    format!("{label}-{}", &Uuid::new_v4().simple().to_string()[..12])
}

async fn seed_user(pool: &PgPool, email: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password) \
         VALUES ($1, $2, 'x', $3, true, false)",
    )
    .bind(id)
    .bind(email)
    .bind(format!("Google 綁定測試 {email}"))
    .execute(pool)
    .await
    .expect("seed user");
    id
}

fn identity(subject: &str, email: &str) -> VerifiedGoogleIdentity {
    VerifiedGoogleIdentity {
        subject: subject.to_string(),
        email: email.to_string(),
    }
}

fn no_context() -> RequestContext<'static> {
    RequestContext {
        ip_address: None,
        user_agent: None,
    }
}

async fn resolve(pool: &PgPool, id: &VerifiedGoogleIdentity) -> GoogleAuthResult {
    AuthService::resolve_google_user(pool, id, no_context())
        .await
        .expect("resolve_google_user 不應有基礎設施錯誤")
}

fn expect_authenticated(result: GoogleAuthResult) -> User {
    match result {
        GoogleAuthResult::Authenticated(user) => *user,
        GoogleAuthResult::Rejected(f) => panic!("預期登入成功，實際被拒：{}", f.code()),
    }
}

fn expect_rejected(result: GoogleAuthResult) -> GoogleLoginFailure {
    match result {
        GoogleAuthResult::Authenticated(user) => {
            panic!("預期被拒，實際登入成帳號 {}", user.email)
        }
        GoogleAuthResult::Rejected(f) => f,
    }
}

async fn identities_of(pool: &PgPool, user_id: Uuid) -> Vec<(String, String)> {
    sqlx::query_as(
        "SELECT subject, email_at_link FROM user_identities \
         WHERE user_id = $1 AND provider = 'google' ORDER BY linked_at",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .expect("query user_identities")
}

async fn count_identities_by_subject(pool: &PgPool, subject: &str) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM user_identities WHERE subject = $1")
        .bind(subject)
        .fetch_one(pool)
        .await
        .expect("count by subject")
}

#[tokio::test]
#[serial]
async fn first_login_links_by_email_case_insensitively_and_writes_audit() {
    let app = common::TestApp::spawn().await;
    let local = unique("Mixed.Case");
    let stored_email = format!("{local}@Example.com");
    let user_id = seed_user(&app.db_pool, &stored_email).await;
    let sub = unique("sub");

    // Google 回的 email 大小寫與建帳時不同，仍應對到同一個帳號
    let google = identity(&sub, &stored_email.to_lowercase());
    let user = expect_authenticated(resolve(&app.db_pool, &google).await);

    assert_eq!(user.id, user_id);
    assert_eq!(
        identities_of(&app.db_pool, user_id).await,
        vec![(sub.clone(), stored_email.to_lowercase())],
        "應建立恰好一筆綁定，email_at_link 記 Google 回報的值"
    );

    let linked_audits: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM user_activity_logs \
         WHERE event_type = 'EXTERNAL_IDENTITY_LINKED' AND entity_id = $1",
    )
    .bind(user_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("count link audit");
    assert_eq!(linked_audits, 1, "自動綁定必須留下稽核");
}

#[tokio::test]
#[serial]
async fn bound_identity_is_found_by_subject_even_after_email_changes() {
    let app = common::TestApp::spawn().await;
    let email = format!("{}@example.com", unique("bound"));
    let user_id = seed_user(&app.db_pool, &email).await;
    let sub = unique("sub");
    expect_authenticated(resolve(&app.db_pool, &identity(&sub, &email)).await);

    // 之後使用者在 Google 端改了 email：仍以 sub 登入同一個帳號，也不會另建綁定
    let renamed = format!("{}@elsewhere.example", unique("renamed"));
    let user = expect_authenticated(resolve(&app.db_pool, &identity(&sub, &renamed)).await);

    assert_eq!(user.id, user_id);
    assert_eq!(identities_of(&app.db_pool, user_id).await.len(), 1);
    let last_login: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT last_login_at FROM user_identities WHERE subject = $1")
            .bind(&sub)
            .fetch_one(&app.db_pool)
            .await
            .expect("read last_login_at");
    assert!(last_login.is_some(), "以 sub 登入要更新 last_login_at");
}

#[tokio::test]
#[serial]
async fn unknown_email_is_rejected_and_nothing_is_created() {
    let app = common::TestApp::spawn().await;
    let sub = unique("sub");
    let email = format!("{}@example.com", unique("nobody"));

    let failure = expect_rejected(resolve(&app.db_pool, &identity(&sub, &email)).await);

    assert_eq!(failure, GoogleLoginFailure::NoAccount);
    assert_eq!(count_identities_by_subject(&app.db_pool, &sub).await, 0);
    let users_created: i64 =
        sqlx::query_scalar("SELECT count(*) FROM users WHERE lower(email) = lower($1)")
            .bind(&email)
            .fetch_one(&app.db_pool)
            .await
            .expect("count users");
    assert_eq!(users_created, 0, "只開放既有帳號，不可自動開帳號");
}

#[tokio::test]
#[serial]
async fn email_matching_two_accounts_case_insensitively_is_rejected() {
    let app = common::TestApp::spawn().await;
    let local = unique("dup");
    // users.email 的唯一約束分大小寫，所以這兩筆可以並存
    seed_user(&app.db_pool, &format!("{local}@example.com")).await;
    seed_user(
        &app.db_pool,
        &format!("{}@example.com", local.to_uppercase()),
    )
    .await;
    let sub = unique("sub");

    let failure = expect_rejected(
        resolve(
            &app.db_pool,
            &identity(&sub, &format!("{local}@example.com")),
        )
        .await,
    );

    assert_eq!(
        failure,
        GoogleLoginFailure::AmbiguousAccount,
        "不猜是哪一個"
    );
    assert_eq!(count_identities_by_subject(&app.db_pool, &sub).await, 0);
}

#[tokio::test]
#[serial]
async fn second_google_account_with_same_email_cannot_take_over() {
    let app = common::TestApp::spawn().await;
    let email = format!("{}@example.com", unique("owner"));
    let user_id = seed_user(&app.db_pool, &email).await;
    let original_sub = unique("sub-original");
    expect_authenticated(resolve(&app.db_pool, &identity(&original_sub, &email)).await);

    // 另一個 Google 帳號宣稱同一個（已驗證）email：不得覆寫、不得疊加
    let intruder_sub = unique("sub-intruder");
    let failure = expect_rejected(resolve(&app.db_pool, &identity(&intruder_sub, &email)).await);

    assert_eq!(failure, GoogleLoginFailure::IdentityConflict);
    assert_eq!(
        identities_of(&app.db_pool, user_id).await,
        vec![(original_sub, email)],
        "原本的綁定必須原封不動"
    );
}

#[tokio::test]
#[serial]
async fn disabled_expired_or_deleted_accounts_are_not_linked() {
    let app = common::TestApp::spawn().await;

    let disabled_email = format!("{}@example.com", unique("disabled"));
    let disabled = seed_user(&app.db_pool, &disabled_email).await;
    sqlx::query("UPDATE users SET is_active = false WHERE id = $1")
        .bind(disabled)
        .execute(&app.db_pool)
        .await
        .expect("disable user");

    let expired_email = format!("{}@example.com", unique("expired"));
    let expired = seed_user(&app.db_pool, &expired_email).await;
    sqlx::query("UPDATE users SET expires_at = NOW() - INTERVAL '1 day' WHERE id = $1")
        .bind(expired)
        .execute(&app.db_pool)
        .await
        .expect("expire user");

    let deleted_email = format!("{}@example.com", unique("deleted"));
    let deleted = seed_user(&app.db_pool, &deleted_email).await;
    sqlx::query("UPDATE users SET deleted_at = NOW() WHERE id = $1")
        .bind(deleted)
        .execute(&app.db_pool)
        .await
        .expect("soft delete user");

    for (email, expected) in [
        (&disabled_email, GoogleLoginFailure::AccountUnavailable),
        (&expired_email, GoogleLoginFailure::AccountUnavailable),
        (&deleted_email, GoogleLoginFailure::NoAccount),
    ] {
        let sub = unique("sub");
        let failure = expect_rejected(resolve(&app.db_pool, &identity(&sub, email)).await);
        assert_eq!(failure, expected, "{email}");
        assert_eq!(
            count_identities_by_subject(&app.db_pool, &sub).await,
            0,
            "被拒的帳號不可建立綁定：{email}"
        );
    }
}

#[tokio::test]
#[serial]
async fn bound_account_disabled_later_is_rejected() {
    let app = common::TestApp::spawn().await;
    let email = format!("{}@example.com", unique("later-disabled"));
    let user_id = seed_user(&app.db_pool, &email).await;
    let sub = unique("sub");
    expect_authenticated(resolve(&app.db_pool, &identity(&sub, &email)).await);

    sqlx::query("UPDATE users SET is_active = false WHERE id = $1")
        .bind(user_id)
        .execute(&app.db_pool)
        .await
        .expect("disable user");

    let failure = expect_rejected(resolve(&app.db_pool, &identity(&sub, &email)).await);
    assert_eq!(failure, GoogleLoginFailure::AccountUnavailable);
}

#[tokio::test]
#[serial]
async fn guest_demo_account_is_never_linked() {
    let app = common::TestApp::spawn().await;
    let guest_email = erp_backend::constants::GUEST_DEMO_EMAIL;
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, display_name, is_active) \
         VALUES ($1, $2, 'x', 'Guest', true) ON CONFLICT (email) DO NOTHING",
    )
    .bind(Uuid::new_v4())
    .bind(guest_email)
    .execute(&app.db_pool)
    .await
    .expect("ensure guest user");
    let sub = unique("sub");

    let failure = expect_rejected(resolve(&app.db_pool, &identity(&sub, guest_email)).await);

    assert_eq!(failure, GoogleLoginFailure::NoAccount);
    assert_eq!(count_identities_by_subject(&app.db_pool, &sub).await, 0);
}

#[tokio::test]
#[serial]
async fn concurrent_first_logins_create_exactly_one_binding() {
    let app = common::TestApp::spawn().await;
    let email = format!("{}@example.com", unique("race"));
    let user_id = seed_user(&app.db_pool, &email).await;
    let sub = unique("sub");
    let google = identity(&sub, &email);

    // 使用者連點兩次：兩個 callback 同時走到「尚未綁定」→ 兩邊都嘗試 INSERT
    let (a, b) = tokio::join!(
        resolve(&app.db_pool, &google),
        resolve(&app.db_pool, &google)
    );

    assert_eq!(expect_authenticated(a).id, user_id);
    assert_eq!(expect_authenticated(b).id, user_id);
    assert_eq!(identities_of(&app.db_pool, user_id).await.len(), 1);
}
