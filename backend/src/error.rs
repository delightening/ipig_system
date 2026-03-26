use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum::extract::rejection::JsonRejection;
use serde_json::json;
use utoipa::ToSchema;

pub type Result<T> = std::result::Result<T, AppError>;

/// API 錯誤回應格式（供 Swagger 文件展示）
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ErrorResponse {
    /// 錯誤資訊
    pub error: ErrorDetail,
}

/// 錯誤詳細資訊
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ErrorDetail {
    /// 錯誤訊息
    pub message: String,
    /// HTTP 狀態碼
    pub code: u16,
    /// 是否阻斷操作
    pub blocking: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Authentication required")]
    Unauthorized,

    /// 登入失敗（帳號或密碼錯誤），回傳 401
    #[error("{0}")]
    InvalidCredentials(String),

    #[error("Permission denied: {0}")]
    Forbidden(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Duplicate warning: {message}")]
    DuplicateWarning {
        message: String,
        existing_animals: Vec<serde_json::Value>,
    },

    #[error("Business rule violation: {0}")]
    BusinessRule(String),

    #[error("Too many requests: {0}")]
    TooManyRequests(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl From<rust_xlsxwriter::XlsxError> for AppError {
    fn from(err: rust_xlsxwriter::XlsxError) -> Self {
        AppError::Internal(format!("Excel generation error: {}", err))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // DuplicateWarning 需要特殊的 JSON 回應格式
        if let AppError::DuplicateWarning { message, existing_animals } = &self {
            let body = Json(json!({
                "error": {
                    "message": message,
                    "code": 409,
                    "blocking": false,
                    "warning_type": "duplicate_ear_tag",
                    "existing_animals": existing_animals
                }
            }));
            return (StatusCode::CONFLICT, body).into_response();
        }

        let (status, error_message) = match &self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::InvalidCredentials(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::DuplicateWarning { .. } => {
                // 已在前面 if let 提前 return，理論上不可達；
                // 但為避免未來重構遺漏，回傳安全預設值而非 panic
                (StatusCode::CONFLICT, "Duplicate warning".to_string())
            }
            AppError::BusinessRule(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::TooManyRequests(msg) => (StatusCode::TOO_MANY_REQUESTS, msg.clone()),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::Database(e) => {
                tracing::error!("Database error: {}", e);
                // PoolTimedOut、PoolClosed 為暫時性資源不足，回傳 503 建議重試
                if matches!(e, sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed) {
                    let body = Json(json!({
                        "error": {
                            "message": "服務暫時忙碌，請稍後再試",
                            "code": 503,
                            "blocking": false
                        }
                    }));
                    return (
                        StatusCode::SERVICE_UNAVAILABLE,
                        [(header::RETRY_AFTER, "2")],
                        body,
                    )
                        .into_response();
                }
                // 依 DB 錯誤碼回傳正確的 HTTP 狀態碼（而非統一 500）
                let (status, user_msg) = match e {
                    sqlx::Error::Database(ref db_err) => {
                        match db_err.code().as_deref() {
                            Some("23505") => (StatusCode::CONFLICT, "資料重複，請檢查輸入欄位是否已存在相同資料".to_string()),
                            Some("23503") => (StatusCode::BAD_REQUEST, "關聯資料不存在，請確認參照的資料是否正確".to_string()),
                            Some("23502") => (StatusCode::BAD_REQUEST, "必填欄位不可為空".to_string()),
                            Some("23514") => (StatusCode::BAD_REQUEST, "資料不符合欄位限制條件".to_string()),
                            _ => (StatusCode::INTERNAL_SERVER_ERROR, "資料庫操作失敗，請稍後再試".to_string()),
                        }
                    }
                    sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, "查無相關資料".to_string()),
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, "資料庫操作失敗，請稍後再試".to_string()),
                };
                (status, user_msg)
            }
            AppError::Anyhow(e) => {
                tracing::error!("Unexpected error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error".to_string())
            }
        };

        let body = Json(json!({
            "error": {
                "message": error_message,
                "code": status.as_u16(),
                "blocking": true
            }
        }));

        (status, body).into_response()
    }
}

// 處理 JSON 反序列化錯誤
impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        tracing::warn!("JSON rejection: {}", rejection);
        let error_message = match rejection {
            JsonRejection::JsonDataError(_) => {
                "請求資料格式錯誤，請確認欄位格式是否正確".to_string()
            }
            JsonRejection::JsonSyntaxError(_) => {
                "請求內容格式有誤，請確認 JSON 語法是否正確".to_string()
            }
            JsonRejection::MissingJsonContentType(_) => {
                "缺少 Content-Type: application/json 標頭".to_string()
            }
            _ => {
                "請求解析失敗，請確認資料格式".to_string()
            }
        };
        AppError::Validation(error_message)
    }
}

// 處理 validator 驗證錯誤
impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        AppError::Validation(format!("Validation failed: {}", errors))
    }
}
