//! #61 回歸：銷貨 / 出庫（SO / DO）開立授權以「該計畫層級 SD」判斷。
//!
//! Bug：舊版守門寫在 handler，只查全域 `STUDY_DIRECTOR`（GLP）角色。但計畫層級 SD
//! （`protocols.study_director_user_id`）是自 `EXPERIMENT_STAFF` 挑選、通常**不具**該全域角色，
//! 導致計畫負責人（如 PIG-115006 的使用者D）替自己負責的計畫開銷貨反被「僅 SD 或管理員」擋下。
//!
//! 修復後（service 層 single source of truth）：
//! - 該計畫的 `study_director_user_id` 使用者 → 可開立（即使只有 EXPERIMENT_STAFF 角色）。
//! - 非該計畫 SD、且無全域 STUDY_DIRECTOR / admin → Forbidden。

mod common;

use chrono::Utc;
use common::TestApp;
use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::models::{CreateDocumentRequest, DocType, DocumentLineInput};
use erp_backend::services::DocumentService;
use rust_decimal::Decimal;
use serial_test::serial;
use uuid::Uuid;

/// 建一個只有 EXPERIMENT_STAFF 角色的內部人員（非 admin、非全域 STUDY_DIRECTOR）。
fn experiment_staff(id: Uuid, email: &str) -> CurrentUser {
    CurrentUser {
        id,
        email: email.into(),
        roles: vec!["EXPERIMENT_STAFF".into()],
        permissions: vec![],
        jti: "test".into(),
        exp: 0,
        impersonated_by: None,
    }
}

async fn seed_user(app: &TestApp, tag: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, email, password_hash, display_name, is_active)
           VALUES ($1, $2, 'x', $3, true)"#,
    )
    .bind(id)
    .bind(format!(
        "{}-{}@test.local",
        tag,
        &Uuid::new_v4().to_string()[..8]
    ))
    .bind(format!("sd-authz-{tag}"))
    .execute(&app.db_pool)
    .await
    .expect("insert user");
    id
}

/// 建一個「已核准」計畫，指定其計畫層級 SD（study_director_user_id）。
async fn seed_approved_protocol(app: &TestApp, sd_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    let suffix = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO protocols (id, protocol_no, title, status, pi_user_id, study_director_user_id, iacuc_no, created_by) \
         VALUES ($1, $2, '銷貨授權測試計畫', 'APPROVED', $3, $3, $4, $3)",
    )
    .bind(id)
    .bind(format!("P-SD-{}", &suffix[..10]))
    .bind(sd_id)
    .bind(format!("IACUC-{}", &suffix[..8]))
    .execute(&app.db_pool)
    .await
    .expect("insert approved protocol");
    id
}

async fn seed_warehouse(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO warehouses (id, code, name) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(format!("WH-{}", &id.to_string()[..8]))
        .bind("測試倉")
        .execute(&app.db_pool)
        .await
        .expect("insert warehouse");
    id
}

async fn seed_shelf(app: &TestApp, warehouse_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO storage_locations (id, warehouse_id, code) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(warehouse_id)
        .bind(format!("SL-{}", &id.to_string()[..8]))
        .execute(&app.db_pool)
        .await
        .expect("insert storage location");
    id
}

async fn seed_product(app: &TestApp) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO products (id, sku, name, base_uom) VALUES ($1, $2, $3, '支')")
        .bind(id)
        .bind(format!("SKU-{}", &id.to_string()[..8]))
        .bind("銷貨授權測試品")
        .execute(&app.db_pool)
        .await
        .expect("insert product");
    id
}

fn so_request(
    protocol_id: Option<Uuid>,
    warehouse_id: Uuid,
    shelf_id: Uuid,
    product_id: Uuid,
) -> CreateDocumentRequest {
    CreateDocumentRequest {
        doc_type: DocType::SO,
        warehouse_id: Some(warehouse_id),
        warehouse_from_id: None,
        warehouse_to_id: None,
        partner_id: None,
        source_doc_id: None,
        doc_date: Utc::now().date_naive(),
        remark: None,
        stocktake_scope: None,
        iacuc_no: None,
        protocol_id,
        lines: vec![DocumentLineInput {
            product_id,
            qty: Decimal::ONE,
            uom: "支".into(),
            // 內部銷貨不記單價（2026-07-21 裁定）：SO 拒收 unit_price。
            unit_price: None,
            batch_no: None,
            expiry_date: None,
            remark: None,
            storage_location_id: Some(shelf_id),
            storage_location_from_id: None,
            storage_location_to_id: None,
        }],
    }
}

