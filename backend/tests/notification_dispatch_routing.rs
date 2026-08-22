//! P0 驗證：`document_submitted` 試點改走統一派送層 `dispatch_event` 後，行為須與原本一致。
//!
//! 同時實證 migration 110（notification_routing 加 target_kind/target_value）可套用於完整
//! migration 鏈，且既有角色型路由（WAREHOUSE_MANAGER）解析、站內通知建立、多規則去重皆正確。

use sqlx::PgPool;
use uuid::Uuid;

use erp_backend::services::NotificationService;

#[path = "common/test_db.rs"]
mod test_db;

async fn setup_pool() -> PgPool {
    // 10 = sqlx `PgPool::connect` 的預設池大小，明寫以保留原行為。
    let pool = test_db::connect_disposable(10).await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations on test db");
    pool
}

async fn seed_user(pool: &PgPool, label: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', $3, true, false)"#,
    )
    .bind(id)
    .bind(format!("{}-{}@test.local", label, &id.to_string()[..8]))
    .bind(format!("dispatch test {label}"))
    .execute(pool)
    .await
    .expect("seed user");
    id
}

async fn assign_role(pool: &PgPool, user_id: Uuid, role_code: &str) {
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE code = $2",
    )
    .bind(user_id)
    .bind(role_code)
    .execute(pool)
    .await
    .expect("assign role");
}

async fn count_notifications(pool: &PgPool, user_id: Uuid, entity_id: Uuid) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND related_entity_id = $2",
    )
    .bind(user_id)
    .bind(entity_id)
    .fetch_one(pool)
    .await
    .expect("count notifications")
}

/// 角色型路由：WAREHOUSE_MANAGER 透過 dispatcher 收到正好 1 則 document_submitted 站內通知。
#[tokio::test]
async fn document_submitted_routes_to_warehouse_manager_via_dispatcher() {
    let pool = setup_pool().await;
    let user = seed_user(&pool, "wm").await;
    assign_role(&pool, user, "WAREHOUSE_MANAGER").await;

    let doc_id = Uuid::new_v4();
    let svc = NotificationService::new(pool.clone());
    let count = svc
        .notify_document_submitted(doc_id, "PO-DISP-TEST", "PO", "tester")
        .await
        .expect("notify_document_submitted");

    assert!(count >= 1, "應至少建立 1 則站內通知，實得 {count}");

    let n = count_notifications(&pool, user, doc_id).await;
    assert_eq!(
        n, 1,
        "WAREHOUSE_MANAGER 應收到正好 1 則 document_submitted 站內通知"
    );

    let ty: String = sqlx::query_scalar(
        "SELECT type::TEXT FROM notifications WHERE user_id = $1 AND related_entity_id = $2",
    )
    .bind(user)
    .bind(doc_id)
    .fetch_one(&pool)
    .await
    .expect("fetch notification type");
    assert_eq!(ty, "document_approval", "型別應為 document_approval");
}

/// P1（migration 145 更新）：leave_submitted 透過 dispatcher 路由給「當前關卡」的審核人，
/// 不再是不分關卡的角色廣播（見 `hr_leave_approval_routing_precision.rs` 的完整覆蓋）。
/// 本測試鎖住終審關（PENDING_DIRECTOR）這一種情境：resolver `leave_current_stage_approvers`
/// 讀 `leave_requests` 目前狀態，只有真正在終審關待簽核時 DIRECTOR 才會收到通知。
#[tokio::test]
async fn leave_submitted_routes_to_director_via_dispatcher() {
    let pool = setup_pool().await;
    let applicant = seed_user(&pool, "applicant").await;
    let director = seed_user(&pool, "dir").await;
    assign_role(&pool, director, "DIRECTOR").await;

    let leave_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO leave_requests
               (id, user_id, leave_type, start_date, end_date, total_days, status)
           VALUES ($1, $2, 'ANNUAL', '2026-07-01', '2026-07-02', 2, 'PENDING_DIRECTOR')"#,
    )
    .bind(leave_id)
    .bind(applicant)
    .execute(&pool)
    .await
    .expect("seed leave request at PENDING_DIRECTOR");

    let svc = NotificationService::new(pool.clone());
    svc.notify_leave_submitted(leave_id, applicant, "特休", "2026-07-01", "2026-07-02")
        .await
        .expect("notify_leave_submitted");

    let n = count_notifications(&pool, director, leave_id).await;
    assert_eq!(
        n, 1,
        "終審關待簽核時，DIRECTOR 應收到正好 1 則 leave_submitted 站內通知"
    );
}

