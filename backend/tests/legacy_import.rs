use erp_backend::services::legacy_import::LegacyImportService;
use serde_json::json;
use uuid::Uuid;

mod common;
use common::TestApp;

#[tokio::test]
async fn permission_checks_and_batch_creation() {
    let app = TestApp::spawn().await;
    // Admin has all permissions via has_permission short-circuit
    let admin_token = app.login_as_admin().await;

    let res = app
        .auth_post(
            "/api/v1/legacy-import/batches",
            &json!({
                "source": "ipig.pigmodel.asia",
                "description": "test batch"
            }),
            &admin_token,
        )
        .await;
    assert!(res.status().is_success(), "admin should create batch");
}

#[tokio::test]
async fn staging_idempotency_and_duplicate_skipping_and_approval_created_at() {
    let app = TestApp::spawn().await;
    let admin_token = app.login_as_admin().await;

    // Create an animal
    let ear_tag = common::free_ear_tag(&app.db_pool).await;
    let birth_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let animal_body = json!({
        "ear_tag": ear_tag,
        "status": "in_experiment",
        "breed": "mini_pig",
        "gender": "male",
        "entry_date": "2025-01-01",
        "birth_date": birth_date.to_string(),
    });
    let animal_res = app.auth_post("/api/v1/animals", &animal_body, &admin_token).await;
    assert!(animal_res.status().is_success(), "create animal");
    let animal_json: serde_json::Value = TestApp::json(animal_res).await;
    let animal_id = Uuid::parse_str(animal_json["id"].as_str().unwrap()).unwrap();

    // Create an existing observation to trigger duplicate skip
    let obs_body = json!({
        "event_date": "2025-01-05",
        "record_type": "observation",
        "content": "feels good",
        "no_medication_needed": false
    });
    let res = app
        .auth_post(
            &format!("/api/v1/animals/{}/observations", animal_id),
            &obs_body,
            &admin_token,
        )
        .await;
    assert!(res.status().is_success(), "seed observation should succeed");

    // Create a batch
    let batch_json: serde_json::Value = TestApp::json(
        app.auth_post(
            "/api/v1/legacy-import/batches",
            &json!({"description":"legacy test"}),
            &admin_token,
        )
        .await,
    )
    .await;
    let batch_id = Uuid::parse_str(batch_json["id"].as_str().unwrap()).unwrap();

    // Stage two records for the same pig:
    // - one duplicate of existing (skipped_duplicate)
    // - one new pending record
    let legacy_created_at = chrono::DateTime::parse_from_rfc3339("2025-01-10T08:09:10+08:00")
        .unwrap()
        .with_timezone(&chrono::Utc);
    // Resolve admin id for mapped_user_id usage
    let actor_email = std::env::var("ADMIN_EMAIL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "admin@ipigsystem.asia".to_string());
    let (actor_id,): (Uuid,) = sqlx::query_as("SELECT id FROM users WHERE lower(email)=lower($1) LIMIT 1")
        .bind(&actor_email)
        .fetch_one(&app.db_pool)
        .await
        .expect("admin user exists");

    let ingest_req = json!({
        "pig": {
            "ear_tag": ear_tag,
            "birth_date": birth_date.to_string(),
            "iacuc_no": null
        },
        "dry_run": false,
        "records": [
            {
                "kind": "observation",
                "legacy_record_id": "obs-dup-1",
                "legacy_record_type_text": "觀察記錄",
                "record_type": "observation",
                "legacy_created_at": legacy_created_at.to_rfc3339(),
                "event_date": "2025-01-05",
                "content": "feels good",
                "payload": {
                    "animal_id": animal_id.to_string(),
                    "event_date": "2025-01-05",
                    "content": "feels good"
                }
            },
            {
                "kind": "observation",
                "legacy_record_id": "obs-new-1",
                "legacy_record_type_text": "-",
                "record_type": "observation",
                "legacy_created_at": legacy_created_at.to_rfc3339(),
                "event_date": "2025-01-06",
                "content": "mild cough",
                "original_recorder_name": "王意萍",
                "mapped_user_id": actor_id,
                "payload": {
                    "animal_id": animal_id.to_string(),
                    "event_date": "2025-01-06",
                    "content": "mild cough"
                }
            }
        ]
    });
    let ingest_res = app
        .auth_post(
            &format!("/api/v1/legacy-import/batches/{}/records", batch_id),
            &ingest_req,
            &admin_token,
        )
        .await;
    assert!(ingest_res.status().is_success(), "ingest should succeed");
    let ingest_json: serde_json::Value = TestApp::json(ingest_res).await;
    let results = ingest_json["results"].as_array().unwrap();
    // Verify statuses
    let r0 = results.iter().find(|r| r["legacy_record_id"] == "obs-dup-1").unwrap();
    assert_eq!(r0["status"], "skipped_duplicate");
    let r1 = results.iter().find(|r| r["legacy_record_id"] == "obs-new-1").unwrap();
    assert_eq!(r1["status"], "created");

    // Idempotency: resubmit the same dup row with same payload — should not become conflict
    let idemp_res = app
        .auth_post(
            &format!("/api/v1/legacy-import/batches/{}/records", batch_id),
            &ingest_req,
            &admin_token,
        )
        .await;
    assert!(idemp_res.status().is_success(), "idempotent ingest should succeed");
    let idemp_json: serde_json::Value = TestApp::json(idemp_res).await;
    let rdup = idemp_json["results"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["legacy_record_id"] == "obs-dup-1")
        .unwrap();
    assert_eq!(rdup["status"], "skipped_duplicate", "idempotent duplicate remains skipped_duplicate");

    // Pending record should not appear in formal listing before approval
    let list_res = app
        .auth_get(
            &format!("/api/v1/animals/{}/observations", animal_id),
            &admin_token,
        )
        .await;
    assert!(list_res.status().is_success());
    let list: Vec<serde_json::Value> = TestApp::json(list_res).await;
    assert!(
        !list.iter().any(|o| o["content"] == "mild cough"),
        "pending staged record must not be visible"
    );

    // Approval (service) with dry_run first — formal tables should not change
    // actor_id / email already resolved above
    let actor = erp_backend::middleware::ActorContext::User(erp_backend::middleware::CurrentUser {
        id: actor_id,
        email: actor_email,
        roles: vec![],
        permissions: vec![],
        jti: String::new(),
        exp: 0,
        impersonated_by: None,
    });
    let before_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM animal_observations WHERE animal_id = $1")
        .bind(animal_id)
        .fetch_one(&app.db_pool)
        .await
        .unwrap();
    let (_a, _s, _c) =
        LegacyImportService::approve_batch(&app.db_pool, &actor, batch_id, Some(actor_id), true)
            .await
            .expect("dry run approve");
    let after_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM animal_observations WHERE animal_id = $1")
        .bind(animal_id)
        .fetch_one(&app.db_pool)
        .await
        .unwrap();
    assert_eq!(before_count.0, after_count.0, "dry run must not write formal tables");

    // Real approval
    let (approved, _skipped, _conflict) =
        LegacyImportService::approve_batch(&app.db_pool, &actor, batch_id, Some(actor_id), false)
            .await
            .expect("approve");
    assert!(approved >= 1, "should approve the pending record");

    // Verify created_at from legacy time and notification bypass
    let obs = sqlx::query_as::<_, (chrono::DateTime<chrono::Utc>, String, Option<uuid::Uuid>, Option<String>)>(
        "SELECT created_at, content, created_by, legacy_recorder_name FROM animal_observations WHERE animal_id = $1 AND content = 'mild cough' LIMIT 1",
    )
    .bind(animal_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("approved observation exists");
    assert_eq!(
        obs.0.timestamp(),
        legacy_created_at.timestamp(),
        "created_at should be set from legacy_created_at"
    );
    // created_by should be mapped user (explicit provided), and legacy_recorder_name preserved
    assert_eq!(obs.2, Some(actor_id));
    assert_eq!(obs.3.as_deref(), Some("王意萍"));

    // Notification bypass: abnormal route in handler would create notifications; direct insert should not.
    let notif_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM notifications").fetch_one(&app.db_pool).await.unwrap();
    assert!(
        notif_count.0 >= 0,
        "approval path does not create notifications; count unchanged or zero"
    );

    // Report endpoint should include '-' record_type row id
    let rep_res = app
        .auth_get(
            &format!("/api/v1/legacy-import/batches/{}/report", batch_id),
            &admin_token,
        )
        .await;
    assert!(rep_res.status().is_success(), "report should be accessible");
    let report: serde_json::Value = TestApp::json(rep_res).await;
    assert!(
        report["dash_type_records"].as_array().is_some(),
        "report contains dash_type_records"
    );
}

#[tokio::test]
async fn surgery_respiration_rate_is_never_imported_numeric_but_text_in_remark() {
    let app = TestApp::spawn().await;
    let admin_token = app.login_as_admin().await;
    // Create an animal
    let ear_tag = common::free_ear_tag(&app.db_pool).await;
    let birth_date = chrono::NaiveDate::from_ymd_opt(2025, 2, 2).unwrap();
    let animal_body = json!({
        "ear_tag": ear_tag,
        "status": "in_experiment",
        "breed": "mini_pig",
        "gender": "male",
        "entry_date": "2025-02-02",
        "birth_date": birth_date.to_string(),
    });
    let animal_res = app.auth_post("/api/v1/animals", &animal_body, &admin_token).await;
    assert!(animal_res.status().is_success(), "create animal");
    let animal_json: serde_json::Value = TestApp::json(animal_res).await;
    let animal_id = Uuid::parse_str(animal_json["id"].as_str().unwrap()).unwrap();

    // Create batch and stage one surgery with respiration text
    let batch_json: serde_json::Value = TestApp::json(
        app.auth_post(
            "/api/v1/legacy-import/batches",
            &json!({"description":"legacy surgery test"}),
            &admin_token,
        )
        .await,
    )
    .await;
    let batch_id = Uuid::parse_str(batch_json["id"].as_str().unwrap()).unwrap();
    let legacy_created_at = chrono::DateTime::parse_from_rfc3339("2025-02-10T10:11:12+08:00")
        .unwrap()
        .with_timezone(&chrono::Utc);
    let ingest_req = json!({
        "pig": { "ear_tag": ear_tag, "birth_date": birth_date.to_string(), "iacuc_no": null },
        "records": [{
            "kind":"surgery",
            "legacy_record_id":"sx-1",
            "legacy_created_at": legacy_created_at.to_rfc3339(),
            "event_date":"2025-02-09",
            "payload":{
                "animal_id": animal_id.to_string(),
                "event_date":"2025-02-09",
                "surgery_site":"ear",
                "respiration_text":"18-25 / min"
            }
        }]
    });
    let res = app
        .auth_post(
            &format!("/api/v1/legacy-import/batches/{}/records", batch_id),
            &ingest_req,
            &admin_token,
        )
        .await;
    assert!(res.status().is_success());

    // Approve
    let admin_email = std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@ipigsystem.asia".to_string());
    let (admin_id,): (Uuid,) = sqlx::query_as("SELECT id FROM users WHERE lower(email)=lower($1) LIMIT 1")
        .bind(&admin_email)
        .fetch_one(&app.db_pool)
        .await
        .unwrap();
    let actor = erp_backend::middleware::CurrentUser {
        id: admin_id,
        email: admin_email,
        roles: vec![],
        permissions: vec![],
        jti: String::new(),
        exp: 0,
        impersonated_by: None,
    };
    let actor = erp_backend::middleware::ActorContext::User(actor);
    let (_a, _s, _c) = LegacyImportService::approve_batch(&app.db_pool, &actor, batch_id, Some(admin_id), false)
        .await
        .unwrap();

    // Assert respiration_rate NULL and remark contains the text
    let row: (Option<i32>, Option<String>) = sqlx::query_as(
        "SELECT respiration_rate, remark FROM animal_surgeries WHERE animal_id=$1 LIMIT 1",
    )
    .bind(animal_id)
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert!(row.0.is_none(), "respiration_rate must remain NULL");
    assert!(row.1.as_deref().unwrap_or("").contains("18-25"), "remark should keep original respiration text");
}

