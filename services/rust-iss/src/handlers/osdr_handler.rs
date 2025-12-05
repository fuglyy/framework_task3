use axum::{extract::State, Json};
use serde_json::{json, Value};
use crate::domain::models::AppState;
use crate::domain::errors::AppError;
use crate::services::osdr_service;

pub async fn osdr_sync(State(st): State<AppState>) -> Result<Json<Value>, AppError> {
    let written = osdr_service::fetch_and_store_osdr(&st).await?;
    Ok(Json(json!({ "written": written })))
}

pub async fn osdr_list(State(st): State<AppState>) -> Result<Json<Value>, AppError> {
    let items = osdr_service::get_osdr_list(&st).await?;
    let out: Vec<Value> = items.into_iter().map(|r| {
        json!({
            "id": r.id,
            "dataset_id": r.dataset_id,
            "title": r.title,
            "status": r.status,
            "updated_at": r.updated_at,
            "inserted_at": r.inserted_at,
            "raw": r.raw,
        })
    }).collect();

    Ok(Json(json!({ "items": out })))
}