/// 去重：同一使用者因多條規則命中（WAREHOUSE_MANAGER + admin），dispatcher 以 user 去重 → 仍 1 則。
/// 同時驗證以新 target_kind/target_value 欄位插入 resolver-ready 的角色列可運作。
#[tokio::test]
async fn dispatcher_dedups_multi_rule_same_user() {
    let pool = setup_pool().await;
    let user = seed_user(&pool, "dedup").await;
    assign_role(&pool, user, "WAREHOUSE_MANAGER").await;
    assign_role(&pool, user, "admin").await;

    // 額外加一條 document_submitted → admin 角色型路由（顯式帶 target_kind/target_value）。
    sqlx::query(
        r#"INSERT INTO notification_routing
              (event_type, role_code, channel, description, frequency, target_kind, target_value)
           VALUES ('document_submitted', 'admin', 'in_app', '採購單提交(admin)', 'immediate', 'role', 'admin')
           ON CONFLICT DO NOTHING"#,
    )
    .execute(&pool)
    .await
    .expect("insert admin routing rule");

    let doc_id = Uuid::new_v4();
    let svc = NotificationService::new(pool.clone());
    svc.notify_document_submitted(doc_id, "PO-DEDUP", "PO", "tester")
        .await
        .expect("notify_document_submitted");

    let n = count_notifications(&pool, user, doc_id).await;
    assert_eq!(n, 1, "多規則命中同一使用者應去重為 1 則，實得 {n}");
}

/// P2a：review_comment_created 收件人＝IACUC_STAFF（角色）+ 計畫 PI（resolver protocol_pi_sd），
/// 全由 notification_routing 決定。首次實證 dispatcher 的 resolver 路徑。
#[tokio::test]
async fn review_comment_notifies_iacuc_staff_and_pi_via_resolver() {
    let pool = setup_pool().await;

    // IACUC_STAFF 角色收件人。
    let staff = seed_user(&pool, "staff").await;
    assign_role(&pool, staff, "IACUC_STAFF").await;

    // 計畫 + PI（user_protocols role_in_protocol='PI'）。
    let pi = seed_user(&pool, "pi").await;
    let protocol_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by) \
         VALUES ($1, $2, $3, '審查意見測試計畫', 'APPROVED', $4, $4)",
    )
    .bind(protocol_id)
    .bind(format!("P-RC-{}", &protocol_id.to_string()[..8]))
    .bind(format!("IACUC-RC-{}", &protocol_id.to_string()[..8]))
    .bind(pi)
    .execute(&pool)
    .await
    .expect("seed protocol");
    sqlx::query(
        "INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol) VALUES ($1, $2, 'PI')",
    )
    .bind(pi)
    .bind(protocol_id)
    .execute(&pool)
    .await
    .expect("seed user_protocols PI");

    let svc = NotificationService::new(pool.clone());
    svc.notify_review_comment_created(protocol_id, "P-RC", "審查委員乙", "意見內容")
        .await
        .expect("notify_review_comment_created");

    let pi_n = count_notifications(&pool, pi, protocol_id).await;
    assert_eq!(pi_n, 1, "PI 應透過 protocol_pi_sd resolver 收到正好 1 則");
    let staff_n = count_notifications(&pool, staff, protocol_id).await;
    assert_eq!(staff_n, 1, "IACUC_STAFF 應透過角色路由收到正好 1 則");
}

