use axum::{routing::get, Router};
use crate::domain::models::AppState;
use crate::handlers::osdr_handler;

pub fn osdr_routes() -> Router<AppState> {
    Router::new()
        .route("/osdr/sync", get(osdr_handler::osdr_sync))
        .route("/osdr/list", get(osdr_handler::osdr_list))
}