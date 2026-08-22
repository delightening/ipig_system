//! Integration tests for authentication flow:
//! login → me → refresh → logout → verify token revoked

mod common;

use erp_backend::constants::REFRESH_TOKEN_REUSE_RACE_WINDOW_SECS;
use serial_test::serial;

// ── Login ────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn login_with_valid_credentials_returns_200_and_tokens() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;
    assert!(!token.is_empty());
}

#[tokio::test]
#[serial]
async fn login_with_wrong_password_returns_401() {
    let app = common::TestApp::spawn().await;

    let email =
        std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@ipigsystem.asia".to_string());

    let res = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&serde_json::json!({
            "email": email,
            "password": "definitely_wrong_password"
        }))
        .send()
        .await
        .expect("HTTP request failed");

    assert_eq!(res.status(), 401);
}

#[tokio::test]
#[serial]
async fn login_with_invalid_email_format_returns_400() {
    let app = common::TestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&serde_json::json!({
            "email": "not-an-email",
            "password": "anything"
        }))
        .send()
        .await
        .expect("HTTP request failed");

    assert_eq!(res.status(), 400);
}

// ── Me ───────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn me_returns_current_user_info() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app.auth_get("/api/v1/me", &token).await;
    assert_eq!(res.status(), 200);

    let body: serde_json::Value = res.json().await.expect("Failed to parse JSON response");
    assert!(body["id"].is_string());
    assert!(body["email"].is_string());
    assert!(body["display_name"].is_string());
}

#[tokio::test]
#[serial]
async fn me_without_token_returns_401() {
    let app = common::TestApp::spawn().await;

    let res = app
        .client
        .get(app.url("/api/v1/me"))
        .send()
        .await
        .expect("HTTP request failed");

    assert_eq!(res.status(), 401);
}

/// GOV: PUT /me 不得讓使用者自行指派 AUP 可擔任角色（aup_roles）。
/// aup_roles（PI / Co-PI / 獸醫師…）屬權責宣告，須由 admin/IACUC 指派；
/// 自填值必須被後端忽略（與 role_ids 等 SEC-PRIV 欄位同樣處理）。
#[tokio::test]
#[serial]
async fn update_me_ignores_self_assigned_aup_roles() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    // 先取基準值，避免依賴「帳號從未有某角色」的 fixture 假設（CodeRabbit 建議）
    let before = app.auth_get("/api/v1/me", &token).await;
    assert_eq!(before.status(), 200);
    let before_body: serde_json::Value =
        before.json().await.expect("Failed to parse JSON response");
    let before_roles = before_body["aup_roles"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let res = app
        .auth_put(
            "/api/v1/me",
            &serde_json::json!({ "aup_roles": ["PI", "Veterinarian"] }),
            &token,
        )
        .await;
    assert_eq!(res.status(), 200);

    let me = app.auth_get("/api/v1/me", &token).await;
    assert_eq!(me.status(), 200);
    let body: serde_json::Value = me.json().await.expect("Failed to parse JSON response");
    let roles = body["aup_roles"].as_array().cloned().unwrap_or_default();
    assert_eq!(roles, before_roles, "PUT /me 自填 aup_roles 不應改變既有值");
}

// ── Refresh ──────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn refresh_with_valid_token_returns_new_tokens() {
    let app = common::TestApp::spawn().await;

    let email =
        std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@ipigsystem.asia".to_string());
    let password =
        std::env::var("ADMIN_INITIAL_PASSWORD").unwrap_or_else(|_| "iPig$ecure1".to_string());

    // Login to get refresh_token
    let login_res = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&serde_json::json!({ "email": email, "password": password }))
        .send()
        .await
        .expect("HTTP request failed");
    assert_eq!(login_res.status(), 200);

    let login_body: serde_json::Value = login_res
        .json()
        .await
        .expect("Failed to parse login response");
    let refresh_token = login_body["refresh_token"]
        .as_str()
        .expect("refresh_token should be present in login response");

    // Use refresh_token to get new tokens
    let refresh_res = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&serde_json::json!({ "refresh_token": refresh_token }))
        .send()
        .await
        .expect("HTTP request failed");

    assert_eq!(refresh_res.status(), 200);
    let refresh_body: serde_json::Value = refresh_res
        .json()
        .await
        .expect("Failed to parse refresh response");
    assert!(refresh_body["access_token"].is_string());
    assert!(refresh_body["refresh_token"].is_string());
}

