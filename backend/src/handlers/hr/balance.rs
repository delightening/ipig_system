// 假勤餘額管理 Handlers

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::{
        AdjustBalanceRequest, AnnualLeaveBalanceView, AnnualLeaveEntitlement, BalanceQuery,
        BalanceSummary, CompTimeBalanceView, CreateAnnualLeaveRequest, ExpiredLeaveReport,
    },
    services::HrService,
    AppState, Result,
};

/// 取得特休餘額
pub async fn get_annual_leave_balances(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<BalanceQuery>,
) -> Result<Json<Vec<AnnualLeaveBalanceView>>> {
    let user_id = params.user_id.unwrap_or(current_user.id);
    if user_id != current_user.id && !current_user.has_permission("hr.balance.view_all") {
        return Err(crate::error::AppError::Forbidden("無權查看他人餘額".to_string()));
    }
    let balances = HrService::get_annual_leave_balances(&state.db, user_id).await?;
    Ok(Json(balances))
}

/// 取得補休餘額
pub async fn get_comp_time_balances(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<BalanceQuery>,
) -> Result<Json<Vec<CompTimeBalanceView>>> {
    let user_id = params.user_id.unwrap_or(current_user.id);
    if user_id != current_user.id && !current_user.has_permission("hr.balance.view_all") {
        return Err(crate::error::AppError::Forbidden("無權查看他人餘額".to_string()));
    }
    let balances = HrService::get_comp_time_balances(&state.db, user_id).await?;
    Ok(Json(balances))
}

/// 取得餘額摘要
pub async fn get_balance_summary(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<BalanceQuery>,
) -> Result<Json<BalanceSummary>> {
    let user_id = params.user_id.unwrap_or(current_user.id);
    if user_id != current_user.id && !current_user.has_permission("hr.balance.view_all") {
        return Err(crate::error::AppError::Forbidden("無權查看他人餘額".to_string()));
    }
    let summary = HrService::get_balance_summary(&state.db, user_id).await?;
    Ok(Json(summary))
}

/// 建立特休額度
pub async fn create_annual_leave_entitlement(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateAnnualLeaveRequest>,
) -> Result<(StatusCode, Json<AnnualLeaveEntitlement>)> {
    if !current_user.is_admin()
        && !current_user.roles.contains(&crate::constants::ROLE_ADMIN_STAFF.to_string())
        && !current_user.has_permission("hr.balance.manage")
    {
        return Err(crate::error::AppError::Forbidden("僅管理員或行政可新增特休額度".to_string()));
    }
    let entitlement = HrService::create_annual_leave_entitlement(&state.db, current_user.id, &payload).await?;
    Ok((StatusCode::CREATED, Json(entitlement)))
}

/// 調整餘額
pub async fn adjust_balance(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AdjustBalanceRequest>,
) -> Result<Json<AnnualLeaveEntitlement>> {
    let entitlement = HrService::adjust_annual_leave(&state.db, id, current_user.id, &payload).await?;
    Ok(Json(entitlement))
}

/// 勞基法 §38：自動計算單一員工特休假
#[derive(serde::Deserialize)]
pub struct AutoCalcRequest {
    pub user_id: Uuid,
    pub entitlement_year: i32,
}

pub async fn auto_calculate_annual_leave(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<AutoCalcRequest>,
) -> Result<Json<serde_json::Value>> {
    if !current_user.is_admin()
        && !current_user.has_permission("hr.balance.manage")
    {
        return Err(crate::error::AppError::Forbidden("僅管理員可執行自動計算".to_string()));
    }
    let result = HrService::auto_calculate_annual_leave(
        &state.db, payload.user_id, payload.entitlement_year, current_user.id,
    ).await?;
    match result {
        Some(ent) => Ok(Json(serde_json::json!({ "created": true, "entitlement": ent }))),
        None => Ok(Json(serde_json::json!({ "created": false, "message": "已存在或年資不足" }))),
    }
}

/// 勞基法 §38：批次自動計算所有在職員工特休假
#[derive(serde::Deserialize)]
pub struct BatchAutoCalcRequest {
    pub entitlement_year: i32,
}

pub async fn batch_auto_calculate_annual_leave(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<BatchAutoCalcRequest>,
) -> Result<Json<serde_json::Value>> {
    if !current_user.is_admin()
        && !current_user.has_permission("hr.balance.manage")
    {
        return Err(crate::error::AppError::Forbidden("僅管理員可執行批次計算".to_string()));
    }
    let count = HrService::batch_auto_calculate_annual_leave(
        &state.db, payload.entitlement_year, current_user.id,
    ).await?;
    Ok(Json(serde_json::json!({ "created_count": count })))
}

/// 取得過期特休假報表
pub async fn get_expired_leave_compensation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<ExpiredLeaveReport>>> {
    if !current_user.has_permission("hr.balance.manage")
        && !current_user.is_admin()
        && !current_user.roles.contains(&crate::constants::ROLE_IACUC_STAFF.to_string())
        && !current_user.roles.contains(&crate::constants::ROLE_ADMIN_STAFF.to_string())
    {
        return Err(crate::error::AppError::Forbidden("無權查看過期特休報表".to_string()));
    }
    let reports = HrService::get_expired_leave_compensation_report(&state.db).await?;
    Ok(Json(reports))
}
