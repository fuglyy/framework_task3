use tracing::{info, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

// Импорты модулей
mod config;
mod domain;
mod repo;
mod clients;
mod services;
mod handlers;
mod routes;
mod utils;
mod background;

use domain::models::AppState;
use domain::errors::AppError;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // 1. Настройка логирования (Trace)
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    // 2. Загрузка конфигурации
        
    let config = crate::config::load_config()?;
    
    // 3. Инициализация пула БД и схем (Repo/Pool)
    let pool = repo::pg_pool::init_pool().await?;
    repo::pg_pool::init_db(&pool).await?;

    // 4. Создание AppState
    let state = AppState::new(pool.clone(), config);

    // 5. Запуск фоновых задач (Background)
    background::spawn_background_tasks(state.clone());

    // 6. Настройка роутера (Routes)
    let app = routes::app_router(state.clone());

    // 7. Запуск сервера Axum
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", 3000)).await?;
    info!("rust_iss listening on 0.0.0.0:3000");
    axum::serve(listener, app.into_make_service()).await.map_err(|e| AppError::AnyhowError(e.into()))?;

    Ok(())
}