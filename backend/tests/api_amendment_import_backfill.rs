//! 補登歷史變更申請（Amendment Import Backfill）P6-2 整合測試。
//!
//! - create_historical：建 is_historical DRAFT + 回填日期，僅限匯入計劃 + 已分類。
//! - finalize_historical：DRAFT → EFFECTIVE + 回填生效日 + 版本快照，無 live 簽章。
//! - 編號接續：歷史佔前號，後續 live 變更自動接續。
//! - 互斥：live 變更不可走補登 finalize；Anonymous 一律拒絕。
//! - access gate：限計劃負責人 SD / 管理者，且須為匯入計劃。

mod common;

use chrono::{TimeZone, Utc};
use common::TestApp;
use serial_test::serial;
use uuid::Uuid;

use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::{
    AmendmentStatus, AmendmentType, CreateAmendmentRequest, CreateHistoricalAmendmentRequest,
    FinalizeHistoricalAmendmentRequest, HistoricalAmendmentReviewer, ImportApprovedProtocolRequest,
    RecordHistoricalReviewsRequest,
};
use erp_backend::services::{access, AmendmentService, ProtocolService};
use erp_backend::AppError;

const SYSTEM_TEST: ActorContext = ActorContext::System {
    reason: "amendment_backfill_test",
};

async fn seed_pi(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'backfill pi', true, false)"#,
    )
    .bind(id)
    .bind(format!("bf-pi-{}@test.local", &Uuid::new_v4().to_string()[..6]))
    .execute(&app.db_pool)
    .await
    .expect("insert pi user");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = 'PI'",
    )
    .bind(id)
    .execute(&app.db_pool)
    .await
    .expect("assign PI role");
    id
}

async fn seed_sd(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, is_internal, must_change_password)
           VALUES ($1, $2, 'fake', 'backfill sd', true, true, false)"#,
    )
    .bind(id)
    .bind(format!("bf-sd-{}@test.local", &Uuid::new_v4().to_string()[..6]))
    .execute(&app.db_pool)
    .await
    .expect("insert sd user");
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = 'EXPERIMENT_STAFF'",
    )
    .bind(id)
    .execute(&app.db_pool)
    .await
    .expect("assign EXPERIMENT_STAFF role");
    id
}

fn import_req(pi: Uuid, sd: Uuid, iacuc_no: &str) -> ImportApprovedProtocolRequest {
    ImportApprovedProtocolRequest {
        title: "補登變更測試計劃".to_string(),
        pi_user_id: Some(pi),
        study_director_user_id: sd,
        iacuc_no: iacuc_no.to_string(),
        application_no: Some("APIG-103001".to_string()),
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
    }
}

/// 匯入 + 完成補登 → 回傳 (protocol_id, sd_id)。證明補登變更可作用於已 finalize（鎖定）的匯入計劃。
async fn seed_imported_protocol(app: &TestApp) -> (Uuid, Uuid) {
    let pi = seed_pi(app).await;
    let sd = seed_sd(app).await;
    let unique = &Uuid::new_v4().to_string()[..8];
    let p = ProtocolService::import_approved(
        &app.db_pool,
        &SYSTEM_TEST,
        &import_req(pi, sd, &format!("PIG-{unique}")),
        pi,
    )
    .await
    .expect("import");
    ProtocolService::finalize_import(&app.db_pool, &SYSTEM_TEST, p.id, None)
        .await
        .expect("finalize import");
    (p.id, sd)
}

/// R75-P4：取得變更申請 PI 寫入證明（用 SYSTEM_ADMIN → authorize 短路，免額外 user_protocols setup）。
async fn amendment_scope(app: &TestApp, pid: Uuid) -> access::Scoped<access::AmendmentWrite> {
    let admin = CurrentUser {
        id: Uuid::new_v4(),
        email: "amd-admin@test.local".to_string(),
        roles: vec!["SYSTEM_ADMIN".to_string()],
        permissions: vec![],
        jti: "test".to_string(),
        exp: 0,
        impersonated_by: None,
    };
    access::Scoped::<access::AmendmentWrite>::authorize(&app.db_pool, &admin, pid)
        .await
        .expect("amendment authorize")
}

fn create_req(
    protocol_id: Uuid,
    amendment_type: AmendmentType,
) -> CreateHistoricalAmendmentRequest {
    CreateHistoricalAmendmentRequest {
        protocol_id,
        title: "歷史變更：延長試驗期程".to_string(),
        description: Some("原核准期程延長三個月".to_string()),
        change_items: Some(vec!["schedule".to_string()]),
        changes_content: None,
        amendment_type,
        submitted_at: Some(
            Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0)
                .single()
                .expect("date"),
        ),
        classified_at: Some(
            Utc.with_ymd_and_hms(2024, 1, 20, 0, 0, 0)
                .single()
                .expect("date"),
        ),
        classification_remark: Some("執秘判定小變更".to_string()),
    }
}

