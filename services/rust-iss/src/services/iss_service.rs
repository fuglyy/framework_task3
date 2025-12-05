use crate::domain::models::{AppState, Trend, IssLog};
use crate::domain::errors::AppError;
use crate::clients::iss_client;
use crate::repo::iss_repo;

pub async fn fetch_and_store_iss(st: &AppState) -> Result<IssLog, AppError> {
    // Используем fallback_iss_url из новой AppState/AppConfig
    let url = st.config.fallback_iss_url.as_str();
    let payload = iss_client::fetch_iss_location(url).await?;
    // Передаем &Value в insert_log
    iss_repo::insert_log(&st.pool, url, &payload).await?;

    let log = iss_repo::get_last_log(&st.pool).await?
        .ok_or_else(|| AppError::NotFound("Failed to retrieve ISS log after insert".to_string()))?;

    Ok(log)
}

pub async fn get_last_iss(st: &AppState) -> Result<Option<IssLog>, AppError> {
    iss_repo::get_last_log(&st.pool).await
}

pub async fn get_iss_trend(st: &AppState) -> Result<Trend, AppError> {
    iss_repo::calculate_trend(&st.pool).await
}