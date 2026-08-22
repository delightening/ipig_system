// 猝死登記 Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{AnimalSuddenDeath, CreateSuddenDeathRequest},
    require_permission,
    services::{access, AnimalMedicalService, AnimalService},
    AppState, Result,
};

/// 取得動物的猝死記錄
pub async fn get_animal_sudden_death(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<Option<AnimalSuddenDeath>>> {
    // R70-3: 猝死為免計畫紀錄，允許未指派計畫的動物存取
    access::require_animal_read_access(&state.db, &current_user, animal_id).await?;
    let record = AnimalMedicalService::get_sudden_death(&state.db, animal_id).await?;
    Ok(Json(record))
}

/// 登記猝死
pub async fn create_animal_sudden_death(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<CreateSuddenDeathRequest>,
) -> Result<Json<AnimalSuddenDeath>> {
    require_permission!(current_user, "animal.record.create");
    // R75-2: 對齊 get 的免計畫紀錄 IDOR 防護（原 create 缺擁有權守衛）
    access::require_animal_read_access(&state.db, &current_user, animal_id).await?;

    // Audit 已收進 service 層（SUDDEN_DEATH，tx 內 + animal 狀態變更同 tx）
    let actor = ActorContext::User(current_user.clone());
    let record =
        AnimalMedicalService::create_sudden_death(&state.db, &actor, animal_id, &req).await?;

    // 猝死通知 → 依路由表 animal_sudden_death（角色 VET）；非阻塞，失敗不影響登記。
    let db = state.db.clone();
    let reporter = current_user.email.clone();
    let reporter_id = current_user.id;
    tokio::spawn(async move {
        let animal = match AnimalService::get_by_id(&db, animal_id).await {
            Ok(a) => a,
            Err(e) => {
                tracing::warn!("猝死通知用 animal fetch 失敗: {e}");
                return;
            }
        };
        let svc = crate::services::NotificationService::new(db);
        if let Err(e) = svc
            .notify_sudden_death(
                animal_id,
                &animal.ear_tag,
                animal.iacuc_no.as_deref(),
                &reporter,
                Some(reporter_id),
            )
            .await
        {
            tracing::warn!("發送動物猝死通知失敗: {e}");
        }
    });

    Ok(Json(record))
}
