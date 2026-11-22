//! 審查意見「回覆 / 回覆草稿」的物件層授權回歸測試。
//!
//! 守的規則：持有回覆權限碼不等於可以對任意計畫的審查意見動手——
//! 還必須與該意見所屬的計畫有關聯（`access::require_review_reply_access`）。
//!
//! 每一個拒絕斷言都配一個**同形狀請求**的放行對照組：同一個使用者、同一種 body，
//! 只把目標換成自己計畫的意見。少了對照組，403 也可能來自壞掉的 fixture
//! （角色代碼錯、body 不合法先被擋下），測不到授權本身。
//!
//! 角色代碼來源：`src/startup/permissions.rs` 的 `"PI"`、`"IACUC_STAFF"` 區塊。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::services::AuthService;

const PASSWORD: &str = "iPig$ecure1";

// ── fixtures ──────────────────────────────────────────────────────────────

/// 建立不可登入的使用者（僅供 FK 參照）。
async fn seed_user(app: &TestApp, label: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', $3, true, false)"#,
    )
    .bind(id)
    .bind(format!("reply-scope-{label}-{}@test.local", &Uuid::new_v4().to_string()[..8]))
    .bind(format!("reply scope {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    id
}

/// 建立可登入、帶指定角色的使用者，回傳 (id, access token)。
async fn seed_login_user(app: &TestApp, label: &str, role_code: &str) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let email = format!(
        "reply-scope-{label}-{}@test.local",
        &Uuid::new_v4().to_string()[..8]
    );
    let hash = AuthService::hash_password(PASSWORD).expect("hash password");
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password)
           VALUES ($1, $2, $3, $4, true, true, false)"#,
    )
    .bind(id)
    .bind(&email)
    .bind(&hash)
    .bind(format!("reply scope {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert login user");
    let assigned = sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
    )
    .bind(id)
    .bind(role_code)
    .execute(&app.db_pool)
    .await
    .expect("assign role");
    // 角色代碼打錯時 INSERT ... SELECT 會安靜地寫入 0 列，使用者變成零角色，
    // 之後的 403 就不是在測授權。這裡直接擋下。
    assert_eq!(
        assigned.rows_affected(),
        1,
        "角色代碼 {role_code} 不存在，fixture 無效"
    );
    let token = app.login(&email, PASSWORD).await.expect("login 應成功");
    (id, token)
}

/// 建立一份計畫（pi_user_id = `pi`）與它的一個版本，回傳 (protocol_id, version_id)。
async fn seed_protocol_with_version(app: &TestApp, pi: Uuid) -> (Uuid, Uuid) {
    let protocol_id = Uuid::new_v4();
    let version_id = Uuid::new_v4();
    let unique = &Uuid::new_v4().to_string()[..8];
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by, created_at, updated_at)
           VALUES ($1, $2, $3, 'reply scope regression', 'UNDER_REVIEW'::protocol_status, $4, $4, NOW(), NOW())"#,
    )
    .bind(protocol_id)
    .bind(format!("PRS-{unique}"))
    .bind(format!("IACUCRS-{unique}"))
    .bind(pi)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol");
    sqlx::query(
        r#"INSERT INTO protocol_versions (id, protocol_id, version_no, content_snapshot, submitted_by)
           VALUES ($1, $2, 1, '{}'::jsonb, $3)"#,
    )
    .bind(version_id)
    .bind(protocol_id)
    .bind(pi)
    .execute(&app.db_pool)
    .await
    .expect("insert protocol version");
    (protocol_id, version_id)
}

/// 建立一則頂層審查意見，可選擇預先帶草稿。
async fn seed_comment(
    app: &TestApp,
    protocol_id: Uuid,
    version_id: Uuid,
    reviewer: Uuid,
    draft: Option<&str>,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO review_comments
               (id, protocol_version_id, protocol_id, reviewer_id, content, draft_content, created_at, updated_at)
           VALUES ($1, $2, $3, $4, 'please clarify', $5, NOW(), NOW())"#,
    )
    .bind(id)
    .bind(version_id)
    .bind(protocol_id)
    .bind(reviewer)
    .bind(draft)
    .execute(&app.db_pool)
    .await
    .expect("insert review comment");
    id
}

async fn reply_count(app: &TestApp, parent: Uuid) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM review_comments WHERE parent_comment_id = $1")
        .bind(parent)
        .fetch_one(&app.db_pool)
        .await
        .expect("count replies")
}

async fn draft_of(app: &TestApp, comment: Uuid) -> Option<String> {
    sqlx::query_scalar("SELECT draft_content FROM review_comments WHERE id = $1")
        .bind(comment)
        .fetch_one(&app.db_pool)
        .await
        .expect("read draft")
}

/// 兩份計畫、各一則意見：`own_*` 屬於 PI 本人，`other_*` 屬於別人。
struct Scene {
    pi_token: String,
    own_comment: Uuid,
    other_comment: Uuid,
}

const OTHER_DRAFT: &str = "draft written by the other protocol";

async fn seed_scene(app: &TestApp) -> Scene {
    let (pi, pi_token) = seed_login_user(app, "pi", "PI").await;
    let other_pi = seed_user(app, "other-pi").await;
    let reviewer = seed_user(app, "reviewer").await;

    let (own_protocol, own_version) = seed_protocol_with_version(app, pi).await;
    let (other_protocol, other_version) = seed_protocol_with_version(app, other_pi).await;

    let own_comment = seed_comment(app, own_protocol, own_version, reviewer, None).await;
    let other_comment = seed_comment(
        app,
        other_protocol,
        other_version,
        reviewer,
        Some(OTHER_DRAFT),
    )
    .await;

    Scene {
        pi_token,
        own_comment,
        other_comment,
    }
}

