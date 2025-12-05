use axum::{routing::get, Router};
use crate::domain::models::AppState;
use crate::handlers::iss_handler;

pub fn iss_routes() -> Router<AppState> {
    Router::new()
        .route("/last", get(iss_handler::last_iss))
        .route("/fetch", get(iss_handler::trigger_iss))
        .route("/iss/trend", get(iss_handler::iss_trend))
        // Space cache
        .route("/space/:src/latest", get(iss_handler::space_latest))
        .route("/space/refresh", get(iss_handler::space_refresh))
        .route("/space/summary", get(iss_handler::space_summary))
}