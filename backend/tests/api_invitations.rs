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
    let pi_role_id = app.pi_role_id().await;

    // 建立邀請
    let create_res = app
        .auth_post(
            "/api/v1/invitations",
            &serde_json::json!({
                "email": &pi_email,
                "display_name": "PI Test Admin Endpoint",
                "organization": "Test Lab",
                "role_ids": [pi_role_id],
                // PI 為外部角色。`is_internal` 是必填（無 serde default），
                // 漏帶會 400 —— 那是刻意的，見 CreateInvitationRequest 的註解。
                "is_internal": false
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

    // PI 不能建立邀請：payload 須含 required 欄位才能通過 Json extractor 跑到
    // require_permission!；否則 Axum 422 短路、test 會收到 422 而非預期的 403
    let invite_res = app
        .auth_post(
            "/api/v1/invitations",
            &serde_json::json!({
                "email": "another@test.local",
                "display_name": "Another User",
                "organization": "Test Lab",
                "role_ids": [pi_role_id],
                // PI 為外部角色。`is_internal` 是必填（無 serde default），
                // 漏帶會 400 —— 那是刻意的，見 CreateInvitationRequest 的註解。
                "is_internal": false
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
    let pi_role_id = app.pi_role_id().await;

    let create_res = app
        .auth_post(
            "/api/v1/invitations",
            &serde_json::json!({
                "email": &pi_email,
                "display_name": "PI ERP Test",
                "organization": "ERP Test Lab",
                "role_ids": [pi_role_id],
                // PI 為外部角色。`is_internal` 是必填（無 serde default），
                // 漏帶會 400 —— 那是刻意的，見 CreateInvitationRequest 的註解。
                "is_internal": false
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

    let inventory_res = app.auth_get("/api/v1/erp/inventory", &pi_token).await;
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
    let pi_role_id = app.pi_role_id().await;

    let create_res = app
        .auth_post(
            "/api/v1/invitations",
            &serde_json::json!({
                "email": &pi_email,
                "display_name": "PI Project Test",
                "organization": "Project Test Lab",
                "role_ids": [pi_role_id],
                // PI 為外部角色。`is_internal` 是必填（無 serde default），
                // 漏帶會 400 —— 那是刻意的，見 CreateInvitationRequest 的註解。
                "is_internal": false
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
    let my_projects_res = app.auth_get("/api/v1/my-projects", &pi_token).await;
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
    let pi_role_id = app.pi_role_id().await;

    let create_res = app
        .auth_post(
            "/api/v1/invitations",
            &serde_json::json!({
                "email": &email,
                "display_name": "Expired User",
                "organization": "Expire Test",
                "role_ids": [pi_role_id],
                // PI 為外部角色。`is_internal` 是必填（無 serde default），
                // 漏帶會 400 —— 那是刻意的，見 CreateInvitationRequest 的註解。
                "is_internal": false
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
    sqlx::query(
        "UPDATE invitations SET expires_at = NOW() - INTERVAL '1 day' WHERE invitation_token = $1",
    )
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
    let pi_role_id = app.pi_role_id().await;

    let create_res = app
        .auth_post(
            "/api/v1/invitations",
            &serde_json::json!({
                "email": &email,
                "display_name": "Used User",
                "organization": "Used Test",
                "role_ids": [pi_role_id],
                // PI 為外部角色。`is_internal` 是必填（無 serde default），
                // 漏帶會 400 —— 那是刻意的，見 CreateInvitationRequest 的註解。
                "is_internal": false
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
async fn expired_invitation_hidden_when_email_already_accepted() {
    // 同一 email 同時存在「已接受」與「已過期」邀請時，列表應隱藏過期那筆，
    // 避免同一人重複出現於「已過期」與「已接受」。
    let app = common::TestApp::spawn().await;
    let admin_token = app.login_as_admin().await;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time should be valid")
        .subsec_nanos();

    let email = format!("dedup_{}@test.local", ts);
    let pi_role_id = app.pi_role_id().await;

    // 1. 建立邀請並接受 → 該 email 取得一筆 accepted 邀請
    let create_res = app
        .auth_post(
            "/api/v1/invitations",
            &serde_json::json!({
                "email": &email,
                "display_name": "Dedup User",
                "organization": "Dedup Lab",
                "role_ids": [pi_role_id],
                // PI 為外部角色。`is_internal` 是必填（無 serde default），
                // 漏帶會 400 —— 那是刻意的，見 CreateInvitationRequest 的註解。
                "is_internal": false
            }),
            &admin_token,
        )
        .await;

    let create_body: serde_json::Value = create_res
        .json()
        .await
        .expect("Failed to parse create invitation response");
    let invitation_token = create_body["invite_link"]
        .as_str()
        .expect("invite_link should be present")
        .rsplit('/')
        .next()
        .expect("Should have token in link");

    let accept_res = app
        .client
        .post(app.url("/api/v1/invitations/accept"))
        .json(&serde_json::json!({
            "invitation_token": invitation_token,
            "display_name": "Dedup User",
            "phone": "0900000006",
            "organization": "Dedup Lab",
            "password": "DedupTest123!",
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

    // 2. 為同一 email 直接插入一筆過期邀請（模擬先前未接受、已轉 expired 的舊邀請）
    let admin_id: uuid::Uuid =
        sqlx::query_scalar("SELECT invited_by FROM invitations WHERE email = $1 LIMIT 1")
            .bind(&email)
            .fetch_one(&app.db_pool)
            .await
            .expect("Failed to fetch invited_by");

    sqlx::query(
        r#"
        -- is_internal 必須明確帶值：migration 154 撤掉了欄位的 DEFAULT，
        -- 漏帶會直接違反 NOT NULL（刻意如此，見該 migration 的註解）。
        -- 這張是「已過期的同 email 邀請」，內外部與斷言無關，取 false。
        INSERT INTO invitations (email, invitation_token, invited_by, status, expires_at, is_internal)
        VALUES ($1, $2, $3, 'expired', NOW() - INTERVAL '1 day', false)
        "#,
    )
    .bind(&email)
    .bind(format!("expired-sibling-{}", ts))
    .bind(admin_id)
    .execute(&app.db_pool)
    .await
    .expect("Failed to insert expired sibling invitation");

    // 3. 「已過期」分頁不應包含此 email（已被 accepted 取代）
    let expired_list: serde_json::Value = common::TestApp::json(
        app.auth_get(
            "/api/v1/invitations?status=expired&per_page=100",
            &admin_token,
        )
        .await,
    )
    .await;
    let expired_has_email = expired_list["data"]
        .as_array()
        .expect("data should be array")
        .iter()
        .any(|inv| inv["email"] == serde_json::json!(email));
    assert!(
        !expired_has_email,
        "已接受後，同 email 的過期邀請不應出現在『已過期』列表"
    );

    // 4. 「已接受」分頁仍應包含此 email
    let accepted_list: serde_json::Value = common::TestApp::json(
        app.auth_get(
            "/api/v1/invitations?status=accepted&per_page=100",
            &admin_token,
        )
        .await,
    )
    .await;
    let accepted_has_email = accepted_list["data"]
        .as_array()
        .expect("data should be array")
        .iter()
        .any(|inv| inv["email"] == serde_json::json!(email));
    assert!(accepted_has_email, "『已接受』列表應包含已接受邀請的 email");
}

#[tokio::test]
#[serial]
async fn resend_expired_invitation_reactivates_it() {
    // 已過期邀請可重新發送：狀態回到 pending、換新 token、展延效期。
    let app = common::TestApp::spawn().await;
    let admin_token = app.login_as_admin().await;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time should be valid")
        .subsec_nanos();

    let email = format!("resend_{}@test.local", ts);
    let pi_role_id = app.pi_role_id().await;

    let create_res = app
        .auth_post(
            "/api/v1/invitations",
            &serde_json::json!({
                "email": &email,
                "display_name": "Resend User",
                "organization": "Resend Lab",
                "role_ids": [pi_role_id],
                // PI 為外部角色。`is_internal` 是必填（無 serde default），
                // 漏帶會 400 —— 那是刻意的，見 CreateInvitationRequest 的註解。
                "is_internal": false
            }),
            &admin_token,
        )
        .await;
    let create_body: serde_json::Value = create_res
        .json()
        .await
        .expect("Failed to parse create invitation response");
    let inv_id = create_body["invitation"]["id"]
        .as_str()
        .expect("invitation id should be present");
    let original_token = create_body["invite_link"]
        .as_str()
        .expect("invite_link should be present")
        .rsplit('/')
        .next()
        .expect("Should have token in link")
        .to_string();

    // 直接在 DB 將邀請設為已過期
    sqlx::query(
        "UPDATE invitations SET status = 'expired', expires_at = NOW() - INTERVAL '1 day' WHERE id = $1",
    )
    .bind(uuid::Uuid::parse_str(inv_id).expect("invalid uuid"))
    .execute(&app.db_pool)
    .await
    .expect("Failed to expire invitation");

    // 重新發送
    let resend_res = app
        .auth_post(
            &format!("/api/v1/invitations/{}/resend", inv_id),
            &serde_json::json!({}),
            &admin_token,
        )
        .await;
    assert!(
        resend_res.status().is_success(),
        "重新發送已過期邀請應成功，實際狀態: {}",
        resend_res.status()
    );

    let resend_body: serde_json::Value = resend_res
        .json()
        .await
        .expect("Failed to parse resend response");
    assert_eq!(resend_body["status"], "pending", "重發後狀態應回到 pending");

    let new_token = resend_body["invite_link"]
        .as_str()
        .expect("invite_link should be present after resend")
        .rsplit('/')
        .next()
        .expect("Should have token in link");
    assert_ne!(new_token, original_token, "重發應換新 token");
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
    let pi_role_id = app.pi_role_id().await;

    let create_res = app
        .auth_post(
            "/api/v1/invitations",
            &serde_json::json!({
                "email": &email,
                "display_name": "Weak Password User",
                "organization": "Weak Pwd Test",
                "role_ids": [pi_role_id],
                // PI 為外部角色。`is_internal` 是必填（無 serde default），
                // 漏帶會 400 —— 那是刻意的，見 CreateInvitationRequest 的註解。
                "is_internal": false
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

    let body: serde_json::Value = res.json().await.expect("Failed to parse verify response");

    assert_eq!(body["valid"], false, "無效 token 應回傳 valid: false");
    assert_eq!(
        body["reason"], "not_found",
        "無效 token 的 reason 應為 not_found"
    );
}

/// 邀請單的 `is_internal = true` 必須原封傳到 `users.is_internal`。
///
/// 為什麼要單獨一支：本檔其餘 fixture **一律** 用 `is_internal: false`（PI 是外部
/// 角色），所以「true 這條路」在此之前完全沒有被測到。而它正是本次變更的核心契約
/// ——接受邀請的 SQL 先前硬編 `false`，導致所有走邀請進來的人（含具內部職能者）
/// 都被記成外部人員，連帶讓他們無法被加入部門、假單找不到單位主管簽核。
///
/// 若日後有人把接受端的 INSERT 改回硬編、或漏接這個欄位，只有這支會紅。
#[tokio::test]
#[serial]
async fn accepted_invitation_preserves_internal_affiliation() {
    let app = common::TestApp::spawn().await;
    let admin_token = app.login_as_admin().await;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time should be valid")
        .subsec_nanos();

    let email = format!("internal_invite_{}@test.local", ts);
    let password = "InternalPass123!";
    let role_id = app.pi_role_id().await;

    let create_res = app
        .auth_post(
            "/api/v1/invitations",
            &serde_json::json!({
                "email": &email,
                "display_name": "Internal Invitee",
                "organization": "Test Lab",
                "role_ids": [role_id],
                // 本支的重點：走 true 這條路。角色本身是不是「內部角色」不影響
                // 判定——同一個角色可能是自己人也可能是外聘（見 migration 154 註解），
                // 所以這個值只由邀請人明確指定。
                "is_internal": true
            }),
            &admin_token,
        )
        .await;
    assert!(
        create_res.status().is_success(),
        "建立邀請應成功，實際狀態: {}",
        create_res.status()
    );

    // ① 邀請單自己要記住 true（不是被 DEFAULT 或硬編蓋掉）
    let stored_on_invitation: bool =
        sqlx::query_scalar("SELECT is_internal FROM invitations WHERE email = $1")
            .bind(&email)
            .fetch_one(&app.db_pool)
            .await
            .expect("Failed to read invitation is_internal");
    assert!(
        stored_on_invitation,
        "邀請單應記錄 is_internal=true，實得 false"
    );

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
            "display_name": "Internal Invitee",
            "phone": "0912345678",
            "organization": "Test Lab",
            "password": password,
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

    // ② 建立出來的使用者要繼承 true——這是整條契約真正的落點
    let user_is_internal: bool =
        sqlx::query_scalar("SELECT is_internal FROM users WHERE email = $1")
            .bind(&email)
            .fetch_one(&app.db_pool)
            .await
            .expect("Failed to read created user is_internal");
    assert!(
        user_is_internal,
        "接受邀請建立的使用者應繼承邀請單的 is_internal=true，實得 false"
    );
}
