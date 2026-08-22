//! 自助新建動物物種（P1）驗收測試。
//!
//! 核心命題：admin 在物種管理新增一個物種後，**不需要改 code、不需要重新部署**，
//! 就能立刻建立該物種的動物。過去這條路會被寫死的 `animal_breed` enum 擋成 422。

mod common;

use serial_test::serial;

/// 建立一個唯一 code 的頂層物種，回傳 (id, code)。
async fn create_species(app: &common::TestApp, token: &str, label: &str) -> (String, String) {
    let code = format!("{}_{}", label, unique_suffix());
    let body = serde_json::json!({
        "code": code,
        "name": format!("測試物種 {}", code),
        "name_en": "Test Species",
        "sort_order": 50,
    });
    let res = app
        .auth_post("/api/v1/facilities/species", &body, token)
        .await;
    assert_eq!(res.status(), 201, "建立物種應回 201，實際 {}", res.status());
    let created: serde_json::Value = res.json().await.expect("物種建立回應應為 JSON");
    let id = created["id"].as_str().expect("物種應有 id").to_string();
    (id, code)
}

/// 耳號只有三位數（100–999），同一顆測試 DB 內跨測試檔很容易撞號，而
/// `AnimalService::create` 的硬阻擋條件是「同耳號**且**同出生日期」——`force_create`
/// 只跳過重複警告，跳不過這一條。因此每個測試都帶自己的 `birth_date`：
/// 撞到耳號時出生日期不同，仍可建立。用 1999 年份是因為整個測試套件沒有別處使用，
/// 不會與其他測試檔直接 INSERT 的動物相撞。
fn animal_body(ear_tag: &str, birth_date: &str) -> serde_json::Value {
    serde_json::json!({
        "ear_tag": ear_tag,
        "gender": "male",
        "birth_date": birth_date,
        "entry_date": "2026-07-13",
        "entry_weight": 59.0,
        "pen_location": "A-01",
        "force_create": true,
    })
}

/// P1 的核心驗收：自助新增物種 → 立刻能建該物種的動物。
#[tokio::test]
#[serial]
async fn newly_created_species_is_immediately_usable_for_animals() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let (species_id, _) = create_species(&app, &token, "goat").await;

    let mut body = animal_body(&format!("{:03}", rand_num()), "1999-01-01");
    body["species_id"] = serde_json::json!(species_id);

    let res = app.auth_post("/api/v1/animals", &body, &token).await;
    assert!(
        res.status() == 201 || res.status() == 200,
        "以新物種建立動物應成功，實際 {}",
        res.status()
    );

    let created: serde_json::Value = res.json().await.expect("動物建立回應應為 JSON");
    assert_eq!(
        created["species_id"].as_str(),
        Some(species_id.as_str()),
        "動物應保存所選物種"
    );
    // 主檔沒有對應 enum 值的物種，相容欄位 breed 一律推導為 other
    assert_eq!(created["breed"], "other", "新物種的 breed 應推導為 other");
}

/// 動物明細要帶出物種名稱，顯示端才不會把山羊印成「其他」。
#[tokio::test]
#[serial]
async fn animal_detail_exposes_species_name() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let (species_id, code) = create_species(&app, &token, "sheep").await;
    let expected_name = format!("測試物種 {}", code);

    let mut body = animal_body(&format!("{:03}", rand_num()), "1999-01-02");
    body["species_id"] = serde_json::json!(species_id);
    let created: serde_json::Value = app
        .auth_post("/api/v1/animals", &body, &token)
        .await
        .json()
        .await
        .expect("動物建立回應應為 JSON");
    let animal_id = created["id"].as_str().expect("動物應有 id");

    let fetched: serde_json::Value = app
        .auth_get(&format!("/api/v1/animals/{}", animal_id), &token)
        .await
        .json()
        .await
        .expect("動物明細應為 JSON");
    assert_eq!(
        fetched["species_name"].as_str(),
        Some(expected_name.as_str()),
        "明細應帶出 species_name"
    );
}