fn cu(id: Uuid, perms: &[&str]) -> CurrentUser {
    CurrentUser {
        id,
        email: "x@test.local".into(),
        roles: vec![],
        permissions: perms.iter().map(|s| s.to_string()).collect(),
        jti: "t".into(),
        exp: 0,
        impersonated_by: None,
    }
}

// ── create_historical：DRAFT + is_historical + 回填日期 + 編號 R01 ──
#[tokio::test]
#[serial]
async fn historical_create_sets_draft_and_fields() {
    let app = TestApp::spawn().await;
    let (protocol_id, _sd) = seed_imported_protocol(&app).await;

    let a = AmendmentService::create_historical(
        &app.db_pool,
        &SYSTEM_TEST,
        &create_req(protocol_id, AmendmentType::Minor),
    )
    .await
    .expect("create_historical 應成功");

    assert_eq!(a.status, AmendmentStatus::Draft, "補登建立應為 DRAFT");
    assert!(a.is_historical, "應標記 is_historical");
    assert_eq!(
        a.amendment_type,
        AmendmentType::Minor,
        "amendment_type 應為 MINOR"
    );
    assert!(
        a.amendment_no.ends_with("-R01"),
        "首筆編號應 R01，實得：{}",
        a.amendment_no
    );
    assert_eq!(a.revision_number, 1);
    assert!(
        a.approved_signature_id.is_none(),
        "歷史變更不應有 live 核准簽章"
    );
    assert!(a.effective_from.is_none(), "DRAFT 階段尚未生效");
    assert_eq!(
        a.submitted_at,
        Some(
            Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0)
                .single()
                .expect("date")
        ),
        "應回填原始送件日"
    );
    assert_eq!(
        a.classified_at,
        Some(
            Utc.with_ymd_and_hms(2024, 1, 20, 0, 0, 0)
                .single()
                .expect("date")
        ),
        "應回填原始分類日"
    );
}

// ── 僅匯入計劃可補登：非匯入計劃 → BusinessRule ──
#[tokio::test]
#[serial]
async fn historical_requires_imported_protocol() {
    let app = TestApp::spawn().await;
    let pi = seed_pi(&app).await;
    let unique = &Uuid::new_v4().to_string()[..8];

    // 直接建一個 APPROVED 但非匯入（imported_at NULL）的計劃
    let protocol_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by)
           VALUES ($1, $2, $3, '一般計劃', 'APPROVED', $4, $4)"#,
    )
    .bind(protocol_id)
    .bind(format!("Pre-{unique}"))
    .bind(format!("PIG-{unique}"))
    .bind(pi)
    .execute(&app.db_pool)
    .await
    .expect("insert normal protocol");

    let err = AmendmentService::create_historical(
        &app.db_pool,
        &SYSTEM_TEST,
        &create_req(protocol_id, AmendmentType::Major),
    )
    .await
    .expect_err("非匯入計劃不可補登歷史變更");
    assert!(
        matches!(err, AppError::BusinessRule(_)),
        "應 BusinessRule，實得：{err:?}"
    );
}

// ── 補登須已分類：PENDING → BadRequest ──
#[tokio::test]
#[serial]
async fn historical_rejects_pending_type() {
    let app = TestApp::spawn().await;
    let (protocol_id, _sd) = seed_imported_protocol(&app).await;

    let err = AmendmentService::create_historical(
        &app.db_pool,
        &SYSTEM_TEST,
        &create_req(protocol_id, AmendmentType::Pending),
    )
    .await
    .expect_err("PENDING 分類應拒絕");
    assert!(
        matches!(err, AppError::BadRequest(_)),
        "應 BadRequest，實得：{err:?}"
    );
}