/// 核心回歸：該計畫層級 SD（僅 EXPERIMENT_STAFF 角色）可替自己負責的計畫開立銷貨單。
/// 修復前：被 handler 全域角色守門擋成 Forbidden。
#[tokio::test]
#[serial]
async fn plan_study_director_can_create_sales_order() {
    let app = TestApp::spawn().await;
    let sd_id = seed_user(&app, "owner").await;
    let protocol_id = seed_approved_protocol(&app, sd_id).await;
    let warehouse_id = seed_warehouse(&app).await;
    let shelf_id = seed_shelf(&app, warehouse_id).await;
    let product_id = seed_product(&app).await;

    let actor = ActorContext::User(experiment_staff(sd_id, "owner@test.local"));
    let req = so_request(Some(protocol_id), warehouse_id, shelf_id, product_id);

    let created = DocumentService::create(&app.db_pool, &actor, &req)
        .await
        .expect("計畫層級 SD 應可開立銷貨單");
    assert_eq!(created.document.doc_type, DocType::SO);
    assert_eq!(created.document.protocol_id, Some(protocol_id));
}

/// 反例：非該計畫 SD、無全域 STUDY_DIRECTOR / admin → 仍被擋（#61 防掛任意計畫）。
#[tokio::test]
#[serial]
async fn non_sd_staff_cannot_create_sales_order() {
    let app = TestApp::spawn().await;
    let sd_id = seed_user(&app, "owner").await;
    let outsider_id = seed_user(&app, "outsider").await;
    let protocol_id = seed_approved_protocol(&app, sd_id).await;
    let warehouse_id = seed_warehouse(&app).await;
    let shelf_id = seed_shelf(&app, warehouse_id).await;
    let product_id = seed_product(&app).await;

    let actor = ActorContext::User(experiment_staff(outsider_id, "outsider@test.local"));
    let req = so_request(Some(protocol_id), warehouse_id, shelf_id, product_id);

    let err = DocumentService::create(&app.db_pool, &actor, &req)
        .await
        .expect_err("非該計畫 SD 不應可開立銷貨單");
    assert!(
        matches!(err, erp_backend::AppError::Forbidden(_)),
        "應回 Forbidden，實際：{err:?}"
    );
    assert!(
        err.to_string().contains("開立銷貨"),
        "錯誤訊息應說明銷貨開立授權，實際：{err}"
    );
}

/// 反例（IDOR 防護）：計畫層級 SD 不應可把自己的銷貨單 update 改指到「非自己負責」的計畫。
/// 防止繞過 create 授權——透過 update 把 protocol_id 換成未授權計畫。應回 Forbidden（非 NotFound）。
#[tokio::test]
#[serial]
async fn plan_sd_cannot_repoint_sales_order_to_unauthorized_protocol() {
    let app = TestApp::spawn().await;
    let sd_id = seed_user(&app, "owner").await;
    let outsider_id = seed_user(&app, "outsider").await;
    let protocol_a = seed_approved_protocol(&app, sd_id).await;
    let protocol_b = seed_approved_protocol(&app, outsider_id).await;
    let warehouse_id = seed_warehouse(&app).await;
    let shelf_id = seed_shelf(&app, warehouse_id).await;
    let product_id = seed_product(&app).await;

    let actor = ActorContext::User(experiment_staff(sd_id, "owner@test.local"));

    // A 計畫的 SD 先合法開立自己計畫的銷貨單。
    let created = DocumentService::create(
        &app.db_pool,
        &actor,
        &so_request(Some(protocol_a), warehouse_id, shelf_id, product_id),
    )
    .await
    .expect("計畫層級 SD 應可開立自己計畫的銷貨單");

    // 嘗試把 protocol_id 改指到 B 計畫（其 SD 為 outsider）→ 應被擋（IDOR 繞過）。
    let update_req = erp_backend::models::UpdateDocumentRequest {
        warehouse_id: None,
        warehouse_from_id: None,
        warehouse_to_id: None,
        partner_id: None,
        protocol_id: Some(protocol_b),
        source_doc_id: None,
        doc_date: None,
        remark: None,
        lines: None,
    };
    let err = DocumentService::update(&app.db_pool, &actor, created.document.id, &update_req)
        .await
        .expect_err("不應允許 update 改指到未授權計畫");
    assert!(
        matches!(err, erp_backend::AppError::Forbidden(_)),
        "應回 Forbidden，實際：{err:?}"
    );

    // 反向確認：改指回自己負責的 A 計畫應可通過授權（不因此測試而誤擋合法編輯）。
    let ok_req = erp_backend::models::UpdateDocumentRequest {
        protocol_id: Some(protocol_a),
        ..update_req
    };
    DocumentService::update(&app.db_pool, &actor, created.document.id, &ok_req)
        .await
        .expect("改指回自己負責的計畫應可通過授權");
}

