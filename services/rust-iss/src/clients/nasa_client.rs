use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use chrono::{Utc, Days};
use crate::domain::errors::AppError;

const TIMEOUT: u64 = 30;

pub async fn fetch_osdr_list(url: &str) -> Result<Vec<Value>, AppError> {
    let client = Client::builder().timeout(Duration::from_secs(TIMEOUT)).build()?;
    let resp = client.get(url).send().await?;

    if !resp.status().is_success() {
        return Err(anyhow::anyhow!("OSDR request status {}", resp.status()).into());
    }
    let json: Value = resp.json().await?;

    let items = if let Some(a) = json.as_array() { a.clone() }
        else if let Some(v) = json.get("items").and_then(|x| x.as_array()) { v.clone() }
        else if let Some(v) = json.get("results").and_then(|x| x.as_array()) { v.clone() }
        else { vec![json.clone()] };

    Ok(items)
}

pub async fn fetch_apod(api_key: &str) -> Result<Value, AppError> {
    let url = "https://api.nasa.gov/planetary/apod";
    let client = Client::builder().timeout(Duration::from_secs(TIMEOUT)).build()?;
    let mut req = client.get(url).query(&[("thumbs","true")]);
    if !api_key.is_empty() { req = req.query(&[("api_key",api_key)]); }
    let json: Value = req.send().await?.json().await?;
    Ok(json)
}

pub async fn fetch_neo_feed(api_key: &str) -> Result<Value, AppError> {
    let today = Utc::now().date_naive();
    let start = today - Days::new(2);
    let url = "https://api.nasa.gov/neo/rest/v1/feed";
    let client = Client::builder().timeout(Duration::from_secs(TIMEOUT)).build()?;
    let mut req = client.get(url).query(&[
        ("start_date", start.to_string()),
        ("end_date", today.to_string()),
    ]);
    if !api_key.is_empty() { req = req.query(&[("api_key",api_key)]); }
    let json: Value = req.send().await?.json().await?;
    Ok(json)
}

pub async fn fetch_donki_flr(api_key: &str) -> Result<Value, AppError> {
    let (from,to) = last_days(5);
    let url = "https://api.nasa.gov/DONKI/FLR";
    let client = Client::builder().timeout(Duration::from_secs(TIMEOUT)).build()?;
    let mut req = client.get(url).query(&[("startDate",from),("endDate",to)]);
    if !api_key.is_empty() { req = req.query(&[("api_key",api_key)]); }
    let json: Value = req.send().await?.json().await?;
    Ok(json)
}

pub async fn fetch_donki_cme(api_key: &str) -> Result<Value, AppError> {
    let (from,to) = last_days(5);
    let url = "https://api.nasa.gov/DONKI/CME";
    let client = Client::builder().timeout(Duration::from_secs(TIMEOUT)).build()?;
    let mut req = client.get(url).query(&[("startDate",from),("endDate",to)]);
    if !api_key.is_empty() { req = req.query(&[("api_key",api_key)]); }
    let json: Value = req.send().await?.json().await?;
    Ok(json)
}

fn last_days(n: i64) -> (String,String) {
    let to = Utc::now().date_naive();
    let from = to - Days::new(n as u64);
    (from.to_string(), to.to_string())
}