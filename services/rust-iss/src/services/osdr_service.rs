use crate::domain::models::{AppState, OsdrItem};
use crate::domain::errors::AppError;
use crate::clients::nasa_client;
use crate::repo::osdr_repo;

pub async fn fetch_and_store_osdr(st: &AppState) -> Result<usize, AppError> {
    let items = nasa_client::fetch_osdr_list(&st.config.nasa_url).await?;

    let mut written = 0usize;
    for item in items {
        if osdr_repo::upsert_item(&st.pool, &item).await? {
            written += 1;
        }
    }
    Ok(written)
}

pub async fn get_osdr_list(st: &AppState) -> Result<Vec<OsdrItem>, AppError> {
    let limit = std::env::var("OSDR_LIST_LIMIT").ok()
        .and_then(|s| s.parse::<i64>().ok()).unwrap_or(20);

    osdr_repo::get_list(&st.pool, limit).await
}

pub async fn get_osdr_count(st: &AppState) -> Result<i64, AppError> {
    osdr_repo::get_count(&st.pool).await
}