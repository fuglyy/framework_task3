use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

// --- 1. Конфигурация приложения (иммутабельные настройки)
// Это то, что загружается из .env, используется для AppState.
#[derive(Clone, Debug)]
pub struct AppConfig {
    // URL/Keys
    pub nasa_url: String, // OSDR
    pub nasa_key: String, // ключ NASA
    pub fallback_iss_url: String, // ISS where-the-iss
    // Интервалы
    pub every_osdr: u64,
    pub every_iss: u64,
    pub every_apod: u64,
    pub every_neo: u64,
    pub every_donki: u64,
    pub every_spacex: u64,
}

// --- 2. Стейт приложения (для Axum::State)
// Объединяет Pool и Config.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: AppConfig, // Используем AppConfig как вложенную структуру
}

impl AppState {
    // Используем метод new для создания из пула и конфига
    pub fn new(pool: PgPool, config: AppConfig) -> Self {
        Self { pool, config }
    }
}

// --- 3. Другие модели (остаются без изменений)
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