/// 舊 client / Excel 匯入只送 breed 時，後端要反查補上 species_id，
/// 否則 P3 要把 species_id 改 NOT NULL 時又會多出一批漏網列。
#[tokio::test]
#[serial]
async fn legacy_breed_only_request_backfills_species_id() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let mut body = animal_body(&format!("{:03}", rand_num()), "1999-01-03");
    body["breed"] = serde_json::json!("white");

    let res = app.auth_post("/api/v1/animals", &body, &token).await;
    assert!(
        res.status() == 201 || res.status() == 200,
        "只送 breed 的舊呼叫端應仍可建立，實際 {}",
        res.status()
    );

    let created: serde_json::Value = res.json().await.expect("動物建立回應應為 JSON");
    assert_eq!(created["breed"], "white");
    assert!(
        created["species_id"].is_string(),
        "只送 breed 時後端應反查補上 species_id，實際 {}",
        created["species_id"]
    );
}

/// species_id 與 breed 都沒給 → 擋下，不要靜默寫入預設品種。
#[tokio::test]
#[serial]
async fn animal_without_species_or_breed_is_rejected() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let body = animal_body(&format!("{:03}", rand_num()), "1999-01-04");
    let res = app.auth_post("/api/v1/animals", &body, &token).await;
    assert_eq!(
        res.status(),
        400,
        "未指定物種應被擋下，實際 {}",
        res.status()
    );
}

/// species 成為真相源後，停用/刪除使用中的物種會讓動物的品種顯示變孤兒，必須擋下。
#[tokio::test]
#[serial]
async fn species_in_use_cannot_be_deleted() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let (species_id, _) = create_species(&app, &token, "inuse").await;

    // 沒有動物引用時可以刪
    let (spare_id, _) = create_species(&app, &token, "spare").await;
    let res = app
        .auth_delete(&format!("/api/v1/facilities/species/{}", spare_id), &token)
        .await;
    assert_eq!(res.status(), 204, "未被引用的物種應可刪除");

    // 建一隻動物指向它之後就不能刪
    let mut body = animal_body(&format!("{:03}", rand_num()), "1999-01-05");
    body["species_id"] = serde_json::json!(species_id);
    let create_res = app.auth_post("/api/v1/animals", &body, &token).await;
    assert!(
        create_res.status() == 201 || create_res.status() == 200,
        "前置：建立動物應成功，實際 {}",
        create_res.status()
    );

    let res = app
        .auth_delete(
            &format!("/api/v1/facilities/species/{}", species_id),
            &token,
        )
        .await;
    assert_eq!(
        res.status(),
        422,
        "使用中的物種不可刪除，實際 {}",
        res.status()
    );

    // 停用（update is_active=false）同樣要擋
    let res = app
        .auth_put(
            &format!("/api/v1/facilities/species/{}", species_id),
            &serde_json::json!({ "is_active": false }),
            &token,
        )
        .await;
    assert_eq!(
        res.status(),
        422,
        "使用中的物種不可停用，實際 {}",
        res.status()
    );
}

/// 重複 code 要在 service 層擋下並回可讀訊息，而不是把 DB 的 unique violation 噴給使用者。
#[tokio::test]
#[serial]
async fn duplicate_species_code_is_rejected() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let (_, code) = create_species(&app, &token, "dup").await;

    let res = app
        .auth_post(
            "/api/v1/facilities/species",
            &serde_json::json!({ "code": code.to_uppercase(), "name": "大小寫不同的重複代碼" }),
            &token,
        )
        .await;
    assert_eq!(
        res.status(),
        409,
        "重複 code（不分大小寫）應回 409，實際 {}",
        res.status()
    );
}

/// 物種階層只有兩層，父物種必須是頂層——這條同時也是 parent 迴圈的結構性防護。
#[tokio::test]
#[serial]
async fn species_parent_must_be_top_level() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let (parent_id, _) = create_species(&app, &token, "lvl1").await;

    // 第二層：合法
    let child_body = serde_json::json!({
        "code": format!("lvl2_{}", unique_suffix()),
        "name": "第二層物種",
        "parent_id": parent_id,
    });
    let res = app
        .auth_post("/api/v1/facilities/species", &child_body, &token)
        .await;
    assert_eq!(res.status(), 201, "第二層物種應可建立");
    let child: serde_json::Value = res.json().await.expect("物種建立回應應為 JSON");
    let child_id = child["id"].as_str().expect("物種應有 id");

    // 第三層：擋下
    let grandchild_body = serde_json::json!({
        "code": format!("lvl3_{}", unique_suffix()),
        "name": "第三層物種",
        "parent_id": child_id,
    });
    let res = app
        .auth_post("/api/v1/facilities/species", &grandchild_body, &token)
        .await;
    assert_eq!(
        res.status(),
        400,
        "第三層物種應被擋下，實際 {}",
        res.status()
    );

    // 指向自己：擋下
    let res = app
        .auth_put(
            &format!("/api/v1/facilities/species/{}", parent_id),
            &serde_json::json!({ "parent_id": parent_id }),
            &token,
        )
        .await;
    assert_eq!(
        res.status(),
        400,
        "父物種指向自己應被擋下，實際 {}",
        res.status()
    );
}

