use axum::{
    extract::{Json, Path, Query, State},
    Extension,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{
        AcceptInvitationRequest, AcceptInvitationResponse, CreateInvitationRequest,
        CreateInvitationResponse, InvitationListQuery, InvitationResponse, PaginatedResponse,
        VerifyInvitationResponse,
    },
    require_permission,
    services::InvitationService,
    AppState, Result,
};

/// POST /api/v1/invitations — 建立邀請
pub async fn create_invitation(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<CreateInvitationRequest>,
) -> Result<Json<CreateInvitationResponse>> {
    require_permission!(user, "invitation.create");
    req.validate()?;

    let result =
        InvitationService::create(&state.db, &state.config, &req, user.id).await?;

    Ok(Json(result))
}

/// GET /api/v1/invitations — 列出邀請
pub async fn list_invitations(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Query(query): Query<InvitationListQuery>,
) -> Result<Json<PaginatedResponse<InvitationResponse>>> {
    require_permission!(user, "invitation.view");

    let result = InvitationService::list(&state.db, &state.config, &query).await?;

    Ok(Json(result))
}

/// DELETE /api/v1/invitations/:id — 撤銷邀請
pub async fn revoke_invitation(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(user, "invitation.revoke");

    InvitationService::revoke(&state.db, id).await?;

    Ok(Json(serde_json::json!({ "message": "邀請已撤銷" })))
}

/// POST /api/v1/invitations/:id/resend — 重新發送邀請
pub async fn resend_invitation(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<InvitationResponse>> {
    require_permission!(user, "invitation.resend");

    let result = InvitationService::resend(&state.db, &state.config, id).await?;

    Ok(Json(result))
}

/// GET /api/v1/invitations/verify/:token — 驗證邀請（公開）
pub async fn verify_invitation(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Json<VerifyInvitationResponse>> {
    let result = InvitationService::verify(&state.db, &token).await?;

    Ok(Json(result))
}

/// POST /api/v1/invitations/accept — 接受邀請（公開）
pub async fn accept_invitation(
    State(state): State<AppState>,
    Json(req): Json<AcceptInvitationRequest>,
) -> Result<Json<AcceptInvitationResponse>> {
    req.validate()?;

    let result =
        InvitationService::accept(&state.db, &state.config, &req).await?;

    Ok(Json(result))
}
