//! 計畫書範本版本登記（院區層級，admin 管理）回歸測試。
//! CRUD + 「現行版本」唯一性（set_current 清除其他）+ Anonymous 拒絕。

mod common;

use common::TestApp;
use serial_test::serial;

use erp_backend::middleware::ActorContext;
use erp_backend::models::{
    CreateProtocolTemplateVersionRequest, UpdateProtocolTemplateVersionRequest,
};
use erp_backend::services::ProtocolTemplateVersionService;
use erp_backend::AppError;

const SYSTEM_TEST: ActorContext = ActorContext::System {
    reason: "template_version_test",
};

fn create_req(label: &str) -> CreateProtocolTemplateVersionRequest {
    CreateProtocolTemplateVersionRequest {
        version_label: label.to_string(),
        effective_date: None,
        notes: Some("測試備註".to_string()),
    }
}

#[tokio::test]
#[serial]
async fn template_version_crud_and_single_current() {
    let app = TestApp::spawn().await;
    // version_label 唯一，避免跨次測試殘留資料撞 UNIQUE
    let uniq = &uuid::Uuid::new_v4().to_string()[..8];

    // 建立兩個版本
    let v1 = ProtocolTemplateVersionService::create(
        &app.db_pool,
        &SYSTEM_TEST,
        &create_req(&format!("v1.0-{uniq}")),
    )
    .await
    .expect("create v1");
    assert!(!v1.is_current, "新建版本預設非現行");
    let v2 = ProtocolTemplateVersionService::create(
        &app.db_pool,
        &SYSTEM_TEST,
        &create_req(&format!("v2.0-{uniq}")),
    )
    .await
    .expect("create v2");

    // 列表含兩者
    let list = ProtocolTemplateVersionService::list(&app.db_pool)
        .await
        .expect("list");
    assert!(list.len() >= 2);

    // 設 v1 為現行
    let v1c = ProtocolTemplateVersionService::set_current(&app.db_pool, &SYSTEM_TEST, v1.id)
        .await
        .expect("set current v1");
    assert!(v1c.is_current);

    // 設 v2 為現行 → v1 應被清除（唯一現行）
    ProtocolTemplateVersionService::set_current(&app.db_pool, &SYSTEM_TEST, v2.id)
        .await
        .expect("set current v2");
    let v1_after = ProtocolTemplateVersionService::get(&app.db_pool, v1.id)
        .await
        .expect("get v1");
    let v2_after = ProtocolTemplateVersionService::get(&app.db_pool, v2.id)
        .await
        .expect("get v2");
    assert!(!v1_after.is_current, "設定新現行版本後，舊現行應被清除");
    assert!(v2_after.is_current, "v2 應為現行");

    // 更新 v1 版本號 + 備註
    let updated = ProtocolTemplateVersionService::update(
        &app.db_pool,
        &SYSTEM_TEST,
        v1.id,
        &UpdateProtocolTemplateVersionRequest {
            version_label: Some(format!("v1.1-{uniq}")),
            effective_date: None,
            notes: Some("更新後備註".to_string()),
        },
    )
    .await
    .expect("update v1");
    assert_eq!(updated.version_label, format!("v1.1-{uniq}"));

    // 刪除 v1
    ProtocolTemplateVersionService::delete(&app.db_pool, &SYSTEM_TEST, v1.id)
        .await
        .expect("delete v1");
    let err = ProtocolTemplateVersionService::get(&app.db_pool, v1.id)
        .await
        .expect_err("已刪除應找不到");
    assert!(
        matches!(err, AppError::NotFound(_)),
        "應 NotFound，實得：{err:?}"
    );
}

