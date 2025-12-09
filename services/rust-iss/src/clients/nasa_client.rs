use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use axum::http::StatusCode;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest::header::CONTENT_TYPE;
use tracing::{info, instrument, warn};

use crate::domain::contracts::NasaClientContract;
use crate::domain::errors::AppError;
use crate::domain::models::IssPosition; // Используем IssPosition

// =========================================================================================
// 1. КОНСТРУКТОР HTTP КЛИЕНТА (С RETRIES, БЕЗ CIRCUIT BREAKER)
// =========================================================================================

/// Создает устойчивый HTTP клиент с Exponential Backoff Retries.
/// Rate Limiter будет применен на уровне Axum-роутов.
pub fn create_api_client() -> ClientWithMiddleware { // FIX: Имя функции совпадает с использованием
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

    // Удален CircuitBreaker из-за проблем совместимости.
    ClientBuilder::new(reqwest::Client::new())
        // Retries (для временных ошибок)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build() 
}

// =========================================================================================
// 2. СТРУКТУРА NASA КЛИЕНТА
// =========================================================================================

pub struct NasaClient {
    client: ClientWithMiddleware,
    api_key: String,
}

pub fn new_nasa_client(client: ClientWithMiddleware, api_key: String) -> NasaClient {
    NasaClient { client, api_key }
}

// =========================================================================================
// 3. РЕАЛИЗАЦИЯ КОНТРАКТА
// =========================================================================================

#[async_trait]
impl NasaClientContract for NasaClient {
    
    // FIX: Теперь возвращает Result<IssPosition, AppError> (согласно контракту)
    #[instrument(skip(self), level = "info")]
    async fn get_iss_position(&self) -> Result<IssPosition, AppError> {
        let url = "http://api.open-notify.org/iss-now.json"; 
        info!("Fetching live ISS position from: {}", url);

        // Вспомогательная структура для десериализации ответа open-notify.org
        #[derive(Debug, Deserialize)]
        struct IssApiResponse {
            iss_position: IssPosition,
            timestamp: i64,
            message: String,
        }

        let response = self.client.get(url)
            .send()
            .await
            .map_err(|e| AppError::ClientError(e.to_string(), StatusCode::BAD_GATEWAY))?; // FIX: Используем From<reqwest::Error>
            
        // Обработка статуса ошибки
        let response = response.error_for_status()
            .map_err(|e| AppError::ClientError(e.to_string(), StatusCode::BAD_GATEWAY))?;

        let full_body: IssApiResponse = response.json().await
            .map_err(|e| AppError::SerializationError(e.to_string()))?;
            
        // Возвращаем только требуемую IssPosition
        Ok(full_body.iss_position)
    }

    #[instrument(skip(self), level = "info")]
    async fn fetch_osdr_list(&self, url: &str) -> Result<Vec<Value>, AppError> {
        info!("Fetching OSDR list from: {}", url);
        
        let request_url = format!("{}&api_key={}", url, self.api_key);

        let response = self.client.get(&request_url)
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await
            .map_err(|e| AppError::ClientError(e.to_string(), StatusCode::BAD_GATEWAY))?;
        
        // Обработка статуса ошибки
        let response = response.error_for_status()
            .map_err(|e| AppError::ClientError(e.to_string(), StatusCode::BAD_GATEWAY))?;

        // Десериализация (мы ожидаем массив)
        let json_body = response.json::<Value>().await
            .map_err(|e| AppError::SerializationError(e.to_string()))?;

        // Поиск массива данных по ключу "results" (типично для NASA APIs)
        let items = json_body["results"].as_array().cloned().unwrap_or_default();

        Ok(items)
    }

    #[instrument(skip(self), level = "info")]
    async fn fetch_apod(&self, _api_key: &str) -> Result<Value, AppError> { 
        warn!("NASA APOD client not implemented yet.");
        Ok(Value::Null)
    }

    #[instrument(skip(self), level = "info")]
    async fn fetch_neo_feed(&self, _start_date: &str, _end_date: &str, _api_key: &str) -> Result<Value, AppError> {
        warn!("NASA NEO Feed client not implemented yet.");
        Ok(Value::Null)
    }

    #[instrument(skip(self), level = "info")]
    async fn fetch_donki_flr(&self, _start_date: &str, _end_date: &str, _api_key: &str) -> Result<Value, AppError> {
        warn!("NASA DONKI FLR client not implemented yet.");
        Ok(Value::Null)
    }

    #[instrument(skip(self), level = "info")]
    async fn fetch_donki_cme(&self, _start_date: &str, _end_date: &str, _api_key: &str) -> Result<Value, AppError> {
        warn!("NASA DONKI CME client not implemented yet.");
        Ok(Value::Null)
    }
}