/// 2026-07-20 裁定放寬：任一「已核准」計畫的計畫層級 SD 可開立「無計畫」外部客戶銷貨單
/// （protocol_id = None），非僅 admin / 全域 STUDY_DIRECTOR。
#[tokio::test]
#[serial]
async fn plan_sd_can_create_no_protocol_sales_order() {
    let app = TestApp::spawn().await;
    let sd_id = seed_user(&app, "owner").await;
    let _protocol_id = seed_approved_protocol(&app, sd_id).await;
    let warehouse_id = seed_warehouse(&app).await;
    let shelf_id = seed_shelf(&app, warehouse_id).await;
    let product_id = seed_product(&app).await;

    let actor = ActorContext::User(experiment_staff(sd_id, "owner@test.local"));
    let req = so_request(None, warehouse_id, shelf_id, product_id);

    let created = DocumentService::create(&app.db_pool, &actor, &req)
        .await
        .expect("已核准計畫的計畫層級 SD 應可開立無計畫（外部客戶）銷貨單");
    assert_eq!(created.document.protocol_id, None);
}

/// 回歸（CodeRabbit #1005 Major）：無計畫（外部客戶）銷貨單的 **update** 也必須過 SD 授權。
/// 修復前 `req.protocol_id.or(existing.protocol_id)` 兩值皆 None 時整個授權被跳過，
/// 任何具編輯權限者都能改無計畫銷貨草稿。
#[tokio::test]
#[serial]
async fn no_protocol_sales_order_update_requires_sd_authorization() {
    let app = TestApp::spawn().await;
    let sd_id = seed_user(&app, "owner").await;
    let outsider_id = seed_user(&app, "outsider").await;
    let _protocol = seed_approved_protocol(&app, sd_id).await;
    let warehouse_id = seed_warehouse(&app).await;
    let shelf_id = seed_shelf(&app, warehouse_id).await;
    let product_id = seed_product(&app).await;

    let sd_actor = ActorContext::User(experiment_staff(sd_id, "owner@test.local"));
    let created = DocumentService::create(
        &app.db_pool,
        &sd_actor,
        &so_request(None, warehouse_id, shelf_id, product_id),
    )
    .await
    .expect("計畫層級 SD 開立無計畫銷貨單");

    let update_req = erp_backend::models::UpdateDocumentRequest {
        warehouse_id: None,
        warehouse_from_id: None,
        warehouse_to_id: None,
        partner_id: None,
        protocol_id: None,
        source_doc_id: None,
        doc_date: None,
        remark: Some("嘗試改單".into()),
        lines: None,
    };

    // 反例：非任何計畫 SD → Forbidden（修復前這裡會靜默放行）。
    let outsider_actor = ActorContext::User(experiment_staff(outsider_id, "outsider@test.local"));
    let err = DocumentService::update(
        &app.db_pool,
        &outsider_actor,
        created.document.id,
        &update_req,
    )
    .await
    .expect_err("非任何計畫 SD 不應可編輯無計畫銷貨草稿");
    assert!(
        matches!(err, erp_backend::AppError::Forbidden(_)),
        "應回 Forbidden，實際：{err:?}"
    );

    // 正向：任一已核准計畫的計畫層級 SD 仍可編輯（授權閘不誤擋）。
    DocumentService::update(&app.db_pool, &sd_actor, created.document.id, &update_req)
        .await
        .expect("計畫層級 SD 應可編輯無計畫銷貨草稿");
}