// ── finalize_historical：DRAFT → EFFECTIVE + 回填生效日 + 版本快照 + 無簽章 ──
#[tokio::test]
#[serial]
async fn historical_finalize_sets_effective() {
    let app = TestApp::spawn().await;
    let (protocol_id, _sd) = seed_imported_protocol(&app).await;

    let a = AmendmentService::create_historical(
        &app.db_pool,
        &SYSTEM_TEST,
        &create_req(protocol_id, AmendmentType::Major),
    )
    .await
    .expect("create");

    let effective = Utc
        .with_ymd_and_hms(2024, 2, 1, 0, 0, 0)
        .single()
        .expect("date");
    let req = FinalizeHistoricalAmendmentRequest {
        effective_from: Some(effective),
        remark: Some("完成補登".to_string()),
    };
    let done = AmendmentService::finalize_historical(&app.db_pool, &SYSTEM_TEST, a.id, &req)
        .await
        .expect("finalize_historical 應成功");

    assert_eq!(done.status, AmendmentStatus::Effective, "應落 EFFECTIVE");
    assert_eq!(done.effective_from, Some(effective), "應回填原始生效日");
    assert!(
        done.approved_signature_id.is_none(),
        "歷史變更不產生 live 簽章"
    );

    let vcount: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM amendment_versions WHERE amendment_id = $1")
            .bind(a.id)
            .fetch_one(&app.db_pool)
            .await
            .expect("count versions");
    assert_eq!(vcount, 1, "完成補登應建 1 筆 amendment_versions");

    // 已生效（非 DRAFT）不可重複 finalize
    let err = AmendmentService::finalize_historical(&app.db_pool, &SYSTEM_TEST, a.id, &req)
        .await
        .expect_err("非 DRAFT 不可再 finalize");
    assert!(
        matches!(err, AppError::BusinessRule(_)),
        "應 BusinessRule，實得：{err:?}"
    );
}

// ── 編號接續：歷史 R01/R02 → 後續 live 變更自動 R03 ──
#[tokio::test]
#[serial]
async fn historical_numbering_continues_to_live() {
    let app = TestApp::spawn().await;
    let (protocol_id, sd) = seed_imported_protocol(&app).await;

    let a1 = AmendmentService::create_historical(
        &app.db_pool,
        &SYSTEM_TEST,
        &create_req(protocol_id, AmendmentType::Minor),
    )
    .await
    .expect("create R01");
    let a2 = AmendmentService::create_historical(
        &app.db_pool,
        &SYSTEM_TEST,
        &create_req(protocol_id, AmendmentType::Major),
    )
    .await
    .expect("create R02");
    assert_eq!(a1.revision_number, 1);
    assert_eq!(a2.revision_number, 2);

    // 後續 live 變更（service 層）應接 R03。PI 寫入授權由 Scoped<AmendmentWrite> 證明。
    let scope = amendment_scope(&app, protocol_id).await;
    let live = AmendmentService::create(
        &app.db_pool,
        scope,
        &CreateAmendmentRequest {
            protocol_id,
            title: "live 新變更".to_string(),
            description: None,
            change_items: None,
            changes_content: None,
        },
        sd,
    )
    .await
    .expect("create live amendment");
    assert_eq!(live.revision_number, 3, "live 變更應自動接 R03");
    assert!(!live.is_historical, "live 變更非 historical");
}

// ── 互斥：live 變更不可走補登 finalize ──
#[tokio::test]
#[serial]
async fn historical_finalize_rejects_live_amendment() {
    let app = TestApp::spawn().await;
    let (protocol_id, sd) = seed_imported_protocol(&app).await;

    let scope = amendment_scope(&app, protocol_id).await;
    let live = AmendmentService::create(
        &app.db_pool,
        scope,
        &CreateAmendmentRequest {
            protocol_id,
            title: "live 變更".to_string(),
            description: None,
            change_items: None,
            changes_content: None,
        },
        sd,
    )
    .await
    .expect("create live amendment");

    let req = FinalizeHistoricalAmendmentRequest {
        effective_from: None,
        remark: None,
    };
    let err = AmendmentService::finalize_historical(&app.db_pool, &SYSTEM_TEST, live.id, &req)
        .await
        .expect_err("live 變更不可走補登 finalize");
    assert!(
        matches!(err, AppError::BusinessRule(_)),
        "應 BusinessRule，實得：{err:?}"
    );
}

// ── Anonymous 一律拒絕 ──
#[tokio::test]
#[serial]
async fn historical_rejects_anonymous() {
    let app = TestApp::spawn().await;
    let (protocol_id, _sd) = seed_imported_protocol(&app).await;

    let err = AmendmentService::create_historical(
        &app.db_pool,
        &ActorContext::Anonymous,
        &create_req(protocol_id, AmendmentType::Minor),
    )
    .await
    .expect_err("Anonymous 不可補登");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應 Forbidden，實得：{err:?}"
    );

    let a = AmendmentService::create_historical(
        &app.db_pool,
        &SYSTEM_TEST,
        &create_req(protocol_id, AmendmentType::Minor),
    )
    .await
    .expect("create");
    let req = FinalizeHistoricalAmendmentRequest {
        effective_from: None,
        remark: None,
    };
    let err2 =
        AmendmentService::finalize_historical(&app.db_pool, &ActorContext::Anonymous, a.id, &req)
            .await
            .expect_err("Anonymous 不可 finalize");
    assert!(
        matches!(err2, AppError::Forbidden(_)),
        "應 Forbidden，實得：{err2:?}"
    );
}

