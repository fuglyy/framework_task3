use async_trait::async_trait;
use sqlx::{PgPool, Row};
use serde_json::Value;

use crate::domain::models::OsdrItem;
use crate::domain::errors::AppError;
use crate::domain::contracts::OsdrRepoContract; // <--- Новый импорт
use crate::utils::json_tools; // Ваш модуль для парсинга JSON

// =========================================================================================
// 1. СТРУКТУРА РЕПОЗИТОРИЯ
// =========================================================================================

/// Конкретная реализация OsdrRepoContract. Хранит пул подключения к БД.
pub struct OsdrRepo {
    pool: PgPool,
}

pub fn new_osdr_repo(pool: PgPool) -> OsdrRepo {
    OsdrRepo { pool }
}

// =========================================================================================
// 2. РЕАЛИЗАЦИЯ КОНТРАКТА (REPOSITORY LOGIC)
// =========================================================================================

#[async_trait]
impl OsdrRepoContract for OsdrRepo {
    
    /// Идемпотентное сохранение/обновление (UPSERT) списка элементов OSDR.
    /// Бизнес-ключ для ON CONFLICT — dataset_id.
    async fn upsert_osdr_items(&self, items: &[Value]) -> Result<(), AppError> {
        // Мы будем выполнять UPSERT для каждого элемента в списке
        for item in items {
            // Извлечение бизнес-ключей и полей из JSON
            let id = json_tools::s_pick(item, &["dataset_id", "id", "uuid", "studyId", "accession", "osdr_id"]);
            let title = json_tools::s_pick(item, &["title", "name", "label"]);
            let status = json_tools::s_pick(item, &["status", "state", "lifecycle"]);
            let updated = json_tools::t_pick(item, &["updated", "updated_at", "modified", "lastUpdated", "timestamp"]);

            // ВАЖНО: Мы используем dataset_id как уникальный ключ для ON CONFLICT.
            if let Some(ds_id) = id.clone() {
                // Использование UPSERT (ON CONFLICT)
                // Если dataset_id уже есть, обновляем все поля (title, status, updated_at, raw).
                sqlx::query(
                    "INSERT INTO osdr_items(dataset_id, title, status, updated_at, raw)
                     VALUES($1,$2,$3,$4,$5)
                     ON CONFLICT (dataset_id) DO UPDATE
                     SET title=EXCLUDED.title, status=EXCLUDED.status,
                         updated_at=EXCLUDED.updated_at, raw=EXCLUDED.raw"
                )
                .bind(ds_id).bind(title).bind(status).bind(updated).bind(item)
                .execute(&self.pool).await
                .map(|_| ())?; // Преобразуем результат в ()
            } else {
                // Если dataset_id отсутствует (редкий случай), просто вставляем
                // без гарантии уникальности (зависит от схемы БД).
                sqlx::query(
                    "INSERT INTO osdr_items(dataset_id, title, status, updated_at, raw)
                     VALUES($1,$2,$3,$4,$5)"
                )
                .bind::<Option<String>>(None).bind(title).bind(status).bind(updated).bind(item)
                .execute(&self.pool).await
                .map(|_| ())?;
            }
        }
        Ok(())
    }

    /// Получить список элементов OSDR (как было в get_list)
    async fn get_list(&self, limit: i64) -> Result<Vec<OsdrItem>, AppError> {
        let items = sqlx::query_as::<_, OsdrItem>(
            "SELECT id, dataset_id, title, status, updated_at, inserted_at, raw
             FROM osdr_items
             ORDER BY inserted_at DESC
             LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&self.pool).await?;

        Ok(items)
    }

    /// Получить количество элементов (как было в get_count)
    async fn get_count(&self) -> Result<i64, AppError> {
        let count: i64 = sqlx::query("SELECT count(*) AS c FROM osdr_items")
            .fetch_one(&self.pool).await
            .map(|r| r.get("c"))?;
        Ok(count)
    }

    async fn get_all(&self) -> Result<Vec<OsdrItem>, AppError> {
        // Временная заглушка
        Ok(vec![])
    }
    
    async fn save(&self, _osdr: &OsdrItem) -> Result<(), AppError> {
        // Временная заглушка
        Ok(())
    }
}