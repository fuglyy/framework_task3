use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

// Импортируем конкретные реализации
use crate::clients::nasa_client::{create_api_client, new_nasa_client}; 
use crate::repo::osdr_repo::{new_osdr_repo, OsdrRepo}; 

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

    // 4. Создание конкретных реализаций и обертывание в Arc (DI Container)
    
    // Создание устойчивого HTTP клиента
    let api_client  = create_api_client(); 

    // Создание NASA клиента (Adapter)
    let nasa_client_impl = new_nasa_client(api_client.clone(), config.nasa_key.clone());
    let nasa_client = Arc::new(nasa_client_impl);

    // Создание OSDR репозитория (Repository)
    let osdr_repo_impl = new_osdr_repo(pool.clone());
    let osdr_repo = Arc::new(osdr_repo_impl);
    
    // !!! ИНИЦИАЛИЗАЦИЯ НОВОГО КЛИЕНТА REDIS !!!
    let legacy_pascal_client = Arc::new(
        crate::clients::legacy_pascal_client::LegacyPascalClient::new(&config.redis_url, None)
            .map_err(|e| AppError::InternalError(format!("Failed to initialize Redis client: {}", e)))?
    );
    // !!! ПЕРЕДАЧА НОВОГО КЛИЕНТА В SpaceService !!!
    let space_service = crate::services::space_service::new_space_service(
        config.clone(),
        nasa_client.clone(),
        osdr_repo.clone(),
        legacy_pascal_client.clone()
    );
    
    // Оберните в Arc один раз
    let space_service_arc = Arc::new(space_service);
    // 5. Создание AppState, внедрение зависимостей
    let state = AppState::new(
        pool.clone(), 
        config, 
        nasa_client, 
        osdr_repo,
        space_service_arc,
        legacy_pascal_client, 
        
    );

    // 6. Запуск фоновых задач (Background)
    background::spawn_background_tasks(state.clone());

    // 7. Настройка роутера (Routes)
    let app = routes::app_router(state.clone());

    // 8. Запуск сервера Axum
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", 3000)).await?;
    info!("rust_iss listening on 0.0.0.0:3000");
    axum::serve(listener, app.into_make_service()).await.map_err(|e| AppError::AnyhowError(e.into()))?;

    Ok(())
}