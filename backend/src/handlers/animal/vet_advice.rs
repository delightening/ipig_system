// 獸醫師建議 Handlers
// - 舊版結構化表單（保留，未來巡場報告用）
// - 新版多筆紀錄 CRUD

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::{ActorContext, CurrentUser},
    require_permission,
    services::{
        access, AnimalVetAdvice, AnimalVetAdviceService, UpsertVetAdviceRequest, VetAdviceRecord,
        VetAdviceRecordService,
    },
    AppState, Result,
};

// ── 舊版結構化表單 ──────────────────────────────

/// 取得動物的獸醫師建議（結構化）
pub async fn get_animal_vet_advice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<Option<AnimalVetAdvice>>> {
    // SEC-IDOR: 驗證使用者是否有權存取該動物（透過計畫成員資格）
    let scope =
        access::Scoped::<access::AnimalRead>::authorize(&state.db, &current_user, animal_id)
            .await?;
    let advice = AnimalVetAdviceService::get_by_animal(&state.db, scope).await?;
    Ok(Json(advice))
}

/// 新增或更新動物的獸醫師建議（結構化）
pub async fn upsert_animal_vet_advice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<UpsertVetAdviceRequest>,
) -> Result<Json<AnimalVetAdvice>> {
    // SEC-IDOR: 寫入操作需權限 + 動物歸屬檢查（原先缺漏）
    require_permission!(current_user, "animal.vet.recommend");
    let scope =
        access::Scoped::<access::AnimalWrite>::authorize(&state.db, &current_user, animal_id)
            .await?;

    let actor = ActorContext::User(current_user.clone());
    let advice = AnimalVetAdviceService::upsert(&state.db, &actor, scope, &req).await?;
    Ok(Json(advice))
}

// ── 新版多筆紀錄 CRUD ──────────────────────────────

/// 列出動物的獸醫師建議紀錄
pub async fn list_vet_advice_records(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<Vec<VetAdviceRecord>>> {
    // SEC-IDOR: 驗證使用者是否有權存取該動物（透過計畫成員資格）
    let scope =
        access::Scoped::<access::AnimalRead>::authorize(&state.db, &current_user, animal_id)
            .await?;
    let records = VetAdviceRecordService::list(&state.db, scope).await?;
    Ok(Json(records))
}

// create/update/delete_vet_advice_record 已移除：獸醫師建議改為唯讀，
// 內容由巡場報告「完成」時自動歸位（VetPatrolReportService，單一來源、單向同步）。