// ── 回覆 ──────────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn pi_cannot_reply_to_comment_on_unrelated_protocol() {
    let app = TestApp::spawn().await;
    let scene = seed_scene(&app).await;

    let res = app
        .auth_post(
            "/api/v1/reviews/comments/reply",
            &serde_json::json!({ "parent_comment_id": scene.other_comment, "content": "injected" }),
            &scene.pi_token,
        )
        .await;
    let status = res.status().as_u16();
    let body = res.text().await.unwrap_or_default();
    assert_eq!(
        status, 403,
        "別人計畫的意見不得回覆，實得 {status}，body={body}"
    );
    assert_eq!(
        reply_count(&app, scene.other_comment).await,
        0,
        "被拒絕的回覆不得寫入"
    );

    // 對照組：同一個人、同形狀的 body，目標換成自己計畫的意見 → 放行且確實寫入。
    let res = app
        .auth_post(
            "/api/v1/reviews/comments/reply",
            &serde_json::json!({ "parent_comment_id": scene.own_comment, "content": "my reply" }),
            &scene.pi_token,
        )
        .await;
    let status = res.status().as_u16();
    let body = res.text().await.unwrap_or_default();
    assert_eq!(
        status, 200,
        "自己計畫的意見應可回覆，實得 {status}，body={body}"
    );
    assert_eq!(reply_count(&app, scene.own_comment).await, 1);
}

// ── 儲存草稿 ──────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn pi_cannot_overwrite_draft_on_unrelated_protocol() {
    let app = TestApp::spawn().await;
    let scene = seed_scene(&app).await;

    let res = app
        .auth_post(
            "/api/v1/reviews/comments/draft",
            &serde_json::json!({ "comment_id": scene.other_comment, "draft_content": "planted" }),
            &scene.pi_token,
        )
        .await;
    let status = res.status().as_u16();
    let body = res.text().await.unwrap_or_default();
    assert_eq!(
        status, 403,
        "別人計畫的回覆草稿不得覆寫，實得 {status}，body={body}"
    );
    assert_eq!(
        draft_of(&app, scene.other_comment).await.as_deref(),
        Some(OTHER_DRAFT),
        "被拒絕的請求不得改動既有草稿"
    );

    // 對照組：自己計畫的意見 → 放行且確實寫入。
    let res = app
        .auth_post(
            "/api/v1/reviews/comments/draft",
            &serde_json::json!({ "comment_id": scene.own_comment, "draft_content": "my draft" }),
            &scene.pi_token,
        )
        .await;
    let status = res.status().as_u16();
    let body = res.text().await.unwrap_or_default();
    assert_eq!(
        status, 200,
        "自己計畫的草稿應可儲存，實得 {status}，body={body}"
    );
    assert_eq!(
        draft_of(&app, scene.own_comment).await.as_deref(),
        Some("my draft")
    );
}

// ── 讀取草稿 ──────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn pi_cannot_read_draft_on_unrelated_protocol() {
    let app = TestApp::spawn().await;
    let scene = seed_scene(&app).await;

    let res = app
        .auth_get(
            &format!("/api/v1/reviews/comments/{}/draft", scene.other_comment),
            &scene.pi_token,
        )
        .await;
    let status = res.status().as_u16();
    let body = res.text().await.unwrap_or_default();
    assert_eq!(
        status, 403,
        "別人計畫的回覆草稿不得讀取，實得 {status}，body={body}"
    );
    assert!(
        !body.contains(OTHER_DRAFT),
        "拒絕回應不得夾帶草稿內容，body={body}"
    );

    // 對照組：先存一份自己的草稿，再讀回來。
    let res = app
        .auth_post(
            "/api/v1/reviews/comments/draft",
            &serde_json::json!({ "comment_id": scene.own_comment, "draft_content": "readable" }),
            &scene.pi_token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 200, "前置：儲存自己的草稿應成功");
    let res = app
        .auth_get(
            &format!("/api/v1/reviews/comments/{}/draft", scene.own_comment),
            &scene.pi_token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 200, "自己計畫的草稿應可讀取");
    let draft: Option<String> = TestApp::json(res).await;
    assert_eq!(draft.as_deref(), Some("readable"));
}

// ── 不得過度封鎖：可跨計畫檢視的角色照舊 ──────────────────────────────────

#[tokio::test]
#[serial]
async fn view_all_role_with_reply_permission_keeps_cross_protocol_access() {
    let app = TestApp::spawn().await;
    let scene = seed_scene(&app).await;
    // IACUC_STAFF：持回覆權限碼，且可檢視所有計畫；與 scene 的兩份計畫都無成員關係。
    let (_, staff_token) = seed_login_user(&app, "staff", "IACUC_STAFF").await;

    let res = app
        .auth_post(
            "/api/v1/reviews/comments/draft",
            &serde_json::json!({ "comment_id": scene.other_comment, "draft_content": "staff draft" }),
            &staff_token,
        )
        .await;
    let status = res.status().as_u16();
    let body = res.text().await.unwrap_or_default();
    assert_eq!(
        status, 200,
        "可檢視所有計畫且持回覆權限者應照舊放行，實得 {status}，body={body}"
    );
}

// ── 不存在的意見 ──────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn reply_to_missing_comment_returns_404() {
    let app = TestApp::spawn().await;
    let scene = seed_scene(&app).await;

    let res = app
        .auth_post(
            "/api/v1/reviews/comments/reply",
            &serde_json::json!({ "parent_comment_id": Uuid::new_v4(), "content": "nobody home" }),
            &scene.pi_token,
        )
        .await;
    assert_eq!(res.status().as_u16(), 404, "不存在的意見應回 404");
}
