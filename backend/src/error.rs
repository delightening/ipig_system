use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum::extract::rejection::JsonRejection;
use serde_json::json;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Authentication required")]
    Unauthorized,

    #[error("Permission denied: {0}")]
    Forbidden(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Business rule violation: {0}")]
    BusinessRule(String),

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
        let (status, error_message) = match &self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::BusinessRule(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
            }
            AppError::Anyhow(e) => {
                tracing::error!("Unexpected error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error".to_string())
            }
        };

        let body = Json(json!({
            "error": {
                "message": error_message,
                "code": status.as_u16()
            }
        }));

        (status, body).into_response()
    }
}

// 處理 JSON 反序列化錯誤
impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        let error_message = match rejection {
            JsonRejection::JsonDataError(err) => {
                format!("JSON 資料格式錯誤: {}", err)
            }
            JsonRejection::JsonSyntaxError(err) => {
                format!("JSON 語法錯誤: {}", err)
            }
            JsonRejection::MissingJsonContentType(_) => {
                "缺少 Content-Type: application/json 標頭".to_string()
            }
            _ => {
                format!("JSON 解析錯誤: {}", rejection)
            }
        };
        tracing::warn!("JSON rejection: {}", error_message);
        AppError::Validation(error_message)
    }
}
