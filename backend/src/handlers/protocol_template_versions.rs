use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    handlers::upload::{handle_upload, UploadResponse},
    middleware::{ActorContext, CurrentUser},
    models::{
        CreateProtocolTemplateVersionRequest, ProtocolTemplateVersion,
        ProtocolTemplateVersionResponse, TemplateVersionDocument,
        UpdateProtocolTemplateVersionRequest,
    },
    require_permission,
    services::{FileCategory, ProtocolTemplateVersionService},
    AppState, Result,
};

const PERM: &str = "admin.protocol_template.manage";
const ENTITY_TYPE: &str = "protocol_template_version";

pub async fn list_template_versions(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<ProtocolTemplateVersionResponse>>> {
    require_permission!(current_user, PERM);
    Ok(Json(ProtocolTemplateVersionService::list(&state.db).await?))
}

pub async fn create_template_version(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateProtocolTemplateVersionRequest>,
) -> Result<(StatusCode, Json<ProtocolTemplateVersion>)> {
    require_permission!(current_user, PERM);
    req.validate()?;
    let actor = ActorContext::User(current_user);
    let v = ProtocolTemplateVersionService::create(&state.db, &actor, &req).await?;
    Ok((StatusCode::CREATED, Json(v)))
}

pub async fn update_template_version(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProtocolTemplateVersionRequest>,
) -> Result<Json<ProtocolTemplateVersion>> {
    require_permission!(current_user, PERM);
    req.validate()?;
    let actor = ActorContext::User(current_user);
    let v = ProtocolTemplateVersionService::update(&state.db, &actor, id, &req).await?;
    Ok(Json(v))
}

pub async fn set_current_template_version(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProtocolTemplateVersion>> {
    require_permission!(current_user, PERM);
    let actor = ActorContext::User(current_user);
    let v = ProtocolTemplateVersionService::set_current(&state.db, &actor, id).await?;
    Ok(Json(v))
}

pub async fn delete_template_version(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    require_permission!(current_user, PERM);
    let actor = ActorContext::User(current_user);
    ProtocolTemplateVersionService::delete(&state.db, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 上傳該版本的現行 SOP / 表單文件（multipart 屬 HTTP 層，沿用 upload 基礎建設）。
pub async fn upload_template_version_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<Vec<UploadResponse>>> {
    require_permission!(current_user, PERM);
    ProtocolTemplateVersionService::get(&state.db, id).await?; // 確認版本存在
    let results = handle_upload(
        &state.db,
        current_user.id,
        FileCategory::ProtocolAttachment,
        ENTITY_TYPE,
        &id.to_string(),
        &mut multipart,
    )
    .await?;
    Ok(Json(results))
}

pub async fn list_template_version_documents(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<TemplateVersionDocument>>> {
    require_permission!(current_user, PERM);
    Ok(Json(
        ProtocolTemplateVersionService::list_documents(&state.db, id).await?,
    ))
}

pub async fn delete_template_version_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((id, doc_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {
    require_permission!(current_user, PERM);
    let actor = ActorContext::User(current_user);
    ProtocolTemplateVersionService::delete_document(&state.db, &actor, id, doc_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
