//! Integration tests for invitation flow:
//! R19-13: 權限隔離測試
//! R19-14: 安全測試

mod common;

use serial_test::serial;

// ── R19-13: 權限隔離測試 ────────────────────────────────

#[tokio::test]
#[serial]
async fn create_invitation_requires_permission() {
    let app = common::TestApp::spawn().await;

    // 未認證的請求應被拒絕
    let res = app
        .client
        .post(app.url("/api/v1/invitations"))
        .json(&serde_json::json!({
            "email": "test@example.com"
        }))
        .send()
        .await
        .expect("HTTP request failed");

    assert_eq!(
        res.status(),
        401,
        "建立邀請應要求認證，未帶 token 應回傳 401"
    );
}

#[tokio::test]
#[serial]
async fn list_invitations_requires_permission() {
    let app = common::TestApp::spawn().await;

    // 未認證的請求應被拒絕
    let res = app
        .client
        .get(app.url("/api/v1/invitations"))
        .send()
        .await
        .expect("HTTP request failed");

    assert_eq!(
        res.status(),
        401,
        "列出邀請應要求認證，未帶 token 應回傳 401"
    );
}

#[tokio::test]
#[serial]
async fn pi_user_cannot_access_admin_endpoints() {
    let app = common::TestApp::spawn().await;
    let admin_token = app.login_as_admin().await;

    // 先透過邀請流程建立一個 PI 使用者
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time should be valid")
        .subsec_nanos();

    let pi_email = format!("pi_test_{}@test.local", ts);
    let pi_password = "PiTestPass123!";

    // 建立邀請
    let create_res = app
        .auth_post(
            "/api/v1/invitations",
            &serde_json::json!({
                "email": &pi_email,
                "organization": "Test Lab"
            }),
            &admin_token,
        )
        .await;

    // 取得邀請 token
    let create_body: serde_json::Value = create_res
        .json()
        .await
        .expect("Failed to parse create invitation response");
    let invite_link = create_body["invite_link"]
        .as_str()
        .expect("invite_link should be present");
    let invitation_token = invite_link
        .rsplit('/')
        .next()
        .expect("Should have token in link");

    // 接受邀請，建立 PI 帳號
    let accept_res = app
        .client
        .post(app.url("/api/v1/invitations/accept"))
        .json(&serde_json::json!({
            "invitation_token": invitation_token,
            "display_name": "PI Test User",
            "phone": "0912345678",
            "organization": "Test Lab",
            "password": pi_password,
            "position": "Researcher",
            "agree_terms": true
        }))
        .send()
        .await
        .expect("HTTP request failed");

    assert!(
        accept_res.status().is_success(),
        "接受邀請應成功，實際狀態: {}",
        accept_res.status()
    );

    // 用 PI 帳號登入
    let pi_token = app
        .login(&pi_email, pi_password)
        .await
        .expect("PI user login should succeed");

    // PI 不能存取使用者管理 (admin)
    let users_res = app.auth_get("/api/v1/users", &pi_token).await;
    assert!(
        users_res.status() == 403 || users_res.status() == 401,
        "PI 不應能存取 /api/v1/users，實際狀態: {}",
        users_res.status()
    );

    // PI 不能建立邀請
    let invite_res = app
        .auth_post(
            "/api/v1/invitations",
            &serde_json::json!({
                "email": "another@test.local"
            }),
            &pi_token,
        )
        .await;
    assert!(
        invite_res.status() == 403 || invite_res.status() == 401,
        "PI 不應能建立邀請，實際狀態: {}",
        invite_res.status()
    );
}

#[tokio::test]
#[serial]
async fn pi_user_cannot_access_erp_endpoints() {
    let app = common::TestApp::spawn().await;
    let admin_token = app.login_as_admin().await;

    // 建立 PI 使用者
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time should be valid")
        .subsec_nanos();

    let pi_email = format!("pi_erp_test_{}@test.local", ts);
    let pi_password = "PiErpTest123!";

    let create_res = app
        .auth_post(
            "/api/v1/invitations",
            &serde_json::json!({
                "email": &pi_email,
                "organization": "ERP Test Lab"
            }),
            &admin_token,
        )
        .await;

    let create_body: serde_json::Value = create_res
        .json()
        .await
        .expect("Failed to parse create invitation response");
    let invite_link = create_body["invite_link"]
        .as_str()
        .expect("invite_link should be present");
    let invitation_token = invite_link
        .rsplit('/')
        .next()
        .expect("Should have token in link");

    let accept_res = app
        .client
        .post(app.url("/api/v1/invitations/accept"))
        .json(&serde_json::json!({
            "invitation_token": invitation_token,
            "display_name": "PI ERP Test",
            "phone": "0912345679",
            "organization": "ERP Test Lab",
            "password": pi_password,
            "position": "PI",
            "agree_terms": true
        }))
        .send()
        .await
        .expect("HTTP request failed");

    assert!(
        accept_res.status().is_success(),
        "接受邀請應成功，實際狀態: {}",
        accept_res.status()
    );

    let pi_token = app
        .login(&pi_email, pi_password)
        .await
        .expect("PI user login should succeed");

    // PI 不能存取 ERP 端點（例如進貨單、庫存等）
    // 路由可能尚未實作（404）或有權限保護（401/403），兩者皆可接受
    let grn_res = app.auth_get("/api/v1/erp/grn", &pi_token).await;
    assert!(
        matches!(grn_res.status().as_u16(), 401 | 403 | 404),
        "PI 不應能存取 ERP GRN，實際狀態: {}",
        grn_res.status()
    );

    let inventory_res = app
        .auth_get("/api/v1/erp/inventory", &pi_token)
        .await;
    assert!(
        matches!(inventory_res.status().as_u16(), 401 | 403 | 404),
        "PI 不應能存取 ERP 庫存，實際狀態: {}",
        inventory_res.status()
    );
}

