use axum::{extract::State, Json, extract::Query};
use serde::Deserialize;

use crate::{
    models::AuditLogQuery,
    services::AuditService,
    AppState, Result,
};

#[derive(Debug, Deserialize)]
pub struct AuditLogQueryParams {
    pub entity_type: Option<String>,
    pub action: Option<String>,
    pub actor_user_id: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

/// 獲取審計日誌列表
pub async fn list_audit_logs(
    State(state): State<AppState>,
    Query(params): Query<AuditLogQueryParams>,
) -> Result<Json<Vec<crate::models::AuditLogWithActor>>> {
    let query = AuditLogQuery {
        entity_type: params.entity_type,
        entity_id: None,
        action: params.action,
        actor_user_id: params.actor_user_id.and_then(|s| s.parse().ok()),
        start_date: params.start_date.and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
        end_date: params.end_date.and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
    };
    
    let logs = AuditService::list(&state.db, &query).await?;
    Ok(Json(logs))
}
