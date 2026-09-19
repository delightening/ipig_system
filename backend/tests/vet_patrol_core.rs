//! R82-3 回歸防護網：`services/animal/vet_patrol.rs` + `handlers/animal/vet_patrol.rs`
//! 原為零測試的獸醫巡場核心（1941 行 / 32 個 async fn）。
//!
//! 涵蓋範圍：
//! (a) 巡場報告 CRUD 與權限 gate（未認證 401 / 角色權限 403）
//! (b) #928 回歸：報告產生（create + get）時各筆觀察的 observation / suggestion 內容
//!     須完整帶出（資料層；PDF 位元組層不在本檔範圍）
//! (c) R39++ 三階段流程（submit → acknowledge → complete）+ audit log 落地驗證（≥2 個事件）
//!
//! 不驗證：照片附件端點、PDF 位元組輸出（依派工範圍排除）。

mod common;

use common::TestApp;
use erp_backend::services::AuthService;
use serial_test::serial;
use uuid::Uuid;

const PASSWORD: &str = "TestPassword123!";

/// 某份巡場報告的置頂待辦現況：`(通知總列數, 其中仍置頂的列數, 收件人)`。
///
/// 這支 PR 的主修正是「狀態轉換時在同一個 tx 內解除置頂待辦」。
/// 那些解除呼叫在 service 層，若只測 reconcile（安全網）而不測這裡，
/// 把 4 個解除全部 revert 掉整套測試仍會全綠 —— 主修正等於裸奔。
///
/// 回傳三個值而非只回置頂數，是為了讓斷言能區分「**降級**」與「**刪除**」：
/// 只看 `priority > 0` 的話，把整列 DELETE 掉也會讓計數變 0 而測試照過。
/// 收件人一併回傳，避免「有建立待辦但發錯人」通過測試。
async fn pinned_state_for(app: &TestApp, report_id: &str) -> (i64, i64, Option<Uuid>) {
    let id: Uuid = report_id.parse().expect("report id");
    // Postgres 沒有 min(uuid)；用 array_agg 取第一個相異收件人。
    // 搭配 total 一起斷言即可涵蓋「發錯人」與「發給多人」兩種錯誤。
    sqlx::query_as(
        r#"SELECT count(*)                              AS total,
                  count(*) FILTER (WHERE priority > 0)  AS pinned,
                  (array_agg(DISTINCT user_id))[1]      AS recipient
           FROM notifications
           WHERE related_entity_type = 'vet_patrol_reports'
             AND related_entity_id = $1
             AND title LIKE '%需您填寫追蹤改善%'"#,
    )
    .bind(id)
    .fetch_one(&app.db_pool)
    .await
    .expect("query pinned notification state")
}

/// 建立指定角色（role code，如 "VET" / "PI" / "STUDY_DIRECTOR"）的使用者並登入，回傳 (user_id, token)。
async fn seed_user_with_role(app: &TestApp, role_code: &str, label: &str) -> (Uuid, String) {
    let id = Uuid::new_v4();
    let email = format!(
        "vp_{}_{}@test.local",
        label,
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
    .bind(format!("VP Test {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
    )
    .bind(id)
    .bind(role_code)
    .execute(&app.db_pool)
    .await
    .expect("assign role");
    let token = app.login(&email, PASSWORD).await.expect("login failed");
    (id, token)
}

/// 建立一隻最小動物 fixture（供 entries.animal_ids 關聯用）。
async fn seed_animal(app: &TestApp, tag_prefix: &str, created_by: Uuid) -> Uuid {
    let aid = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO animals (id, ear_tag, breed, gender, entry_date, created_by)
           VALUES ($1, $2, 'miniature', 'male', '2026-01-01', $3)"#,
    )
    .bind(aid)
    .bind(format!("{}{}", tag_prefix, &aid.to_string()[..5]))
    .bind(created_by)
    .execute(&app.db_pool)
    .await
    .expect("insert animal");
    aid
}

/// 查某 report_id 在 user_activity_logs 中特定 event_type 的筆數。
async fn audit_count(app: &TestApp, event_type: &str, report_id: Uuid) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_activity_logs WHERE event_type = $1 AND entity_id = $2",
    )
    .bind(event_type)
    .bind(report_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("count audit rows")
}

/// 查某動物在 animal_vet_advice_records（病歷獸醫師建議）的有效筆數。
async fn advice_count(app: &TestApp, animal_id: Uuid) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM animal_vet_advice_records WHERE animal_id = $1 AND deleted_at IS NULL",
    )
    .bind(animal_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("count advice rows")
}

// ── (a) 權限 gate：未認證 ──────────────────────────────

#[tokio::test]
#[serial]
async fn list_vet_patrol_reports_without_auth_returns_401() {
    let app = TestApp::spawn().await;
    let res = app
        .client
        .get(app.url("/api/v1/vet-patrol-reports"))
        .send()
        .await
        .expect("request failed");
    assert_eq!(res.status(), 401);
}

#[tokio::test]
#[serial]
async fn create_vet_patrol_report_without_auth_returns_401() {
    let app = TestApp::spawn().await;
    let res = app
        .client
        .post(app.url("/api/v1/vet-patrol-reports"))
        .json(&serde_json::json!({"patrol_date": "2026-07-01", "entries": []}))
        .send()
        .await
        .expect("request failed");
    assert_eq!(res.status(), 401);
}

// ── (a) 權限 gate：角色權限不足 ──────────────────────────────

#[tokio::test]
#[serial]
async fn create_vet_patrol_report_forbidden_without_write_permission() {
    // PI 無 animal.vet.recommend → require_permission! 應擋下（403）
    let app = TestApp::spawn().await;
    let (_id, token) = seed_user_with_role(&app, "PI", "pi-create").await;
    let res = app
        .auth_post(
            "/api/v1/vet-patrol-reports",
            &serde_json::json!({"patrol_date": "2026-07-01", "entries": []}),
            &token,
        )
        .await;
    assert_eq!(res.status(), 403);
}

#[tokio::test]
#[serial]
async fn list_vet_patrol_reports_forbidden_without_view_permission() {
    // PI 無 animal.animal.view_all → require_vet_patrol_view 應擋下（403）
    let app = TestApp::spawn().await;
    let (_id, token) = seed_user_with_role(&app, "PI", "pi-view").await;
    let res = app.auth_get("/api/v1/vet-patrol-reports", &token).await;
    assert_eq!(res.status(), 403);
}