/// 只有葉節點能指派給動物：頂層的父物種（如「豬」）是分類用的，指給牠語意不完整。
/// 前端下拉本來就只列葉節點，但後端不能倚賴前端當安全邊界。
#[tokio::test]
#[serial]
async fn animal_cannot_be_assigned_to_a_parent_species() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let (parent_id, _) = create_species(&app, &token, "genus").await;
    let child_body = serde_json::json!({
        "code": format!("child_{}", unique_suffix()),
        "name": "子物種",
        "parent_id": parent_id,
    });
    let res = app
        .auth_post("/api/v1/facilities/species", &child_body, &token)
        .await;
    assert_eq!(res.status(), 201, "前置：子物種應可建立");

    let mut body = animal_body(&format!("{:03}", rand_num()), "1999-01-07");
    body["species_id"] = serde_json::json!(parent_id);
    let res = app.auth_post("/api/v1/animals", &body, &token).await;
    assert_eq!(
        res.status(),
        400,
        "指定非葉節點物種應被擋下，實際 {}",
        res.status()
    );
}

/// 物種代碼是全域唯一且不重用的（軟刪除仍佔用代碼）。撞到停用物種時，錯誤訊息必須
/// 講清楚是「被停用的物種佔用」並指向重新啟用，否則使用者只會看到「已存在」而卡死。
#[tokio::test]
#[serial]
async fn deleted_species_code_reports_it_is_reusable_by_reactivating() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let (species_id, code) = create_species(&app, &token, "retire").await;
    let res = app
        .auth_delete(
            &format!("/api/v1/facilities/species/{}", species_id),
            &token,
        )
        .await;
    assert_eq!(res.status(), 204, "前置：未被引用的物種應可刪除");

    let res = app
        .auth_post(
            "/api/v1/facilities/species",
            &serde_json::json!({ "code": code, "name": "重新建立" }),
            &token,
        )
        .await;
    assert_eq!(res.status(), 409, "代碼仍被停用物種佔用，應回 409");

    let body: serde_json::Value = res.json().await.expect("錯誤回應應為 JSON");
    let message = body["error"]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("已停用"),
        "訊息要點出是被停用物種佔用，實際：{}",
        message
    );
}

/// 停用物種預設不出現在清單（動物表單的品種下拉吃同一支 API），
/// 但帶 include_inactive 就要看得到——否則沒有任何途徑把它啟用回來。
#[tokio::test]
#[serial]
async fn inactive_species_are_listed_only_when_requested_and_can_be_reactivated() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let (species_id, _) = create_species(&app, &token, "revive").await;
    app.auth_delete(
        &format!("/api/v1/facilities/species/{}", species_id),
        &token,
    )
    .await;

    let contains = |list: &serde_json::Value| -> bool {
        list.as_array()
            .map(|rows| {
                rows.iter()
                    .any(|s| s["id"].as_str() == Some(species_id.as_str()))
            })
            .unwrap_or(false)
    };

    let default_list: serde_json::Value = app
        .auth_get("/api/v1/facilities/species", &token)
        .await
        .json()
        .await
        .expect("物種清單應為 JSON");
    assert!(!contains(&default_list), "預設清單不應含停用物種");

    let with_inactive: serde_json::Value = app
        .auth_get("/api/v1/facilities/species?include_inactive=true", &token)
        .await
        .json()
        .await
        .expect("物種清單應為 JSON");
    assert!(contains(&with_inactive), "include_inactive 應帶出停用物種");

    // 重新啟用後，代碼即可正常使用
    let res = app
        .auth_put(
            &format!("/api/v1/facilities/species/{}", species_id),
            &serde_json::json!({ "is_active": true }),
            &token,
        )
        .await;
    assert_eq!(res.status(), 200, "重新啟用應成功");

    let default_list: serde_json::Value = app
        .auth_get("/api/v1/facilities/species", &token)
        .await
        .json()
        .await
        .expect("物種清單應為 JSON");
    assert!(contains(&default_list), "啟用後應回到預設清單");
}

