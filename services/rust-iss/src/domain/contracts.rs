// src/domain/contracts.rs

use async_trait::async_trait;
use serde_json::Value;

use super::errors::AppError;
use crate::domain::models::{IssPosition, OsdrItem};

// ------------------------------------------------------------
// Контракт для внешних API (Adapter Pattern)
// ------------------------------------------------------------
#[async_trait]
pub trait NasaClientContract: Send + Sync {
    /// Запрашивает список данных OSDR
    async fn fetch_osdr_list(&self, url: &str) -> Result<Vec<Value>, AppError>;

    /// Запрашивает APOD
    async fn fetch_apod(&self, api_key: &str) -> Result<Value, AppError>;
    
    async fn get_iss_position(&self) -> Result<IssPosition, AppError>;
    
    /// Запрашивает NEO feed - ДОБАВЬТЕ ЕСЛИ ИСПОЛЬЗУЕТЕ
    async fn fetch_neo_feed(&self, start_date: &str, end_date: &str, api_key: &str) -> Result<Value, AppError>;
    
    /// Запрашивает DONKI FLR - ДОБАВЬТЕ ЕСЛИ ИСПОЛЬЗУЕТЕ
    async fn fetch_donki_flr(&self, start_date: &str, end_date: &str, api_key: &str) -> Result<Value, AppError>;
    
    /// Запрашивает DONKI CME - ДОБАВЬТЕ ЕСЛИ ИСПОЛЬЗУЕТЕ
    async fn fetch_donki_cme(&self, start_date: &str, end_date: &str, api_key: &str) -> Result<Value, AppError>;}


// ------------------------------------------------------------
// Контракт для Репозиториев (Repository Pattern)
// ------------------------------------------------------------
#[async_trait]
pub trait OsdrRepoContract: Send + Sync {
    /// Сохраняет или обновляет список данных в таблице osdr_items
    async fn upsert_osdr_items(&self, items: &[Value]) -> Result<(), AppError>;
    
    async fn get_list(&self, limit: i64) -> Result<Vec<OsdrItem>, AppError>;
    
    /// Получает количество записей - ДОБАВЬТЕ ЭТОТ МЕТОД
    async fn get_count(&self) -> Result<i64, AppError>;
    
    /// Получает все записи - ДОБАВЬТЕ ЕСЛИ ИСПОЛЬЗУЕТЕ
    async fn get_all(&self) -> Result<Vec<OsdrItem>, AppError>;
    
    /// Сохраняет одну запись - ДОБАВЬТЕ ЕСЛИ ИСПОЛЬЗУЕТЕ
    async fn save(&self, osdr: &OsdrItem) -> Result<(), AppError>;}

// ------------------------------------------------------------
// Контракт для Бизнес-Логики (Service Layer)
// ------------------------------------------------------------
#[async_trait]
pub trait SpaceServiceContract: Send + Sync {
    /// Выполняет полный цикл: запрашивает данные у NASA и сохраняет в БД
    async fn fetch_and_save_osdr_data(&self) -> Result<(), AppError>;
    
    async fn fetch_and_store_iss(&self) -> Result<(), AppError>;
    
    async fn get_iss_position(&self) -> Result<IssPosition, AppError>;
    
    /// Получает список OSDR - ДОБАВЬТЕ ЭТОТ МЕТОД
    async fn get_osdr_list(&self) -> Result<Vec<OsdrItem>, AppError>;

    // Cache/APOD/NEO/DONKI/SPACEX (методы, которые использует background/mod.rs)
    async fn fetch_and_cache_apod(&self) -> Result<(), AppError>;
    async fn fetch_and_cache_neo(&self) -> Result<(), AppError>;
    async fn fetch_and_cache_donki(&self) -> Result<(), AppError>;
    async fn fetch_and_cache_spacex(&self) -> Result<(), AppError>;

}