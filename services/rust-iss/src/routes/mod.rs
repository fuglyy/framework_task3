use axum::{routing::get, Router};
use crate::domain::models::AppState;
use crate::handlers::{iss_handler, osdr_handler};

pub mod iss;
pub mod osdr;

pub fn app_router(state: AppState) -> Router {
    Router::new()
        // Общее
        .route("/health", get(iss_handler::health_check))
        // Подключаем роуты модулей
        .merge(iss::iss_routes())
        .merge(osdr::osdr_routes())
        .with_state(state)
}