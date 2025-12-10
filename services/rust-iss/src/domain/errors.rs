use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use uuid::Uuid;
use thiserror::Error;

#[derive(Serialize)]
pub struct ApiErrorDetail {
    pub code: &'static str,
    pub message: String,
    pub trace_id: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub ok: bool,
    pub error: ApiErrorDetail,
}

// Универсальный тип ошибок для всего приложения
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    
    #[error("Reqwest client error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Reqwest middleware error: {0}")]
    ReqwestMiddlewareError(#[from] reqwest_middleware::Error),
    
    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),
    
    #[error("Configuration Error: {0}")]
    ConfigError(String),
    
    #[error("Resource Not Found: {0}")]
    NotFound(String),
    
    #[error("Upstream API error: {0}")]
    ClientError(String, StatusCode),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Internal error: {0}")]
    InternalError(String),
    
    #[error("External service error: {0}")]
    ExternalServiceError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("External API error from {api_name}: {source}")]
    ExternalApiError {
        source: reqwest_middleware::Error,
        api_name: String,
    },
    
    #[error("External API returned error status: {status}")]
    ExternalApiStatusError {
        status: StatusCode,
        api_name: String,
        message: String,
    },
    
    #[error("Deserialization error: {message} - {source}")]
    DeserializationError {
        source: reqwest::Error,
        message: String,
    },
}

// Преобразование ошибок в Response для Axum
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let trace_id = Uuid::new_v4().to_string();
        
        let (status, error_detail) = match &self {
            AppError::Sqlx(e) => {
                tracing::error!("SQLx error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, ApiErrorDetail {
                    code: "DB_ERROR",
                    message: "Internal database operation failed.".to_string(),
                    trace_id: trace_id.clone(),
                })
            }
            AppError::ReqwestError(e) => {
                tracing::error!("Reqwest error: {}", e);
                (StatusCode::BAD_GATEWAY, ApiErrorDetail {
                    code: "UPSTREAM_API_ERROR",
                    message: format!("API request failed: {}", e),
                    trace_id: trace_id.clone(),
                })
            }
            AppError::ReqwestMiddlewareError(e) => {
                tracing::error!("Reqwest middleware error: {}", e);
                (StatusCode::BAD_GATEWAY, ApiErrorDetail {
                    code: "MIDDLEWARE_ERROR",
                    message: format!("Middleware error: {}", e),
                    trace_id: trace_id.clone(),
                })
            }
            AppError::AnyhowError(e) => {
                tracing::error!("Anyhow error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, ApiErrorDetail {
                    code: "INTERNAL_ERROR",
                    message: format!("Internal error: {}", e),
                    trace_id: trace_id.clone(),
                })
            }
            AppError::ConfigError(e) => {
                tracing::error!("Config error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, ApiErrorDetail {
                    code: "CONFIG_ERROR",
                    message: format!("Configuration error: {}", e),
                    trace_id: trace_id.clone(),
                })
            }
            AppError::NotFound(e) => {
                (StatusCode::NOT_FOUND, ApiErrorDetail {
                    code: "NOT_FOUND",
                    message: e.clone(),
                    trace_id: trace_id.clone(),
                })
            }
            AppError::ClientError(msg, code) => {
                (*code, ApiErrorDetail {
                    code: "CLIENT_ERROR",
                    message: format!("Client error: {} (Status: {})", msg, code),
                    trace_id: trace_id.clone(),
                })
            }
            AppError::IoError(e) => {
                tracing::error!("I/O error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, ApiErrorDetail {
                    code: "IO_ERROR",
                    message: format!("I/O error: {}", e),
                    trace_id: trace_id.clone(),
                })
            }
            AppError::InternalError(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, ApiErrorDetail {
                    code: "INTERNAL_ERROR",
                    message: msg.clone(),
                    trace_id: trace_id.clone(),
                })
            }
            AppError::ExternalServiceError(msg) => {
                tracing::error!("External service error: {}", msg);
                (StatusCode::BAD_GATEWAY, ApiErrorDetail {
                    code: "EXTERNAL_SERVICE_ERROR",
                    message: msg.clone(),
                    trace_id: trace_id.clone(),
                })
            }
            AppError::SerializationError(msg) => {
                tracing::error!("Serialization error: {}", msg);
                (StatusCode::BAD_REQUEST, ApiErrorDetail {
                    code: "SERIALIZATION_ERROR",
                    message: msg.clone(),
                    trace_id: trace_id.clone(),
                })
            }
            AppError::ExternalApiError { source: e, api_name } => {
                tracing::error!("External API '{}' error: {}", api_name, e);
                (StatusCode::BAD_GATEWAY, ApiErrorDetail {
                    code: "EXTERNAL_API_ERROR",
                    message: format!("External API '{}' error", api_name),
                    trace_id: trace_id.clone(),
                })
            }
            AppError::ExternalApiStatusError { status, api_name, message } => {
                tracing::error!("External API '{}' status error {}: {}", api_name, status, message);
                (*status, ApiErrorDetail {
                    code: "EXTERNAL_API_STATUS_ERROR",
                    message: format!("External API '{}' returned error: {}", api_name, message),
                    trace_id: trace_id.clone(),
                })
            }
            AppError::DeserializationError { source: e, message } => {
                tracing::error!("Deserialization error: {} - {}", message, e);
                (StatusCode::BAD_REQUEST, ApiErrorDetail {
                    code: "DESERIALIZATION_ERROR",
                    message: message.clone(),
                    trace_id: trace_id.clone(),
                })
            }
        };

        let body = ErrorResponse {
            ok: false,
            error: error_detail,
        };

        (StatusCode::OK, Json(body)).into_response()
    }
}