#[tokio::test]
#[serial]
async fn pi_user_can_access_my_projects() {
    let app = common::TestApp::spawn().await;
    let admin_token = app.login_as_admin().await;

    // 建立 PI 使用者
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time should be valid")
        .subsec_nanos();

    let pi_email = format!("pi_proj_test_{}@test.local", ts);
    let pi_password = "PiProjTest123!";

    let create_res = app
        .auth_post(
            "/api/v1/invitations",
            &serde_json::json!({
                "email": &pi_email,
                "organization": "Project Test Lab"
            }),
            &admin_token,
        )
        .await;

    let create_body: serde_json::Value = create_res
        .json()
        .await
        .expect("Failed to parse create invitation response");
    let invite_link = create_body["invite_link"]
        .as_str()
        .expect("invite_link should be present");
    let invitation_token = invite_link
        .rsplit('/')
        .next()
        .expect("Should have token in link");

    let accept_res = app
        .client
        .post(app.url("/api/v1/invitations/accept"))
        .json(&serde_json::json!({
            "invitation_token": invitation_token,
            "display_name": "PI Project Test",
            "phone": "0912345680",
            "organization": "Project Test Lab",
            "password": pi_password,
            "position": "PI",
            "agree_terms": true
        }))
        .send()
        .await
        .expect("HTTP request failed");

    assert!(
        accept_res.status().is_success(),
        "接受邀請應成功，實際狀態: {}",
        accept_res.status()
    );

    let pi_token = app
        .login(&pi_email, pi_password)
        .await
        .expect("PI user login should succeed");

    // PI 應該能存取自己的計劃書
    let my_projects_res = app
        .auth_get("/api/v1/my-projects", &pi_token)
        .await;
    assert_eq!(
        my_projects_res.status(),
        200,
        "PI 應能存取 /api/v1/my-projects，實際狀態: {}",
        my_projects_res.status()
    );
}

// ── R19-14: 安全測試 ────────────────────────────────────

#[tokio::test]
#[serial]
async fn accept_with_expired_token_fails() {
    let app = common::TestApp::spawn().await;
    let admin_token = app.login_as_admin().await;

    // 建立邀請
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time should be valid")
        .subsec_nanos();

    let email = format!("expire_test_{}@test.local", ts);

    let create_res = app
        .auth_post(
            "/api/v1/invitations",
            &serde_json::json!({
                "email": &email,
                "organization": "Expire Test"
            }),
            &admin_token,
        )
        .await;

    let create_body: serde_json::Value = create_res
        .json()
        .await
        .expect("Failed to parse create invitation response");
    let invite_link = create_body["invite_link"]
        .as_str()
        .expect("invite_link should be present");
    let invitation_token = invite_link
        .rsplit('/')
        .next()
        .expect("Should have token in link");

    // 直接在 DB 把過期時間設為過去
    sqlx::query("UPDATE invitations SET expires_at = NOW() - INTERVAL '1 day' WHERE invitation_token = $1")
        .bind(invitation_token)
        .execute(&app.db_pool)
        .await
        .expect("Failed to update invitation expiry");

    // 嘗試接受過期邀請
    let accept_res = app
        .client
        .post(app.url("/api/v1/invitations/accept"))
        .json(&serde_json::json!({
            "invitation_token": invitation_token,
            "display_name": "Expired User",
            "phone": "0900000001",
            "organization": "Expire Test",
            "password": "ExpiredTest123!",
            "position": "Researcher",
            "agree_terms": true
        }))
        .send()
        .await
        .expect("HTTP request failed");

    assert_eq!(
        accept_res.status(),
        400,
        "接受過期邀請應回傳 400，實際狀態: {}",
        accept_res.status()
    );
}

