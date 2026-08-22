//! 整合測試：疫苗/驅蟲紀錄的後端驗證（`CreateVaccinationRequest` / `UpdateVaccinationRequest`）。
//!
//! - 合法值（含「其他」自訂文字）→ 成功
//! - 空白字串（非 null，模擬繞過前端下拉塞垃圾值）→ 400 擋下
//! - 更新路徑先前漏呼叫 `req.validate()`，本測試同時涵蓋建立與更新兩條路徑

mod common;

use serial_test::serial;

#[tokio::test]
#[serial]
async fn vaccination_validation_rejects_blank_but_allows_custom_text() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let ear_tag = common::free_ear_tag(&app.db_pool).await;
    let create_body = serde_json::json!({
        "ear_tag": ear_tag,
        "breed": "white",
        "gender": "female",
        "entry_date": "2026-01-15",
        "entry_weight": 25.5,
        "pen_location": "A-01",
        "force_create": true
    });
    let created = app.auth_post("/api/v1/animals", &create_body, &token).await;
    assert!(
        created.status() == 200 || created.status() == 201,
        "建立動物失敗：{}",
        created.status()
    );
    let created: serde_json::Value = created.json().await.expect("parse create response");
    let animal_id = created["id"]
        .as_str()
        .expect("created animal id")
        .to_string();
    let path = format!("/api/v1/animals/{}/vaccinations", animal_id);

    // 1) 合法代碼 + 「其他」自訂文字驅蟲藥名 → 成功
    let body_ok = serde_json::json!({
        "administered_date": "2026-06-30",
        "vaccine": "SEP",
        "deworming_dose": "驅蟲寧 2ml"
    });
    let r_ok = app.auth_post(&path, &body_ok, &token).await;
    assert!(
        r_ok.status() == 200 || r_ok.status() == 201,
        "合法疫苗代碼＋自訂驅蟲藥名應成功，得到 {}",
        r_ok.status()
    );
    let created_vac: serde_json::Value = r_ok.json().await.expect("parse vaccination response");
    let vaccination_id = created_vac["id"]
        .as_str()
        .expect("created vaccination id")
        .to_string();

    // 2) 空白字串（非 null）→ 400 擋下（繞過前端下拉塞垃圾值）
    let body_blank = serde_json::json!({
        "administered_date": "2026-06-30",
        "vaccine": "   "
    });
    let r_blank = app.auth_post(&path, &body_blank, &token).await;
    assert_eq!(r_blank.status(), 400, "空白疫苗欄位應被 400 擋下");

    // 3) 更新路徑：先前漏呼叫 req.validate()，同一條空白字串規則須同樣生效
    let update_path = format!("/api/v1/vaccinations/{}", vaccination_id);
    let update_blank = serde_json::json!({ "deworming_dose": "   " });
    let r_update_blank = app.auth_put(&update_path, &update_blank, &token).await;
    assert_eq!(
        r_update_blank.status(),
        400,
        "更新路徑空白驅蟲劑量應被 400 擋下"
    );

    // 4) 更新路徑：合法值仍可正常放行
    let update_ok = serde_json::json!({ "vaccine": "OTHER" });
    let r_update_ok = app.auth_put(&update_path, &update_ok, &token).await;
    assert!(
        r_update_ok.status() == 200 || r_update_ok.status() == 201,
        "更新路徑合法值應成功，得到 {}",
        r_update_ok.status()
    );

    // 5) 更新路徑：明確送 null 應能清空既有值（不得被 COALESCE 誤解讀為「維持舊值」）
    let update_clear = serde_json::json!({
        "administered_date": "2026-06-30",
        "vaccine": null,
        "deworming_dose": null
    });
    let r_clear = app.auth_put(&update_path, &update_clear, &token).await;
    assert!(
        r_clear.status() == 200 || r_clear.status() == 201,
        "清空欄位的更新應成功，得到 {}",
        r_clear.status()
    );
    let cleared: serde_json::Value = r_clear.json().await.expect("parse clear response");
    assert!(
        cleared["vaccine"].is_null(),
        "送 null 應清空 vaccine，實得 {:?}",
        cleared["vaccine"]
    );
    assert!(
        cleared["deworming_dose"].is_null(),
        "送 null 應清空 deworming_dose，實得 {:?}",
        cleared["deworming_dose"]
    );

    // 6) 空白填充不得繞過長度上限：驗證只看 trim 後長度，但寫入前 handler 會先 trim，
    // 故短內容外包大量空白應成功且「原樣存入的是 trim 後的乾淨值」，不是帶填充的原字串
    let padded = format!("   SEP{}   ", " ".repeat(200));
    let body_padded = serde_json::json!({
        "administered_date": "2026-06-30",
        "vaccine": padded
    });
    let r_padded = app.auth_post(&path, &body_padded, &token).await;
    assert!(
        r_padded.status() == 200 || r_padded.status() == 201,
        "trim 後合法長度應成功，得到 {}",
        r_padded.status()
    );
    let padded_result: serde_json::Value = r_padded.json().await.expect("parse padded response");
    assert_eq!(
        padded_result["vaccine"], "SEP",
        "寫入值應為 trim 後的乾淨值，不帶填充空白，實得 {:?}",
        padded_result["vaccine"]
    );
}