/// P2b：leave_approved 透過 resolver event_subject 通知申請人本人；停用帳號不通知（GDPR）。
#[tokio::test]
async fn leave_approved_notifies_active_applicant_only_via_resolver() {
    let pool = setup_pool().await;
    let svc = NotificationService::new(pool.clone());

    // active 申請人 → 收到 1 則。
    let applicant = seed_user(&pool, "appl").await;
    let leave_id = Uuid::new_v4();
    svc.notify_leave_approved(leave_id, applicant, "特休", "2026-07-01", "2026-07-02")
        .await
        .expect("notify_leave_approved");
    assert_eq!(
        count_notifications(&pool, applicant, leave_id).await,
        1,
        "active 申請人應透過 event_subject resolver 收到 1 則"
    );

    // inactive 申請人 → 不收到（resolver 的 active 過濾）。
    let inactive = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'inactive applicant', false, false)"#,
    )
    .bind(inactive)
    .bind(format!("inact-{}@test.local", &inactive.to_string()[..8]))
    .execute(&pool)
    .await
    .expect("seed inactive user");
    let leave_id2 = Uuid::new_v4();
    svc.notify_leave_approved(leave_id2, inactive, "特休", "2026-07-03", "2026-07-04")
        .await
        .expect("notify_leave_approved inactive");
    assert_eq!(
        count_notifications(&pool, inactive, leave_id2).await,
        0,
        "停用申請人不應收到通知"
    );
}

/// P2b：migration 112 移除 leave_cancelled 的死角色列（ADMIN_STAFF/admin），改為 approvers resolver。
/// 確保取消通知不再擴散給全體 admin。
#[tokio::test]
async fn leave_cancelled_role_rows_removed_resolver_added() {
    let pool = setup_pool().await;

    let role_rows: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notification_routing \
         WHERE event_type = 'leave_cancelled' AND target_kind = 'role'",
    )
    .fetch_one(&pool)
    .await
    .expect("count role rows");
    assert_eq!(
        role_rows, 0,
        "leave_cancelled 不應再有角色型路由列（死列已移除）"
    );

    let resolver_rows: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notification_routing \
         WHERE event_type = 'leave_cancelled' AND target_kind = 'resolver' \
           AND target_value = 'leave_request_approvers'",
    )
    .fetch_one(&pool)
    .await
    .expect("count resolver rows");
    assert_eq!(
        resolver_rows, 1,
        "leave_cancelled 應有 approvers resolver 列"
    );
}

/// P2b-animal：animal_sudden_death 啟動——透過 dispatcher 路由給 VET（原本完全不發通知）。
#[tokio::test]
async fn sudden_death_notifies_vet_via_dispatcher() {
    let pool = setup_pool().await;
    let vet = seed_user(&pool, "vet").await;
    assign_role(&pool, vet, "VET").await;

    let animal_id = Uuid::new_v4();
    let svc = NotificationService::new(pool.clone());
    svc.notify_sudden_death(animal_id, "PIG-SD", Some("IACUC-SD"), "reporter@test", None)
        .await
        .expect("notify_sudden_death");

    assert_eq!(
        count_notifications(&pool, vet, animal_id).await,
        1,
        "VET 應收到 1 則 animal_sudden_death 站內通知"
    );
}

/// P2b-animal：emergency_medication 透過 dispatcher 通知 VET（角色）+ 計畫 PI（resolver protocol_pi）。
/// 順帶驗證原 up.role bug 修復（PI 改走正確的 role_in_protocol 欄位）。
#[tokio::test]
async fn emergency_medication_notifies_vet_and_pi_via_dispatcher() {
    let pool = setup_pool().await;
    let vet = seed_user(&pool, "evet").await;
    assign_role(&pool, vet, "VET").await;

    let pi = seed_user(&pool, "epi").await;
    let protocol_id = Uuid::new_v4();
    let iacuc = format!("IACUC-EM-{}", &protocol_id.to_string()[..8]);
    sqlx::query(
        "INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, created_by) \
         VALUES ($1, $2, $3, '緊急給藥測試計畫', 'APPROVED', $4, $4)",
    )
    .bind(protocol_id)
    .bind(format!("P-EM-{}", &protocol_id.to_string()[..8]))
    .bind(&iacuc)
    .bind(pi)
    .execute(&pool)
    .await
    .expect("seed protocol");
    sqlx::query(
        "INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol) VALUES ($1, $2, 'PI')",
    )
    .bind(pi)
    .bind(protocol_id)
    .execute(&pool)
    .await
    .expect("seed PI");

    let animal_id = Uuid::new_v4();
    let svc = NotificationService::new(pool.clone());
    svc.notify_emergency_medication(
        animal_id,
        Uuid::new_v4(),
        "PIG-EM",
        Some(&iacuc),
        "操作員",
        "獸醫不在",
    )
    .await
    .expect("notify_emergency_medication");

    assert_eq!(
        count_notifications(&pool, vet, animal_id).await,
        1,
        "VET 應收到 1 則緊急給藥通知"
    );
    assert_eq!(
        count_notifications(&pool, pi, animal_id).await,
        1,
        "PI 應透過 protocol_pi resolver 收到 1 則（修復原 up.role bug）"
    );
}

