//! 匯入已核准計劃「補登審查文件」P2 回歸測試。
//!
//! - record_import_reviews 寫入真實審查表（review_comments / vet_review_assignments）。
//! - 支援系統內審查者（FK）與院外審查者（填姓名）。
//! - 全量取代語意；限 import_pending 計劃；Anonymous 拒絕。
//! - PDF 匯出資料（get_review_reply_export_data）正確含補登的審查意見。

mod common;

use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::{
    ImportApprovedProtocolRequest, ImportCommitteeReviewer, ImportReviewComment,
    ImportReviewsRequest, ImportVetReview, VetReviewItem,
};
use erp_backend::services::access::{ProtocolId, Scoped};
use erp_backend::services::ProtocolService;
use erp_backend::AppError;

const SYSTEM_TEST: ActorContext = ActorContext::System {
    reason: "import_p2_regression",
};

/// 構造已授權的 `Scoped<ProtocolId>`：viewer 持 `aup.protocol.view_all` 權限，
/// 故走真實 `authorize` 必過、無需額外 DB 角色 setup（R75-P4 Phase 2）。
async fn scope(app: &TestApp, pid: Uuid) -> Scoped<ProtocolId> {
    let viewer = CurrentUser {
        id: Uuid::new_v4(),
        email: "viewer@test.local".to_string(),
        roles: vec![],
        permissions: vec!["aup.protocol.view_all".to_string()],
        jti: "test".to_string(),
        exp: 0,
        impersonated_by: None,
    };
    Scoped::<ProtocolId>::authorize(&app.db_pool, &viewer, pid)
        .await
        .expect("authorize scope")
}

async fn seed_user_with_role(app: &TestApp, role: &str, label: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', $3, true, false)"#,
    )
    .bind(id)
    .bind(format!("import-p2-{label}-{}@test.local", &Uuid::new_v4().to_string()[..6]))
    .bind(format!("import p2 {label}"))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
    )
    .bind(id)
    .bind(role)
    .execute(&app.db_pool)
    .await
    .expect("assign role");
    id
}

async fn import_pending_protocol(app: &TestApp, pi: Uuid, sd: Uuid) -> Uuid {
    let unique = &Uuid::new_v4().to_string()[..8];
    let req = ImportApprovedProtocolRequest {
        title: "P2 補登審查測試".to_string(),
        pi_user_id: Some(pi),
        study_director_user_id: sd,
        iacuc_no: format!("PIG-{unique}"),
        application_no: None,
        working_content: None,
        start_date: None,
        end_date: None,
        submitted_at: None,
        pre_review_at: None,
        vet_review_at: None,
        committee_first_review_at: None,
        revision_required_at: None,
        committee_second_review_at: None,
        approved_at: None,
        remark: None,
        notice_version_label: None,
        notice_attachment_id: None,
        notice_acknowledged_at: None,
        source_form_version: None,
    };
    ProtocolService::import_approved(&app.db_pool, &SYSTEM_TEST, &req, pi)
        .await
        .expect("import")
        .id
}

fn comment(
    reviewer_id: Option<Uuid>,
    name: Option<&str>,
    content: &str,
    reply: Option<&str>,
) -> ImportReviewComment {
    ImportReviewComment {
        reviewer_id,
        reviewer_name: name.map(String::from),
        content: content.to_string(),
        reply: reply.map(String::from),
        section_no: None,
    }
}

