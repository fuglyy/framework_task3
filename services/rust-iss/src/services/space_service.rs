use std::sync::Arc;
use async_trait::async_trait;
use tracing::{error, info, instrument, warn};
use serde_json::Value;

use crate::domain::errors::AppError;
use crate::domain::contracts::{NasaClientContract, OsdrRepoContract, SpaceServiceContract};
use crate::domain::models::{AppConfig, IssPosition, OsdrItem}; // Добавьте импорты
use crate::clients::legacy_pascal_client::LegacyPascalClientContract; // !!! НОВЫЙ ИМПОРТ !!!

// =========================================================================================
// 1. СТРУКТУРА СЕРВИСА (ОРКЕСТРАТОР БИЗНЕС-ЛОГИКИ)
// =========================================================================================

/// Конкретная реализация SpaceServiceContract.
/// Хранит зависимости для доступа к данным и внешним API.
pub struct SpaceService {
    // Dependencies
    config: AppConfig, // Нужна для URL'ов
    nasa_client: Arc<dyn NasaClientContract>,
    osdr_repo: Arc<dyn OsdrRepoContract>,
    // !!! ДОБАВЛЕННЫЙ КЛИЕНТ ДЛЯ GOLANG ВОРКЕРА !!!
    legacy_pascal_client: Arc<dyn LegacyPascalClientContract>, 
}

impl SpaceService {
    /// Конструктор для создания нового экземпляра SpaceService
    pub fn new(
        config: AppConfig, 
        nasa_client: Arc<dyn NasaClientContract>, 
        osdr_repo: Arc<dyn OsdrRepoContract>,
        legacy_pascal_client: Arc<dyn LegacyPascalClientContract>
    ) -> Self {
        Self {
            config,
            nasa_client,
            osdr_repo,
            legacy_pascal_client,
        }
    }
}

/// Конструктор для внедрения зависимостей.
pub fn new_space_service(
    config: AppConfig, 
    nasa_client: Arc<dyn NasaClientContract>, 
    osdr_repo: Arc<dyn OsdrRepoContract>,
    // !!! НОВЫЙ ПАРАМЕТР В КОНСТРУКТОРЕ !!!
    legacy_pascal_client: Arc<dyn LegacyPascalClientContract>
) -> impl SpaceServiceContract {
    SpaceService::new(config, nasa_client, osdr_repo, legacy_pascal_client)
}

// =========================================================================================
// 2. РЕАЛИЗАЦИЯ КОНТРАКТА (БИЗНЕС-ЛОГИКА)
// =========================================================================================

#[async_trait]
impl SpaceServiceContract for SpaceService {

    /// -------------------------------------------------------------------------------------
    /// ОСНОВНАЯ ЗАДАЧА: OSDR (Orchestrates Client call and Repo UPSERT)
    /// -------------------------------------------------------------------------------------
    #[instrument(skip(self), level = "info")]
    async fn fetch_and_save_osdr_data(&self) -> Result<(), AppError> {
        info!("Starting fetch_and_save_osdr_data cycle...");

        let osdr_url = &self.config.nasa_url;
        
        // 1. Fetch
        let items = self.nasa_client.fetch_osdr_list(osdr_url).await?;
        info!("Successfully fetched {} items from OSDR API.", items.len());

        // 2. Save/Upsert
        self.osdr_repo.upsert_osdr_items(&items).await?;
        info!("Successfully UPSERTED OSDR data into database.");
        
        Ok(())
    }

    /// -------------------------------------------------------------------------------------
    /// Получение позиции МКС (ОБНОВЛЕННАЯ ЛОГИКА FALLBACK)
    /// -------------------------------------------------------------------------------------
    #[instrument(skip(self), level = "info")]
    async fn get_iss_position(&self) -> Result<IssPosition, AppError> {
        // 1. Попробуем получить данные из Redis (Pascal клиент)
        match self.legacy_pascal_client.calculate_iss_position().await {
            Ok(redis_value) => {
                // Преобразуем JsonValue в IssPosition
                serde_json::from_value(redis_value)
                    .map_err(|e| AppError::SerializationError(format!("Failed to parse ISS position from Redis: {}", e)))
            }
            Err(e) => {
                warn!("Failed to get ISS position from Redis: {}. Falling back to NASA API.", e);
                
                // 2. Fallback к NASA API
                self.nasa_client.get_iss_position().await
            }
        }
    }

    /// -------------------------------------------------------------------------------------
    /// Получение списка OSDR
    /// -------------------------------------------------------------------------------------
    #[instrument(skip(self), level = "info")]
    async fn get_osdr_list(&self) -> Result<Vec<OsdrItem>, AppError> {
        info!("Getting OSDR list...");
        // Получаем 100 записей или все, в зависимости от того, что реализовано
        self.osdr_repo.get_list(100).await
    }

    /// -------------------------------------------------------------------------------------
    /// СЛУЖЕБНЫЕ ЗАДАЧИ: ISS, APOD, NEO, DONKI, SPACEX (Stubs)
    /// -------------------------------------------------------------------------------------

    #[instrument(skip(self), level = "info")]
    async fn fetch_and_store_iss(&self) -> Result<(), AppError> {
        warn!("STUB: fetch_and_store_iss not yet implemented in Service Layer.");
        // TODO: Реализовать логику ISS, используя self.nasa_client и соответствующий Repo
        Ok(())
    }

    #[instrument(skip(self), level = "info")]
    async fn fetch_and_cache_apod(&self) -> Result<(), AppError> {
        warn!("STUB: fetch_and_cache_apod not yet implemented in Service Layer.");
        // TODO: Реализовать логику APOD
        Ok(())
    }

    #[instrument(skip(self), level = "info")]
    async fn fetch_and_cache_neo(&self) -> Result<(), AppError> {
        warn!("STUB: fetch_and_cache_neo not yet implemented in Service Layer.");
        // TODO: Реализовать логику NEO
        Ok(())
    }
    
    #[instrument(skip(self), level = "info")]
    async fn fetch_and_cache_donki(&self) -> Result<(), AppError> {
        warn!("STUB: fetch_and_cache_donki not yet implemented in Service Layer.");
        // TODO: Реализовать логику DONKI
        Ok(())
    }

    #[instrument(skip(self), level = "info")]
    async fn fetch_and_cache_spacex(&self) -> Result<(), AppError> {
        warn!("STUB: fetch_and_cache_spacex not yet implemented in Service Layer.");
        // TODO: Реализовать логику SpaceX
        Ok(())
    }
}

// Добавьте также функцию-помощник, если нужна
pub async fn get_last_iss(_st: &crate::domain::models::AppState) -> Result<Option<Value>, AppError> {
    warn!("STUB: get_last_iss not yet implemented");
    Ok(None)
}

pub async fn get_iss_trend(_st: &crate::domain::models::AppState) -> Result<crate::domain::models::Trend, AppError> {
    warn!("STUB: get_iss_trend not yet implemented");
    
    // ИСПОЛЬЗУЙТЕ ВСЕ ПОЛЯ из вашей структуры Trend:
    Ok(crate::domain::models::Trend { 
        movement: false,          // bool
        delta_km: 0.0,            // f64
        dt_sec: 0.0,              // f64
        velocity_kmh: None,       // Option<f64>
        from_time: None,          // Option<DateTime<Utc>>
        to_time: None,            // Option<DateTime<Utc>>
        from_lat: None,           // Option<f64>
        from_lon: None,           // Option<f64>
        to_lat: None,             // Option<f64>
        to_lon: None,             // Option<f64>
        status: "ok".to_string(), // String
        message: "not implemented".to_string() // String
    })
}