#[tokio::test]
#[serial]
async fn accept_with_used_token_fails() {
    let app = common::TestApp::spawn().await;
    let admin_token = app.login_as_admin().await;

    // 建立邀請
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time should be valid")
        .subsec_nanos();

    let email = format!("used_test_{}@test.local", ts);

    let create_res = app
        .auth_post(
            "/api/v1/invitations",
            &serde_json::json!({
                "email": &email,
                "organization": "Used Test"
            }),
            &admin_token,
        )
        .await;

    let create_body: serde_json::Value = create_res
        .json()
        .await
        .expect("Failed to parse create invitation response");
    let invite_link = create_body["invite_link"]
        .as_str()
        .expect("invite_link should be present");
    let invitation_token = invite_link
        .rsplit('/')
        .next()
        .expect("Should have token in link");

    // 第一次接受（應成功）
    let accept_res = app
        .client
        .post(app.url("/api/v1/invitations/accept"))
        .json(&serde_json::json!({
            "invitation_token": invitation_token,
            "display_name": "First Accept",
            "phone": "0900000002",
            "organization": "Used Test",
            "password": "UsedTest1234!",
            "position": "Researcher",
            "agree_terms": true
        }))
        .send()
        .await
        .expect("HTTP request failed");

    assert!(
        accept_res.status().is_success(),
        "第一次接受應成功，實際狀態: {}",
        accept_res.status()
    );

    // 第二次接受（應失敗 — token 已使用）
    let accept_again = app
        .client
        .post(app.url("/api/v1/invitations/accept"))
        .json(&serde_json::json!({
            "invitation_token": invitation_token,
            "display_name": "Second Accept",
            "phone": "0900000003",
            "organization": "Used Test",
            "password": "UsedTest5678!",
            "position": "Researcher",
            "agree_terms": true
        }))
        .send()
        .await
        .expect("HTTP request failed");

    assert!(
        accept_again.status() == 400 || accept_again.status() == 409,
        "重複接受應回傳 400 或 409，實際狀態: {}",
        accept_again.status()
    );
}

#[tokio::test]
#[serial]
async fn accept_with_invalid_token_returns_404() {
    let app = common::TestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/invitations/accept"))
        .json(&serde_json::json!({
            "invitation_token": "completely-invalid-nonexistent-token",
            "display_name": "Invalid Token User",
            "phone": "0900000004",
            "organization": "Test",
            "password": "InvalidToken123!",
            "position": "Researcher",
            "agree_terms": true
        }))
        .send()
        .await
        .expect("HTTP request failed");

    assert!(
        res.status() == 404 || res.status() == 400,
        "無效 token 應回傳 404 或 400，實際狀態: {}",
        res.status()
    );
}

#[tokio::test]
#[serial]
async fn accept_with_weak_password_fails() {
    let app = common::TestApp::spawn().await;
    let admin_token = app.login_as_admin().await;

    // 建立邀請
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time should be valid")
        .subsec_nanos();

    let email = format!("weak_pwd_{}@test.local", ts);

    let create_res = app
        .auth_post(
            "/api/v1/invitations",
            &serde_json::json!({
                "email": &email,
                "organization": "Weak Pwd Test"
            }),
            &admin_token,
        )
        .await;

    let create_body: serde_json::Value = create_res
        .json()
        .await
        .expect("Failed to parse create invitation response");
    let invite_link = create_body["invite_link"]
        .as_str()
        .expect("invite_link should be present");
    let invitation_token = invite_link
        .rsplit('/')
        .next()
        .expect("Should have token in link");

    // 使用太短的密碼（< 8 字元，model 驗證 min=8）
    let res = app
        .client
        .post(app.url("/api/v1/invitations/accept"))
        .json(&serde_json::json!({
            "invitation_token": invitation_token,
            "display_name": "Weak Password User",
            "phone": "0900000005",
            "organization": "Weak Pwd Test",
            "password": "ab",
            "position": "Researcher",
            "agree_terms": true
        }))
        .send()
        .await
        .expect("HTTP request failed");

    assert!(
        res.status() == 400 || res.status() == 422,
        "弱密碼應被拒絕（400 或 422），實際狀態: {}",
        res.status()
    );
}

#[tokio::test]
#[serial]
async fn verify_with_invalid_token_returns_appropriate_response() {
    let app = common::TestApp::spawn().await;

    // 驗證不存在的 token（公開端點，不需認證）
    let res = app
        .client
        .get(app.url("/api/v1/invitations/verify/nonexistent-token-xyz"))
        .send()
        .await
        .expect("HTTP request failed");

    assert_eq!(
        res.status(),
        200,
        "verify 端點應回傳 200（含 valid: false），實際狀態: {}",
        res.status()
    );

    let body: serde_json::Value = res
        .json()
        .await
        .expect("Failed to parse verify response");

    assert_eq!(
        body["valid"], false,
        "無效 token 應回傳 valid: false"
    );
    assert_eq!(
        body["reason"], "not_found",
        "無效 token 的 reason 應為 not_found"
    );
}