/// P4a：routing_recipients_preview 列出角色型實際成員 + resolver 型說明且標不可調。
#[tokio::test]
async fn recipients_preview_lists_role_members_and_resolver_meta() {
    let pool = setup_pool().await;
    let vet = seed_user(&pool, "pvet").await;
    assign_role(&pool, vet, "VET").await;

    let svc = NotificationService::new(pool.clone());
    let preview = svc.routing_recipients_preview().await.expect("preview");

    // animal_abnormal_record → 角色 VET，成員應含剛建的 vet。
    let abnormal = preview
        .iter()
        .find(|p| p.event_type == "animal_abnormal_record")
        .expect("應有 animal_abnormal_record");
    let vet_rule = abnormal
        .rules
        .iter()
        .find(|r| r.target_kind == "role" && r.target_value == "VET")
        .expect("應有 VET 角色規則");
    assert!(!vet_rule.fixed, "角色型規則 fixed 應為 false");
    assert!(
        vet_rule.members.iter().any(|m| m.id == vet),
        "VET 成員應含剛建使用者"
    );

    // review_comment_created → resolver protocol_pi_sd，應標 fixed + 有說明 + 不列成員。
    let rc = preview
        .iter()
        .find(|p| p.event_type == "review_comment_created")
        .expect("應有 review_comment_created");
    let resolver_rule = rc
        .rules
        .iter()
        .find(|r| r.target_kind == "resolver" && r.target_value == "protocol_pi_sd")
        .expect("應有 protocol_pi_sd resolver 規則");
    assert!(resolver_rule.fixed, "resolver 型應標 fixed");
    assert!(resolver_rule.description.is_some(), "resolver 型應有說明");
    assert!(resolver_rule.members.is_empty(), "resolver 型不列舉成員");
}

/// P3：channel='email' 的路由規則經 dispatch_event 寄信——入 event_outbox 且不建站內通知。
#[tokio::test]
async fn dispatch_email_channel_enqueues_outbox_email() {
    let pool = setup_pool().await;
    // 全域 app_url（OnceLock，整個 test process 設一次即可）。
    erp_backend::services::init_notification_app_url("http://test.local".to_string());

    let user = seed_user(&pool, "otwm").await;
    assign_role(&pool, user, "ADMIN_STAFF").await;

    // 將 overtime_submitted 改為 email channel（其他測試未斷言此事件）；測後還原避免洩漏到其他測試。
    sqlx::query(
        "UPDATE notification_routing SET channel='email' WHERE event_type='overtime_submitted'",
    )
    .execute(&pool)
    .await
    .expect("set email channel");

    let ot_id = Uuid::new_v4();
    let svc = NotificationService::new(pool.clone());
    svc.notify_overtime_submitted(ot_id, "申請人甲", "2026-07-01", 2.0)
        .await
        .expect("notify_overtime_submitted");

    // 還原 channel，避免共用 DB 下洩漏到其他測試。
    sqlx::query(
        "UPDATE notification_routing SET channel='in_app' WHERE event_type='overtime_submitted'",
    )
    .execute(&pool)
    .await
    .expect("restore channel");

    // email channel → 不建站內通知。
    assert_eq!(
        count_notifications(&pool, user, ot_id).await,
        0,
        "email channel 不應建站內通知"
    );

    // email 排入 event_outbox（>=1：可能有其他 ADMIN_STAFF 測試使用者一併入列）。
    let outbox: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM event_outbox \
         WHERE channel='email' AND source_entity='overtime_record' AND source_entity_id=$1",
    )
    .bind(ot_id)
    .fetch_one(&pool)
    .await
    .expect("count outbox");
    assert!(outbox >= 1, "應至少 1 封 email 排入 outbox，實得 {outbox}");
}