// 刪除版本應一併清除其附件 DB 列（避免孤兒；CodeRabbit #539）
#[tokio::test]
#[serial]
async fn delete_version_cascades_attachment_rows() {
    let app = TestApp::spawn().await;
    let uniq = &uuid::Uuid::new_v4().to_string()[..8];
    let v = ProtocolTemplateVersionService::create(
        &app.db_pool,
        &SYSTEM_TEST,
        &create_req(&format!("vdoc-{uniq}")),
    )
    .await
    .expect("create");
    // 直接插一筆 attachment（模擬已上傳文件）
    let uploader: (uuid::Uuid,) = sqlx::query_as("SELECT id FROM users LIMIT 1")
        .fetch_one(&app.db_pool)
        .await
        .expect("a user");
    sqlx::query(
        r#"INSERT INTO attachments (id, category, entity_type, entity_id, file_name, file_path, file_size, mime_type, uploaded_by)
           VALUES (gen_random_uuid(), 'protocol_attachment', 'protocol_template_version', $1, 'sop.pdf', '/tmp/nonexist.pdf', 100, 'application/pdf', $2)"#,
    )
    .bind(v.id)
    .bind(uploader.0)
    .execute(&app.db_pool)
    .await
    .expect("insert attachment");

    ProtocolTemplateVersionService::delete(&app.db_pool, &SYSTEM_TEST, v.id)
        .await
        .expect("delete version");

    let remaining: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM attachments WHERE entity_type = 'protocol_template_version' AND entity_id::text = $1",
    )
    .bind(v.id.to_string())
    .fetch_one(&app.db_pool)
    .await
    .expect("count");
    assert_eq!(remaining, 0, "刪除版本應一併清除其附件 DB 列");
}

// 透過真實 HTTP multipart 端點上傳文件應成功（回歸：attachments.category NOT NULL
// 但 save_attachment INSERT 漏填 category，導致 23502 → 400）。
#[tokio::test]
#[serial]
async fn upload_template_version_document_succeeds() {
    let app = TestApp::spawn().await;
    let uniq = &uuid::Uuid::new_v4().to_string()[..8];
    let v = ProtocolTemplateVersionService::create(
        &app.db_pool,
        &SYSTEM_TEST,
        &create_req(&format!("vup-{uniq}")),
    )
    .await
    .expect("create version");

    let token = app.login_as_admin().await;
    // 最小合法 PDF（magic number %PDF 通過 SEC-14 檔案簽名檢查）
    let pdf: &[u8] = b"%PDF-1.4\n1 0 obj<<>>endobj\ntrailer<<>>\n%%EOF\n";
    let part = reqwest::multipart::Part::bytes(pdf.to_vec())
        .file_name("sop.pdf")
        .mime_str("application/pdf")
        .expect("mime");
    let form = reqwest::multipart::Form::new().part("file", part);
    let res = app
        .client
        .post(app.url(&format!(
            "/api/v1/protocol-template-versions/{}/documents",
            v.id
        )))
        .bearer_auth(&token)
        .multipart(form)
        .send()
        .await
        .expect("upload request");

    let status = res.status();
    let body = res.text().await.unwrap_or_default();
    assert_eq!(status, 200, "上傳應回 200，實得 {status} body={body}");

    // 文件應出現在版本文件清單
    let docs = ProtocolTemplateVersionService::list_documents(&app.db_pool, v.id)
        .await
        .expect("list documents");
    assert_eq!(docs.len(), 1, "應有 1 份已上傳文件");
    assert_eq!(docs[0].file_name, "sop.pdf");

    // 清理實體檔（避免 uploads 目錄殘留）
    ProtocolTemplateVersionService::delete(&app.db_pool, &SYSTEM_TEST, v.id)
        .await
        .expect("cleanup version");
}

#[tokio::test]
#[serial]
async fn template_version_rejects_anonymous() {
    let app = TestApp::spawn().await;
    let err = ProtocolTemplateVersionService::create(
        &app.db_pool,
        &ActorContext::Anonymous,
        &create_req("anon-v1"),
    )
    .await
    .expect_err("Anonymous 不可建立");
    assert!(
        matches!(err, AppError::Forbidden(_)),
        "應 Forbidden，實得：{err:?}"
    );
}
