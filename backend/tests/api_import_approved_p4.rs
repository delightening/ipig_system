//! 匯入已核准計劃「補登」P4：端到端整合 + 列印 PDF 資料驗證。
//!
//! 串接 P1（匯入 + 完成補登）/ P2（補登審查文件寫真實表）全流程，驗證：
//! import → record_import_reviews → finalize_import 後，
//! 列印用匯出資料（審查回覆 review_reply / 審核結果 review_result）正確含補登資料，
//! 且資料在「完成補登」後仍可取得（PDF 列印不受鎖定影響）。
//!
//! PDF 實際渲染走外部 print-pdf 服務，無法於單元測試執行；匯出資料 JSON 即模板
//! 消費的契約，故以匯出資料結構作為列印正確性的代理驗證。

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

const SYSTEM_TEST: ActorContext = ActorContext::System {
    reason: "import_p4_integration",
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
    .bind(format!("import-p4-{label}-{}@test.local", &Uuid::new_v4().to_string()[..6]))
    .bind(format!("import p4 {label}"))
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

fn cmt(content: &str, reply: Option<&str>) -> ImportReviewComment {
    ImportReviewComment {
        reviewer_id: None,
        reviewer_name: None,
        content: content.to_string(),
        reply: reply.map(String::from),
        section_no: None,
    }
}

fn make_import_request(pi: Uuid, sd: Uuid, iacuc_no: String) -> ImportApprovedProtocolRequest {
    ImportApprovedProtocolRequest {
        title: "P4 端到端補登".to_string(),
        pi_user_id: Some(pi),
        study_director_user_id: sd,
        iacuc_no,
        application_no: Some("APIG-104001".to_string()),
        working_content: Some(serde_json::json!({ "basic": { "study_title": "P4" } })),
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
    }
}

/// 補登審查：執秘 + 系統內委員一審/二審 + 院外委員一審 + 院外獸醫。
fn make_reviews_request(sys_reviewer: Uuid) -> ImportReviewsRequest {
    ImportReviewsRequest {
        secretary_comments: vec![ImportReviewComment {
            reviewer_id: None,
            reviewer_name: Some("執秘".to_string()),
            content: "請補充麻醉劑量".to_string(),
            reply: Some("已補充".to_string()),
            section_no: Some("4.1.2".to_string()),
        }],
        committee_reviewers: vec![
            ImportCommitteeReviewer {
                reviewer_id: Some(sys_reviewer),
                reviewer_name: None,
                first_round: vec![cmt("樣本數需說明", Some("已說明"))],
                second_round: vec![cmt("同意通過", None)],
            },
            ImportCommitteeReviewer {
                reviewer_id: None,
                reviewer_name: Some("王教授（院外）".to_string()),
                first_round: vec![cmt("倫理考量需強化", None)],
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
    }
}

// ── 全流程：匯入 → 補登審查 → 完成補登 → 列印匯出資料正確 ──
#[tokio::test]
#[serial]
async fn full_lifecycle_export_data_reflects_imported_reviews() {
    let app = TestApp::spawn().await;
    let pi = seed_user_with_role(&app, "PI", "pi").await;
    let sd = seed_user_with_role(&app, "EXPERIMENT_STAFF", "sd").await;
    let sys_reviewer = seed_user_with_role(&app, "REVIEWER", "rev").await;
    let unique = &Uuid::new_v4().to_string()[..8];

    // 1) 匯入（帶歷史里程碑）
    let import_req = make_import_request(pi, sd, format!("PIG-{unique}"));
    let protocol = ProtocolService::import_approved(&app.db_pool, &SYSTEM_TEST, &import_req, pi)
        .await
        .expect("import");
    assert!(protocol.import_pending, "匯入即補登中");

    // 2) 補登審查文件（執秘 + 系統內委員一審/二審 + 院外委員 + 院外獸醫）
    let reviews = make_reviews_request(sys_reviewer);
    ProtocolService::record_import_reviews(&app.db_pool, &SYSTEM_TEST, protocol.id, &reviews)
        .await
        .expect("record reviews");

    // 3) 完成補登（鎖定）
    let finalized = ProtocolService::finalize_import(
        &app.db_pool,
        &SYSTEM_TEST,
        protocol.id,
        Some("v1.0".into()),
    )
    .await
    .expect("finalize");
    assert!(!finalized.import_pending, "完成補登後解除補登中");

    // 4) 審查回覆 PDF 匯出資料：完成補登後仍可取得且含補登審查
    let reply =
        ProtocolService::get_review_reply_export_data(&app.db_pool, scope(&app, protocol.id).await)
            .await
            .expect("review_reply export");
    assert_eq!(
        reply["secretary_items"].as_array().map(|a| a.len()),
        Some(1),
        "執秘意見 1 條"
    );
    // 委員 slot（committee_1..4）順序依 reviewer 出現序（UUID），非固定 → 以集合包含驗證
    let committee_items: Vec<serde_json::Value> = (1..=4)
        .filter_map(|n| {
            reply
                .get(format!("committee_{n}"))
                .and_then(|v| v.as_array())
                .cloned()
        })
        .flatten()
        .collect();
    assert!(
        committee_items
            .iter()
            .any(|it| it["opinion_1st"] == "樣本數需說明" && it["opinion_2nd"] == "同意通過"),
        "系統內委員一審+二審應配對呈現（opinion_1st + opinion_2nd）"
    );
    assert!(
        committee_items
            .iter()
            .any(|it| it["opinion_1st"] == "倫理考量需強化"),
        "院外委員一審應呈現"
    );
    assert_eq!(
        reply["vet_items"].as_array().map(|a| a.len()),
        Some(1),
        "獸醫評比 1 項"
    );

    // 5) 審核結果 PDF 匯出資料：初審 + 複審意見皆含補登資料
    let result = ProtocolService::get_review_result_export_data(
        &app.db_pool,
        scope(&app, protocol.id).await,
    )
    .await
    .expect("review_result export");
    let initial: Vec<String> = result["initial_comments"]
        .as_array()
        .expect("initial_comments")
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    let final_c: Vec<String> = result["final_comments"]
        .as_array()
        .expect("final_comments")
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    // 初審含兩位委員一審意見（系統內 + 院外）
    assert!(
        initial.iter().any(|c| c == "樣本數需說明"),
        "初審應含系統內委員一審意見"
    );
    assert!(
        initial.iter().any(|c| c == "倫理考量需強化"),
        "初審應含院外委員一審意見"
    );
    // 複審含系統內委員二審意見
    assert!(
        final_c.iter().any(|c| c == "同意通過"),
        "複審應含委員二審意見"
    );
}