/// 收斂：stock_email_recipients 來源同站內（notification_routing 角色），含 PURCHASING；
/// 並受個人偏好 notification_settings.email_low_stock 過濾。
#[tokio::test]
async fn stock_email_recipients_from_routing_with_pref_filter() {
    let pool = setup_pool().await;
    let purchasing = seed_user(&pool, "purch").await;
    assign_role(&pool, purchasing, "PURCHASING").await;

    let svc = NotificationService::new(pool.clone());
    let recipients = svc
        .stock_email_recipients("low_stock_alert")
        .await
        .expect("stock_email_recipients");
    assert!(
        recipients.iter().any(|(id, _, _)| *id == purchasing),
        "PURCHASING 使用者應在低庫存 email 收件人中（路由表來源，與站內一致）"
    );

    // 個人 email 偏好關閉 → 不在收件人中。
    sqlx::query("UPDATE notification_settings SET email_low_stock = false WHERE user_id = $1")
        .bind(purchasing)
        .execute(&pool)
        .await
        .expect("disable pref");
    let recipients2 = svc
        .stock_email_recipients("low_stock_alert")
        .await
        .expect("stock_email_recipients 2");
    assert!(
        !recipients2.iter().any(|(id, _, _)| *id == purchasing),
        "個人 email 偏好關閉的使用者不應在收件人中"
    );

    // 軟刪除帳號不應在收件人中（GDPR）。
    let deleted = seed_user(&pool, "delpurch").await;
    assign_role(&pool, deleted, "PURCHASING").await;
    sqlx::query("UPDATE users SET deleted_at = NOW() WHERE id = $1")
        .bind(deleted)
        .execute(&pool)
        .await
        .expect("soft delete user");
    let recipients3 = svc
        .stock_email_recipients("low_stock_alert")
        .await
        .expect("stock_email_recipients 3");
    assert!(
        !recipients3.iter().any(|(id, _, _)| *id == deleted),
        "軟刪除帳號不應在收件人中"
    );
}

/// 審核結果回通知：設備報廢核准 / 駁回都透過 resolver event_subject 通知申請人本人。
#[tokio::test]
async fn equipment_disposal_result_notifies_applicant_via_resolver() {
    let pool = setup_pool().await;
    let svc = NotificationService::new(pool.clone());
    let applicant = seed_user(&pool, "disp-appl").await;

    // 核准 → 申請人收到 1 則（型別 system_alert，與既有設備通知一致）。
    let disposal_id = Uuid::new_v4();
    let n = svc
        .notify_equipment_disposal_result(disposal_id, applicant, Uuid::new_v4(), true, None)
        .await
        .expect("notify disposal approved");
    assert!(n >= 1, "核准應至少建立 1 則站內通知，實得 {n}");
    assert_eq!(
        count_notifications(&pool, applicant, disposal_id).await,
        1,
        "申請人應透過 event_subject 收到正好 1 則報廢核准通知"
    );
    let ty: String = sqlx::query_scalar(
        "SELECT type::TEXT FROM notifications WHERE user_id = $1 AND related_entity_id = $2",
    )
    .bind(applicant)
    .bind(disposal_id)
    .fetch_one(&pool)
    .await
    .expect("fetch type");
    assert_eq!(ty, "system_alert", "設備通知型別應為 system_alert");

    // 駁回（另一張單，含原因）→ 申請人收到 1 則。
    let disposal_id2 = Uuid::new_v4();
    svc.notify_equipment_disposal_result(
        disposal_id2,
        applicant,
        Uuid::new_v4(),
        false,
        Some("設備仍堪用"),
    )
    .await
    .expect("notify disposal rejected");
    assert_eq!(
        count_notifications(&pool, applicant, disposal_id2).await,
        1,
        "申請人應收到正好 1 則報廢駁回通知"
    );
}

