use sqlx::{PgPool, Row};
use serde_json::Value;
use chrono::{DateTime, Utc};
use crate::domain::models::{IssLog, Trend};
use crate::domain::errors::AppError;
use crate::utils::haversine::haversine_km;

// Соответствует `insert_iss_log`
pub async fn insert_log(pool: &PgPool, source_url: &str, payload: &Value) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO iss_fetch_log (source_url, payload)
         VALUES ($1, $2)"
    )
    .bind(source_url)
    .bind(payload)
    .execute(pool).await?;
    Ok(())
}

// Соответствует `last_iss` (возвращает структурированный IssLog вместо Value)
pub async fn get_last_log(pool: &PgPool) -> Result<Option<IssLog>, AppError> {
    // Используем `query_as` для автоматического маппинга
    let row_opt = sqlx::query_as::<_, IssLog>(
        "SELECT id, fetched_at, source_url, payload
         FROM iss_fetch_log
         ORDER BY id DESC LIMIT 1"
    ).fetch_optional(pool).await?;

    Ok(row_opt)
}

pub async fn calculate_trend(pool: &PgPool) -> Result<Trend, AppError> {
    let rows = sqlx::query("SELECT fetched_at, payload FROM iss_fetch_log ORDER BY id DESC LIMIT 2")
        .fetch_all(pool).await?;

    if rows.len() < 2 {
        return Ok(Trend {
            movement: false, delta_km: 0.0, dt_sec: 0.0, velocity_kmh: None,
            from_time: None, to_time: None,
            from_lat: None, from_lon: None, to_lat: None, to_lon: None,
            message: "calculated successfully".to_string(), // ДОБАВЬТЕ
            status: "ok".to_string(), 
        });
    }

    let t2: chrono::DateTime<Utc> = rows[0].get("fetched_at");
    let t1: chrono::DateTime<Utc> = rows[1].get("fetched_at");
    let p2: Value = rows[0].get("payload");
    let p1: Value = rows[1].get("payload");

    let lat1 = crate::utils::json_tools::num(&p1["latitude"]);
    let lon1 = crate::utils::json_tools::num(&p1["longitude"]);
    let lat2 = crate::utils::json_tools::num(&p2["latitude"]);
    let lon2 = crate::utils::json_tools::num(&p2["longitude"]);
    let v2 = crate::utils::json_tools::num(&p2["velocity"]);

    let mut delta_km = 0.0;
    let mut movement = false;
    if let (Some(a1), Some(o1), Some(a2), Some(o2)) = (lat1, lon1, lat2, lon2) {
        delta_km = haversine_km(a1, o1, a2, o2);
        movement = delta_km > 0.1;
    }
    let dt_sec = (t2 - t1).num_milliseconds() as f64 / 1000.0;

    Ok(Trend {
        movement,
        delta_km,
        dt_sec,
        velocity_kmh: v2,
        from_time: Some(t1),
        to_time: Some(t2),
        from_lat: lat1, from_lon: lon1, to_lat: lat2, to_lon: lon2,
        message: "calculated successfully".to_string(), // ДОБАВЬТЕ ЗДЕСЬ ТОЖЕ
        status: "ok".to_string(),
    })
}

// Функции для универсального кэша space_cache
pub async fn write_cache(pool: &PgPool, source: &str, payload: &Value) -> Result<(), AppError> {
    sqlx::query("INSERT INTO space_cache(source, payload) VALUES ($1,$2)")
        .bind(source).bind(payload).execute(pool).await?;
    Ok(())
}

// Соответствует `last_space`
pub async fn get_latest_from_cache(pool: &PgPool, source: &str) -> Result<Option<Value>, AppError> {
    let row = sqlx::query(
        "SELECT payload, fetched_at
         FROM space_cache
         WHERE source = $1
         ORDER BY fetched_at DESC
         LIMIT 1"
    )
    .bind(source)
    .fetch_optional(pool)
    .await?;

    if let Some(r) = row {
        use sqlx::Row;
        Ok(Some(serde_json::json!({
            "fetched_at": r.get::<DateTime<Utc>, _>("fetched_at"),
            "payload": r.get::<Value, _>("payload"),
        })))
    } else {
        Ok(None)
    }
}