fn reviewer(id: Option<Uuid>, name: Option<&str>, decision: &str) -> HistoricalAmendmentReviewer {
    HistoricalAmendmentReviewer {
        reviewer_id: id,
        reviewer_name: name.map(String::from),
        decision: Some(decision.to_string()),
        comment: Some("歷史委員意見".to_string()),
        decided_at: Some(
            Utc.with_ymd_and_hms(2024, 1, 25, 0, 0, 0)
                .single()
                .expect("date"),
        ),
    }
}

async fn draft_major(app: &TestApp, protocol_id: Uuid) -> Uuid {
    AmendmentService::create_historical(
        &app.db_pool,
        &SYSTEM_TEST,
        &create_req(protocol_id, AmendmentType::Major),
    )
    .await
    .expect("create historical major")
    .id
}

// ── 補登審查：院外委員（姓名）+ 系統內委員，get_review_assignments 都能顯示 ──
#[tokio::test]
#[serial]
async fn historical_reviews_external_and_system() {
    let app = TestApp::spawn().await;
    let (protocol_id, _sd) = seed_imported_protocol(&app).await;
    let sys_reviewer = seed_pi(&app).await;
    let aid = draft_major(&app, protocol_id).await;

    let req = RecordHistoricalReviewsRequest {
        reviewers: vec![
            reviewer(None, Some("外部李委員"), "APPROVE"),
            reviewer(Some(sys_reviewer), None, "REVISION"),
        ],
    };
    AmendmentService::record_historical_reviews(&app.db_pool, &SYSTEM_TEST, aid, &req)
        .await
        .expect("record reviews");

    let list = AmendmentService::get_review_assignments(&app.db_pool, aid)
        .await
        .expect("list assignments");
    assert_eq!(list.len(), 2, "應有 2 位委員");
    let names: Vec<&str> = list.iter().map(|x| x.reviewer_name.as_str()).collect();
    assert!(
        names.contains(&"外部李委員"),
        "院外委員應以姓名顯示，實得：{names:?}"
    );
    assert!(
        names.iter().any(|n| n.contains("backfill pi")),
        "系統委員應顯示 display_name，實得：{names:?}"
    );
    let external = list
        .iter()
        .find(|x| x.reviewer_name == "外部李委員")
        .expect("find external");
    assert!(
        external.reviewer_id.is_none(),
        "院外委員 reviewer_id 應 NULL"
    );
}

// ── 全量取代：重送清掉先前 ──
#[tokio::test]
#[serial]
async fn historical_reviews_full_replace() {
    let app = TestApp::spawn().await;
    let (protocol_id, _sd) = seed_imported_protocol(&app).await;
    let aid = draft_major(&app, protocol_id).await;

    let r1 = RecordHistoricalReviewsRequest {
        reviewers: vec![
            reviewer(None, Some("委員A"), "APPROVE"),
            reviewer(None, Some("委員B"), "APPROVE"),
        ],
    };
    AmendmentService::record_historical_reviews(&app.db_pool, &SYSTEM_TEST, aid, &r1)
        .await
        .expect("first");
    let r2 = RecordHistoricalReviewsRequest {
        reviewers: vec![reviewer(None, Some("委員C"), "REJECT")],
    };
    AmendmentService::record_historical_reviews(&app.db_pool, &SYSTEM_TEST, aid, &r2)
        .await
        .expect("second");

    let list = AmendmentService::get_review_assignments(&app.db_pool, aid)
        .await
        .expect("list");
    assert_eq!(list.len(), 1, "全量取代後應只剩 1 位");
    assert_eq!(list[0].reviewer_name, "委員C");
}

// ── 驗證：委員缺 id + 缺姓名 → Validation ──
#[tokio::test]
#[serial]
async fn historical_reviews_requires_identity() {
    let app = TestApp::spawn().await;
    let (protocol_id, _sd) = seed_imported_protocol(&app).await;
    let aid = draft_major(&app, protocol_id).await;

    let req = RecordHistoricalReviewsRequest {
        reviewers: vec![reviewer(None, None, "APPROVE")],
    };
    let err = AmendmentService::record_historical_reviews(&app.db_pool, &SYSTEM_TEST, aid, &req)
        .await
        .expect_err("缺身分應拒絕");
    assert!(
        matches!(err, AppError::Validation(_)),
        "應 Validation，實得：{err:?}"
    );
}