/// 審核結果回通知：設備維修驗收通過 / 退回都透過 resolver event_subject 通知登錄人本人。
#[tokio::test]
async fn equipment_maintenance_result_notifies_reporter_via_resolver() {
    let pool = setup_pool().await;
    let svc = NotificationService::new(pool.clone());
    let reporter = seed_user(&pool, "maint-rep").await;

    let rec_id = Uuid::new_v4();
    svc.notify_equipment_maintenance_result(rec_id, reporter, Uuid::new_v4(), true, None)
        .await
        .expect("notify maintenance approved");
    assert_eq!(
        count_notifications(&pool, reporter, rec_id).await,
        1,
        "登錄人應透過 event_subject 收到正好 1 則驗收通過通知"
    );

    let rec_id2 = Uuid::new_v4();
    svc.notify_equipment_maintenance_result(
        rec_id2,
        reporter,
        Uuid::new_v4(),
        false,
        Some("需補附照片"),
    )
    .await
    .expect("notify maintenance rejected");
    assert_eq!(
        count_notifications(&pool, reporter, rec_id2).await,
        1,
        "登錄人應收到正好 1 則驗收退回通知"
    );
}

/// 審核結果回通知：請假駁回透過 resolver event_subject 通知申請人本人；停用帳號不通知（GDPR）。
#[tokio::test]
async fn leave_rejected_notifies_active_applicant_only_via_resolver() {
    let pool = setup_pool().await;
    let svc = NotificationService::new(pool.clone());

    // active 申請人 → 收到 1 則（型別 leave_approval）。
    let applicant = seed_user(&pool, "rej-appl").await;
    let leave_id = Uuid::new_v4();
    svc.notify_leave_rejected(
        leave_id,
        applicant,
        "特休",
        "2026-07-01",
        "2026-07-02",
        Some("人力吃緊"),
    )
    .await
    .expect("notify_leave_rejected");
    assert_eq!(
        count_notifications(&pool, applicant, leave_id).await,
        1,
        "active 申請人應透過 event_subject 收到 1 則駁回通知"
    );
    let ty: String = sqlx::query_scalar(
        "SELECT type::TEXT FROM notifications WHERE user_id = $1 AND related_entity_id = $2",
    )
    .bind(applicant)
    .bind(leave_id)
    .fetch_one(&pool)
    .await
    .expect("fetch type");
    assert_eq!(ty, "leave_approval", "請假通知型別應為 leave_approval");

    // inactive 申請人 → 不收到（resolver 的 active 過濾，GDPR）。
    let inactive = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active, must_change_password)
           VALUES ($1, $2, 'fake', 'inactive rej applicant', false, false)"#,
    )
    .bind(inactive)
    .bind(format!("inact-rej-{}@test.local", &inactive.to_string()[..8]))
    .execute(&pool)
    .await
    .expect("seed inactive user");
    let leave_id2 = Uuid::new_v4();
    svc.notify_leave_rejected(
        leave_id2,
        inactive,
        "特休",
        "2026-07-03",
        "2026-07-04",
        None,
    )
    .await
    .expect("notify_leave_rejected inactive");
    assert_eq!(
        count_notifications(&pool, inactive, leave_id2).await,
        0,
        "停用申請人不應收到駁回通知"
    );
}

/// P4c：固定通知目錄含安樂死流程（供路由頁唯讀 notes 顯示）。靜態目錄，不需 DB。
#[tokio::test]
async fn fixed_notifications_catalog_includes_euthanasia() {
    let fixed = erp_backend::services::NotificationService::list_fixed_notifications();
    assert!(
        fixed.len() >= 4,
        "至少含 4 條安樂死流程通知，實得 {}",
        fixed.len()
    );
    assert!(
        fixed.iter().any(|f| f.event_label.contains("安樂死單開立")),
        "應含『安樂死單開立』"
    );
    assert!(
        fixed
            .iter()
            .all(|f| !f.recipient_label.is_empty() && !f.trigger.is_empty()),
        "每筆固定通知都應有收件人與觸發說明"
    );
}
