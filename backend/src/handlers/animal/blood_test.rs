// 血液檢查管理 Handlers（檢查紀錄 + 模板 + 組合）

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{
        AnimalBloodTestWithItems, BloodTestListItem, BloodTestPanelWithItems, BloodTestPreset,
        BloodTestTemplate, CreateBloodTestPanelRequest, CreateBloodTestPresetRequest,
        CreateBloodTestRequest, CreateBloodTestTemplateRequest, DeleteRequest, RecordFilterQuery,
        UpdateBloodTestPanelItemsRequest, UpdateBloodTestPanelRequest,
        UpdateBloodTestPresetRequest, UpdateBloodTestRequest, UpdateBloodTestTemplateRequest,
    },
    require_permission,
    services::{access, AnimalBloodTestService},
    AppState, Result,
};

// ============================================
// 血液檢查紀錄
// ============================================

/// 列出動物的所有血液檢查紀錄
pub async fn list_animal_blood_tests(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Query(filter): Query<RecordFilterQuery>,
) -> Result<Json<Vec<BloodTestListItem>>> {
    // SEC-IDOR: 驗證使用者是否有權存取該動物（透過計畫成員資格）
    access::require_animal_access(&state.db, &current_user, animal_id).await?;
    let tests =
        AnimalBloodTestService::list_blood_tests(&state.db, animal_id, filter.after).await?;
    Ok(Json(tests))
}