/// R35-15 / R46: 真 reuse（超過 race window）→ 整 family 撤銷 + 寫 security_alert。
/// 本測試在第一次 refresh 後手動 backdate `rotated_at` 跳過 `REFRESH_TOKEN_REUSE_RACE_WINDOW_SECS`
/// 模擬「使用者離開很久後 token 被重用」的真 leak 情境。
#[tokio::test]
#[serial]
async fn refresh_token_reuse_revokes_entire_family() {
    let app = common::TestApp::spawn().await;

    let email =
        std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@ipigsystem.asia".to_string());
    let password =
        std::env::var("ADMIN_INITIAL_PASSWORD").unwrap_or_else(|_| "iPig$ecure1".to_string());

    // Login → token T1
    let login_res = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&serde_json::json!({ "email": email, "password": password }))
        .send()
        .await
        .expect("HTTP request failed");
    let login_body: serde_json::Value = login_res.json().await.expect("login body");
    let t1 = login_body["refresh_token"]
        .as_str()
        .expect("T1")
        .to_string();

    // First refresh: T1 → T2（T1 revoked normal_rotation, rotated_at = NOW()）
    let r1_res = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&serde_json::json!({ "refresh_token": t1 }))
        .send()
        .await
        .expect("HTTP request failed");
    assert_eq!(r1_res.status(), 200, "first refresh should succeed");
    let r1_body: serde_json::Value = r1_res.json().await.expect("r1 body");
    let t2 = r1_body["refresh_token"].as_str().expect("T2").to_string();

    // R46-1: backdate rotated_at 跳過 race window，模擬真 leak（非併發 race）。
    // backdate 用 race window 常數 × 2 動態計算，避免常數調整時測試漂移（CodeRabbit
    // PR #472 nitpick）。WHERE 限縮到本測試使用者避免汙染其他 #[serial] 測試。
    let backdate_secs = REFRESH_TOKEN_REUSE_RACE_WINDOW_SECS * 2;
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "UPDATE refresh_tokens
         SET rotated_at = NOW() - INTERVAL '{backdate_secs} seconds'
         WHERE rotated_at IS NOT NULL
           AND user_id = (SELECT id FROM users WHERE email = $1)"
    )))
    .bind(&email)
    .execute(&app.db_pool)
    .await
    .expect("backdate rotated_at");

    // Reuse T1 → reuse detected → T2 應一併撤銷
    let r2_res = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&serde_json::json!({ "refresh_token": t1 }))
        .send()
        .await
        .expect("HTTP request failed");
    assert!(!r2_res.status().is_success(), "reused T1 must be rejected");

    // T2 也應該無法使用（family revoke）
    let r3_res = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&serde_json::json!({ "refresh_token": t2 }))
        .send()
        .await
        .expect("HTTP request failed");
    assert!(
        !r3_res.status().is_success(),
        "T2 must also be rejected after reuse detection (family revoke)"
    );
}

/// R46-1: race window 內的 reuse 視為併發 race（多分頁同時 refresh），
/// 僅拒絕當次請求，**不撤銷整 family**、不寫 security_alert。
#[tokio::test]
#[serial]
async fn refresh_token_reuse_within_race_window_does_not_revoke_family() {
    let app = common::TestApp::spawn().await;

    let email =
        std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@ipigsystem.asia".to_string());
    let password =
        std::env::var("ADMIN_INITIAL_PASSWORD").unwrap_or_else(|_| "iPig$ecure1".to_string());

    // 清空本測試使用者的既有 alert 避免汙染斷言；限縮 WHERE 保持 #[serial] 測試隔離。
    sqlx::query(
        "DELETE FROM security_alerts
         WHERE alert_type = 'REFRESH_TOKEN_REUSE'
           AND user_id = (SELECT id FROM users WHERE email = $1)",
    )
    .bind(&email)
    .execute(&app.db_pool)
    .await
    .expect("clear alerts");

    // Login → T1
    let login_res = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&serde_json::json!({ "email": email, "password": password }))
        .send()
        .await
        .expect("HTTP request failed");
    let login_body: serde_json::Value = login_res.json().await.expect("login body");
    let t1 = login_body["refresh_token"]
        .as_str()
        .expect("T1")
        .to_string();

    // T1 → T2（rotated_at = NOW()，race window 起算點）
    let r1_res = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&serde_json::json!({ "refresh_token": t1 }))
        .send()
        .await
        .expect("HTTP request failed");
    assert_eq!(r1_res.status(), 200);
    let r1_body: serde_json::Value = r1_res.json().await.expect("r1 body");
    let t2 = r1_body["refresh_token"].as_str().expect("T2").to_string();

    // Reuse T1 在 race window 內 → 應該被拒，但不觸發 family revoke
    let r2_res = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&serde_json::json!({ "refresh_token": t1 }))
        .send()
        .await
        .expect("HTTP request failed");
    assert!(
        !r2_res.status().is_success(),
        "reused T1 in race window still rejected"
    );

    // T2 應該還能用（race window 抑制 family revoke）
    let r3_res = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&serde_json::json!({ "refresh_token": t2 }))
        .send()
        .await
        .expect("HTTP request failed");
    assert_eq!(
        r3_res.status(),
        200,
        "T2 must still be usable — race window suppresses family revoke"
    );

    // 不應產生 REFRESH_TOKEN_REUSE security_alert
    let alert_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM security_alerts WHERE alert_type = 'REFRESH_TOKEN_REUSE'",
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("count alerts");
    assert_eq!(
        alert_count, 0,
        "race window reuse must not write security_alert"
    );
}

