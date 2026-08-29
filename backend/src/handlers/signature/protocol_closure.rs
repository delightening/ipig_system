//! 結案雙簽 handlers（設計 A）。
//!
//! 設計文件：`docs/design/protocol-sd-close-and-handover.md` §5.6 / §5.7
//!
//! # 🔴 三條實作紅線（§5.7，缺一不可）
//!
//! 1. **絕對不要為了讓 PI 簽結案而放寬 `check_protocol_signing_authority`。**
//!    那個函式是**核准**路徑的守衛（僅 IACUC 主席／秘書／admin 可簽），
//!    動它就是動 21 CFR §11.10(g) 的合規路徑，屬 CLAUDE.md 必問紅線。
//!    本模組完全不呼叫它——結案簽章走獨立的 `entity_type`，兩條路徑互不干擾。
//!
//! 2. **新端點必須有自己的權責檢查**，不是「沒有守衛」。
//!    `SignatureService::sign_record_tx` 不驗權責（§3.3），
//!    `closure::sign_closure` 也不驗（它的 doc comment 明寫由呼叫端負責）。
//!    唯一驗的地方就是這裡。
//!
//! 3. **兩支端點分開**，不做成「依角色自動判斷要寫哪一欄」的單一端點。
//!    分開才能讓每支各自只有一種權責檢查、各自只寫一欄，
//!    也才擋得住「PI 誤觸 SD 那一簽」。
//!
//! # 為什麼「PI 不得簽自己的核准」與「PI 必須簽自己的結案」可以並存
//!
//! 兩者管的是**不同的簽章**：前者是審查方對申請方的把關（`entity_type='protocol'`，
//! meaning=approval），後者是申請方對「試驗確實做完了」的具結
//! （`entity_type='protocol_closure'`，meaning=responsibility）。
//!
//! ⚠️ 這個「不衝突」**完全依賴 entity_type 分離**（裁定 5）。若哪天有人把結案簽章
//! 改回共用 `'protocol'`，就會出現「同一個 entity_type 底下有些簽章 PI 能簽、
//! 有些不行」，權責檢查得改成依 `signature_type` 分流——**那才是真的動到合規路徑**。

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::Protocol,
    services::ClosureSigner,
    AppError, AppState, Result,
};

use super::SignRecordRequest;

/// 讀出計畫並檢查「這個人是不是該簽這一邊的人」。
///
/// ⚠️ 這裡的讀取**不加鎖**，`sign_closure` 內會再以 `FOR UPDATE` 讀一次權威值。
/// 也就是說這一關是「快速回饋 + 明確錯誤訊息」，真正的把關在 tx 內。
/// 兩處都要有：只有這裡會有 TOCTOU；只有 tx 內則使用者拿到的錯誤訊息會很模糊。
async fn authorize_closure_signer(
    state: &AppState,
    protocol_id: Uuid,
    user: &CurrentUser,
    signer: ClosureSigner,
) -> Result<Protocol> {
    let protocol = sqlx::query_as::<_, Protocol>("SELECT * FROM protocols WHERE id = $1")
        .bind(protocol_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到計劃書".into()))?;

    let expected = match signer {
        ClosureSigner::Pi => Some(protocol.pi_user_id),
        ClosureSigner::StudyDirector => protocol.study_director_user_id,
    };

    // ⚠️ **沒有 admin escape。** admin 要繞過雙簽走 §5.5 的旁路
    //（強制填理由 + 專屬 audit action），那條路比讓 admin 直接代簽更可稽核。
    // 在這裡開 admin 例外等於讓 admin 可以「以 PI 的身分」留下一張簽章，
    // 而簽章的意義是具結——代簽的簽章在稽核上是沒有價值的。
    match expected {
        Some(uid) if uid == user.id => Ok(protocol),
        Some(_) => Err(AppError::Forbidden(format!(
            "只有本計畫的{}可以簽這一欄。",
            match signer {
                ClosureSigner::Pi => "計畫主持人（PI）",
                ClosureSigner::StudyDirector => "計劃負責人（Study Director）",
            }
        ))),
        None => Err(AppError::BusinessRule(
            "本計畫尚未指派計劃負責人（Study Director），無法進行結案雙簽。\
             請先請執行秘書指派。"
                .into(),
        )),
    }
}

/// PI 簽結案
#[utoipa::path(
    post,
    path = "/api/v1/signatures/protocol-closure/{id}/pi",
    request_body = SignRecordRequest,
    responses(
        (status = 200, description = "簽章成功；若此為第二簽，計畫同時轉為已結案"),
        (status = 400, description = "請提供密碼或手寫簽名"),
        (status = 403, description = "不是本計畫的 PI"),
        (status = 404, description = "找不到計劃書"),
        (status = 422, description = "狀態不允許簽結案，或已重複簽署")
    ),
    params(("id" = Uuid, Path, description = "計劃書 ID")),
    tag = "電子簽章",
    security(("bearer" = []))
)]
pub async fn sign_protocol_closure_pi(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(protocol_id): Path<Uuid>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<Protocol>> {
    authorize_closure_signer(&state, protocol_id, &current_user, ClosureSigner::Pi).await?;

    let actor = ActorContext::User(current_user.clone());
    let updated = crate::services::protocol_closure_sign(
        &state.db,
        &actor,
        protocol_id,
        ClosureSigner::Pi,
        current_user.id,
        req.password.as_deref(),
        req.handwriting_svg.as_deref(),
        req.stroke_data.as_ref(),
    )
    .await?;

    Ok(Json(updated))
}

/// SD（計劃負責人）簽結案
///
/// ⚠️ **不檢查 `aup.protocol.close_own`**（設計文件 §5.6）。SD 是
/// `EXPERIMENT_STAFF`，該角色原本就沒有那個權限；要求它等於回到 R98-1 的
/// (a) 路線，而本案已否定該路線。這裡改用**身分即授權**：
/// `actor.id == protocol.study_director_user_id`，與 `disposal.rs` 的申請人簽章同構。
#[utoipa::path(
    post,
    path = "/api/v1/signatures/protocol-closure/{id}/sd",
    request_body = SignRecordRequest,
    responses(
        (status = 200, description = "簽章成功；若此為第二簽，計畫同時轉為已結案"),
        (status = 400, description = "請提供密碼或手寫簽名"),
        (status = 403, description = "不是本計畫的計劃負責人"),
        (status = 404, description = "找不到計劃書"),
        (status = 422, description = "狀態不允許簽結案，或已重複簽署")
    ),
    params(("id" = Uuid, Path, description = "計劃書 ID")),
    tag = "電子簽章",
    security(("bearer" = []))
)]
pub async fn sign_protocol_closure_sd(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(protocol_id): Path<Uuid>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<Protocol>> {
    authorize_closure_signer(
        &state,
        protocol_id,
        &current_user,
        ClosureSigner::StudyDirector,
    )
    .await?;

    let actor = ActorContext::User(current_user.clone());
    let updated = crate::services::protocol_closure_sign(
        &state.db,
        &actor,
        protocol_id,
        ClosureSigner::StudyDirector,
        current_user.id,
        req.password.as_deref(),
        req.handwriting_svg.as_deref(),
        req.stroke_data.as_ref(),
    )
    .await?;

    Ok(Json(updated))
}
