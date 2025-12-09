use axum::{extract::State, Json};
use serde_json::{json, Value};
use crate::domain::models::AppState;
use crate::domain::errors::AppError;
//use crate::services::osdr_service;

pub async fn osdr_sync(State(st): State<AppState>) -> Result<Json<Value>, AppError> {
    let written = st.space_service.fetch_and_save_osdr_data().await?;
    Ok(Json(json!({ "written": written })))
}

pub async fn osdr_list(State(st): State<AppState>) -> Result<Json<Value>, AppError> {
    let items = st.space_service.get_osdr_list().await?;
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