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
    let birth_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 1).expect("valid birth date");
    let created = common::create_animal_with_free_ear_tag(&app, &admin_token, |ear_tag| {
        serde_json::json!({
            "ear_tag": ear_tag,
            "breed": "minipig",
            "gender": "male",
            "entry_date": "2025-01-01",
            "entry_weight": 20.0,
            "pen_location": "A-01",
            "birth_date": birth_date.to_string(),
            "force_create": true
        })
    })
    .await;
    let animal_id = Uuid::parse_str(created["id"].as_str().expect("created animal id"))
        .expect("parse animal uuid");
    let ear_tag_for_import = created["ear_tag"]
        .as_str()
        .expect("created ear_tag")
        .to_string();

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
    let batch_id = Uuid::parse_str(batch_json["id"].as_str().expect("batch id present"))
        .expect("parse batch uuid");

    // Stage two records for the same pig:
    // - one duplicate of existing (skipped_duplicate)
    // - one new pending record
    let legacy_created_at = chrono::DateTime::parse_from_rfc3339("2025-01-10T08:09:10+08:00")
        .expect("parse legacy datetime")
        .with_timezone(&chrono::Utc);
    // Resolve admin id for mapped_user_id usage
    let actor_email = std::env::var("ADMIN_EMAIL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "admin@ipigsystem.asia".to_string());
    let (actor_id,): (Uuid,) =
        sqlx::query_as("SELECT id FROM users WHERE lower(email)=lower($1) LIMIT 1")
            .bind(&actor_email)
            .fetch_one(&app.db_pool)
            .await
            .expect("admin user exists");

    let ingest_req = json!({
        "pig": {
            "ear_tag": ear_tag_for_import,
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
    let ingest_status = ingest_res.status();
    let ingest_text = ingest_res.text().await.unwrap_or_default();
    assert!(
        ingest_status.is_success(),
        "ingest should succeed: status={} body={}",
        ingest_status,
        ingest_text
    );
    let ingest_json: serde_json::Value = serde_json::from_str(&ingest_text).expect("ingest json");
    let results = ingest_json["results"].as_array().expect("results array");
    // Verify statuses
    let r0 = results
        .iter()
        .find(|r| r["legacy_record_id"] == "obs-dup-1")
        .expect("dup row");
    assert_eq!(r0["status"], "skipped_duplicate");
    let r1 = results
        .iter()
        .find(|r| r["legacy_record_id"] == "obs-new-1")
        .expect("new row");
    assert_eq!(r1["status"], "created");

    // Idempotency: resubmit the same dup row with same payload — should not become conflict
    let idemp_res = app
        .auth_post(
            &format!("/api/v1/legacy-import/batches/{}/records", batch_id),
            &ingest_req,
            &admin_token,
        )
        .await;
    assert!(
        idemp_res.status().is_success(),
        "idempotent ingest should succeed"
    );
    let idemp_json: serde_json::Value = TestApp::json(idemp_res).await;
    let rdup = idemp_json["results"]
        .as_array()
        .expect("results array")
        .iter()
        .find(|r| r["legacy_record_id"] == "obs-dup-1")
        .expect("dup row present");
    assert_eq!(
        rdup["status"], "skipped_duplicate",
        "idempotent duplicate remains skipped_duplicate"
    );

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
    let before_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM animal_observations WHERE animal_id = $1")
            .bind(animal_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("count before");
    let (_a, _s, _e) = LegacyImportService::approve_batch(
        &app.db_pool,
        &actor,
        batch_id,
        Some(actor_id),
        true,
        None,
    )
    .await
    .expect("dry run approve");
    let after_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM animal_observations WHERE animal_id = $1")
            .bind(animal_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("count after");
    assert_eq!(
        before_count.0, after_count.0,
        "dry run must not write formal tables"
    );

    // Real approval
    let notif_before: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM notifications")
        .fetch_one(&app.db_pool)
        .await
        .expect("notif before");
    let (approved, _skipped, _error) = LegacyImportService::approve_batch(
        &app.db_pool,
        &actor,
        batch_id,
        Some(actor_id),
        false,
        None,
    )
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

    // Notification bypass: ensure exactly zero new notifications were created by approval
    let notif_after: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM notifications")
        .fetch_one(&app.db_pool)
        .await
        .expect("notif after");
    assert_eq!(
        notif_before.0, notif_after.0,
        "approval path must not create notifications"
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
    let birth_date = chrono::NaiveDate::from_ymd_opt(2025, 2, 2).expect("valid date");
    let created = common::create_animal_with_free_ear_tag(&app, &admin_token, |ear_tag| {
        serde_json::json!({
            "ear_tag": ear_tag,
            "breed": "minipig",
            "gender": "male",
            "entry_date": "2025-02-02",
            "entry_weight": 18.5,
            "pen_location": "A-01",
            "birth_date": birth_date.to_string(),
            "force_create": true
        })
    })
    .await;
    let animal_id = Uuid::parse_str(created["id"].as_str().expect("animal id")).expect("uuid");
    let ear_tag = created["ear_tag"].as_str().expect("ear_tag").to_string();

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
    let batch_id = Uuid::parse_str(batch_json["id"].as_str().expect("batch id")).expect("uuid");
    let legacy_created_at = chrono::DateTime::parse_from_rfc3339("2025-02-10T10:11:12+08:00")
        .expect("parse legacy dt")
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
    let admin_email =
        std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@ipigsystem.asia".to_string());
    let (admin_id,): (Uuid,) =
        sqlx::query_as("SELECT id FROM users WHERE lower(email)=lower($1) LIMIT 1")
            .bind(&admin_email)
            .fetch_one(&app.db_pool)
            .await
            .expect("actor exists");
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
    let (_a, _s, _e) = LegacyImportService::approve_batch(
        &app.db_pool,
        &actor,
        batch_id,
        Some(admin_id),
        false,
        None,
    )
    .await
    .expect("approve ok");

    // Assert respiration_rate NULL and remark contains the text
    let row: (Option<i32>, Option<String>) = sqlx::query_as(
        "SELECT respiration_rate, remark FROM animal_surgeries WHERE animal_id=$1 LIMIT 1",
    )
    .bind(animal_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("select surgery row");
    assert!(row.0.is_none(), "respiration_rate must remain NULL");
    assert!(
        row.1.as_deref().unwrap_or("").contains("18-25"),
        "remark should keep original respiration text"
    );
}

#[tokio::test]
async fn surgery_approval_maps_fields_and_blank_site_defaults() {
    let app = TestApp::spawn().await;
    let admin_token = app.login_as_admin().await;
    // Create an animal
    let birth_date = chrono::NaiveDate::from_ymd_opt(2025, 3, 3).expect("valid date");
    let created = common::create_animal_with_free_ear_tag(&app, &admin_token, |ear_tag| {
        serde_json::json!({
            "ear_tag": ear_tag,
            "breed": "minipig",
            "gender": "female",
            "entry_date": "2025-03-03",
            "entry_weight": 19.0,
            "pen_location": "A-01",
            "birth_date": birth_date.to_string(),
            "force_create": true
        })
    })
    .await;
    let animal_id = Uuid::parse_str(created["id"].as_str().expect("animal id")).expect("uuid");
    let ear_tag = created["ear_tag"].as_str().expect("ear_tag").to_string();

    // Batch
    let batch_json: serde_json::Value = TestApp::json(
        app.auth_post(
            "/api/v1/legacy-import/batches",
            &json!({"description":"sx fields"}),
            &admin_token,
        )
        .await,
    )
    .await;
    let batch_id = Uuid::parse_str(batch_json["id"].as_str().expect("batch id")).expect("uuid");
    let legacy_created_at = chrono::DateTime::parse_from_rfc3339("2025-03-10T12:34:56+08:00")
        .expect("parse legacy dt")
        .with_timezone(&chrono::Utc);
    let ingest_req = json!({
        "pig": { "ear_tag": ear_tag, "birth_date": birth_date.to_string(), "iacuc_no": null },
        "records": [{
            "kind":"surgery",
            "legacy_record_id":"sx-2",
            "legacy_created_at": legacy_created_at.to_rfc3339(),
            "event_date":"2025-03-09",
            "payload":{
                "animal_id": animal_id.to_string(),
                "event_date":"2025-03-09",
                "surgery_site":"  ",  // blank/whitespace
                "anesthesia_observation":"維持平穩",
                "vital_signs":{"hr":"100"},
                "reflex_recovery":"30 分",
                "positioning_list":["仰臥","右側臥"],
                "positioning_other":"頭部墊高",
                "post_surgery_medication":{"drug":"Meloxicam"}
            }
        }]
    });
    let _ = app
        .auth_post(
            &format!("/api/v1/legacy-import/batches/{}/records", batch_id),
            &ingest_req,
            &admin_token,
        )
        .await;
    // Approve
    let admin_email =
        std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@ipigsystem.asia".to_string());
    let (admin_id,): (Uuid,) =
        sqlx::query_as("SELECT id FROM users WHERE lower(email)=lower($1) LIMIT 1")
            .bind(&admin_email)
            .fetch_one(&app.db_pool)
            .await
            .expect("approve ok");
    let actor = erp_backend::middleware::ActorContext::User(erp_backend::middleware::CurrentUser {
        id: admin_id,
        email: admin_email,
        roles: vec![],
        permissions: vec![],
        jti: String::new(),
        exp: 0,
        impersonated_by: None,
    });
    let (_a, _s, _e) = LegacyImportService::approve_batch(
        &app.db_pool,
        &actor,
        batch_id,
        Some(admin_id),
        false,
        None,
    )
    .await
    .expect("approve ok");
    // Verify field mappings
    let row: (String, Option<String>, Option<serde_json::Value>, Option<String>) = sqlx::query_as(
        "SELECT surgery_site, anesthesia_observation, vital_signs, reflex_recovery FROM animal_surgeries WHERE animal_id=$1 LIMIT 1",
    )
    .bind(animal_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("select surgery row");
    assert_eq!(row.0, "未記載", "blank site defaults to 未記載");
    assert_eq!(row.1.as_deref(), Some("維持平穩"));
    assert!(row.2.as_ref().expect("vitals").get("hr").is_some());
    assert_eq!(row.3.as_deref(), Some("30 分"));
}

#[tokio::test]
async fn weight_and_vaccine_and_sacrifice_approval_and_duplicate_sacrifice() {
    let app = TestApp::spawn().await;
    let admin_token = app.login_as_admin().await;
    // Create animal
    let birth_date = chrono::NaiveDate::from_ymd_opt(2025, 4, 4).expect("valid date");
    let created = common::create_animal_with_free_ear_tag(&app, &admin_token, |ear_tag| {
        serde_json::json!({
            "ear_tag": ear_tag,
            "breed": "minipig",
            "gender": "male",
            "entry_date": "2025-04-04",
            "entry_weight": 21.0,
            "pen_location": "A-01",
            "birth_date": birth_date.to_string(),
            "force_create": true
        })
    })
    .await;
    let animal_id = Uuid::parse_str(created["id"].as_str().expect("animal id")).expect("uuid");
    let ear_tag = created["ear_tag"].as_str().expect("ear_tag").to_string();

    // Batch
    let batch_json: serde_json::Value = TestApp::json(
        app.auth_post(
            "/api/v1/legacy-import/batches",
            &json!({"description":"wvsc"}),
            &admin_token,
        )
        .await,
    )
    .await;
    let batch_id = Uuid::parse_str(batch_json["id"].as_str().expect("batch id")).expect("uuid");
    let legacy_created_at = chrono::DateTime::parse_from_rfc3339("2025-04-10T08:00:00+08:00")
        .expect("ingest ok")
        .with_timezone(&chrono::Utc);

    // Stage weight, vaccine, and two sacrifices (second will be skipped at approval)
    let ingest_req = json!({
        "pig": { "ear_tag": ear_tag, "birth_date": birth_date.to_string(), "iacuc_no": null },
        "records": [
            {
                "kind": "weight",
                "legacy_record_id": "w-1",
                "legacy_created_at": legacy_created_at.to_rfc3339(),
                "event_date": "2025-04-05",
                "payload": { "event_date":"2025-04-05", "weight":"12.3" }
            },
            {
                "kind": "vaccine",
                "legacy_record_id": "v-1",
                "legacy_created_at": legacy_created_at.to_rfc3339(),
                "event_date": "2025-04-06",
                "payload": { "event_date":"2025-04-06", "vaccine":"APP", "deworming_dose":"5ml" }
            },
            {
                "kind": "sacrifice",
                "legacy_record_id": "s-1",
                "legacy_created_at": legacy_created_at.to_rfc3339(),
                "event_date": "2025-04-07",
                "payload": {
                    "event_date":"2025-04-07",
                    "zoletil_dose":"10 mg/kg",
                    "method_electrocution": true,
                    "method_bloodletting": false,
                    "sampling":"血液",
                    "blood_volume_ml":"20.0"
                }
            },
            {
                "kind": "sacrifice",
                "legacy_record_id": "s-2",
                "legacy_created_at": legacy_created_at.to_rfc3339(),
                "event_date": "2025-04-08",
                "payload": {
                    "event_date":"2025-04-08",
                    "zoletil_dose":"11 mg/kg",
                    "method_electrocution": false,
                    "method_bloodletting": true,
                    "sampling":"血液",
                    "blood_volume_ml":"25.0"
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
    assert!(ingest_res.status().is_success());

    // Approve
    let admin_email =
        std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@ipigsystem.asia".to_string());
    let (admin_id,): (Uuid,) =
        sqlx::query_as("SELECT id FROM users WHERE lower(email)=lower($1) LIMIT 1")
            .bind(&admin_email)
            .fetch_one(&app.db_pool)
            .await
            .expect("approve ok");
    let actor = erp_backend::middleware::ActorContext::User(erp_backend::middleware::CurrentUser {
        id: admin_id,
        email: admin_email,
        roles: vec![],
        permissions: vec![],
        jti: String::new(),
        exp: 0,
        impersonated_by: None,
    });
    let (_a, skipped, _e) = LegacyImportService::approve_batch(
        &app.db_pool,
        &actor,
        batch_id,
        Some(admin_id),
        false,
        None,
    )
    .await
    .expect("weight count");
    // Weight exists
    let w: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM animal_weights WHERE animal_id=$1 AND weight=12.3")
            .bind(animal_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("vaccine count");
    assert_eq!(w.0, 1);
    // Vaccine exists
    let v: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM animal_vaccinations WHERE animal_id=$1 AND vaccine='APP'",
    )
    .bind(animal_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("sacrifice count");
    assert_eq!(v.0, 1);
    // One sacrifice exists, second skipped at approval
    let s: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM animal_sacrifices WHERE animal_id=$1")
        .bind(animal_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("sacrifice count");
    assert_eq!(s.0, 1, "only one sacrifice should be created");
    assert!(
        skipped >= 1,
        "approval should report skipped duplicate sacrifice"
    );
}

#[tokio::test]
async fn approval_permission_denied_for_non_privileged_user() {
    let app = TestApp::spawn().await;
    // Create a non-admin user via helper
    let uid = common::test_db::seed_other_user(&app.db_pool, "perm-deny").await;
    let actor = erp_backend::middleware::ActorContext::User(erp_backend::middleware::CurrentUser {
        id: uid,
        email: "tester@example.com".into(),
        roles: vec![],
        permissions: vec![],
        jti: String::new(),
        exp: 0,
        impersonated_by: None,
    });
    // Create empty batch to attempt approval
    let admin_token = app.login_as_admin().await;
    let batch: serde_json::Value = TestApp::json(
        app.auth_post(
            "/api/v1/legacy-import/batches",
            &json!({"description":"perm deny"}),
            &admin_token,
        )
        .await,
    )
    .await;
    let batch_id = Uuid::parse_str(batch["id"].as_str().expect("batch id")).expect("uuid");
    let err = LegacyImportService::approve_batch(&app.db_pool, &actor, batch_id, None, true, None)
        .await
        .expect_err("should be forbidden");
    let err_text = format!("{}", err);
    assert!(
        err_text.contains("無權核准匯入"),
        "expected Forbidden error, got {}",
        err_text
    );
}

#[tokio::test]
async fn ingest_dry_run_reports_pending_and_writes_nothing() {
    let app = TestApp::spawn().await;
    let admin_token = app.login_as_admin().await;
    // Animal
    let birth_date = chrono::NaiveDate::from_ymd_opt(2025, 5, 5).expect("valid date");
    let created = common::create_animal_with_free_ear_tag(&app, &admin_token, |ear_tag| {
        serde_json::json!({
            "ear_tag": ear_tag,
            "breed": "minipig",
            "gender": "male",
            "entry_date": "2025-05-05",
            "entry_weight": 20.5,
            "pen_location": "A-01",
            "birth_date": birth_date.to_string(),
            "force_create": true
        })
    })
    .await;
    let ear_tag = created["ear_tag"].as_str().expect("ear_tag").to_string();
    // Batch
    let batch: serde_json::Value = TestApp::json(
        app.auth_post(
            "/api/v1/legacy-import/batches",
            &json!({"description":"dryrun"}),
            &admin_token,
        )
        .await,
    )
    .await;
    let batch_id = Uuid::parse_str(batch["id"].as_str().expect("batch id")).expect("uuid");
    let req = json!({
        "pig": { "ear_tag": ear_tag, "birth_date": birth_date.to_string(), "iacuc_no": null },
        "dry_run": true,
        "records": [{
            "kind":"observation",
            "legacy_record_id":"dry-1",
            "legacy_created_at":"2025-05-06T01:02:03Z",
            "event_date":"2025-05-06",
            "content":"just note",
            "payload":{"event_date":"2025-05-06","content":"just note"}
        }]
    });
    let res = app
        .auth_post(
            &format!("/api/v1/legacy-import/batches/{}/records", batch_id),
            &req,
            &admin_token,
        )
        .await;
    assert!(res.status().is_success());
    let json: serde_json::Value = TestApp::json(res).await;
    assert_eq!(json["results"][0]["status"], "pending");
    // Ensure no staging rows inserted (dry_run write nothing)
    let staged: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM legacy_import_records WHERE import_batch_id = $1")
            .bind(batch_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("staging count");
    assert_eq!(staged.0, 0);
}

#[tokio::test]
async fn changed_resend_after_approval_reports_conflict_and_does_not_overwrite() {
    let app = TestApp::spawn().await;
    let admin_token = app.login_as_admin().await;
    // Animal
    let birth_date = chrono::NaiveDate::from_ymd_opt(2025, 6, 6).expect("valid date");
    let created = common::create_animal_with_free_ear_tag(&app, &admin_token, |ear_tag| {
        serde_json::json!({
            "ear_tag": ear_tag,
            "breed": "minipig",
            "gender": "female",
            "entry_date": "2025-06-06",
            "entry_weight": 23.0,
            "pen_location": "A-01",
            "birth_date": birth_date.to_string(),
            "force_create": true
        })
    })
    .await;
    let animal_id = Uuid::parse_str(created["id"].as_str().expect("animal id")).expect("uuid");
    let ear_tag = created["ear_tag"].as_str().expect("ear_tag").to_string();
    // Batch
    let batch: serde_json::Value = TestApp::json(
        app.auth_post(
            "/api/v1/legacy-import/batches",
            &json!({"description":"resend"}),
            &admin_token,
        )
        .await,
    )
    .await;
    let batch_id = Uuid::parse_str(batch["id"].as_str().expect("batch id")).expect("uuid");
    let admin_email =
        std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@ipigsystem.asia".to_string());
    let (admin_id,): (Uuid,) =
        sqlx::query_as("SELECT id FROM users WHERE lower(email)=lower($1) LIMIT 1")
            .bind(&admin_email)
            .fetch_one(&app.db_pool)
            .await
            .expect("admin lookup");
    // Ingest and approve one observation
    let req1 = json!({
        "pig": { "ear_tag": ear_tag, "birth_date": birth_date.to_string(), "iacuc_no": null },
        "records": [{
            "kind":"observation","legacy_record_id":"x-1","legacy_created_at":"2025-06-07T00:00:00Z","event_date":"2025-06-07","content":"A","payload":{"event_date":"2025-06-07","content":"A"}
        }]
    });
    let _ = app
        .auth_post(
            &format!("/api/v1/legacy-import/batches/{}/records", batch_id),
            &req1,
            &admin_token,
        )
        .await;
    let actor = erp_backend::middleware::ActorContext::User(erp_backend::middleware::CurrentUser {
        id: admin_id,
        email: admin_email,
        roles: vec![],
        permissions: vec![],
        jti: String::new(),
        exp: 0,
        impersonated_by: None,
    });
    let _ = LegacyImportService::approve_batch(
        &app.db_pool,
        &actor,
        batch_id,
        Some(admin_id),
        false,
        None,
    )
    .await
    .expect("approve ok");
    // Re-send same legacy_record_id with changed content
    let req2 = json!({
        "pig": { "ear_tag": ear_tag, "birth_date": birth_date.to_string(), "iacuc_no": null },
        "records": [{
            "kind":"observation","legacy_record_id":"x-1","legacy_created_at":"2025-06-07T00:00:00Z","event_date":"2025-06-07","content":"B","payload":{"event_date":"2025-06-07","content":"B"}
        }]
    });
    let res = app
        .auth_post(
            &format!("/api/v1/legacy-import/batches/{}/records", batch_id),
            &req2,
            &admin_token,
        )
        .await;
    assert!(res.status().is_success());
    let js: serde_json::Value = TestApp::json(res).await;
    assert_eq!(js["results"][0]["status"], "conflict");
    // The formal observation content should remain "A"
    let row: (String,) = sqlx::query_as("SELECT content FROM animal_observations WHERE animal_id=$1 ORDER BY created_at DESC LIMIT 1")
        .bind(animal_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("select observation");
    assert_eq!(row.0, "A");
}

#[tokio::test]
async fn recorder_mapping_csv_active_disabled_unmatched_and_legacy_name_written() {
    let app = TestApp::spawn().await;
    let admin_token = app.login_as_admin().await;
    // Seed mapped users: one active, one disabled
    let active_id = common::test_db::seed_other_user(&app.db_pool, "rec-active").await;
    // Disable the second user
    let disabled_id = common::test_db::seed_other_user(&app.db_pool, "rec-disabled").await;
    sqlx::query("UPDATE users SET is_active = false WHERE id = $1")
        .bind(disabled_id)
        .execute(&app.db_pool)
        .await
        .expect("disable user");
    // Create one animal
    let bd = chrono::NaiveDate::from_ymd_opt(2025, 7, 7).expect("valid date");
    let created = common::create_animal_with_free_ear_tag(&app, &admin_token, |ear_tag| {
        json!({
            "ear_tag": ear_tag,
            "breed": "minipig",
            "gender": "male",
            "entry_date": "2025-07-01",
            "entry_weight": 20.0,
            "pen_location": "A-01",
            "birth_date": bd.to_string(),
            "force_create": true
        })
    })
    .await;
    let animal_id = Uuid::parse_str(created["id"].as_str().expect("id")).expect("uuid");
    let ear_tag = created["ear_tag"].as_str().expect("ear_tag").to_string();
    // Batch and stage three observations with different recorder names
    let batch: serde_json::Value = TestApp::json(
        app.auth_post(
            "/api/v1/legacy-import/batches",
            &json!({"description":"recorder map"}),
            &admin_token,
        )
        .await,
    )
    .await;
    let batch_id = Uuid::parse_str(batch["id"].as_str().expect("batch id")).expect("uuid");
    let ingest = json!({
        "pig": { "ear_tag": ear_tag, "birth_date": bd.to_string(), "iacuc_no": null },
        "records": [
            { "kind":"observation","legacy_record_id":"rm-1","legacy_created_at":"2025-07-02T00:00:00Z","event_date":"2025-07-02","content":"A","original_recorder_name":"Active User","payload":{"event_date":"2025-07-02","content":"A"} },
            { "kind":"observation","legacy_record_id":"rm-2","legacy_created_at":"2025-07-03T00:00:00Z","event_date":"2025-07-03","content":"B","original_recorder_name":"Disabled User","payload":{"event_date":"2025-07-03","content":"B"} },
            { "kind":"observation","legacy_record_id":"rm-3","legacy_created_at":"2025-07-04T00:00:00Z","event_date":"2025-07-04","content":"C","original_recorder_name":"Unknown User","payload":{"event_date":"2025-07-04","content":"C"} }
        ]
    });
    let _ = app
        .auth_post(
            &format!("/api/v1/legacy-import/batches/{}/records", batch_id),
            &ingest,
            &admin_token,
        )
        .await;
    // Build mapping: Active User -> active_id email, Disabled User -> disabled_id email
    // emails not required for assertions
    let mut map: std::collections::HashMap<String, uuid::Uuid> = std::collections::HashMap::new();
    map.insert("Active User".to_string(), active_id);
    map.insert("Disabled User".to_string(), disabled_id);
    // Approve with import_user = admin (fallback target)
    let admin_email =
        std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@ipigsystem.asia".to_string());
    let (admin_id,): (Uuid,) =
        sqlx::query_as("SELECT id FROM users WHERE lower(email)=lower($1) LIMIT 1")
            .bind(&admin_email)
            .fetch_one(&app.db_pool)
            .await
            .expect("admin lookup");
    let actor = erp_backend::middleware::ActorContext::User(erp_backend::middleware::CurrentUser {
        id: admin_id,
        email: admin_email,
        roles: vec![],
        permissions: vec![],
        jti: String::new(),
        exp: 0,
        impersonated_by: None,
    });
    let (_a, _s, _e) = LegacyImportService::approve_batch(
        &app.db_pool,
        &actor,
        batch_id,
        Some(admin_id),
        false,
        Some(&map),
    )
    .await
    .expect("approve ok");
    // Assert created_by per mapping, and legacy_recorder_name filled
    let rows: Vec<(String, uuid::Uuid, Option<String>)> = sqlx::query_as(
        "SELECT content, created_by, legacy_recorder_name FROM animal_observations WHERE animal_id = $1 ORDER BY event_date",
    )
    .bind(animal_id)
    .fetch_all(&app.db_pool)
    .await
    .expect("select observations");
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].0, "A");
    assert_eq!(rows[0].1, active_id);
    assert_eq!(rows[0].2.as_deref(), Some("Active User"));
    assert_eq!(rows[1].0, "B");
    assert_eq!(rows[1].1, disabled_id);
    assert_eq!(rows[1].2.as_deref(), Some("Disabled User"));
    assert_eq!(rows[2].0, "C");
    assert_eq!(rows[2].1, admin_id, "unmapped falls back to import user");
    assert_eq!(rows[2].2.as_deref(), Some("Unknown User"));
}

#[tokio::test]
async fn per_animal_error_rollback_and_counts_with_duplicate_sacrifice_kept() {
    let app = TestApp::spawn().await;
    let admin_token = app.login_as_admin().await;
    // Animal A and B
    let bd_a = chrono::NaiveDate::from_ymd_opt(2025, 8, 1).expect("valid date");
    let created_a = common::create_animal_with_free_ear_tag(&app, &admin_token, |ear_tag| {
        json!({"ear_tag":ear_tag,"breed":"minipig","gender":"male","entry_date":"2025-08-01","entry_weight":20.0,"pen_location":"A-01","birth_date":bd_a.to_string(),"force_create":true})
    }).await;
    let a_id = Uuid::parse_str(created_a["id"].as_str().expect("id")).expect("uuid");
    let a_et = created_a["ear_tag"].as_str().expect("ear_tag").to_string();
    let bd_b = chrono::NaiveDate::from_ymd_opt(2025, 8, 2).expect("valid date");
    let created_b = common::create_animal_with_free_ear_tag(&app, &admin_token, |ear_tag| {
        json!({"ear_tag":ear_tag,"breed":"minipig","gender":"female","entry_date":"2025-08-02","entry_weight":19.0,"pen_location":"A-01","birth_date":bd_b.to_string(),"force_create":true})
    }).await;
    let b_id = Uuid::parse_str(created_b["id"].as_str().expect("id")).expect("uuid");
    let b_et = created_b["ear_tag"].as_str().expect("ear_tag").to_string();
    let admin_email =
        std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@ipigsystem.asia".to_string());
    let (admin_id,): (Uuid,) =
        sqlx::query_as("SELECT id FROM users WHERE lower(email)=lower($1) LIMIT 1")
            .bind(&admin_email)
            .fetch_one(&app.db_pool)
            .await
            .expect("admin lookup");
    // Batch: for A include invalid weight (abc) to trigger DB error, and a duplicate sacrifice; for B a valid observation
    let batch: serde_json::Value = TestApp::json(
        app.auth_post(
            "/api/v1/legacy-import/batches",
            &json!({"description":"per-animal-error"}),
            &admin_token,
        )
        .await,
    )
    .await;
    let batch_id = Uuid::parse_str(batch["id"].as_str().expect("batch id")).expect("uuid");
    // A records
    let ingest_a = json!({
        "pig": { "ear_tag": a_et, "birth_date": bd_a.to_string(), "iacuc_no": null },
        "records": [
            { "kind":"weight","legacy_record_id":"a-w","legacy_created_at":"2025-08-04T00:00:00Z","event_date":"2025-08-04","payload":{"event_date":"2025-08-04","weight":"abc"} },
            { "kind":"sacrifice","legacy_record_id":"a-s","legacy_created_at":"2025-08-05T00:00:00Z","event_date":"2025-08-05","payload":{"event_date":"2025-08-05"} }
        ]
    });
    let res_a = app
        .auth_post(
            &format!("/api/v1/legacy-import/batches/{}/records", batch_id),
            &ingest_a,
            &admin_token,
        )
        .await;
    assert!(res_a.status().is_success(), "ingest A should succeed");
    // B records
    let ingest_b = json!({
        "pig": { "ear_tag": b_et, "birth_date": bd_b.to_string(), "iacuc_no": null },
        "records": [
            { "kind":"observation","legacy_record_id":"b-o","legacy_created_at":"2025-08-04T00:00:00Z","event_date":"2025-08-04","content":"ok","payload":{"event_date":"2025-08-04","content":"ok"} }
        ]
    });
    let res_b = app
        .auth_post(
            &format!("/api/v1/legacy-import/batches/{}/records", batch_id),
            &ingest_b,
            &admin_token,
        )
        .await;
    assert!(res_b.status().is_success(), "ingest B should succeed");
    // Now pre-create one sacrifice for A so approval sees it as an existing duplicate
    sqlx::query(
        "INSERT INTO animal_sacrifices (animal_id, sacrifice_date, created_by, created_at) VALUES ($1,$2,$3,NOW())",
    )
    .bind(a_id)
    .bind(chrono::NaiveDate::from_ymd_opt(2025, 8, 3).expect("valid date"))
    .bind(admin_id)
    .execute(&app.db_pool)
    .await
    .expect("seed sacrifice");
    // Approve
    let actor = erp_backend::middleware::ActorContext::User(erp_backend::middleware::CurrentUser {
        id: admin_id,
        email: admin_email,
        roles: vec![],
        permissions: vec![],
        jti: String::new(),
        exp: 0,
        impersonated_by: None,
    });
    let (approved, skipped, error_count) = LegacyImportService::approve_batch(
        &app.db_pool,
        &actor,
        batch_id,
        Some(admin_id),
        false,
        None,
    )
    .await
    .expect("approve ok");
    assert!(approved >= 1, "animal B should be approved");
    assert!(skipped >= 1, "duplicate sacrifice should be skipped");
    assert!(error_count >= 1, "animal A should be counted as error");
    // Animal A should have no new weight or sacrifice created
    let w_cnt: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM animal_weights WHERE animal_id=$1")
        .bind(a_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("count weights A");
    assert_eq!(w_cnt.0, 0, "failed animal should not write formal rows");
    // Its staging rows should be marked error, except duplicate sacrifice which remains skipped_duplicate
    let rows_a: Vec<(String,)> = sqlx::query_as(
        "SELECT status FROM legacy_import_records WHERE import_batch_id=$1 AND matched_animal_id=$2 ORDER BY created_at",
    )
    .bind(batch_id)
    .bind(a_id)
    .fetch_all(&app.db_pool)
    .await
    .expect("select staging rows");
    assert_eq!(rows_a.len(), 2);
    assert!(rows_a.iter().any(|r| r.0 == "error"));
    assert!(rows_a.iter().any(|r| r.0 == "skipped_duplicate"));
    // Animal B observation exists
    let b_cnt: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM animal_observations WHERE animal_id=$1")
            .bind(b_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("count obs B");
    assert_eq!(b_cnt.0, 1);
}
