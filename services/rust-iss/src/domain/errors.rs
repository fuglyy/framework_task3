use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiErrorDetail {
    pub code: &'static str, // Уникальный код ошибки (например, "UPSTREAM_403", "DB_ERROR")
    pub message: String,
    pub trace_id: String, // Для корреляции с логами
}

/// Финальный формат ответа для клиента (всегда HTTP 200)

#[derive(Serialize)]
pub struct ErrorResponse {
    pub ok: bool, // Всегда false
    pub error: ApiErrorDetail,
}
// Универсальный тип ошибок для всего приложения
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Reqwest client error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),
    #[error("Configuration Error: {0}")]
    ConfigError(String),
    #[error("Resource Not Found: {0}")]
    NotFound(String),
    #[error("Upstream API error: {0}")]
    ClientError(String, StatusCode),
    #[error("I/O error: {0}")] // <-- Добавить новую ошибку
    IoError(#[from] std::io::Error),
}

// Преобразование ошибок в Response для Axum
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let trace_id = uuid::Uuid::new_v4().to_string();

        let detail = match &self {
            AppError::Sqlx(_) => ApiErrorDetail {
                code: "DB_ERROR",
                message: "Internal database operation failed.".to_string(),
                trace_id: trace_id.clone(),
            },
            AppError::ClientError(msg, code) => ApiErrorDetail {
                code: "UPSTREAM_API_ERROR",
                message: format!("External API call failed. Status: {}. Details: {}", code, msg),
                trace_id: trace_id.clone(),
            },
            _ => ApiErrorDetail { // <--- ИСПРАВЛЕНО
                code: "INTERNAL_ERROR",
                message: self.to_string(), // Используем Display реализацию для сообщения
                trace_id: trace_id.clone(),
            },
        }; // <-- Добавлена закрывающая скобка и точка с запятой

        let (status, _error_message) = match self {
            AppError::Sqlx(e) => {
                tracing::error!("SQLx error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
            }
            AppError::ReqwestError(e) => {
                tracing::error!("Reqwest error: {}", e);
                (StatusCode::BAD_GATEWAY, format!("API request failed: {}", e))
            }
            AppError::AnyhowError(e) => {
                tracing::error!("Anyhow error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal error: {}", e))
            }
            AppError::ConfigError(e) => {
                tracing::error!("Config error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Configuration error: {}", e))
            }
            AppError::NotFound(e) => (StatusCode::NOT_FOUND, e),
            
            AppError::ClientError(msg, code) => (code, msg), // Используем код, переданный клиентом
            AppError::IoError(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("I/O Error: {}", e)),
        };

        let body = ErrorResponse {
            ok: false,
            error: detail,
        };

        (status, axum::Json(body)).into_response()
    }
}

// Упрощение преобразования из других типов ошибок
