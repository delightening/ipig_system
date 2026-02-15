// 犧牲/安樂死 + 病理報告 Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::{CreateSacrificeRequest, AnimalSacrifice},
    require_permission,
    services::{AnimalService, AuditService},
    AppState, Result,
};

// ============================================
// 犧牲/安樂死記錄管理
// ============================================

/// 取得動物的犧牲記錄
pub async fn get_animal_sacrifice(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<Option<AnimalSacrifice>>> {
    let sacrifice = AnimalService::get_sacrifice(&state.db, animal_id).await?;
    Ok(Json(sacrifice))
}

/// 建立或更新犧牲記錄
pub async fn upsert_animal_sacrifice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<CreateSacrificeRequest>,
) -> Result<Json<AnimalSacrifice>> {
    require_permission!(current_user, "animal.record.create");
    
    let sacrifice = AnimalService::upsert_sacrifice(&state.db, animal_id, &req, current_user.id).await?;

    // 取得動物資訊用於日誌顯示
    let method = {
        let mut methods = Vec::new();
        if req.method_electrocution { methods.push("電擊"); }
        if req.method_bloodletting { methods.push("放血"); }
        if let Some(ref other) = req.method_other { methods.push(other); }
        if methods.is_empty() { "未指定方式".to_string() } else { methods.join("+") }
    };
    let sac_display = match AnimalService::get_by_id(&state.db, animal_id).await {
        Ok(animal) => {
            let iacuc = animal.iacuc_no.as_deref().unwrap_or("未指派");
            format!("[{}] {} - {}", iacuc, animal.ear_tag, method)
        }
        _ => format!("犧牲/安樂死紀錄 (animal: {})", animal_id),
    };

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "SACRIFICE_UPSERT",
        Some("animal_sacrifice"), Some(animal_id),
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

/// 取得動物的病理報告
pub async fn get_animal_pathology_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<Option<crate::models::AnimalPathologyReport>>> {
    require_permission!(current_user, "animal.pathology.view");
    
    let report = AnimalService::get_pathology_report(&state.db, animal_id).await?;
    Ok(Json(report))
}

/// 建立或更新病理報告
pub async fn upsert_animal_pathology_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<crate::models::AnimalPathologyReport>> {
    require_permission!(current_user, "animal.pathology.upload");
    
    let report = AnimalService::upsert_pathology_report(&state.db, animal_id, current_user.id).await?;

    // 取得動物資訊用於日誌顯示
    let path_display = match AnimalService::get_by_id(&state.db, animal_id).await {
        Ok(animal) => {
            let iacuc = animal.iacuc_no.as_deref().unwrap_or("未指派");
            format!("[{}] {}", iacuc, animal.ear_tag)
        }
        _ => format!("病理報告 (animal: {})", animal_id),
    };

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "PATHOLOGY_UPSERT",
        Some("animal_pathology"), Some(animal_id),
        Some(&path_display),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (PATHOLOGY_UPSERT): {}", e);
    }

    Ok(Json(report))
}