/// 取得單筆血液檢查（含明細）
pub async fn get_animal_blood_test(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<AnimalBloodTestWithItems>> {
    let test = AnimalBloodTestService::get_blood_test_by_id(&state.db, id).await?;
    // SEC-IDOR: 透過血液檢查所屬動物驗證計畫存取權限
    access::require_animal_access(&state.db, &current_user, test.blood_test.animal_id).await?;
    Ok(Json(test))
}

/// 建立血液檢查
pub async fn create_animal_blood_test(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<CreateBloodTestRequest>,
) -> Result<Json<AnimalBloodTestWithItems>> {
    require_permission!(current_user, "animal.record.create");
    req.validate()?;

    // Audit 已收進 service 層（BLOOD_TEST_CREATE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let test =
        AnimalBloodTestService::create_blood_test(&state.db, &actor, animal_id, &req).await?;
    Ok(Json(test))
}

/// 更新血液檢查
pub async fn update_animal_blood_test(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateBloodTestRequest>,
) -> Result<Json<AnimalBloodTestWithItems>> {
    require_permission!(current_user, "animal.record.edit");
    req.validate()?;

    // Audit 已收進 service 層（BLOOD_TEST_UPDATE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let test = AnimalBloodTestService::update_blood_test(&state.db, &actor, id, &req).await?;
    Ok(Json(test))
}

/// 刪除血液檢查（軟刪除 + 刪除原因）
pub async fn delete_animal_blood_test(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<DeleteRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.record.delete");
    req.validate()?;

    // Audit 已收進 service 層（BLOOD_TEST_DELETE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    AnimalBloodTestService::soft_delete_blood_test(&state.db, &actor, id, &req.reason).await?;

    Ok(Json(
        serde_json::json!({ "message": "Blood test record deleted successfully" }),
    ))
}

// ============================================
// 血液檢查項目模板管理
// ============================================

/// 列出啟用中的血液檢查項目模板
pub async fn list_blood_test_templates(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<BloodTestTemplate>>> {
    let templates = AnimalBloodTestService::list_blood_test_templates(&state.db).await?;
    Ok(Json(templates))
}

/// 列出所有模板（含停用）- 與動物權限綁定：具 animal.record.view 者可取得（供血檢分析頁使用）
pub async fn list_all_blood_test_templates(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<BloodTestTemplate>>> {
    require_permission!(current_user, "animal.record.view");
    let templates = AnimalBloodTestService::list_all_blood_test_templates(&state.db).await?;
    Ok(Json(templates))
}

/// 建立血液檢查項目模板
pub async fn create_blood_test_template(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateBloodTestTemplateRequest>,
) -> Result<Json<BloodTestTemplate>> {
    require_permission!(current_user, "animal.blood_test_template.manage");
    req.validate()?;

    // Audit 已收進 service 層（TEMPLATE_CREATE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let template =
        AnimalBloodTestService::create_blood_test_template(&state.db, &actor, &req).await?;
    Ok(Json(template))
}

/// 更新血液檢查項目模板
pub async fn update_blood_test_template(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateBloodTestTemplateRequest>,
) -> Result<Json<BloodTestTemplate>> {
    require_permission!(current_user, "animal.blood_test_template.manage");

    // Audit 已收進 service 層（TEMPLATE_UPDATE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let template =
        AnimalBloodTestService::update_blood_test_template(&state.db, &actor, id, &req).await?;
    Ok(Json(template))
}

/// 刪除血液檢查項目模板（停用）
pub async fn delete_blood_test_template(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.blood_test_template.manage");

    // Audit 已收進 service 層（TEMPLATE_DELETE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    AnimalBloodTestService::delete_blood_test_template(&state.db, &actor, id).await?;
    Ok(Json(
        serde_json::json!({ "message": "Template deactivated successfully" }),
    ))
}

// ============================================
// 血液檢查組合 (Panel) 管理
// ============================================

/// 列出啟用中的血液檢查組合（含項目）
pub async fn list_blood_test_panels(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<BloodTestPanelWithItems>>> {
    let panels = AnimalBloodTestService::list_blood_test_panels(&state.db).await?;
    Ok(Json(panels))
}

/// 列出所有組合（含停用）- 與動物權限綁定：具 animal.record.view 者可取得（供血檢分析頁使用）
pub async fn list_all_blood_test_panels(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<BloodTestPanelWithItems>>> {
    require_permission!(current_user, "animal.record.view");
    let panels = AnimalBloodTestService::list_all_blood_test_panels(&state.db).await?;
    Ok(Json(panels))
}

/// 建立血液檢查組合
pub async fn create_blood_test_panel(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateBloodTestPanelRequest>,
) -> Result<Json<BloodTestPanelWithItems>> {
    require_permission!(current_user, "animal.blood_test_template.manage");
    req.validate()?;

    // Audit 已收進 service 層（PANEL_CREATE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let panel = AnimalBloodTestService::create_blood_test_panel(&state.db, &actor, &req).await?;
    Ok(Json(panel))
}

/// 更新血液檢查組合
pub async fn update_blood_test_panel(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateBloodTestPanelRequest>,
) -> Result<Json<BloodTestPanelWithItems>> {
    require_permission!(current_user, "animal.blood_test_template.manage");

    // Audit 已收進 service 層（PANEL_UPDATE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let panel =
        AnimalBloodTestService::update_blood_test_panel(&state.db, &actor, id, &req).await?;
    Ok(Json(panel))
}

/// 更新組合內的項目
pub async fn update_blood_test_panel_items(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateBloodTestPanelItemsRequest>,
) -> Result<Json<BloodTestPanelWithItems>> {
    require_permission!(current_user, "animal.blood_test_template.manage");

    // Audit 已收進 service 層（PANEL_UPDATE items，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let panel =
        AnimalBloodTestService::update_blood_test_panel_items(&state.db, &actor, id, &req).await?;
    Ok(Json(panel))
}

/// 刪除血液檢查組合（停用）
pub async fn delete_blood_test_panel(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.blood_test_template.manage");

    // Audit 已收進 service 層（PANEL_DELETE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    AnimalBloodTestService::delete_blood_test_panel(&state.db, &actor, id).await?;
    Ok(Json(
        serde_json::json!({ "message": "Panel deactivated successfully" }),
    ))
}

// ============================================
// 血液檢查常用組合 (Preset) 管理
// ============================================

/// 列出啟用中的常用組合（供分析頁使用）
pub async fn list_blood_test_presets(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<BloodTestPreset>>> {
    let presets = AnimalBloodTestService::list_blood_test_presets(&state.db).await?;
    Ok(Json(presets))
}

/// 列出所有常用組合（含停用）- 與動物權限綁定：具 animal.record.view 者可取得（供血檢分析頁使用）
pub async fn list_all_blood_test_presets(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<BloodTestPreset>>> {
    require_permission!(current_user, "animal.record.view");
    let presets = AnimalBloodTestService::list_all_blood_test_presets(&state.db).await?;
    Ok(Json(presets))
}

/// 建立常用組合
pub async fn create_blood_test_preset(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateBloodTestPresetRequest>,
) -> Result<Json<BloodTestPreset>> {
    require_permission!(current_user, "animal.blood_test_template.manage");
    req.validate()?;

    // Audit 已收進 service 層（PRESET_CREATE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let preset = AnimalBloodTestService::create_blood_test_preset(&state.db, &actor, &req).await?;
    Ok(Json(preset))
}

/// 更新常用組合
pub async fn update_blood_test_preset(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateBloodTestPresetRequest>,
) -> Result<Json<BloodTestPreset>> {
    require_permission!(current_user, "animal.blood_test_template.manage");

    // Audit 已收進 service 層（PRESET_UPDATE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let preset =
        AnimalBloodTestService::update_blood_test_preset(&state.db, &actor, id, &req).await?;
    Ok(Json(preset))
}

/// 刪除常用組合（軟刪除）
pub async fn delete_blood_test_preset(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.blood_test_template.manage");

    // Audit 已收進 service 層（PRESET_DELETE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    AnimalBloodTestService::delete_blood_test_preset(&state.db, &actor, id).await?;
    Ok(Json(
        serde_json::json!({ "message": "Preset deactivated successfully" }),
    ))
}