// ── finalize 後（非 DRAFT）不可再補審查 ──
#[tokio::test]
#[serial]
async fn historical_reviews_rejected_after_finalize() {
    let app = TestApp::spawn().await;
    let (protocol_id, _sd) = seed_imported_protocol(&app).await;
    let aid = draft_major(&app, protocol_id).await;
    AmendmentService::finalize_historical(
        &app.db_pool,
        &SYSTEM_TEST,
        aid,
        &FinalizeHistoricalAmendmentRequest {
            effective_from: None,
            remark: None,
        },
    )
    .await
    .expect("finalize");

    let req = RecordHistoricalReviewsRequest {
        reviewers: vec![reviewer(None, Some("委員X"), "APPROVE")],
    };
    let err = AmendmentService::record_historical_reviews(&app.db_pool, &SYSTEM_TEST, aid, &req)
        .await
        .expect_err("finalize 後不可補審查");
    assert!(
        matches!(err, AppError::BusinessRule(_)),
        "應 BusinessRule，實得：{err:?}"
    );
}

// ── access gate：限 SD / 管理者，且須為匯入計劃 ──
#[tokio::test]
#[serial]
async fn access_gate_sd_and_admin_only() {
    let app = TestApp::spawn().await;
    let (protocol_id, sd) = seed_imported_protocol(&app).await;
    let other = Uuid::new_v4();

    // SD 本人 → 允許
    assert!(
        access::can_backfill_historical_amendment(&app.db_pool, protocol_id, &cu(sd, &[]))
            .await
            .expect("check sd"),
        "計劃負責人 SD 應可補登"
    );
    // 真實管理者（SYSTEM_ADMIN 角色）→ 允許
    let admin = CurrentUser {
        id: other,
        email: "admin@test.local".into(),
        roles: vec!["SYSTEM_ADMIN".into()],
        permissions: vec![],
        jti: "t".into(),
        exp: 0,
        impersonated_by: None,
    };
    assert!(
        access::can_backfill_historical_amendment(&app.db_pool, protocol_id, &admin)
            .await
            .expect("check admin"),
        "管理者（SYSTEM_ADMIN）應可補登"
    );
    // 僅有 import_approved 權限、非 SD 非 admin → 拒絕（gate 已收斂為 SD/admin）
    assert!(
        !access::can_backfill_historical_amendment(
            &app.db_pool,
            protocol_id,
            &cu(other, &["aup.protocol.import_approved"])
        )
        .await
        .expect("check perm-only"),
        "僅有 import_approved 權限（非 SD 非 admin）應拒絕"
    );
    // 非 SD 非管理者 → 拒絕
    assert!(
        !access::can_backfill_historical_amendment(&app.db_pool, protocol_id, &cu(other, &[]))
            .await
            .expect("check other"),
        "非 SD 非管理者應拒絕"
    );

    // 非匯入計劃 → 即使是該計劃的 SD 也拒絕
    let unique = &Uuid::new_v4().to_string()[..8];
    let normal_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by, study_director_user_id)
           VALUES ($1, $2, $3, '一般計劃', 'APPROVED', $4, $4, $4)"#,
    )
    .bind(normal_id)
    .bind(format!("Pre-{unique}"))
    .bind(format!("PIG-{unique}"))
    .bind(sd)
    .execute(&app.db_pool)
    .await
    .expect("insert normal protocol");
    assert!(
        !access::can_backfill_historical_amendment(&app.db_pool, normal_id, &cu(sd, &[]))
            .await
            .expect("check normal"),
        "非匯入計劃不可補登歷史變更"
    );

    // import_pending 期間（匯入但未 finalize）→ 即使 SD 也拒絕（須完成補登後才開放）
    let pi2 = seed_pi(&app).await;
    let sd2 = seed_sd(&app).await;
    let u2 = &Uuid::new_v4().to_string()[..8];
    let pending = ProtocolService::import_approved(
        &app.db_pool,
        &SYSTEM_TEST,
        &import_req(pi2, sd2, &format!("PIG-{u2}")),
        pi2,
    )
    .await
    .expect("import pending");
    assert!(
        !access::can_backfill_historical_amendment(&app.db_pool, pending.id, &cu(sd2, &[]))
            .await
            .expect("check pending"),
        "補登中（import_pending）不可補登歷史變更"
    );
}
