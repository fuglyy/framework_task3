use axum::{extract::State, Json};
use serde_json::{json, Value};
use tracing::warn;
use crate::domain::models::{AppState, Health, Trend};
use crate::domain::errors::AppError;
use chrono::Utc;

// Импортируем функции из space_service
use crate::services::space_service::{get_last_iss, get_iss_trend};

// Общая ручка для health check
pub async fn health_check() -> Json<Health> {
    Json(Health { status: "ok", now: Utc::now() })
}

// Получение последней записи ISS
pub async fn last_iss(State(st): State<AppState>) -> Result<Json<Value>, AppError> {
    let log_opt = get_last_iss(&st).await?;
    let json_resp = log_opt.unwrap_or(json!({"message":"no data"}));
    Ok(Json(json_resp))
}

// Запуск принудительного фетча ISS
pub async fn trigger_iss(State(st): State<AppState>) -> Result<Json<Value>, AppError> {
    // Используем метод из сервиса
    st.space_service.fetch_and_store_iss().await?;
    Ok(Json(json!({
        "message": "ISS fetch triggered"
    })))
}

// Расчет тренда ISS
pub async fn iss_trend(State(st): State<AppState>) -> Result<Json<Trend>, AppError> {
    let trend = get_iss_trend(&st).await?;
    Ok(Json(trend))
}

// Универсальная витрина space_cache
pub async fn space_latest(
    axum::extract::Path(src): axum::extract::Path<String>,
    State(st): State<AppState>
) -> Result<Json<Value>, AppError> {
    use sqlx::Row;
    
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
        .collect();

    let mut refreshed = 0;
    
    for source in sources_vec.iter() {
        match source.as_str() {
            "apod" => {
                st.space_service.fetch_and_cache_apod().await?;
                refreshed += 1;
            }
            "neo" => {
                st.space_service.fetch_and_cache_neo().await?;
                refreshed += 1;
            }
            "flr" | "cme" => {
                st.space_service.fetch_and_cache_donki().await?;
                refreshed += 1;
            }
            "spacex" => {
                st.space_service.fetch_and_cache_spacex().await?;
                refreshed += 1;
            }
            _ => warn!("Unknown source: {}", source),
        }
    }

    Ok(Json(json!({ "refreshed": refreshed })))
}

pub async fn space_summary(State(st): State<AppState>) -> Result<Json<Value>, AppError> {
    let osdr_count = st.osdr_repo.get_count().await.unwrap_or(0);
    
    // Получаем позицию МКС
    let iss_position = st.space_service.get_iss_position().await.ok();
    
    Ok(Json(json!({
        "osdr_count": osdr_count,
        "iss_position": iss_position,
        "status": "ok"
    })))
}