use crate::domain::models::AppState;
use tokio::time::{sleep, Duration};
use tracing::{info, error};

// Удаляем старые импорты модульных сервисов, так как теперь мы используем space_service

// =========================================================================================
// 1. ФОНОВЫЕ ЦИКЛЫ (Теперь используют внедренный space_service)
// =========================================================================================

async fn osdr_task(state: AppState) {
    let interval = state.config.every_osdr;
    let duration = Duration::from_secs(interval);
    info!("Starting OSDR background task with interval: {}s.", interval);
    loop {
        if let Err(e) = state.space_service.fetch_and_save_osdr_data().await { 
            error!("OSDR task failed: {:?}", e); 
        }
        sleep(duration).await;
    }
}

async fn iss_task(state: AppState) {
    let interval = state.config.every_iss;
    let duration = Duration::from_secs(interval);
    info!("Starting ISS background task with interval: {}s.", interval);
    loop {
        if let Err(e) = state.space_service.fetch_and_store_iss().await { 
            error!("ISS task failed: {:?}", e); 
        }
        sleep(duration).await;
    }
}

async fn apod_task(state: AppState) {
    let interval = state.config.every_apod;
    let duration = Duration::from_secs(interval);
    info!("Starting APOD background task with interval: {}s.", interval);
    loop {
        if let Err(e) = state.space_service.fetch_and_cache_apod().await { 
            error!("APOD task failed: {:?}", e); 
        }
        sleep(duration).await;
    }
}

async fn neo_task(state: AppState) {
    let interval = state.config.every_neo;
    let duration = Duration::from_secs(interval);
    info!("Starting NeoWs background task with interval: {}s.", interval);
    loop {
        if let Err(e) = state.space_service.fetch_and_cache_neo().await { 
            error!("NeoWs task failed: {:?}", e); 
        }
        sleep(duration).await;
    }
}

async fn donki_task(state: AppState) {
    let interval = state.config.every_donki;
    let duration = Duration::from_secs(interval);
    info!("Starting DONKI background task with interval: {}s.", interval);
    loop {
        if let Err(e) = state.space_service.fetch_and_cache_donki().await { 
            error!("DONKI task failed: {:?}", e); 
        }
        sleep(duration).await;
    }
}

async fn spacex_task(state: AppState) {
    let interval = state.config.every_spacex;
    let duration = Duration::from_secs(interval);
    info!("Starting SpaceX background task with interval: {}s.", interval);
    loop {
        if let Err(e) = state.space_service.fetch_and_cache_spacex().await { 
            error!("SpaceX task failed: {:?}", e); 
        }
        sleep(duration).await;
    }
}

// =========================================================================================
// 2. ЗАПУСК ВСЕХ ЗАДАЧ
// =========================================================================================

/// Запускает все фоновые задачи в отдельных Tokio тасках.
pub fn spawn_background_tasks(state: AppState) {
    info!("Spawning all background tasks...");
    
    // Клонируем стейт для каждой задачи
    tokio::spawn(osdr_task(state.clone()));
    tokio::spawn(iss_task(state.clone()));
    tokio::spawn(apod_task(state.clone()));
    tokio::spawn(neo_task(state.clone()));
    tokio::spawn(donki_task(state.clone()));
    tokio::spawn(spacex_task(state.clone()));
}