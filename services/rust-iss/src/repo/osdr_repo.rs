use sqlx::{PgPool, Row};
use serde_json::Value;
use crate::domain::models::OsdrItem;
use crate::domain::errors::AppError;
use chrono::Utc;

// Соответствует `upsert_osdr_item` (немного изменена логика приёма аргументов)
// Для рефакторинга мы используем старый подход (Value)
pub async fn upsert_item(pool: &PgPool, item: &Value) -> Result<bool, AppError> {
    let id = crate::utils::json_tools::s_pick(item, &["dataset_id","id","uuid","studyId","accession","osdr_id"]);
    let title = crate::utils::json_tools::s_pick(item, &["title","name","label"]);
    let status = crate::utils::json_tools::s_pick(item, &["status","state","lifecycle"]);
    let updated = crate::utils::json_tools::t_pick(item, &["updated","updated_at","modified","lastUpdated","timestamp"]);

    let result = if let Some(ds) = id.clone() {
        sqlx::query(
            "INSERT INTO osdr_items(dataset_id, title, status, updated_at, raw)
             VALUES($1,$2,$3,$4,$5)
             ON CONFLICT (dataset_id) DO UPDATE
             SET title=EXCLUDED.title, status=EXCLUDED.status,
                 updated_at=EXCLUDED.updated_at, raw=EXCLUDED.raw"
        )
        .bind(ds).bind(title).bind(status).bind(updated).bind(item).execute(pool).await
    } else {
        sqlx::query(
            "INSERT INTO osdr_items(dataset_id, title, status, updated_at, raw)
             VALUES($1,$2,$3,$4,$5)"
        )
        .bind::<Option<String>>(None).bind(title).bind(status).bind(updated).bind(item).execute(pool).await
    };

    Ok(result.map(|r| r.rows_affected() > 0)?)
}

// Соответствует `list_osdr` (добавлен limit)
pub async fn get_list(pool: &PgPool, limit: i64) -> Result<Vec<OsdrItem>, AppError> {
    let items = sqlx::query_as::<_, OsdrItem>(
        "SELECT id, dataset_id, title, status, updated_at, inserted_at, raw
         FROM osdr_items
         ORDER BY inserted_at DESC
         LIMIT $1"
    )
    .bind(limit)
    .fetch_all(pool).await?;

    Ok(items)
}

pub async fn get_count(pool: &PgPool) -> Result<i64, AppError> {
    let count: i64 = sqlx::query("SELECT count(*) AS c FROM osdr_items")
        .fetch_one(pool).await
        .map(|r| r.get("c"))?;
    Ok(count)
}