use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

// ── DB Entity ──

#[derive(Debug, Clone, FromRow)]
pub struct ProtocolAiReview {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub protocol_version_id: Option<Uuid>,
    pub review_type: String,
    pub rule_result: Option<serde_json::Value>,
    pub ai_result: Option<serde_json::Value>,
    pub ai_model: Option<String>,
    pub ai_input_tokens: Option<i32>,
    pub ai_output_tokens: Option<i32>,
    pub total_errors: i32,
    pub total_warnings: i32,
    pub score: Option<i32>,
    pub triggered_by: Option<Uuid>,
    pub duration_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

// ── Validation Result (Level 1) ──

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidationResult {
    pub passed: Vec<String>,
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidationIssue {
    pub code: String,
    pub category: String,
    pub section: String,
    pub message: String,
    pub suggestion: String,
}

// ── AI Review Result (Level 2) ──

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AiReviewIssue {
    pub severity: String,
    pub category: String,
    pub section: String,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AiReviewAiResult {
    pub summary: String,
    pub score: Option<i32>,
    pub issues: Vec<AiReviewIssue>,
    pub passed: Vec<String>,
}

/// 執行秘書標註 flag
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StaffReviewFlag {
    pub flag_type: String,
    pub section: String,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StaffAiResult {
    pub summary: String,
    pub flags: Vec<StaffReviewFlag>,
}

// ── API Response ──

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AiReviewResponse {
    pub id: Uuid,
    pub protocol_id: Uuid,
    pub review_type: String,
    pub rule_result: Option<ValidationResult>,
    pub ai_result: Option<serde_json::Value>,
    pub ai_model: Option<String>,
    pub total_errors: i32,
    pub total_warnings: i32,
    pub score: Option<i32>,
    pub duration_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

impl From<ProtocolAiReview> for AiReviewResponse {
    fn from(r: ProtocolAiReview) -> Self {
        let rule_result = r
            .rule_result
            .and_then(|v| serde_json::from_value(v).ok());
        Self {
            id: r.id,
            protocol_id: r.protocol_id,
            review_type: r.review_type,
            rule_result,
            ai_result: r.ai_result,
            ai_model: r.ai_model,
            total_errors: r.total_errors,
            total_warnings: r.total_warnings,
            score: r.score,
            duration_ms: r.duration_ms,
            created_at: r.created_at,
        }
    }
}
