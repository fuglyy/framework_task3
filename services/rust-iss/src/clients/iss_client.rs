use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use crate::domain::errors::AppError;

pub async fn fetch_iss_location(url: &str) -> Result<Value, AppError> {
    let client = Client::builder().timeout(Duration::from_secs(20)).build()?;
    let resp = client.get(url).send().await?;

    if !resp.status().is_success() {
        return Err(AppError::ReqwestError(resp.error_for_status().unwrap_err()));
    }

    let json: Value = resp.json().await?;
    Ok(json)
}