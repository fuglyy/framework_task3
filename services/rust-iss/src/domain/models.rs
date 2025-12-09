use std::sync::Arc; // <--- Новый импорт для Arc
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use reqwest_middleware::ClientWithMiddleware; // Оставим пока, если используется где-то еще

// Импорты контрактов
use super::contracts::{NasaClientContract, OsdrRepoContract, SpaceServiceContract}; 
use crate::clients::legacy_pascal_client::LegacyPascalClientContract;

// --- 1. Конфигурация приложения
// Убираем nasa_client из AppConfig, он теперь внедряется отдельно
#[derive(Clone, Debug)]
pub struct AppConfig {
    // URL/Keys
    pub nasa_url: String, // OSDR
    pub nasa_key: String, // ключ NASA
    pub fallback_iss_url: String,
    pub redis_url: String, // ISS where-the-iss
    // Интервалы
    pub every_osdr: u64,
    pub every_iss: u64,
    pub every_apod: u64,
    pub every_neo: u64,
    pub every_donki: u64,
    pub every_spacex: u64,
    // УБРАНО: pub nasa_client: ClientWithMiddleware, 
}



// --- 2. Стейт приложения (для Axum::State)
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool, // Оставляем пул, т.к. он нужен для создания репозиториев
    pub config: AppConfig, // Используем AppConfig как вложенную структуру
    
    // ВМЕСТО КОНКРЕТНЫХ СТРУКТУР, ХРАНИМ ТРЕЙТ-ОБЪЕКТЫ (DI)
    pub nasa_client: Arc<dyn NasaClientContract>, // <--- ИЗМЕНЕНО
    pub osdr_repo: Arc<dyn OsdrRepoContract>, 
    
    pub space_service: Arc<dyn SpaceServiceContract>,
    pub legacy_pascal_client: Arc<dyn LegacyPascalClientContract>,// <--- НОВОЕ ПОЛЕ
}

impl AppState {
    // Новый конструктор AppState для DI
    pub fn new(
        pool: PgPool, 
        config: AppConfig, 
        nasa_client: Arc<dyn NasaClientContract>, 
        osdr_repo: Arc<dyn OsdrRepoContract>,
        space_service: Arc<dyn SpaceServiceContract>,
        legacy_pascal_client: Arc<dyn LegacyPascalClientContract>, // Добавьте этот параметр
    ) -> Self {
        Self {
            pool,
            config,
            nasa_client,
            osdr_repo,
            space_service,
            legacy_pascal_client, // Добавьте это поле
        }
    }
}

// --- 3. Другие модели (остаются без изменений)
// ... (Health, Trend, IssLog, OsdrItem, ApiSuccessResponse, ToSuccessResponse)
// Оставим остальные модели без изменений
// Структура для /health
#[derive(Serialize)]
pub struct Health {
    pub status: &'static str,
    pub now: DateTime<Utc>,
}

// Структура для iss_trend
#[derive(Serialize)]
pub struct Trend {
    pub movement: bool,
    pub delta_km: f64,
    pub dt_sec: f64,
    pub velocity_kmh: Option<f64>,
    pub from_time: Option<DateTime<Utc>>,
    pub to_time: Option<DateTime<Utc>>,
    pub from_lat: Option<f64>,
    pub from_lon: Option<f64>,
    pub to_lat: Option<f64>,
    pub to_lon: Option<f64>,
    pub status: String,      // ДОБАВЬТЕ ЭТО
    pub message: String,
}

// Структура для последней записи ISS (как хранится в DB)
#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct IssLog {
    pub id: i64,
    pub fetched_at: DateTime<Utc>,
    pub source_url: String,
    pub payload: serde_json::Value,
}

// Структура для OSDR (как хранится в DB)
#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct OsdrItem {
    pub id: i64,
    pub dataset_id: Option<String>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
    pub inserted_at: DateTime<Utc>,
    pub raw: serde_json::Value,
}

#[derive(Serialize)]
pub struct ApiSuccessResponse<T> {
    pub ok: bool, // Всегда true
    #[serde(flatten)] // Встраивает поля T напрямую в тело ответа
    pub data: T,
}

// Новый трейт для удобного преобразования (не обязательно, но полезно)
pub trait ToSuccessResponse: Sized + Serialize {
    fn to_success_response(self) -> ApiSuccessResponse<Self> {
        ApiSuccessResponse {
            ok: true,
            data: self,
        }
    }
}

// Структура для позиции МКС
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IssPosition {
    pub timestamp: i64,
    pub latitude: f64,
    pub longitude: f64,
}
impl TryFrom<serde_json::Value> for IssPosition {
    type Error = serde_json::Error;
    
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}