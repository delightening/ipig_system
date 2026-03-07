// 血液檢查管理 Handlers（檢查紀錄 + 模板 + 組合）

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{
        BloodTestListItem, BloodTestPanelWithItems, BloodTestPreset, BloodTestTemplate,
        CreateBloodTestPanelRequest, CreateBloodTestPresetRequest, CreateBloodTestRequest,
        CreateBloodTestTemplateRequest, DeleteRequest, AnimalBloodTestWithItems,
        UpdateBloodTestPanelItemsRequest, UpdateBloodTestPanelRequest, UpdateBloodTestPresetRequest,
        UpdateBloodTestRequest, UpdateBloodTestTemplateRequest, RecordFilterQuery,
    },
    require_permission,
    services::{AnimalService, AuditService},
    AppState, Result,
};

// ============================================
// 血液檢查紀錄
// ============================================

/// 列出動物的所有血液檢查紀錄
pub async fn list_animal_blood_tests(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Query(filter): Query<RecordFilterQuery>,
) -> Result<Json<Vec<BloodTestListItem>>> {
    let tests = AnimalService::list_blood_tests(&state.db, animal_id, filter.after).await?;
    Ok(Json(tests))
}

/// 取得單筆血液檢查（含明細）
pub async fn get_animal_blood_test(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<AnimalBloodTestWithItems>> {
    let test = AnimalService::get_blood_test_by_id(&state.db, id).await?;
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

    let test = AnimalService::create_blood_test(&state.db, animal_id, &req, current_user.id).await?;

    let display_name = match sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT ear_tag, iacuc_no FROM animals WHERE id = $1"
    ).bind(animal_id).fetch_optional(&state.db).await {
        Ok(Some((ear_tag, iacuc_no))) => {
            let iacuc = iacuc_no.unwrap_or_else(|| "未指派".to_string());
            format!("[{}] {}", iacuc, ear_tag)
        }
        _ => format!("血液檢查紀錄 (animal: {})", animal_id),
    };

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "BLOOD_TEST_CREATE",
        Some("animal_blood_test"), Some(animal_id), Some(&display_name),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (BLOOD_TEST_CREATE): {}", e);
    }

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

    let test = AnimalService::update_blood_test(&state.db, id, &req).await?;

    let animal_id = test.blood_test.animal_id;
    let display_name = match sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT ear_tag, iacuc_no FROM animals WHERE id = $1"
    ).bind(animal_id).fetch_optional(&state.db).await {
        Ok(Some((ear_tag, iacuc_no))) => {
            let iacuc = iacuc_no.unwrap_or_else(|| "未指派".to_string());
            format!("[{}] {}", iacuc, ear_tag)
        }
        _ => format!("血液檢查紀錄 #{}", id),
    };

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "BLOOD_TEST_UPDATE",
        Some("animal_blood_test"), Some(animal_id), Some(&display_name),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (BLOOD_TEST_UPDATE): {}", e);
    }

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

    let animal_info = sqlx::query_as::<_, (Uuid, String, Option<String>)>(
        "SELECT p.id, p.ear_tag, p.iacuc_no FROM animals p INNER JOIN animal_blood_tests bt ON bt.animal_id = p.id WHERE bt.id = $1"
    ).bind(id).fetch_optional(&state.db).await;

    AnimalService::soft_delete_blood_test(&state.db, id, &req.reason, current_user.id).await?;

    let (pid, display_name) = match animal_info {
        Ok(Some((pid, ear_tag, iacuc_no))) => {
            let iacuc = iacuc_no.unwrap_or_else(|| "未指派".to_string());
            (Some(pid), format!("[{}] {} (原因: {})", iacuc, ear_tag, req.reason))
        }
        _ => (None, format!("血液檢查紀錄 #{} (原因: {})", id, req.reason)),
    };

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "BLOOD_TEST_DELETE",
        Some("animal_blood_test"), pid, Some(&display_name),
        None, Some(serde_json::json!({ "reason": req.reason })), None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (BLOOD_TEST_DELETE): {}", e);
    }

    Ok(Json(serde_json::json!({ "message": "Blood test record deleted successfully" })))
}

// ============================================
// 血液檢查項目模板管理
// ============================================

/// 列出啟用中的血液檢查項目模板
pub async fn list_blood_test_templates(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<BloodTestTemplate>>> {
    let templates = AnimalService::list_blood_test_templates(&state.db).await?;
    Ok(Json(templates))
}

/// 列出所有模板（含停用）- 與動物權限綁定：具 animal.record.view 者可取得（供血檢分析頁使用）
pub async fn list_all_blood_test_templates(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<BloodTestTemplate>>> {
    require_permission!(current_user, "animal.record.view");
    let templates = AnimalService::list_all_blood_test_templates(&state.db).await?;
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
    let template = AnimalService::create_blood_test_template(&state.db, &req).await?;
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "TEMPLATE_CREATE",
        Some("blood_test_template"), Some(template.id),
        Some(&format!("建立血檢模板: {}", req.name)), None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (TEMPLATE_CREATE): {}", e);
    }
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
    let template = AnimalService::update_blood_test_template(&state.db, id, &req).await?;
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "TEMPLATE_UPDATE",
        Some("blood_test_template"), Some(id),
        Some(&format!("更新血檢模板: {}", template.name)), None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (TEMPLATE_UPDATE): {}", e);
    }
    Ok(Json(template))
}