/// 反例：完全不是任何計畫 SD 的內部人員，仍不可開立無計畫銷貨單（放寬不外溢）。
#[tokio::test]
#[serial]
async fn non_sd_staff_cannot_create_no_protocol_sales_order() {
    let app = TestApp::spawn().await;
    let outsider_id = seed_user(&app, "outsider").await;
    let warehouse_id = seed_warehouse(&app).await;
    let shelf_id = seed_shelf(&app, warehouse_id).await;
    let product_id = seed_product(&app).await;

    let actor = ActorContext::User(experiment_staff(outsider_id, "outsider@test.local"));
    let req = so_request(None, warehouse_id, shelf_id, product_id);

    let err = DocumentService::create(&app.db_pool, &actor, &req)
        .await
        .expect_err("非任何計畫 SD 不應可開立無計畫銷貨單");
    assert!(
        matches!(err, erp_backend::AppError::Forbidden(_)),
        "應回 Forbidden，實際：{err:?}"
    );
}

/// SO 銷貨計畫下拉 SD 收斂（2026-07-22 裁定）：`get_my_protocols(sd_only=true)`
/// 只回「自己是 `study_director_user_id`」的計畫；僅為成員（非 SD）的計畫被濾除，
/// 避免下拉列出選了也會被 `authorize_sales_document` 擋下的計畫。
#[tokio::test]
#[serial]
async fn sd_only_protocol_list_excludes_member_only_protocols() {
    let app = TestApp::spawn().await;
    let sd_id = seed_user(&app, "sdonly-sd").await;
    let other_sd_id = seed_user(&app, "sdonly-other").await;

    // 計畫 A：自己是 SD；計畫 B：他人是 SD、自己僅為成員（CLIENT）。
    let own_protocol = seed_approved_protocol(&app, sd_id).await;
    let member_protocol = seed_approved_protocol(&app, other_sd_id).await;
    sqlx::query(
        r#"INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol)
           VALUES ($1, $2, 'CLIENT')"#,
    )
    .bind(sd_id)
    .bind(member_protocol)
    .execute(&app.db_pool)
    .await
    .expect("insert user_protocols membership");

    // 對齊舊測試語義：assignable=true（seeded 皆 APPROVED + iacuc_no），其餘過濾預設不套。
    let assignable_q = erp_backend::models::ProtocolQuery {
        assignable: true,
        ..Default::default()
    };

    // 未收斂：成員制範圍含兩案（既有行為不變）。
    let all = erp_backend::services::ProtocolService::get_my_protocols(
        &app.db_pool,
        sd_id,
        &assignable_q,
        false,
    )
    .await
    .expect("get_my_protocols sd_only=false");
    let ids: Vec<Uuid> = all.iter().map(|p| p.id).collect();
    assert!(
        ids.contains(&own_protocol),
        "sd_only=false 應含自己是 SD 的計畫"
    );
    assert!(
        ids.contains(&member_protocol),
        "sd_only=false 應含僅為成員的計畫"
    );

    // 收斂：只剩自己是 SD 的計畫。
    let sd_only = erp_backend::services::ProtocolService::get_my_protocols(
        &app.db_pool,
        sd_id,
        &assignable_q,
        true,
    )
    .await
    .expect("get_my_protocols sd_only=true");
    let ids: Vec<Uuid> = sd_only.iter().map(|p| p.id).collect();
    assert!(
        ids.contains(&own_protocol),
        "sd_only=true 應含自己是 SD 的計畫"
    );
    assert!(
        !ids.contains(&member_protocol),
        "sd_only=true 不應含僅為成員的計畫"
    );
}

/// #1048 回歸：`build_list_sql` 產生 `start_date` / `end_date` 佔位符後，`list` 必須綁定
/// 對應參數；缺綁定時帶日期過濾的管理端查詢會因未綁定參數噴 sqlx 錯誤（500）。
#[tokio::test]
#[serial]
async fn admin_list_with_date_filters_binds_all_params() {
    let app = TestApp::spawn().await;
    let admin_id = seed_user(&app, "datefilter-admin").await;
    let _p = seed_approved_protocol(&app, admin_id).await;

    let query = erp_backend::models::ProtocolQuery {
        start_date: Some(chrono::NaiveDate::from_ymd_opt(2020, 1, 1).expect("valid date")),
        end_date: Some(chrono::NaiveDate::from_ymd_opt(2030, 12, 31).expect("valid date")),
        ..Default::default()
    };
    // is_admin=true、viewer_sees_all_drafts=true 走 build_list_sql；日期佔位符須全數綁定。
    let res =
        erp_backend::services::ProtocolService::list(&app.db_pool, &query, admin_id, true, true)
            .await;
    assert!(
        res.is_ok(),
        "帶 start_date/end_date 的 list 應成功綁定所有參數，實得：{res:?}"
    );
}
