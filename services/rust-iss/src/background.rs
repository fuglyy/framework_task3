use crate::domain::models::AppState;
use crate::services::{iss_service, osdr_service, space_cache_service};
use tokio::time::{sleep, Duration};
use tracing::error;

pub fn spawn_background_tasks(state: AppState) {
    // фон OSDR
    {
        let st = state.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = osdr_service::fetch_and_store_osdr(&st).await { error!("osdr err {e:?}") }
                sleep(Duration::from_secs(st.config.every_osdr)).await;
            }
        });
    }
    // фон ISS
    {
        let st = state.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = iss_service::fetch_and_store_iss(&st).await { error!("iss err {e:?}") }
                sleep(Duration::from_secs(st.config.every_iss)).await;
            }
        });
    }
    // фон APOD
    {
        let st = state.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = space_cache_service::fetch_and_cache_apod(&st).await { error!("apod err {e:?}") }
                sleep(Duration::from_secs(st.config.every_apod)).await;
            }
        });
    }
    // фон NeoWs
    {
        let st = state.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = space_cache_service::fetch_and_cache_neo(&st).await { error!("neo err {e:?}") }
                sleep(Duration::from_secs(st.config.every_neo)).await;
            }
        });
    }
    // фон DONKI
    {
        let st = state.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = space_cache_service::fetch_and_cache_donki(&st).await { error!("donki err {e:?}") }
                sleep(Duration::from_secs(st.config.every_donki)).await;
            }
        });
    }
    // фон SpaceX
    {
        let st = state.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = space_cache_service::fetch_and_cache_spacex(&st).await { error!("spacex err {e:?}") }
                sleep(Duration::from_secs(st.config.every_spacex)).await;
            }
        });
    }
}