/// 刪除血液檢查項目模板（停用）
pub async fn delete_blood_test_template(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.blood_test_template.manage");
    AnimalService::delete_blood_test_template(&state.db, id).await?;
    let tmpl_name = sqlx::query_scalar::<_, String>("SELECT name FROM blood_test_templates WHERE id = $1")
        .bind(id).fetch_optional(&state.db).await.ok().flatten().unwrap_or_else(|| id.to_string());
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "TEMPLATE_DELETE",
        Some("blood_test_template"), Some(id),
        Some(&format!("停用血檢模板: {}", tmpl_name)), None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (TEMPLATE_DELETE): {}", e);
    }
    Ok(Json(serde_json::json!({ "message": "Template deactivated successfully" })))
}

// ============================================
// 血液檢查組合 (Panel) 管理
// ============================================

/// 列出啟用中的血液檢查組合（含項目）
pub async fn list_blood_test_panels(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<BloodTestPanelWithItems>>> {
    let panels = AnimalService::list_blood_test_panels(&state.db).await?;
    Ok(Json(panels))
}

/// 列出所有組合（含停用）- 與動物權限綁定：具 animal.record.view 者可取得（供血檢分析頁使用）
pub async fn list_all_blood_test_panels(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<BloodTestPanelWithItems>>> {
    require_permission!(current_user, "animal.record.view");
    let panels = AnimalService::list_all_blood_test_panels(&state.db).await?;
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
    let panel = AnimalService::create_blood_test_panel(&state.db, &req).await?;
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "PANEL_CREATE",
        Some("blood_test_panel"), Some(panel.panel.id),
        Some(&format!("建立血檢組合: {}", req.name)), None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (PANEL_CREATE): {}", e);
    }
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
    let panel = AnimalService::update_blood_test_panel(&state.db, id, &req).await?;
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "PANEL_UPDATE",
        Some("blood_test_panel"), Some(id),
        Some(&format!("更新血檢組合: {}", panel.panel.name)), None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (PANEL_UPDATE): {}", e);
    }
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
    let panel = AnimalService::update_blood_test_panel_items(&state.db, id, &req).await?;
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "PANEL_UPDATE",
        Some("blood_test_panel"), Some(id),
        Some(&format!("更新血檢組合項目: {}", panel.panel.name)), None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (PANEL_UPDATE items): {}", e);
    }
    Ok(Json(panel))
}

/// 刪除血液檢查組合（停用）
pub async fn delete_blood_test_panel(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.blood_test_template.manage");
    AnimalService::delete_blood_test_panel(&state.db, id).await?;
    let panel_name = sqlx::query_scalar::<_, String>("SELECT name FROM blood_test_panels WHERE id = $1")
        .bind(id).fetch_optional(&state.db).await.ok().flatten().unwrap_or_else(|| id.to_string());
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "PANEL_DELETE",
        Some("blood_test_panel"), Some(id),
        Some(&format!("停用血檢組合: {}", panel_name)), None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (PANEL_DELETE): {}", e);
    }
    Ok(Json(serde_json::json!({ "message": "Panel deactivated successfully" })))
}

// ============================================
// 血液檢查常用組合 (Preset) 管理
// ============================================

/// 列出啟用中的常用組合（供分析頁使用）
pub async fn list_blood_test_presets(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<BloodTestPreset>>> {
    let presets = AnimalService::list_blood_test_presets(&state.db).await?;
    Ok(Json(presets))
}

/// 列出所有常用組合（含停用）- 與動物權限綁定：具 animal.record.view 者可取得（供血檢分析頁使用）
pub async fn list_all_blood_test_presets(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<BloodTestPreset>>> {
    require_permission!(current_user, "animal.record.view");
    let presets = AnimalService::list_all_blood_test_presets(&state.db).await?;
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
    let preset = AnimalService::create_blood_test_preset(&state.db, &req).await?;
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "PRESET_CREATE",
        Some("blood_test_preset"), Some(preset.id),
        Some(&format!("建立常用組合: {}", req.name)), None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (PRESET_CREATE): {}", e);
    }
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
    let preset = AnimalService::update_blood_test_preset(&state.db, id, &req).await?;
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "PRESET_UPDATE",
        Some("blood_test_preset"), Some(id),
        Some(&format!("更新常用組合: {}", preset.name)), None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (PRESET_UPDATE): {}", e);
    }
    Ok(Json(preset))
}

/// 刪除常用組合（軟刪除）
pub async fn delete_blood_test_preset(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.blood_test_template.manage");
    AnimalService::delete_blood_test_preset(&state.db, id).await?;
    let preset_name = sqlx::query_scalar::<_, String>("SELECT name FROM blood_test_presets WHERE id = $1")
        .bind(id).fetch_optional(&state.db).await.ok().flatten().unwrap_or_else(|| id.to_string());
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "PRESET_DELETE",
        Some("blood_test_preset"), Some(id),
        Some(&format!("停用常用組合: {}", preset_name)), None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (PRESET_DELETE): {}", e);
    }
    Ok(Json(serde_json::json!({ "message": "Preset deactivated successfully" })))
}