// ── Logout ───────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn logout_invalidates_token() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    // Verify token works
    let me_res = app.auth_get("/api/v1/me", &token).await;
    assert_eq!(me_res.status(), 200);

    // Logout
    let logout_res = app.auth_post("/api/v1/auth/logout", &(), &token).await;
    assert!(logout_res.status().is_success() || logout_res.status() == 200);

    // Token should now be blacklisted
    let me_after = app.auth_get("/api/v1/me", &token).await;
    assert_eq!(me_after.status(), 401);
}

// ── Validation error message ─────────────────────────────────

#[tokio::test]
#[serial]
async fn login_validation_error_does_not_leak_field_names() {
    let app = common::TestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&serde_json::json!({
            "email": "not-an-email",
            "password": "anything"
        }))
        .send()
        .await
        .expect("HTTP request failed");

    assert_eq!(res.status(), 400);

    let body: serde_json::Value = res.json().await.expect("Failed to parse JSON");
    let message = body["message"].as_str().unwrap_or("");

    // The response must NOT expose field-level details like "email: is not a valid..."
    // which would leak our API schema to attackers.
    assert!(
        !message.contains("email:"),
        "Error message leaks field name 'email:' — got: {:?}",
        message
    );
    assert!(
        !message.to_lowercase().contains("is not a valid email"),
        "Error message leaks validator detail — got: {:?}",
        message
    );
}

// ── 2FA rate limiting ─────────────────────────────────────────

#[tokio::test]
#[serial]
async fn two_factor_verify_locks_after_5_failures() {
    use chrono::{Duration, Utc};
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

    let app = common::TestApp::spawn().await;

    // Fetch admin user (guaranteed to exist after spawn)
    let (user_id_str, email): (String, String) = sqlx::query_as(
        "SELECT id::text, email FROM users WHERE is_active = true ORDER BY created_at LIMIT 1",
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch test user");

    // Build a valid 2FA temp token using the same ES256 private key as the running server
    // (common::TestApp::spawn() sets JWT_EC_PRIVATE_KEY in env)
    let private_pem = std::env::var("JWT_EC_PRIVATE_KEY")
        .expect("JWT_EC_PRIVATE_KEY must be set (done by TestApp::spawn)");
    let encoding_key = EncodingKey::from_ec_pem(private_pem.as_bytes())
        .expect("Failed to parse test EC private key");

    let now = Utc::now();
    let exp = (now + Duration::seconds(300)).timestamp() as usize;
    let claims = serde_json::json!({
        "sub": user_id_str,
        "purpose": "2fa_pending",
        "exp": exp,
        "iat": now.timestamp(),
    });

    let temp_token = encode(&Header::new(Algorithm::ES256), &claims, &encoding_key)
        .expect("Failed to sign temp token");

    // Pre-seed 5 2fa_failure events for this email within the 5-minute window
    for _ in 0..5 {
        sqlx::query(
            r#"INSERT INTO login_events (id, user_id, email, event_type, created_at)
               VALUES (gen_random_uuid(), $1::uuid, $2, '2fa_failure', NOW())"#,
        )
        .bind(&user_id_str)
        .bind(&email)
        .execute(&app.db_pool)
        .await
        .expect("Failed to insert 2fa_failure event");
    }

    // The next attempt (6th failure) must be rate-limited
    let res = app
        .client
        .post(app.url("/api/v1/auth/2fa/verify"))
        .json(&serde_json::json!({
            "temp_token": temp_token,
            "code": "123456"
        }))
        .send()
        .await
        .expect("HTTP request failed");

    // Cleanup before asserting so the DB is clean even if the test fails
    sqlx::query("DELETE FROM login_events WHERE email = $1 AND event_type = '2fa_failure'")
        .bind(&email)
        .execute(&app.db_pool)
        .await
        .expect("Failed to clean up 2fa_failure events");

    assert_eq!(
        res.status(),
        429,
        "Expected 429 TooManyRequests after 5 pre-seeded 2FA failures"
    );
}

// ── Password change ──────────────────────────────────────────

#[tokio::test]
#[serial]
async fn change_password_with_wrong_current_returns_error() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let res = app
        .auth_put(
            "/api/v1/me/password",
            &serde_json::json!({
                "current_password": "wrong_current_password",
                "new_password": "NewPassword123!",
                // C3：新密碼確認；不填會被 ChangeOwnPasswordRequest 反序列化拒絕
                "new_password_confirmation": "NewPassword123!"
            }),
            &token,
        )
        .await;

    // Should be 400 or 401 (wrong current password)
    assert!(res.status() == 400 || res.status() == 401);
}
