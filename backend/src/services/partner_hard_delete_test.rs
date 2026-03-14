use sqlx::PgPool;
use uuid::Uuid;
use crate::services::PartnerService;
use crate::models::{CreatePartnerRequest, PartnerType, SupplierCategory};

#[sqlx::test]
async fn test_admin_hard_delete_partner(pool: PgPool) {
    // 1. 建立一個夥伴
    let req = CreatePartnerRequest {
        partner_type: PartnerType::Supplier,
        code: Some("TEST001".to_string()),
        name: "Test Partner".to_string(),
        supplier_category: Some(SupplierCategory::Drug),
        customer_category: None,
        tax_id: None,
        phone: None,
        phone_ext: None,
        email: None,
        address: None,
        payment_terms: None,
    };
    let partner = PartnerService::create(&pool, &req).await.unwrap();
    let partner_id = partner.id;

    // 2. 測試軟刪除 (is_hard = false)
    PartnerService::delete(&pool, partner_id, false).await.unwrap();
    
    // 驗證：記錄還在，但 is_active = false
    let is_active: bool = sqlx::query_scalar("SELECT is_active FROM partners WHERE id = $1")
        .bind(partner_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(!is_active);

    // 3. 測試硬刪除 (is_hard = true)
    PartnerService::delete(&pool, partner_id, true).await.unwrap();

    // 驗證：記錄已從資料庫消失
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM partners WHERE id = $1)")
        .bind(partner_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(!exists);
}
