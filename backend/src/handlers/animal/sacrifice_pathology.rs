// 犧牲/安樂死 + 病理報告 Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::{CreateSacrificeRequest, PigSacrifice},
    require_permission,
    services::{AnimalService, AuditService},
    AppState, Result,
};

// ============================================
// 犧牲/安樂死記錄管理
// ============================================

/// 取得豬的犧牲記錄
pub async fn get_pig_sacrifice(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
) -> Result<Json<Option<PigSacrifice>>> {
    let sacrifice = AnimalService::get_sacrifice(&state.db, pig_id).await?;
    Ok(Json(sacrifice))
}

/// 建立或更新犧牲記錄
pub async fn upsert_pig_sacrifice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
    Json(req): Json<CreateSacrificeRequest>,
) -> Result<Json<PigSacrifice>> {
    require_permission!(current_user, "animal.record.create");
    
    let sacrifice = AnimalService::upsert_sacrifice(&state.db, pig_id, &req, current_user.id).await?;

    // 取得豬隻資訊用於日誌顯示
    let method = {
        let mut methods = Vec::new();
        if req.method_electrocution { methods.push("電擊"); }
        if req.method_bloodletting { methods.push("放血"); }
        if let Some(ref other) = req.method_other { methods.push(other); }
        if methods.is_empty() { "未指定方式".to_string() } else { methods.join("+") }
    };
    let sac_display = match AnimalService::get_by_id(&state.db, pig_id).await {
        Ok(pig) => {
            let iacuc = pig.iacuc_no.as_deref().unwrap_or("未指派");
            format!("[{}] {} - {}", iacuc, pig.ear_tag, method)
        }
        _ => format!("犧牲/安樂死紀錄 (pig: {})", pig_id),
    };

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "SACRIFICE_UPSERT",
        Some("pig_sacrifice"), Some(pig_id),
        Some(&sac_display),
        None,
        Some(serde_json::json!({
            "method": method,
            "confirmed_sacrifice": req.confirmed_sacrifice,
        })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (SACRIFICE_UPSERT): {}", e);
    }

    Ok(Json(sacrifice))
}

// ============================================
// 病理報告管理
// ============================================

/// 取得豬的病理報告
pub async fn get_pig_pathology_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
) -> Result<Json<Option<crate::models::PigPathologyReport>>> {
    require_permission!(current_user, "animal.pathology.view");
    
    let report = AnimalService::get_pathology_report(&state.db, pig_id).await?;
    Ok(Json(report))
}

/// 建立或更新病理報告
pub async fn upsert_pig_pathology_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
) -> Result<Json<crate::models::PigPathologyReport>> {
    require_permission!(current_user, "animal.pathology.upload");
    
    let report = AnimalService::upsert_pathology_report(&state.db, pig_id, current_user.id).await?;

    // 取得豬隻資訊用於日誌顯示
    let path_display = match AnimalService::get_by_id(&state.db, pig_id).await {
        Ok(pig) => {
            let iacuc = pig.iacuc_no.as_deref().unwrap_or("未指派");
            format!("[{}] {}", iacuc, pig.ear_tag)
        }
        _ => format!("病理報告 (pig: {})", pig_id),
    };

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "PATHOLOGY_UPSERT",
        Some("pig_pathology"), Some(pig_id),
        Some(&path_display),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (PATHOLOGY_UPSERT): {}", e);
    }

    Ok(Json(report))
}