/// 讀寫 gate 分離的回歸：具 `animal.animal.view_all` 者看得到巡場報告，
/// 但沒有 `animal.vet.recommend` 就不能新增。
///
/// 原本這支測試用的是全域 `STUDY_DIRECTOR` 角色（靠角色身分通過讀取閘）。
/// 該角色已於 2026-08-17 移除——SD 只存在於 `protocols.study_director_user_id`，
/// 而擔任 SD 的人一律具 `EXPERIMENT_STAFF`，本來就從 view_all 那條通過。
/// 故改用 EXPERIMENT_STAFF 驗證同一組契約。
#[tokio::test]
#[serial]
async fn experiment_staff_can_view_but_cannot_create() {
    let app = TestApp::spawn().await;
    let (_id, token) = seed_user_with_role(&app, "EXPERIMENT_STAFF", "expstaff").await;

    let list_res = app.auth_get("/api/v1/vet-patrol-reports", &token).await;
    assert_eq!(list_res.status(), 200);

    let create_res = app
        .auth_post(
            "/api/v1/vet-patrol-reports",
            &serde_json::json!({"patrol_date": "2026-07-01", "entries": []}),
            &token,
        )
        .await;
    assert_eq!(create_res.status(), 403);
}

#[tokio::test]
#[serial]
async fn update_draft_forbidden_for_non_creator() {
    let app = TestApp::spawn().await;
    let (_owner_id, owner_token) = seed_user_with_role(&app, "VET", "update-owner").await;
    let (_other_id, other_token) = seed_user_with_role(&app, "VET", "update-other").await;

    let create_res = app
        .auth_post(
            "/api/v1/vet-patrol-reports",
            &serde_json::json!({"patrol_date": "2026-07-02", "entries": []}),
            &owner_token,
        )
        .await;
    assert_eq!(create_res.status(), 200);
    let created: serde_json::Value = create_res.json().await.expect("parse create response");
    let report_id = created["id"].as_str().expect("id").to_string();

    let update_res = app
        .auth_put(
            &format!("/api/v1/vet-patrol-reports/{report_id}"),
            &serde_json::json!({"patrol_date": "2026-07-03", "entries": []}),
            &other_token,
        )
        .await;
    assert_eq!(
        update_res.status(),
        403,
        "非發起者不可編輯他人草稿，實得 {}",
        update_res.status()
    );
}

#[tokio::test]
#[serial]
async fn submit_for_followup_forbidden_for_non_creator() {
    let app = TestApp::spawn().await;
    let (_owner_id, owner_token) = seed_user_with_role(&app, "VET", "submit-owner").await;
    let (_other_id, other_token) = seed_user_with_role(&app, "VET", "submit-other").await;
    let (tracker_id, _tracker_token) = seed_user_with_role(&app, "VET", "submit-tracker").await;

    let create_res = app
        .auth_post(
            "/api/v1/vet-patrol-reports",
            &serde_json::json!({"patrol_date": "2026-07-04", "entries": []}),
            &owner_token,
        )
        .await;
    let created: serde_json::Value = create_res.json().await.expect("parse");
    let report_id = created["id"].as_str().expect("id").to_string();

    // other（非發起者）嘗試送出 → 403
    let res = app
        .auth_post(
            &format!("/api/v1/vet-patrol-reports/{report_id}/submit-for-followup"),
            &serde_json::json!({"follow_up_user_id": tracker_id}),
            &other_token,
        )
        .await;
    assert_eq!(res.status(), 403);
}

// ── (b) #928 回歸：各筆觀察內容 + 建議須完整帶出（資料層） ──────────────────────────────