/// 物種清單要帶出引用數，前端才能標「使用中」並停用刪除鈕。
#[tokio::test]
#[serial]
async fn species_list_reports_animal_usage() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let (species_id, _) = create_species(&app, &token, "usage").await;

    let mut body = animal_body(&format!("{:03}", rand_num()), "1999-01-06");
    body["species_id"] = serde_json::json!(species_id);
    let create_res = app.auth_post("/api/v1/animals", &body, &token).await;
    assert!(
        create_res.status() == 201 || create_res.status() == 200,
        "前置：建立動物應成功，實際 {}",
        create_res.status()
    );

    let list: serde_json::Value = app
        .auth_get("/api/v1/facilities/species", &token)
        .await
        .json()
        .await
        .expect("物種清單應為 JSON");
    let row = list
        .as_array()
        .expect("物種清單應為陣列")
        .iter()
        .find(|s| s["id"].as_str() == Some(species_id.as_str()))
        .expect("清單應含剛建立的物種");
    assert!(
        row["animal_count"].as_i64().unwrap_or(0) >= 1,
        "animal_count 應反映引用數，實際 {}",
        row["animal_count"]
    );
}

/// R86-5：`PUT /animals/:id` 的 `species_id` 過去直接 `COALESCE` 寫入，
/// 不驗存在／啟用／葉節點，也不重推 `breed`。以下四支涵蓋建立流程
/// （`SpeciesLink::resolve_from_species`）已有的三種擋法在更新路徑上同樣成立，
/// 加上合法變更會連動重推 breed，不留下 species_id 與 breed 矛盾的列。
#[tokio::test]
#[serial]
async fn update_animal_species_id_rejects_nonexistent_species() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let mut body = animal_body(&format!("{:03}", rand_num()), "1999-02-01");
    body["breed"] = serde_json::json!("white");
    let created: serde_json::Value = app
        .auth_post("/api/v1/animals", &body, &token)
        .await
        .json()
        .await
        .expect("動物建立回應應為 JSON");
    let animal_id = created["id"].as_str().expect("動物應有 id");

    let res = app
        .auth_put(
            &format!("/api/v1/animals/{}", animal_id),
            &serde_json::json!({ "species_id": uuid::Uuid::new_v4() }),
            &token,
        )
        .await;
    assert_eq!(
        res.status(),
        400,
        "不存在的物種應被擋下，實際 {}",
        res.status()
    );
}

#[tokio::test]
#[serial]
async fn update_animal_species_id_rejects_parent_species() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let (parent_id, _) = create_species(&app, &token, "upd_genus").await;
    let child_body = serde_json::json!({
        "code": format!("upd_child_{}", unique_suffix()),
        "name": "更新測試子物種",
        "parent_id": parent_id,
    });
    let child: serde_json::Value = app
        .auth_post("/api/v1/facilities/species", &child_body, &token)
        .await
        .json()
        .await
        .expect("物種建立回應應為 JSON");
    let child_id = child["id"].as_str().expect("物種應有 id");

    let mut body = animal_body(&format!("{:03}", rand_num()), "1999-02-02");
    body["species_id"] = serde_json::json!(child_id);
    let created: serde_json::Value = app
        .auth_post("/api/v1/animals", &body, &token)
        .await
        .json()
        .await
        .expect("動物建立回應應為 JSON");
    let animal_id = created["id"].as_str().expect("動物應有 id");

    let res = app
        .auth_put(
            &format!("/api/v1/animals/{}", animal_id),
            &serde_json::json!({ "species_id": parent_id }),
            &token,
        )
        .await;
    assert_eq!(
        res.status(),
        400,
        "改指派非葉節點物種應被擋下，實際 {}",
        res.status()
    );
}

#[tokio::test]
#[serial]
async fn update_animal_species_id_rejects_inactive_species() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let (active_id, _) = create_species(&app, &token, "upd_active").await;
    let (inactive_id, _) = create_species(&app, &token, "upd_inactive").await;
    let del_res = app
        .auth_delete(
            &format!("/api/v1/facilities/species/{}", inactive_id),
            &token,
        )
        .await;
    assert_eq!(del_res.status(), 204, "前置：未被引用的物種應可停用");

    let mut body = animal_body(&format!("{:03}", rand_num()), "1999-02-03");
    body["species_id"] = serde_json::json!(active_id);
    let created: serde_json::Value = app
        .auth_post("/api/v1/animals", &body, &token)
        .await
        .json()
        .await
        .expect("動物建立回應應為 JSON");
    let animal_id = created["id"].as_str().expect("動物應有 id");

    let res = app
        .auth_put(
            &format!("/api/v1/animals/{}", animal_id),
            &serde_json::json!({ "species_id": inactive_id }),
            &token,
        )
        .await;
    assert_eq!(
        res.status(),
        400,
        "改指派已停用物種應被擋下，實際 {}",
        res.status()
    );
}

