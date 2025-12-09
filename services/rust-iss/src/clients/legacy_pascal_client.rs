use async_trait::async_trait;
use redis::{AsyncCommands, Client as RedisClient, RedisResult};
use serde_json::{Value, json};
use std::sync::Arc;

// Импортируем AppError
use crate::domain::errors::AppError;

// --- КОНТРАКТ ---
#[async_trait]
pub trait LegacyPascalClientContract: Send + Sync {
    // Метод для получения "горячих" данных ISS (теперь из Redis).
    async fn calculate_iss_position(&self) -> Result<Value, AppError>;
    
    // Опционально: метод для проверки соединения
    async fn check_connection(&self) -> Result<(), AppError>;
}

// --- СТРУКТУРА КЛИЕНТА ---
pub struct LegacyPascalClient {
    // RedisClient (из крейта redis) используется для подключения.
    redis_client: Arc<RedisClient>,
    // Ключ, который используется Go-воркером для записи данных.
    redis_key: String,
}

// --- КОНСТРУКТОР ---
impl LegacyPascalClient {
    pub fn new(redis_url: &str, redis_key: Option<String>) -> Result<Self, AppError> {
        // Создание клиента Redis.
        let client = RedisClient::open(redis_url)
            .map_err(|e| AppError::InternalError(format!("Failed to connect to Redis client: {}", e)))?;
        
        Ok(Self {
            redis_client: Arc::new(client),
            redis_key: redis_key.unwrap_or_else(|| "latest_telemetry_data".to_string()),
        })
    }
}

// --- РЕАЛИЗАЦИЯ КОНТРАКТА ---
#[async_trait]
impl LegacyPascalClientContract for LegacyPascalClient {
    // Чтение данных телеметрии из Redis.
    async fn calculate_iss_position(&self) -> Result<Value, AppError> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| {
                AppError::ExternalServiceError(format!("Redis connection failed: {}", e))
            })?;

        // 1. Получение JSON-строки из Redis
        let data_json: RedisResult<String> = conn.get(&self.redis_key).await;

        match data_json {
            Ok(json_str) => {
                // Проверка на пустую строку
                if json_str.trim().is_empty() {
                    return Err(AppError::NotFound("Empty telemetry data in cache.".to_string()));
                }
                
                // 2. Парсинг JSON-строки в serde_json::Value
                let value: Value = serde_json::from_str(&json_str)
                    .map_err(|e| AppError::InternalError(format!("Failed to parse JSON from Redis: {}", e)))?;
                
                // ВАЖНО: Мы возвращаем сырой JSON, чтобы SpaceService мог его использовать.
                Ok(value)
            },
            Err(e) => {
                // Если данных нет (ключ не найден или ошибка)
                log::warn!("Telemetry data not found in Redis cache. Error: {}", e);
                // Возвращаем ошибку "Данные не найдены"
                Err(AppError::NotFound(format!("Latest telemetry data not available in cache: {}", e)))
            }
        }
    }
    
    // Проверка соединения с Redis
    async fn check_connection(&self) -> Result<(), AppError> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| {
                AppError::ExternalServiceError(format!("Redis connection failed: {}", e))
            })?;
        
        // Простой PING запрос
        let _: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                AppError::ExternalServiceError(format!("Redis PING failed: {}", e))
            })?;
        
        Ok(())
    }
}

// Тип для удобного использования в Arc
pub type LegacyPascalClientArc = Arc<dyn LegacyPascalClientContract>;