#[tokio::test]
#[serial]
async fn create_report_returns_all_entries_with_observation_and_suggestion() {
    let app = TestApp::spawn().await;
    let (vet_id, token) = seed_user_with_role(&app, "VET", "entries-creator").await;
    let a1 = seed_animal(&app, "OB1", vet_id).await;
    let a2 = seed_animal(&app, "OB2", vet_id).await;

    let entries = serde_json::json!([
        {
            "category": "health",
            "animal_ids": [a1],
            "observation": "觀察一：食慾不振",
            "suggestion": "建議一：追蹤體重",
            "follow_up": "",
            "sort_order": 0
        },
        {
            "category": "health",
            "animal_ids": [a2],
            "observation": "觀察二：跛行",
            "suggestion": "建議二：安排X光",
            "follow_up": "",
            "sort_order": 1
        },
        {
            "category": "other",
            "animal_ids": [],
            "observation": "觀察三：欄舍清潔",
            "suggestion": "建議三：加強消毒",
            "follow_up": "",
            "sort_order": 2
        }
    ]);
    let body = serde_json::json!({
        "patrol_date": "2026-07-05",
        "accompanying_personnel": "測試陪同員",
        "entries": entries
    });

    let create_res = app
        .auth_post("/api/v1/vet-patrol-reports", &body, &token)
        .await;
    assert_eq!(create_res.status(), 200);
    let created: serde_json::Value = create_res.json().await.expect("parse create response");
    let report_id_str = created["id"].as_str().expect("id").to_string();
    let report_id = Uuid::parse_str(&report_id_str).expect("valid uuid");

    let created_entries = created["entries"].as_array().expect("entries array");
    assert_eq!(
        created_entries.len(),
        3,
        "create 回應應含全部 3 筆觀察條目（#928 回歸），實得 {}",
        created_entries.len()
    );
    let observations: Vec<&str> = created_entries
        .iter()
        .map(|e| e["observation"].as_str().expect("observation"))
        .collect();
    let suggestions: Vec<&str> = created_entries
        .iter()
        .map(|e| e["suggestion"].as_str().expect("suggestion"))
        .collect();
    for expected in ["觀察一：食慾不振", "觀察二：跛行", "觀察三：欄舍清潔"] {
        assert!(
            observations.contains(&expected),
            "create 回應缺少觀察內容：{expected}"
        );
    }
    for expected in ["建議一：追蹤體重", "建議二：安排X光", "建議三：加強消毒"]
    {
        assert!(
            suggestions.contains(&expected),
            "create 回應缺少建議內容：{expected}"
        );
    }

    // GET 單一報告（PDF 匯出資料來源同一路徑）亦須含全部 3 筆
    let get_res = app
        .auth_get(
            &format!("/api/v1/vet-patrol-reports/{report_id_str}"),
            &token,
        )
        .await;
    assert_eq!(get_res.status(), 200);
    let get_json: serde_json::Value = get_res.json().await.expect("parse get response");
    let get_entries = get_json["entries"].as_array().expect("entries array");
    assert_eq!(
        get_entries.len(),
        3,
        "GET 單一報告應含全部 3 筆觀察條目（#928 回歸：PDF 主表資料來源），實得 {}",
        get_entries.len()
    );
    let get_observations: Vec<&str> = get_entries
        .iter()
        .map(|e| e["observation"].as_str().expect("observation"))
        .collect();
    for expected in ["觀察一：食慾不振", "觀察二：跛行", "觀察三：欄舍清潔"] {
        assert!(
            get_observations.contains(&expected),
            "GET 回應缺少觀察內容：{expected}"
        );
    }

    // audit snapshot（after_data）亦須完整保留全部 3 筆（GLP 醫療紀錄完整保留）
    let after_data: serde_json::Value = sqlx::query_scalar(
        "SELECT after_data FROM user_activity_logs \
         WHERE event_type = 'VET_PATROL_REPORT_CREATED' AND entity_id = $1 \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(report_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("VET_PATROL_REPORT_CREATED audit row should exist");
    let audit_entries = after_data["entries"]
        .as_array()
        .expect("audit snapshot entries array");
    assert_eq!(
        audit_entries.len(),
        3,
        "audit snapshot 應含全部 3 筆觀察內容，實得 {}",
        audit_entries.len()
    );

    assert_eq!(
        audit_count(&app, "VET_PATROL_REPORT_CREATED", report_id).await,
        1
    );
}

// ── 撤回：已送出未完成 → draft（建立獸醫或 admin；非建立者 403；草稿不可撤回）──────────
#[tokio::test]
#[serial]
async fn retract_submitted_report_back_to_draft_and_permission_gate() {
    let app = TestApp::spawn().await;
    let (_vet_id, vet_token) = seed_user_with_role(&app, "VET", "retract-vet").await;
    let (tracker_id, _tracker_token) =
        seed_user_with_role(&app, "EXPERIMENT_STAFF", "retract-tracker").await;
    let (_outsider_id, outsider_token) = seed_user_with_role(&app, "VET", "retract-outsider").await;

    // 建立 + 送出（→ awaiting_acknowledgement）
    let create_body = serde_json::json!({
        "patrol_date": "2026-07-09",
        "entries": [{
            "category": "health", "animal_ids": [],
            "observation": "o", "suggestion": "s", "follow_up": "", "sort_order": 0
        }]
    });
    let create_res = app
        .auth_post("/api/v1/vet-patrol-reports", &create_body, &vet_token)
        .await;
    assert_eq!(create_res.status(), 200);
    let created: serde_json::Value = create_res.json().await.expect("parse");
    let report_id_str = created["id"].as_str().expect("id").to_string();

    let submit_res = app
        .auth_post(
            &format!("/api/v1/vet-patrol-reports/{report_id_str}/submit-for-followup"),
            &serde_json::json!({ "follow_up_user_id": tracker_id }),
            &vet_token,
        )
        .await;
    assert_eq!(submit_res.status(), 200);
    let submitted: serde_json::Value = submit_res.json().await.expect("parse");
    assert_eq!(submitted["status"], "awaiting_acknowledgement");

    // 送出必須替**指派的追蹤者**建立置頂待辦（priority=1）。
    // 這條與下方撤回後的斷言，是「解除 hook 有沒有接上」的唯一端對端保護 ——
    // 沒有它們的話，把 service 內的 resolve 全部 revert 掉，整套測試仍會全綠。
    assert_eq!(
        pinned_state_for(&app, &report_id_str).await,
        (1, 1, Some(tracker_id)),
        "送出後應有 1 則待辦、仍置頂、且收件人是指派的追蹤者"
    );

    // 非建立者非 admin 的其他獸醫撤回 → 403（service 層 created_by/admin gate）
    let wrong = app
        .auth_post(
            &format!("/api/v1/vet-patrol-reports/{report_id_str}/retract"),
            &serde_json::json!({}),
            &outsider_token,
        )
        .await;
    assert_eq!(wrong.status(), 403, "非建立者非 admin 不可撤回");

    // 建立獸醫撤回自己送出的報告 → 200
    let retract_res = app
        .auth_post(
            &format!("/api/v1/vet-patrol-reports/{report_id_str}/retract"),
            &serde_json::json!({}),
            &vet_token,
        )
        .await;
    assert_eq!(retract_res.status(), 200, "建立獸醫可撤回自己送出的報告");

    // 撤回後：GET 應回 draft、follow_up_user_id 清空
    let get_res = app
        .auth_get(
            &format!("/api/v1/vet-patrol-reports/{report_id_str}"),
            &vet_token,
        )
        .await;
    assert_eq!(get_res.status(), 200);
    let after: serde_json::Value = get_res.json().await.expect("parse");
    assert_eq!(after["status"], "draft", "撤回後應回草稿");
    assert!(
        after["follow_up_user_id"].is_null(),
        "撤回後 follow_up_user_id 應清空，實得 {:?}",
        after["follow_up_user_id"]
    );

    // 撤回＝追蹤者不再被指派 → 置頂待辦必須解除。漏接的話那則通知會綁在一份
    // 沒有人能再操作的報告上永久卡死，而待辦依設計不可手動已讀（2026-08-07 事故）。
    //
    // total 仍為 1：解除是**降級**不是刪除 —— 通知留在鈴鐺歷史裡，
    // 只是離開待處理清單。若哪天改成 DELETE，這個斷言會抓到。
    assert_eq!(
        pinned_state_for(&app, &report_id_str).await,
        (1, 0, Some(tracker_id)),
        "撤回後待辦應降級（priority=0）而非被刪除，且列仍在"
    );

    // 已是草稿再撤回 → client error（僅已送出未完成可撤回）
    let retract_draft = app
        .auth_post(
            &format!("/api/v1/vet-patrol-reports/{report_id_str}/retract"),
            &serde_json::json!({}),
            &vet_token,
        )
        .await;
    assert!(
        retract_draft.status().is_client_error(),
        "草稿不可撤回，實得 {}",
        retract_draft.status()
    );
}

// ── (c) R39++ 三階段流程 + audit 落地（≥2 個事件） ──────────────────────────────

#[tokio::test]
#[serial]
async fn full_lifecycle_submit_acknowledge_complete_locks_report_and_writes_audit() {
    let app = TestApp::spawn().await;
    let (_vet_id, vet_token) = seed_user_with_role(&app, "VET", "lifecycle-vet").await;
    // 追蹤者（陪同人員）在真實情境為非獸醫 staff，不具 animal.vet.recommend。
    // 用 EXPERIMENT_STAFF 才能重現「追蹤者被 handler 層 403 擋下」的回歸：
    // acknowledge / update(追蹤改善) / complete 三個追蹤者端點須改守 view gate。
    let (tracker_id, tracker_token) =
        seed_user_with_role(&app, "EXPERIMENT_STAFF", "lifecycle-tracker").await;
    let (_outsider_id, outsider_token) =
        seed_user_with_role(&app, "VET", "lifecycle-outsider").await;

    let create_body = serde_json::json!({
        "patrol_date": "2026-07-08",
        "entries": [{
            "category": "health",
            "animal_ids": [],
            "observation": "待追蹤觀察",
            "suggestion": "待追蹤建議",
            "follow_up": "",
            "sort_order": 0
        }]
    });
    let create_res = app
        .auth_post("/api/v1/vet-patrol-reports", &create_body, &vet_token)
        .await;
    assert_eq!(create_res.status(), 200);
    let created: serde_json::Value = create_res.json().await.expect("parse");
    let report_id_str = created["id"].as_str().expect("id").to_string();
    let report_id = Uuid::parse_str(&report_id_str).expect("valid uuid");
    let entry_id = created["entries"][0]["id"]
        .as_str()
        .expect("entry id")
        .to_string();

    // phase 1: 獸醫送出給追蹤者
    let submit_res = app
        .auth_post(
            &format!("/api/v1/vet-patrol-reports/{report_id_str}/submit-for-followup"),
            &serde_json::json!({"follow_up_user_id": tracker_id}),
            &vet_token,
        )
        .await;
    assert_eq!(submit_res.status(), 200);
    let submitted: serde_json::Value = submit_res.json().await.expect("parse");
    assert_eq!(submitted["status"], "awaiting_acknowledgement");

    // 送出 → 指派的追蹤者取得置頂待辦
    assert_eq!(
        pinned_state_for(&app, &report_id_str).await,
        (1, 1, Some(tracker_id)),
        "送出後應有 1 則待辦、仍置頂、且收件人是指派的追蹤者"
    );

    // 非指派追蹤者確認收到 → 403
    let wrong_ack = app
        .auth_post(
            &format!("/api/v1/vet-patrol-reports/{report_id_str}/acknowledge"),
            &serde_json::json!({}),
            &outsider_token,
        )
        .await;
    assert_eq!(wrong_ack.status(), 403);

    // phase 2: 指派追蹤者確認收到
    let ack_res = app
        .auth_post(
            &format!("/api/v1/vet-patrol-reports/{report_id_str}/acknowledge"),
            &serde_json::json!({}),
            &tracker_token,
        )
        .await;
    assert_eq!(ack_res.status(), 200);
    let acked: serde_json::Value = ack_res.json().await.expect("parse");
    assert_eq!(acked["status"], "awaiting_follow_up");

    // phase 2 **不得**解除待辦：確認收到只代表「我看到了」，追蹤改善還沒填。
    // 少了這條斷言，acknowledge 若誤解除，後面「完成後為 0」仍會通過 ——
    // 測試就抓不到 phase 2 的錯誤行為。
    assert_eq!(
        pinned_state_for(&app, &report_id_str).await,
        (1, 1, Some(tracker_id)),
        "確認收到後待辦必須維持置頂（事情還沒做完）"
    );

    // 追蹤者填寫「追蹤改善」欄位（其他欄位需與原值一致，才能通過待追蹤期限制）
    let update_body = serde_json::json!({
        "entries": [{
            "id": entry_id,
            "category": "health",
            "animal_ids": [],
            "observation": "待追蹤觀察",
            "suggestion": "待追蹤建議",
            "follow_up": "已完成消毒處置",
            "sort_order": 0
        }]
    });
    let update_res = app
        .auth_put(
            &format!("/api/v1/vet-patrol-reports/{report_id_str}"),
            &update_body,
            &tracker_token,
        )
        .await;
    assert_eq!(update_res.status(), 200);

    // phase 3: 追蹤者確認完成 → 鎖定
    let complete_res = app
        .auth_post(
            &format!("/api/v1/vet-patrol-reports/{report_id_str}/complete-followup"),
            &serde_json::json!({}),
            &tracker_token,
        )
        .await;
    assert_eq!(complete_res.status(), 200);
    let completed: serde_json::Value = complete_res.json().await.expect("parse");
    assert_eq!(completed["status"], "completed");

    // phase 3 完成才解除。total 仍為 1：解除是降級不是刪除。
    assert_eq!(
        pinned_state_for(&app, &report_id_str).await,
        (1, 0, Some(tracker_id)),
        "完成追蹤後待辦應降級（priority=0）而非被刪除"
    );

    // 已完成報告再更新 → 422（GLP 不可變醫療紀錄鎖定）
    let locked_update = app
        .auth_put(
            &format!("/api/v1/vet-patrol-reports/{report_id_str}"),
            &update_body,
            &tracker_token,
        )
        .await;
    assert_eq!(locked_update.status(), 422);

    // audit：三階段各恰好一筆（c 要求 ≥2 個 audit 寫入路徑；此處驗證三個）
    assert_eq!(
        audit_count(&app, "VET_PATROL_REPORT_SUBMITTED_FOR_FOLLOWUP", report_id).await,
        1
    );
    assert_eq!(
        audit_count(&app, "VET_PATROL_REPORT_ACKNOWLEDGED", report_id).await,
        1
    );
    assert_eq!(
        audit_count(&app, "VET_PATROL_REPORT_COMPLETED", report_id).await,
        1
    );
}

// ── 巡場「完成」→ 歸位病歷獸醫師建議（觸發點 = complete，非 create）──────────────
//
// 掛豬的觀察在報告「三階段完成」時，才 upsert 到該豬 animal_vet_advice_records
// （觀察 + 建議 + 追蹤改善 + source 連結）。建立/待追蹤階段不同步（避免草稿污染病歷）。
#[tokio::test]
#[serial]
async fn complete_syncs_advice_to_animal_medical_record() {
    let app = TestApp::spawn().await;
    let (vet_id, vet_token) = seed_user_with_role(&app, "VET", "advsync-vet").await;
    let (tracker_id, tracker_token) =
        seed_user_with_role(&app, "EXPERIMENT_STAFF", "advsync-tracker").await;
    let animal_id = seed_animal(&app, "VPADV", vet_id).await;

    // 建立含掛豬觀察的報告（觀察 + 建議；follow_up 由追蹤者後填）
    let create_res = app
        .auth_post(
            "/api/v1/vet-patrol-reports",
            &serde_json::json!({
                "patrol_date": "2026-07-08",
                "entries": [{
                    "category": "health", "animal_ids": [animal_id],
                    "observation": "皮膚紅疹", "suggestion": "維持環境乾燥",
                    "follow_up": "", "sort_order": 0
                }]
            }),
            &vet_token,
        )
        .await;
    assert_eq!(create_res.status(), 200);
    let created: serde_json::Value = create_res.json().await.expect("parse");
    let report_id = created["id"].as_str().expect("id").to_string();
    let entry_id = created["entries"][0]["id"]
        .as_str()
        .expect("entry id")
        .to_string();

    // 建立草稿階段「不」同步 → 病歷 0 筆
    assert_eq!(
        advice_count(&app, animal_id).await,
        0,
        "建立草稿不應同步病歷"
    );

    // submit → acknowledge → awaiting_follow_up
    let submit = app
        .auth_post(
            &format!("/api/v1/vet-patrol-reports/{report_id}/submit-for-followup"),
            &serde_json::json!({"follow_up_user_id": tracker_id}),
            &vet_token,
        )
        .await;
    assert_eq!(submit.status(), 200);
    let ack = app
        .auth_post(
            &format!("/api/v1/vet-patrol-reports/{report_id}/acknowledge"),
            &serde_json::json!({}),
            &tracker_token,
        )
        .await;
    assert_eq!(ack.status(), 200);

    // 追蹤者補填追蹤改善
    let upd = app
        .auth_put(
            &format!("/api/v1/vet-patrol-reports/{report_id}"),
            &serde_json::json!({
                "entries": [{
                    "id": entry_id, "category": "health", "animal_ids": [animal_id],
                    "observation": "皮膚紅疹", "suggestion": "維持環境乾燥",
                    "follow_up": "已投藥追蹤", "sort_order": 0
                }]
            }),
            &tracker_token,
        )
        .await;
    assert_eq!(upd.status(), 200);

    // 完成前仍不同步
    assert_eq!(
        advice_count(&app, animal_id).await,
        0,
        "完成前（待追蹤）不應同步"
    );

    // 完成 → 同步
    let complete = app
        .auth_post(
            &format!("/api/v1/vet-patrol-reports/{report_id}/complete-followup"),
            &serde_json::json!({}),
            &tracker_token,
        )
        .await;
    assert_eq!(complete.status(), 200);

    // 病歷應恰有 1 筆，內容 = 觀察 + 建議 + 追蹤改善，且連結來源巡場條目
    assert_eq!(
        advice_count(&app, animal_id).await,
        1,
        "完成後應同步 1 筆到病歷"
    );
    let entry_uuid = Uuid::parse_str(&entry_id).expect("entry uuid");
    let rec: (String, String, String, Option<Uuid>) = sqlx::query_as(
        "SELECT observation, suggested_treatment, follow_up, source_vet_patrol_entry_id \
         FROM animal_vet_advice_records WHERE animal_id = $1 AND deleted_at IS NULL",
    )
    .bind(animal_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("advice row");
    assert_eq!(rec.0, "皮膚紅疹", "觀察應帶入");
    assert_eq!(rec.1, "維持環境乾燥", "建議應帶入 suggested_treatment");
    assert_eq!(rec.2, "已投藥追蹤", "追蹤改善應帶入 follow_up");
    assert_eq!(rec.3, Some(entry_uuid), "應連結來源巡場條目");
}

// ── #972 回歸：軟刪已完成報告應連動軟刪同步到病歷的建議（避免孤兒） ──────────────

#[tokio::test]
#[serial]
async fn delete_completed_report_soft_deletes_synced_advice() {
    let app = TestApp::spawn().await;
    let (vet_id, vet_token) = seed_user_with_role(&app, "VET", "advdel-vet").await;
    let (tracker_id, tracker_token) =
        seed_user_with_role(&app, "EXPERIMENT_STAFF", "advdel-tracker").await;
    // 用 legacy `admin` role code（migration 002 建立；is_admin() 認 admin/SYSTEM_ADMIN，
    // 但測試 DB 僅有 admin）→ is_admin() true 才能刪已完成報告。
    let (_admin_id, admin_token) = seed_user_with_role(&app, "admin", "advdel-admin").await;
    let animal_id = seed_animal(&app, "VPDEL", vet_id).await;

    // 建立含掛豬觀察的報告 → 送出 → 確認 → 補填追蹤 → 完成（完成時同步病歷）
    let create_res = app
        .auth_post(
            "/api/v1/vet-patrol-reports",
            &serde_json::json!({
                "patrol_date": "2026-07-09",
                "entries": [{
                    "category": "health", "animal_ids": [animal_id],
                    "observation": "跛行", "suggestion": "限制活動",
                    "follow_up": "", "sort_order": 0
                }]
            }),
            &vet_token,
        )
        .await;
    assert_eq!(create_res.status(), 200);
    let created: serde_json::Value = create_res.json().await.expect("parse");
    let report_id = created["id"].as_str().expect("id").to_string();
    let entry_id = created["entries"][0]["id"]
        .as_str()
        .expect("entry id")
        .to_string();

    let submit = app
        .auth_post(
            &format!("/api/v1/vet-patrol-reports/{report_id}/submit-for-followup"),
            &serde_json::json!({"follow_up_user_id": tracker_id}),
            &vet_token,
        )
        .await;
    assert_eq!(submit.status(), 200);
    let ack = app
        .auth_post(
            &format!("/api/v1/vet-patrol-reports/{report_id}/acknowledge"),
            &serde_json::json!({}),
            &tracker_token,
        )
        .await;
    assert_eq!(ack.status(), 200);
    let upd = app
        .auth_put(
            &format!("/api/v1/vet-patrol-reports/{report_id}"),
            &serde_json::json!({
                "entries": [{
                    "id": entry_id, "category": "health", "animal_ids": [animal_id],
                    "observation": "跛行", "suggestion": "限制活動",
                    "follow_up": "已恢復", "sort_order": 0
                }]
            }),
            &tracker_token,
        )
        .await;
    assert_eq!(upd.status(), 200);
    let complete = app
        .auth_post(
            &format!("/api/v1/vet-patrol-reports/{report_id}/complete-followup"),
            &serde_json::json!({}),
            &tracker_token,
        )
        .await;
    assert_eq!(complete.status(), 200);

    // 前置：完成後病歷已同步 1 筆
    assert_eq!(advice_count(&app, animal_id).await, 1, "完成後應同步 1 筆");

    // 非 admin 不可刪已完成報告（GLP 鎖定）→ 建立者獸醫刪應被拒、病歷不受影響
    let forbidden = app
        .auth_post(
            &format!("/api/v1/vet-patrol-reports/{report_id}/delete"),
            &serde_json::json!({}),
            &vet_token,
        )
        .await;
    assert_eq!(forbidden.status(), 403, "已完成報告限管理員刪除");
    assert_eq!(
        advice_count(&app, animal_id).await,
        1,
        "被拒的刪除不應動到病歷"
    );

    // admin 刪除已完成報告 → 連動軟刪同步建議
    let delete_res = app
        .auth_post(
            &format!("/api/v1/vet-patrol-reports/{report_id}/delete"),
            &serde_json::json!({}),
            &admin_token,
        )
        .await;
    assert_eq!(delete_res.status(), 200);

    // 報告軟刪後，同步到病歷的建議應連動軟刪（避免孤兒）
    assert_eq!(
        advice_count(&app, animal_id).await,
        0,
        "報告軟刪應連動軟刪同步到病歷的建議"
    );

    // 為 soft delete：列仍在、deleted_at 設定、source 連結保留
    let entry_uuid = Uuid::parse_str(&entry_id).expect("entry uuid");
    let (deleted, src): (bool, Option<Uuid>) = sqlx::query_as(
        "SELECT deleted_at IS NOT NULL, source_vet_patrol_entry_id \
         FROM animal_vet_advice_records WHERE source_vet_patrol_entry_id = $1",
    )
    .bind(entry_uuid)
    .fetch_one(&app.db_pool)
    .await
    .expect("advice row 仍在（soft delete，非硬刪）");
    assert!(deleted, "建議應為 soft delete（deleted_at 設定）");
    assert_eq!(src, Some(entry_uuid), "soft delete 應保留 source 連結");
}

// ── #955 回歸：待追蹤階段追蹤者不可竄改報告基本資訊 ──────────────────────────────

#[tokio::test]
#[serial]
async fn tracker_cannot_modify_report_meta_during_follow_up() {
    // 放寬 update handler gate（改守 view）後，非獸醫追蹤者可呼叫 update。
    // service 層須確保待追蹤階段追蹤者僅能補填「追蹤改善」，不得改 patrol_date /
    // accompanying_personnel（PR #955 review 指出的資料完整性缺口）。
    let app = TestApp::spawn().await;
    let (_vet_id, vet_token) = seed_user_with_role(&app, "VET", "meta-vet").await;
    let (tracker_id, tracker_token) =
        seed_user_with_role(&app, "EXPERIMENT_STAFF", "meta-tracker").await;

    let create_res = app
        .auth_post(
            "/api/v1/vet-patrol-reports",
            &serde_json::json!({
                "patrol_date": "2026-07-08",
                "accompanying_personnel": "原始陪同員",
                "entries": [{
                    "category": "health",
                    "animal_ids": [],
                    "observation": "待追蹤觀察",
                    "suggestion": "待追蹤建議",
                    "follow_up": "",
                    "sort_order": 0
                }]
            }),
            &vet_token,
        )
        .await;
    assert_eq!(create_res.status(), 200);
    let created: serde_json::Value = create_res.json().await.expect("parse");
    let report_id = created["id"].as_str().expect("id").to_string();
    let entry_id = created["entries"][0]["id"]
        .as_str()
        .expect("entry id")
        .to_string();

    // 送出給追蹤者 → 追蹤者確認收到 → 進入待追蹤
    let submit_res = app
        .auth_post(
            &format!("/api/v1/vet-patrol-reports/{report_id}/submit-for-followup"),
            &serde_json::json!({"follow_up_user_id": tracker_id}),
            &vet_token,
        )
        .await;
    assert_eq!(submit_res.status(), 200);
    let ack_res = app
        .auth_post(
            &format!("/api/v1/vet-patrol-reports/{report_id}/acknowledge"),
            &serde_json::json!({}),
            &tracker_token,
        )
        .await;
    assert_eq!(ack_res.status(), 200);

    // 追蹤者改巡場日期 → 422（BusinessRule）
    let bad_date = app
        .auth_put(
            &format!("/api/v1/vet-patrol-reports/{report_id}"),
            &serde_json::json!({
                "patrol_date": "2026-07-09",
                "entries": [{
                    "id": entry_id, "category": "health", "animal_ids": [],
                    "observation": "待追蹤觀察", "suggestion": "待追蹤建議",
                    "follow_up": "追蹤中", "sort_order": 0
                }]
            }),
            &tracker_token,
        )
        .await;
    assert_eq!(bad_date.status(), 422, "追蹤者改 patrol_date 應被擋");

    // 追蹤者改陪同人員 → 422
    let bad_ap = app
        .auth_put(
            &format!("/api/v1/vet-patrol-reports/{report_id}"),
            &serde_json::json!({
                "accompanying_personnel": "偷改的陪同員",
                "entries": [{
                    "id": entry_id, "category": "health", "animal_ids": [],
                    "observation": "待追蹤觀察", "suggestion": "待追蹤建議",
                    "follow_up": "追蹤中", "sort_order": 0
                }]
            }),
            &tracker_token,
        )
        .await;
    assert_eq!(
        bad_ap.status(),
        422,
        "追蹤者改 accompanying_personnel 應被擋"
    );

    // 只補填追蹤改善（不動基本資訊）→ 200
    let ok = app
        .auth_put(
            &format!("/api/v1/vet-patrol-reports/{report_id}"),
            &serde_json::json!({
                "entries": [{
                    "id": entry_id, "category": "health", "animal_ids": [],
                    "observation": "待追蹤觀察", "suggestion": "待追蹤建議",
                    "follow_up": "已完成消毒處置", "sort_order": 0
                }]
            }),
            &tracker_token,
        )
        .await;
    assert_eq!(ok.status(), 200, "只填追蹤改善應放行");

    // 基本資訊未被竄改：patrol_date / accompanying_personnel 維持原值
    let get_json: serde_json::Value = app
        .auth_get(
            &format!("/api/v1/vet-patrol-reports/{report_id}"),
            &vet_token,
        )
        .await
        .json()
        .await
        .expect("parse get");
    assert_eq!(get_json["patrol_date"], "2026-07-08");
    assert_eq!(get_json["accompanying_personnel"], "原始陪同員");
}

// ── 回歸：待追蹤階段「多動物條目」的追蹤改善可正常儲存（不誤判 422） ──────────
//
// 前端 R39+++ 只送 animal_ids（多）。lock-check 曾比對 DB 既有單一 animal_id，但讀取路徑
// 回傳的 animal_ids 依 ear_tag 排序，與建立時依「選取順序」取的主要 animal_id 未必同序 →
// 多動物條目時，追蹤者 echo 動物集補填追蹤改善會被誤判 422（陪同人員存不了追蹤改善）。
// #959 首修只比主要動物（ear_tag 序首 vs 選取序首）對多動物仍不足；最終改為「待追蹤階段
// 不比對動物」（寫入路徑本就跳過 animal/junction，比對零防護價值）。本測試以「選取序 ≠
// ear_tag 序」的多動物條目重現並鎖定修復。
#[tokio::test]
#[serial]
async fn tracker_can_save_followup_on_multi_animal_entry() {
    let app = TestApp::spawn().await;
    let (vet_id, vet_token) = seed_user_with_role(&app, "VET", "fu-multi-vet").await;
    let (tracker_id, tracker_token) =
        seed_user_with_role(&app, "EXPERIMENT_STAFF", "fu-multi-tracker").await;
    // ear_tag 前綴 VPA<VPB<VPC → 讀取路徑（ORDER BY ear_tag）回傳 [a1,a2,a3]；
    // 但建立時選取序刻意為 [a2,a1,a3] → DB 主要 animal_id = a2（≠ ear_tag 序首 a1）。
    let a1 = seed_animal(&app, "VPA", vet_id).await;
    let a2 = seed_animal(&app, "VPB", vet_id).await;
    let a3 = seed_animal(&app, "VPC", vet_id).await;

    let create_res = app
        .auth_post(
            "/api/v1/vet-patrol-reports",
            &serde_json::json!({
                "patrol_date": "2026-07-08",
                "entries": [{
                    "category": "health",
                    "animal_ids": [a2, a1, a3],
                    "observation": "多隻觀察",
                    "suggestion": "多隻建議",
                    "follow_up": "",
                    "sort_order": 0
                }]
            }),
            &vet_token,
        )
        .await;
    assert_eq!(create_res.status(), 200);
    let created: serde_json::Value = create_res.json().await.expect("parse");
    let report_id = created["id"].as_str().expect("id").to_string();
    let entry_id = created["entries"][0]["id"]
        .as_str()
        .expect("entry id")
        .to_string();
    let entry_uuid = Uuid::parse_str(&entry_id).expect("valid entry uuid");

    // 前置：DB 主要 animal_id = 建立時選取序第一隻 = a2（非 ear_tag 序首 a1）——
    // 這正是舊「比主要動物」修法對多動物仍誤判的成因。
    let primary: Option<Uuid> =
        sqlx::query_scalar("SELECT animal_id FROM vet_patrol_entries WHERE id = $1")
            .bind(entry_uuid)
            .fetch_one(&app.db_pool)
            .await
            .expect("query primary animal");
    assert_eq!(
        primary,
        Some(a2),
        "前置：主要 animal_id 應為選取序第一隻 a2"
    );

    // 送出 → 追蹤者確認收到 → 待追蹤
    let submit = app
        .auth_post(
            &format!("/api/v1/vet-patrol-reports/{report_id}/submit-for-followup"),
            &serde_json::json!({"follow_up_user_id": tracker_id}),
            &vet_token,
        )
        .await;
    assert_eq!(submit.status(), 200);
    let ack = app
        .auth_post(
            &format!("/api/v1/vet-patrol-reports/{report_id}/acknowledge"),
            &serde_json::json!({}),
            &tracker_token,
        )
        .await;
    assert_eq!(ack.status(), 200);

    // 追蹤者 echo 動物集（依 ear_tag 序 [a1,a2,a3]，其首 a1 ≠ 主要 a2）+ 補填追蹤改善 → 200
    // （修前：比主要動物 a1 != a2 → 誤判 422）
    let ok = app
        .auth_put(
            &format!("/api/v1/vet-patrol-reports/{report_id}"),
            &serde_json::json!({
                "entries": [{
                    "id": entry_id,
                    "category": "health",
                    "animal_ids": [a1, a2, a3],
                    "observation": "多隻觀察",
                    "suggestion": "多隻建議",
                    "follow_up": "已投藥並隔離觀察",
                    "sort_order": 0
                }]
            }),
            &tracker_token,
        )
        .await;
    assert_eq!(
        ok.status(),
        200,
        "多動物條目補填追蹤改善應可儲存（不得誤判 422）"
    );

    // 追蹤改善已寫入
    let fu: String = sqlx::query_scalar("SELECT follow_up FROM vet_patrol_entries WHERE id = $1")
        .bind(entry_uuid)
        .fetch_one(&app.db_pool)
        .await
        .expect("query follow_up");
    assert_eq!(fu, "已投藥並隔離觀察", "追蹤改善應已寫入");

    // 竄改防護模型：待追蹤階段送不同動物集 → 回 200，但寫入路徑跳過 animal/junction（no-op），
    // DB 動物集合維持不變（保護來自寫入路徑、非 lock-check）。
    let other = seed_animal(&app, "VPD", vet_id).await;
    let ignored = app
        .auth_put(
            &format!("/api/v1/vet-patrol-reports/{report_id}"),
            &serde_json::json!({
                "entries": [{
                    "id": entry_id,
                    "category": "health",
                    "animal_ids": [other],
                    "observation": "多隻觀察",
                    "suggestion": "多隻建議",
                    "follow_up": "追蹤中",
                    "sort_order": 0
                }]
            }),
            &tracker_token,
        )
        .await;
    assert_eq!(
        ignored.status(),
        200,
        "待追蹤階段動物集變更應被忽略（no-op），非擋下"
    );
    let mut animal_set: Vec<Uuid> =
        sqlx::query_scalar("SELECT animal_id FROM vet_patrol_entry_animals WHERE entry_id = $1")
            .bind(entry_uuid)
            .fetch_all(&app.db_pool)
            .await
            .expect("query junction set");
    animal_set.sort();
    let mut expected = vec![a1, a2, a3];
    expected.sort();
    assert_eq!(
        animal_set, expected,
        "動物關聯應維持不變（動物變更為 no-op）"
    );
    assert!(!animal_set.contains(&other), "竄改的動物不應寫入");

    // 保留的防護：追蹤者若改 observation（非追蹤改善）仍應被擋 422。
    let bad_obs = app
        .auth_put(
            &format!("/api/v1/vet-patrol-reports/{report_id}"),
            &serde_json::json!({
                "entries": [{
                    "id": entry_id,
                    "category": "health",
                    "animal_ids": [a1, a2, a3],
                    "observation": "偷改的觀察",
                    "suggestion": "多隻建議",
                    "follow_up": "追蹤中",
                    "sort_order": 0
                }]
            }),
            &tracker_token,
        )
        .await;
    assert_eq!(bad_obs.status(), 422, "追蹤者改 observation 仍應被擋");
}

// ── discard / delete + audit ──────────────────────────────

#[tokio::test]
#[serial]
async fn discard_draft_hard_deletes_and_writes_audit() {
    let app = TestApp::spawn().await;
    let (_id, token) = seed_user_with_role(&app, "VET", "discard").await;

    let create_res = app
        .auth_post(
            "/api/v1/vet-patrol-reports",
            &serde_json::json!({"patrol_date": "2026-07-09", "entries": []}),
            &token,
        )
        .await;
    assert_eq!(create_res.status(), 200);
    let created: serde_json::Value = create_res.json().await.expect("parse");
    let report_id_str = created["id"].as_str().expect("id").to_string();
    let report_id = Uuid::parse_str(&report_id_str).expect("valid uuid");

    let discard_res = app
        .auth_post(
            &format!("/api/v1/vet-patrol-reports/{report_id_str}/discard"),
            &serde_json::json!({}),
            &token,
        )
        .await;
    assert_eq!(discard_res.status(), 204);

    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM vet_patrol_reports WHERE id = $1)")
            .bind(report_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("exists check");
    assert!(!exists, "discard 應為 hard delete，row 不應殘留");

    assert_eq!(
        audit_count(&app, "VET_PATROL_REPORT_DISCARDED", report_id).await,
        1
    );
}

#[tokio::test]
#[serial]
async fn delete_report_soft_deletes_hides_from_get_and_writes_audit() {
    let app = TestApp::spawn().await;
    let (_id, token) = seed_user_with_role(&app, "VET", "softdelete").await;

    let create_res = app
        .auth_post(
            "/api/v1/vet-patrol-reports",
            &serde_json::json!({"patrol_date": "2026-07-10", "entries": []}),
            &token,
        )
        .await;
    assert_eq!(create_res.status(), 200);
    let created: serde_json::Value = create_res.json().await.expect("parse");
    let report_id_str = created["id"].as_str().expect("id").to_string();
    let report_id = Uuid::parse_str(&report_id_str).expect("valid uuid");

    let delete_res = app
        .auth_post(
            &format!("/api/v1/vet-patrol-reports/{report_id_str}/delete"),
            &serde_json::json!({}),
            &token,
        )
        .await;
    assert_eq!(delete_res.status(), 200);

    let deleted_at_set: bool =
        sqlx::query_scalar("SELECT deleted_at IS NOT NULL FROM vet_patrol_reports WHERE id = $1")
            .bind(report_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("check deleted_at");
    assert!(deleted_at_set, "delete 應為 soft delete（deleted_at 設定）");

    let get_res = app
        .auth_get(
            &format!("/api/v1/vet-patrol-reports/{report_id_str}"),
            &token,
        )
        .await;
    assert_eq!(get_res.status(), 200);
    let get_json: serde_json::Value = get_res.json().await.expect("parse get response");
    assert!(get_json.is_null(), "軟刪報告的 GET 應回 null");

    assert_eq!(
        audit_count(&app, "VET_PATROL_REPORT_DELETED", report_id).await,
        1
    );
}

/// P1-5 迴歸：巡場報告要能用**標準 `DELETE` 方法**刪除。
///
/// 2026-09-08 之前 `/vet-patrol-reports/{id}` 只掛了 `get` 與 `put`，刪除只能走
/// `POST …/delete`——它是全系統 40 條 `POST …/delete` 裡唯一「base 沒有 `DELETE`」的例外
///（其餘 39 條都是「`DELETE` 為主、`POST` 為備用」，備用是為了避開部分代理／tunnel
/// 對 `DELETE` 回 405，見 `handlers/warehouse.rs:120`）。這個例外很危險：任何人照
/// 「移除備用路由」的字面意思整批清掉 `POST …/delete`，就會刪掉巡場報告的唯一刪除途徑。
///
/// **鑑別力**：對舊碼這支會紅——axum 對只掛 get/put 的路徑收到 `DELETE` 會回 405，
/// 第一個 assert 就會炸。上面那支 `delete_report_soft_deletes_…` 走的是 POST，
/// 改動前後都綠，證明不了這件事。
#[tokio::test]
#[serial]
async fn delete_report_via_standard_delete_method() {
    let app = TestApp::spawn().await;
    let (_id, token) = seed_user_with_role(&app, "VET", "httpdelete").await;

    let create_res = app
        .auth_post(
            "/api/v1/vet-patrol-reports",
            &serde_json::json!({"patrol_date": "2026-07-11", "entries": []}),
            &token,
        )
        .await;
    assert_eq!(create_res.status(), 200);
    let created: serde_json::Value = create_res.json().await.expect("parse");
    let report_id_str = created["id"].as_str().expect("id").to_string();
    let report_id = Uuid::parse_str(&report_id_str).expect("valid uuid");

    let delete_res = app
        .auth_delete(
            &format!("/api/v1/vet-patrol-reports/{report_id_str}"),
            &token,
        )
        .await;
    assert_eq!(
        delete_res.status(),
        200,
        "DELETE /vet-patrol-reports/{{id}} 應可用；405 代表該路徑又只剩 POST 備用路由"
    );

    // 與 POST 備用路由走同一個 handler，所以行為必須完全一致：軟刪 + 稽核。
    let deleted_at_set: bool =
        sqlx::query_scalar("SELECT deleted_at IS NOT NULL FROM vet_patrol_reports WHERE id = $1")
            .bind(report_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("check deleted_at");
    assert!(deleted_at_set, "DELETE 方法也應為 soft delete");

    assert_eq!(
        audit_count(&app, "VET_PATROL_REPORT_DELETED", report_id).await,
        1,
        "DELETE 方法也應寫稽核，與 POST 備用路由一致"
    );
}

// ── list filter：預設 completed，不含草稿（回歸 filter 邏輯，非僅 permission） ──────────

#[tokio::test]
#[serial]
async fn list_default_filter_excludes_draft_reports() {
    let app = TestApp::spawn().await;
    let (_id, token) = seed_user_with_role(&app, "VET", "list-filter").await;

    let create_res = app
        .auth_post(
            "/api/v1/vet-patrol-reports",
            &serde_json::json!({"patrol_date": "2026-07-11", "entries": []}),
            &token,
        )
        .await;
    assert_eq!(create_res.status(), 200);
    let created: serde_json::Value = create_res.json().await.expect("parse");
    let report_id = created["id"].as_str().expect("id").to_string();

    // 預設 filter = completed；剛建立的草稿不應出現
    let list_res = app.auth_get("/api/v1/vet-patrol-reports", &token).await;
    assert_eq!(list_res.status(), 200);
    let list: serde_json::Value = list_res.json().await.expect("parse list response");
    let ids: Vec<&str> = list
        .as_array()
        .expect("array")
        .iter()
        .map(|r| r["id"].as_str().expect("id"))
        .collect();
    assert!(
        !ids.contains(&report_id.as_str()),
        "預設 completed filter 不應含剛建立的草稿"
    );

    // my_drafts filter 應看得到
    let mydrafts_res = app
        .auth_get("/api/v1/vet-patrol-reports?status=my_drafts", &token)
        .await;
    assert_eq!(mydrafts_res.status(), 200);
    let mydrafts: serde_json::Value = mydrafts_res.json().await.expect("parse my_drafts response");
    let mydrafts_ids: Vec<&str> = mydrafts
        .as_array()
        .expect("array")
        .iter()
        .map(|r| r["id"].as_str().expect("id"))
        .collect();
    assert!(
        mydrafts_ids.contains(&report_id.as_str()),
        "my_drafts filter 應看到自己剛建立的草稿"
    );
}
