use axum::{extract::State, Json};
use serde_json::{json, Value};
use crate::domain::models::{AppState, Health, Trend};
use crate::domain::errors::AppError;
use crate::services::{iss_service, space_cache_service};
use chrono::Utc;

// Общая ручка для health check
pub async fn health_check() -> Json<Health> {
    Json(Health { status: "ok", now: Utc::now() })
}

// Получение последней записи ISS
pub async fn last_iss(State(st): State<AppState>) -> Result<Json<Value>, AppError> {
    let log_opt = iss_service::get_last_iss(&st).await?;
    let json_resp = log_opt.map(|log| json!({
        "id": log.id, "fetched_at": log.fetched_at, "source_url": log.source_url, "payload": log.payload
    })).unwrap_or(json!({"message":"no data"}));

    Ok(Json(json_resp))
}

// Запуск принудительного фетча ISS
pub async fn trigger_iss(State(st): State<AppState>) -> Result<Json<Value>, AppError> {
    let log = iss_service::fetch_and_store_iss(&st).await?;
    Ok(Json(json!({
        "id": log.id, "fetched_at": log.fetched_at, "source_url": log.source_url, "payload": log.payload
    })))
}

// Расчет тренда ISS
pub async fn iss_trend(State(st): State<AppState>) -> Result<Json<Trend>, AppError> {
    let trend = iss_service::get_iss_trend(&st).await?;
    Ok(Json(trend))
}

// Универсальная витрина space_cache
pub async fn space_latest(
    axum::extract::Path(src): axum::extract::Path<String>,
    State(st): State<AppState>
) -> Result<Json<Value>, AppError> {
    use sqlx::Row;
    use chrono::Utc;

    let row = sqlx::query(
        "SELECT fetched_at, payload FROM space_cache
         WHERE source = $1 ORDER BY id DESC LIMIT 1"
    ).bind(&src).fetch_optional(&st.pool).await?;

    if let Some(r) = row {
        let fetched_at: chrono::DateTime<Utc> = r.get("fetched_at");
        let payload: Value = r.get("payload");
        return Ok(Json(json!({ "source": src, "fetched_at": fetched_at, "payload": payload })));
    }
    Ok(Json(json!({ "source": src, "message":"no data" })))
}

pub async fn space_refresh(
    axum::extract::Query(q): axum::extract::Query<std::collections::HashMap<String,String>>,
    State(st): State<AppState>
) -> Result<Json<Value>, AppError> {
    let list = q.get("src").cloned().unwrap_or_else(|| "apod,neo,flr,cme,spacex".to_string());
    let sources_vec: Vec<String> = list.split(',')
        .map(|x| x.trim().to_lowercase())
        .filter(|x| !x.is_empty())
        .collect(); // <--- Сохраняем владеющий вектор здесь

    let sources: Vec<&str> = sources_vec.iter()
        .map(|s| s.as_str())
        .collect();
    let done = space_cache_service::refresh_cache(&st, &sources).await;

    Ok(Json(json!({ "refreshed": done })))
}

pub async fn space_summary(State(st): State<AppState>) -> Result<Json<Value>, AppError> {
    let osdr_count = crate::repo::osdr_repo::get_count(&st.pool).await.unwrap_or(0);
    let summary = space_cache_service::get_summary(&st, osdr_count).await?;

    Ok(Json(summary))
}