// ── 補登審查文件寫入真實表（系統內 + 院外）+ PDF 匯出資料正確 ──
#[tokio::test]
#[serial]
async fn record_import_reviews_writes_real_tables_and_pdf_data() {
    let app = TestApp::spawn().await;
    let pi = seed_user_with_role(&app, "PI", "pi").await;
    let sd = seed_user_with_role(&app, "EXPERIMENT_STAFF", "sd").await;
    let committee_member = seed_user_with_role(&app, "REVIEWER", "rev").await;
    let protocol_id = import_pending_protocol(&app, pi, sd).await;

    let req = ImportReviewsRequest {
        secretary_comments: vec![comment(
            None,
            Some("林執秘"),
            "請補充麻醉劑量",
            Some("已補充於第三節"),
        )],
        committee_reviewers: vec![
            // 系統內委員
            ImportCommitteeReviewer {
                reviewer_id: Some(committee_member),
                reviewer_name: None,
                first_round: vec![comment(None, None, "樣本數需說明", Some("已說明"))],
                second_round: vec![comment(None, None, "同意通過", None)],
            },
            // 院外委員（填姓名）
            ImportCommitteeReviewer {
                reviewer_id: None,
                reviewer_name: Some("王教授（院外）".to_string()),
                first_round: vec![comment(None, None, "倫理考量需強化", None)],
                second_round: vec![],
            },
        ],
        vet_review: Some(ImportVetReview {
            vet_id: None,
            vet_name: Some("陳獸醫（院外）".to_string()),
            decision: Some("APPROVED".to_string()),
            decision_remark: Some("符合動物福利".to_string()),
            items: vec![VetReviewItem {
                item_name: "麻醉計畫".to_string(),
                compliance: "V".to_string(),
                comment: Some("適當".to_string()),
                pi_reply: None,
            }],
            signed_at: None,
        }),
    };

    ProtocolService::record_import_reviews(&app.db_pool, &SYSTEM_TEST, protocol_id, &req)
        .await
        .expect("record_import_reviews 應成功");

    // 執秘意見：1 主意見 + 1 申請人回覆（共 2 列 PRE_REVIEW）
    let pre_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM review_comments WHERE protocol_id = $1 AND review_stage = 'PRE_REVIEW'",
    )
    .bind(protocol_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("count pre");
    assert_eq!(pre_count, 2, "執秘意見應有主意見 + 回覆 2 列");

    // 院外委員：reviewer_id NULL + reviewer_name 有值
    let ext: (Option<Uuid>, Option<String>) = sqlx::query_as(
        "SELECT reviewer_id, reviewer_name FROM review_comments \
         WHERE protocol_id = $1 AND reviewer_name = '王教授（院外）' LIMIT 1",
    )
    .bind(protocol_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("fetch external reviewer");
    assert!(ext.0.is_none(), "院外委員 reviewer_id 應為 NULL");
    assert_eq!(ext.1.as_deref(), Some("王教授（院外）"));

    // 系統內委員：reviewer_id 有值 + reviewer_name NULL
    let sys: (Option<Uuid>, Option<String>) = sqlx::query_as(
        "SELECT reviewer_id, reviewer_name FROM review_comments \
         WHERE protocol_id = $1 AND reviewer_id = $2 LIMIT 1",
    )
    .bind(protocol_id)
    .bind(committee_member)
    .fetch_one(&app.db_pool)
    .await
    .expect("fetch system reviewer");
    assert_eq!(sys.0, Some(committee_member));
    assert!(
        sys.1.is_none(),
        "系統內委員 reviewer_name 應為 NULL（顯示用 display_name）"
    );

    // 獸醫師：院外，vet_name 有值 + review_form items
    let vet: (Option<Uuid>, Option<String>, Option<serde_json::Value>) = sqlx::query_as(
        "SELECT vet_id, vet_name, review_form FROM vet_review_assignments WHERE protocol_id = $1",
    )
    .bind(protocol_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("fetch vet");
    assert!(vet.0.is_none(), "院外獸醫 vet_id 應為 NULL");
    assert_eq!(vet.1.as_deref(), Some("陳獸醫（院外）"));
    assert!(vet.2.is_some(), "review_form 應寫入");

    // PDF 匯出資料：secretary_items / committee_1 / committee_2 / vet_items 皆有值
    let data =
        ProtocolService::get_review_reply_export_data(&app.db_pool, scope(&app, protocol_id).await)
            .await
            .expect("export data");
    assert_eq!(data["secretary_items"].as_array().map(|a| a.len()), Some(1));
    assert_eq!(
        data["committee_1"].as_array().map(|a| a.len()),
        Some(1),
        "系統內委員一審 1 條"
    );
    assert_eq!(
        data["committee_2"].as_array().map(|a| a.len()),
        Some(1),
        "院外委員一審 1 條"
    );
    assert_eq!(
        data["vet_items"].as_array().map(|a| a.len()),
        Some(1),
        "獸醫評比 1 項"
    );
}

// ── 補登審查意見「項次」(section_no) 持久化 + PDF 匯出 round-trip ──
#[tokio::test]
#[serial]
async fn import_review_section_no_persists_and_exports() {
    let app = TestApp::spawn().await;
    let pi = seed_user_with_role(&app, "PI", "pi").await;
    let sd = seed_user_with_role(&app, "EXPERIMENT_STAFF", "sd").await;
    let rev = seed_user_with_role(&app, "REVIEWER", "rev").await;
    let protocol_id = import_pending_protocol(&app, pi, sd).await;

    let secretary = ImportReviewComment {
        reviewer_id: None,
        reviewer_name: Some("林執秘".to_string()),
        content: "請補充麻醉劑量".to_string(),
        reply: None,
        section_no: Some("4.1.2".to_string()),
    };
    let committee = ImportReviewComment {
        reviewer_id: None,
        reviewer_name: None,
        content: "樣本數需說明".to_string(),
        reply: None,
        section_no: Some("5.3".to_string()),
    };
    let req = ImportReviewsRequest {
        secretary_comments: vec![secretary],
        committee_reviewers: vec![ImportCommitteeReviewer {
            reviewer_id: Some(rev),
            reviewer_name: None,
            first_round: vec![committee],
            second_round: vec![],
        }],
        vet_review: None,
    };
    ProtocolService::record_import_reviews(&app.db_pool, &SYSTEM_TEST, protocol_id, &req)
        .await
        .expect("record");

    // 執秘意見 section_no 持久化
    let sec_section: Option<String> = sqlx::query_scalar(
        "SELECT section_no FROM review_comments \
         WHERE protocol_id = $1 AND review_stage = 'PRE_REVIEW' AND parent_comment_id IS NULL",
    )
    .bind(protocol_id)
    .fetch_one(&app.db_pool)
    .await
    .expect("secretary section_no");
    assert_eq!(
        sec_section.as_deref(),
        Some("4.1.2"),
        "執秘意見項次應持久化"
    );

    // PDF 匯出資料帶 section_no（執秘 + 委員）
    let data =
        ProtocolService::get_review_reply_export_data(&app.db_pool, scope(&app, protocol_id).await)
            .await
            .expect("export");
    assert_eq!(
        data["secretary_items"][0]["section_no"], "4.1.2",
        "執秘匯出帶項次"
    );
    assert_eq!(
        data["committee_1"][0]["section_no"], "5.3",
        "委員匯出帶項次"
    );
}

// ── 全量取代：重送會清掉先前補登資料 ──
#[tokio::test]
#[serial]
async fn record_import_reviews_replaces_previous() {
    let app = TestApp::spawn().await;
    let pi = seed_user_with_role(&app, "PI", "pi").await;
    let sd = seed_user_with_role(&app, "EXPERIMENT_STAFF", "sd").await;
    let protocol_id = import_pending_protocol(&app, pi, sd).await;

    let first = ImportReviewsRequest {
        secretary_comments: vec![
            comment(None, Some("執秘A"), "意見1", None),
            comment(None, Some("執秘A"), "意見2", None),
        ],
        committee_reviewers: vec![],
        vet_review: None,
    };
    ProtocolService::record_import_reviews(&app.db_pool, &SYSTEM_TEST, protocol_id, &first)
        .await
        .expect("first");

    let second = ImportReviewsRequest {
        secretary_comments: vec![comment(None, Some("執秘A"), "唯一意見", None)],
        committee_reviewers: vec![],
        vet_review: None,
    };
    ProtocolService::record_import_reviews(&app.db_pool, &SYSTEM_TEST, protocol_id, &second)
        .await
        .expect("second");

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM review_comments WHERE protocol_id = $1")
            .bind(protocol_id)
            .fetch_one(&app.db_pool)
            .await
            .expect("count");
    assert_eq!(count, 1, "重送應全量取代，僅留最新 1 列");
}

// ── 非補登中計劃拒絕 ──
#[tokio::test]
#[serial]
async fn record_import_reviews_rejects_non_pending() {
    let app = TestApp::spawn().await;
    let pi = seed_user_with_role(&app, "PI", "pi").await;
    let sd = seed_user_with_role(&app, "EXPERIMENT_STAFF", "sd").await;
    let protocol_id = import_pending_protocol(&app, pi, sd).await;

    // 完成補登 → 不再 import_pending
    ProtocolService::finalize_import(&app.db_pool, &SYSTEM_TEST, protocol_id, None)
        .await
        .expect("finalize");

    let req = ImportReviewsRequest {
        secretary_comments: vec![comment(None, Some("執秘"), "意見", None)],
        committee_reviewers: vec![],
        vet_review: None,
    };
    let err = ProtocolService::record_import_reviews(&app.db_pool, &SYSTEM_TEST, protocol_id, &req)
        .await
        .expect_err("非補登中應拒絕");
    assert!(
        matches!(err, AppError::BusinessRule(_)),
        "應 BusinessRule，實得：{err:?}"
    );
}

// ── Anonymous 拒絕 + 院外審查者缺 id/name 驗證 ──
#[tokio::test]
#[serial]
async fn record_import_reviews_guards() {
    let app = TestApp::spawn().await;
    let pi = seed_user_with_role(&app, "PI", "pi").await;
    let sd = seed_user_with_role(&app, "EXPERIMENT_STAFF", "sd").await;
    let protocol_id = import_pending_protocol(&app, pi, sd).await;

    let ok_req = ImportReviewsRequest {
        secretary_comments: vec![comment(None, Some("執秘"), "意見", None)],
        committee_reviewers: vec![],
        vet_review: None,
    };
    let err = ProtocolService::record_import_reviews(
        &app.db_pool,
        &ActorContext::Anonymous,
        protocol_id,
        &ok_req,
    )
    .await
    .expect_err("Anonymous 應拒絕");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應 Forbidden，實得：{err:?}"
    );

    // 委員既無 id 也無 name → Validation
    let bad_req = ImportReviewsRequest {
        secretary_comments: vec![],
        committee_reviewers: vec![ImportCommitteeReviewer {
            reviewer_id: None,
            reviewer_name: None,
            first_round: vec![comment(None, None, "意見", None)],
            second_round: vec![],
        }],
        vet_review: None,
    };
    let err2 =
        ProtocolService::record_import_reviews(&app.db_pool, &SYSTEM_TEST, protocol_id, &bad_req)
            .await
            .expect_err("委員缺 id/name 應拒絕");
    assert!(
        matches!(err2, AppError::Validation(_)),
        "應 Validation，實得：{err2:?}"
    );

    // 執秘意見既無 id 也無 name → Validation（Gemini #534：避免落到 DB CHECK 500）
    let bad_secretary = ImportReviewsRequest {
        secretary_comments: vec![comment(None, None, "無身分意見", None)],
        committee_reviewers: vec![],
        vet_review: None,
    };
    let err3 = ProtocolService::record_import_reviews(
        &app.db_pool,
        &SYSTEM_TEST,
        protocol_id,
        &bad_secretary,
    )
    .await
    .expect_err("執秘意見缺 id/name 應拒絕");
    assert!(
        matches!(err3, AppError::Validation(_)),
        "應 Validation，實得：{err3:?}"
    );
}
