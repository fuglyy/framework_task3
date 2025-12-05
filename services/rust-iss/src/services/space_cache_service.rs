use crate::domain::models::{AppState, Health};
use crate::domain::errors::AppError;
use crate::clients::nasa_client;
use crate::repo::iss_repo; // использует write_cache
use serde_json::Value;
use sqlx::PgPool;

// --- APOD
pub async fn fetch_and_cache_apod(st: &AppState) -> Result<(), AppError> {
    let json = nasa_client::fetch_apod(&st.config.nasa_key).await?;
    iss_repo::write_cache(&st.pool, "apod", &json).await
}

// --- NeoWs
pub async fn fetch_and_cache_neo(st: &AppState) -> Result<(), AppError> {
    let json = nasa_client::fetch_neo_feed(&st.config.nasa_key).await?;
    iss_repo::write_cache(&st.pool, "neo", &json).await
}

// --- DONKI
pub async fn fetch_and_cache_donki(st: &AppState) -> Result<(), AppError> {
    let _ = fetch_and_cache_donki_flr(st).await;
    let _ = fetch_and_cache_donki_cme(st).await;
    Ok(())
}

pub async fn fetch_and_cache_donki_flr(st: &AppState) -> Result<(), AppError> {
    let json = nasa_client::fetch_donki_flr(&st.config.nasa_key).await?;
    iss_repo::write_cache(&st.pool, "flr", &json).await
}

pub async fn fetch_and_cache_donki_cme(st: &AppState) -> Result<(), AppError> {
    let json = nasa_client::fetch_donki_cme(&st.config.nasa_key).await?;
    iss_repo::write_cache(&st.pool, "cme", &json).await
}

// --- SpaceX (нужен отдельный клиент в `clients/`)
pub async fn fetch_spacex_next() -> Result<Value, AppError> {
    let url = "https://api.spacexdata.com/v4/launches/next";
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(30)).build()?;
    let json: Value = client.get(url).send().await?.json().await?;
    Ok(json)
}

pub async fn fetch_and_cache_spacex(st: &AppState) -> Result<(), AppError> {
    let json = fetch_spacex_next().await?;
    iss_repo::write_cache(&st.pool, "spacex", &json).await
}

// --- Summary (переместим сюда, т.к. работа с кэшем)
async fn latest_from_cache(pool: &sqlx::PgPool, src: &str) -> Value {
    iss_repo::get_latest_from_cache(pool, src).await.ok().flatten()
        .unwrap_or(serde_json::json!({}))
}

pub async fn get_summary(st: &AppState, osdr_count: i64) -> Result<Value, AppError> {
    let apod = latest_from_cache(&st.pool, "apod").await;
    let neo = latest_from_cache(&st.pool, "neo").await;
    let flr = latest_from_cache(&st.pool, "flr").await;
    let cme = latest_from_cache(&st.pool, "cme").await;
    let spacex = latest_from_cache(&st.pool, "spacex").await;

    let iss_last = crate::repo::iss_repo::get_last_log(&st.pool).await.ok().flatten()
        .map(|log| serde_json::json!({"at": log.fetched_at, "payload": log.payload}))
        .unwrap_or(serde_json::json!({}));

    Ok(serde_json::json!({
        "apod": apod, "neo": neo, "flr": flr, "cme": cme, "spacex": spacex,
        "iss": iss_last, "osdr_count": osdr_count
    }))
}

pub async fn refresh_cache(st: &AppState, sources: &[&str]) -> Vec<&'static str> {
    let mut done = Vec::new();
    for s in sources {
        match *s {
            "apod"   => { if fetch_and_cache_apod(st).await.is_ok() { done.push("apod"); } }
            "neo"    => { if fetch_and_cache_neo(st).await.is_ok() { done.push("neo"); } }
            "flr"    => { if fetch_and_cache_donki_flr(st).await.is_ok() { done.push("flr"); } }
            "cme"    => { if fetch_and_cache_donki_cme(st).await.is_ok() { done.push("cme"); } }
            "spacex" => { if fetch_and_cache_spacex(st).await.is_ok() { done.push("spacex"); } }
            _ => {}
        }
    }
    done
}