/// 核心迴歸案例：改 species_id 必須連動重推 breed，否則兩欄互相矛盾
/// （改之前 `breed` 卡在 `white`，改之後 `species_id` 已指向自助新物種但 `breed` 沒跟著變）。
#[tokio::test]
#[serial]
async fn update_animal_species_id_recomputes_breed() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    let mut body = animal_body(&format!("{:03}", rand_num()), "1999-02-04");
    body["breed"] = serde_json::json!("white");
    let created: serde_json::Value = app
        .auth_post("/api/v1/animals", &body, &token)
        .await
        .json()
        .await
        .expect("動物建立回應應為 JSON");
    let animal_id = created["id"].as_str().expect("動物應有 id");
    assert_eq!(created["breed"], "white", "前置：應以 white 建立");

    let (new_species_id, _) = create_species(&app, &token, "upd_recompute").await;
    let res = app
        .auth_put(
            &format!("/api/v1/animals/{}", animal_id),
            &serde_json::json!({ "species_id": new_species_id }),
            &token,
        )
        .await;
    assert_eq!(res.status(), 200, "合法變更應成功，實際 {}", res.status());
    let updated: serde_json::Value = res.json().await.expect("更新回應應為 JSON");
    assert_eq!(
        updated["species_id"].as_str(),
        Some(new_species_id.as_str()),
        "species_id 應已更新"
    );
    assert_eq!(
        updated["breed"], "other",
        "breed 應隨新 species_id 重推為 other，不可停留在舊值 white"
    );
}

/// Qodo review 於 #125 指出：resolve_from_species 拒絕已停用物種，但合法舊資料
/// 可能就是停用物種（如 LYD，相容路徑 resolve_from_breed 刻意不檢查 is_active）。
/// 若呼叫端只是把動物目前的 species_id 原樣回填（常見於整份表單送出的編輯流程），
/// 不應該因為物種本身停用就被 400——那不是使用者想改的欄位。
#[tokio::test]
#[serial]
async fn update_animal_with_unchanged_inactive_species_id_still_succeeds() {
    let app = common::TestApp::spawn().await;
    let token = app.login_as_admin().await;

    // LYD 走相容路徑建立：主檔為停用狀態，但既有動物合法持有這個 species_id。
    let mut body = animal_body(&format!("{:03}", rand_num()), "1999-02-05");
    body["breed"] = serde_json::json!("lyd");
    let created: serde_json::Value = app
        .auth_post("/api/v1/animals", &body, &token)
        .await
        .json()
        .await
        .expect("動物建立回應應為 JSON");
    let animal_id = created["id"].as_str().expect("動物應有 id");
    let species_id = created["species_id"]
        .as_str()
        .expect("LYD 相容路徑應反查補上 species_id")
        .to_string();

    // 原樣回填目前的 species_id（未實際變更），只改 remark。
    let res = app
        .auth_put(
            &format!("/api/v1/animals/{}", animal_id),
            &serde_json::json!({ "species_id": species_id, "remark": "echo unchanged species_id" }),
            &token,
        )
        .await;
    assert_eq!(
        res.status(),
        200,
        "species_id 原樣回填（值未變）不應因物種本身停用而被擋，實際 {}",
        res.status()
    );
    let updated: serde_json::Value = res.json().await.expect("更新回應應為 JSON");
    assert_eq!(updated["species_id"].as_str(), Some(species_id.as_str()));
    assert_eq!(
        updated["breed"], "lyd",
        "breed 不應因未變更的 species_id 而被改動"
    );
}

fn rand_num() -> u32 {
    use std::time::SystemTime;
    let d = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time should be valid");
    (d.as_nanos() % 900) as u32 + 100
}

/// 供 species code 去重用（同一顆測試 DB 內多個測試共用 species 表）。
fn unique_suffix() -> String {
    use std::time::SystemTime;
    let d = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time should be valid");
    format!("{}", d.as_nanos() % 1_000_000_000)
}
