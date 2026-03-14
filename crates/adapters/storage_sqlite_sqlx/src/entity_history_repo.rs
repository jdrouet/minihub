//! `SQLite` implementation of [`EntityHistoryRepository`].

use std::collections::HashMap;

use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqlitePool};

use minihub_app::ports::storage::EntityHistoryRepository;
use minihub_domain::entity::{AttributeValue, EntityState};
use minihub_domain::entity_history::{EntityHistory, EntityHistoryId};
use minihub_domain::error::MiniHubError;
use minihub_domain::id::EntityId;
use minihub_domain::time::Timestamp;

use crate::error::StorageError;

/// Wrapper for converting database rows into domain types without polluting
/// domain structs with database concerns.
struct Wrapper(EntityHistory);

impl<'r> FromRow<'r, SqliteRow> for Wrapper {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let id: uuid::Uuid = row.try_get("id")?;
        let entity_id: uuid::Uuid = row.try_get("entity_id")?;
        let state_str: String = row.try_get("state")?;
        let attributes_json: String = row.try_get("attributes")?;
        let recorded_at_str: String = row.try_get("recorded_at")?;

        let id = EntityHistoryId::from_uuid(id);
        let entity_id = EntityId::from_uuid(entity_id);
        let state: EntityState = serde_json::from_str(&format!("\"{state_str}\""))
            .map_err(|err| sqlx::Error::Decode(Box::new(err)))?;
        let attributes: HashMap<String, AttributeValue> = serde_json::from_str(&attributes_json)
            .map_err(|err| sqlx::Error::Decode(Box::new(err)))?;
        let recorded_at = chrono::DateTime::parse_from_rfc3339(&recorded_at_str)
            .map_err(|err| sqlx::Error::Decode(Box::new(err)))?
            .to_utc();

        Ok(Self(EntityHistory {
            id,
            entity_id,
            state,
            attributes,
            recorded_at,
        }))
    }
}

const INSERT: &str = r"
    INSERT INTO entity_history (id, entity_id, state, attributes, recorded_at)
    VALUES (?, ?, ?, ?, ?)
";

const SELECT_BY_ENTITY_IN_RANGE: &str = r"
    SELECT * FROM entity_history
    WHERE entity_id = ? AND recorded_at >= ? AND recorded_at <= ?
    ORDER BY recorded_at ASC
    LIMIT ?
";

const SELECT_BY_ENTITY_IN_RANGE_NO_LIMIT: &str = r"
    SELECT * FROM entity_history
    WHERE entity_id = ? AND recorded_at >= ? AND recorded_at <= ?
    ORDER BY recorded_at ASC
";

const DELETE_BEFORE: &str = "DELETE FROM entity_history WHERE recorded_at < ?";

/// `SQLite`-backed entity history repository.
pub struct SqliteEntityHistoryRepository {
    pool: SqlitePool,
}

impl SqliteEntityHistoryRepository {
    /// Create a new repository using the given connection pool.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl EntityHistoryRepository for SqliteEntityHistoryRepository {
    async fn record(&self, history: EntityHistory) -> Result<EntityHistory, MiniHubError> {
        let attributes_json =
            serde_json::to_string(&history.attributes).map_err(StorageError::from)?;

        sqlx::query(INSERT)
            .bind(history.id.as_uuid())
            .bind(history.entity_id.as_uuid())
            .bind(history.state.to_string())
            .bind(&attributes_json)
            .bind(history.recorded_at.to_rfc3339())
            .execute(&self.pool)
            .await
            .map_err(StorageError::from)?;

        Ok(history)
    }

    async fn find_by_entity_in_range(
        &self,
        entity_id: EntityId,
        from: Timestamp,
        to: Timestamp,
        limit: Option<usize>,
    ) -> Result<Vec<EntityHistory>, MiniHubError> {
        let rows: Vec<Wrapper> = if let Some(limit) = limit {
            let limit_i64 = i64::try_from(limit).unwrap_or(i64::MAX);
            sqlx::query_as(SELECT_BY_ENTITY_IN_RANGE)
                .bind(entity_id.as_uuid())
                .bind(from.to_rfc3339())
                .bind(to.to_rfc3339())
                .bind(limit_i64)
                .fetch_all(&self.pool)
                .await
                .map_err(StorageError::from)?
        } else {
            sqlx::query_as(SELECT_BY_ENTITY_IN_RANGE_NO_LIMIT)
                .bind(entity_id.as_uuid())
                .bind(from.to_rfc3339())
                .bind(to.to_rfc3339())
                .fetch_all(&self.pool)
                .await
                .map_err(StorageError::from)?
        };

        Ok(rows.into_iter().map(|w| w.0).collect())
    }

    async fn purge_before(&self, before: Timestamp) -> Result<usize, MiniHubError> {
        let result = sqlx::query(DELETE_BEFORE)
            .bind(before.to_rfc3339())
            .execute(&self.pool)
            .await
            .map_err(StorageError::from)?;

        // Saturating cast is safe here: realistically we'll never delete usize::MAX rows
        #[allow(clippy::cast_possible_truncation)]
        Ok(result.rows_affected() as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::Config;
    use chrono::Duration;
    use minihub_domain::entity::EntityState;
    use minihub_domain::time::now;

    async fn setup() -> (SqliteEntityHistoryRepository, EntityId) {
        let db = Config {
            database_url: "sqlite::memory:".to_string(),
        }
        .build()
        .await
        .unwrap();
        let pool = db.pool().clone();
        let entity_id = EntityId::new();

        // Insert a test device and entity
        let device_id = uuid::Uuid::new_v4();
        sqlx::query("INSERT INTO devices (id, name) VALUES (?, ?)")
            .bind(device_id)
            .bind("Test Device")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO entities (id, device_id, entity_id, friendly_name, state, attributes, last_changed, last_updated) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(entity_id.as_uuid())
            .bind(device_id)
            .bind("sensor.test")
            .bind("Test Sensor")
            .bind("unknown")
            .bind("{}")
            .bind(now().to_rfc3339())
            .bind(now().to_rfc3339())
            .execute(&pool)
            .await
            .unwrap();

        (SqliteEntityHistoryRepository::new(pool), entity_id)
    }

    fn test_history(entity_id: EntityId, recorded_at: Timestamp) -> EntityHistory {
        EntityHistory::builder()
            .entity_id(entity_id)
            .state(EntityState::On)
            .attribute("temperature", AttributeValue::Float(22.5))
            .recorded_at(recorded_at)
            .build()
    }

    #[tokio::test]
    async fn should_record_and_retrieve_history_when_valid() {
        let (repo, entity_id) = setup().await;
        let timestamp = now();
        let history = test_history(entity_id, timestamp);
        let id = history.id;

        let recorded = repo.record(history).await.unwrap();
        assert_eq!(recorded.id, id);

        let from = timestamp - Duration::hours(1);
        let to = timestamp + Duration::hours(1);
        let found = repo
            .find_by_entity_in_range(entity_id, from, to, None)
            .await
            .unwrap();

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, id);
        assert_eq!(found[0].entity_id, entity_id);
        assert_eq!(found[0].state, EntityState::On);
        assert_eq!(
            found[0].get_attribute("temperature"),
            Some(&AttributeValue::Float(22.5))
        );
    }

    #[tokio::test]
    async fn should_return_empty_when_no_history_in_range() {
        let (repo, entity_id) = setup().await;
        let timestamp = now();
        let history = test_history(entity_id, timestamp);
        repo.record(history).await.unwrap();

        let from = timestamp - Duration::hours(5);
        let to = timestamp - Duration::hours(2);
        let found = repo
            .find_by_entity_in_range(entity_id, from, to, None)
            .await
            .unwrap();

        assert!(found.is_empty());
    }

    #[tokio::test]
    async fn should_order_history_by_recorded_at_ascending() {
        let (repo, entity_id) = setup().await;
        let base_time = now();

        let history1 = test_history(entity_id, base_time);
        let history2 = test_history(entity_id, base_time + Duration::hours(1));
        let history3 = test_history(entity_id, base_time + Duration::hours(2));

        repo.record(history2.clone()).await.unwrap();
        repo.record(history1.clone()).await.unwrap();
        repo.record(history3.clone()).await.unwrap();

        let from = base_time - Duration::hours(1);
        let to = base_time + Duration::hours(3);
        let found = repo
            .find_by_entity_in_range(entity_id, from, to, None)
            .await
            .unwrap();

        assert_eq!(found.len(), 3);
        assert_eq!(found[0].id, history1.id);
        assert_eq!(found[1].id, history2.id);
        assert_eq!(found[2].id, history3.id);
    }

    #[tokio::test]
    async fn should_respect_limit_when_provided() {
        let (repo, entity_id) = setup().await;
        let base_time = now();

        for i in 0..5 {
            let history = test_history(entity_id, base_time + Duration::hours(i));
            repo.record(history).await.unwrap();
        }

        let from = base_time - Duration::hours(1);
        let to = base_time + Duration::hours(10);
        let found = repo
            .find_by_entity_in_range(entity_id, from, to, Some(3))
            .await
            .unwrap();

        assert_eq!(found.len(), 3);
    }

    #[tokio::test]
    async fn should_return_all_when_no_limit_provided() {
        let (repo, entity_id) = setup().await;
        let base_time = now();

        for i in 0..5 {
            let history = test_history(entity_id, base_time + Duration::hours(i));
            repo.record(history).await.unwrap();
        }

        let from = base_time - Duration::hours(1);
        let to = base_time + Duration::hours(10);
        let found = repo
            .find_by_entity_in_range(entity_id, from, to, None)
            .await
            .unwrap();

        assert_eq!(found.len(), 5);
    }

    #[tokio::test]
    async fn should_purge_records_before_timestamp() {
        let (repo, entity_id) = setup().await;
        let base_time = now();

        let old1 = test_history(entity_id, base_time - Duration::days(5));
        let old2 = test_history(entity_id, base_time - Duration::days(3));
        let recent = test_history(entity_id, base_time - Duration::hours(1));

        repo.record(old1).await.unwrap();
        repo.record(old2).await.unwrap();
        repo.record(recent.clone()).await.unwrap();

        let purge_before = base_time - Duration::days(2);
        let deleted_count = repo.purge_before(purge_before).await.unwrap();

        assert_eq!(deleted_count, 2);

        let from = base_time - Duration::days(10);
        let to = base_time + Duration::hours(1);
        let remaining = repo
            .find_by_entity_in_range(entity_id, from, to, None)
            .await
            .unwrap();

        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, recent.id);
    }

    #[tokio::test]
    async fn should_return_zero_when_purging_with_no_old_records() {
        let (repo, entity_id) = setup().await;
        let base_time = now();
        let history = test_history(entity_id, base_time);
        repo.record(history).await.unwrap();

        let purge_before = base_time - Duration::days(10);
        let deleted_count = repo.purge_before(purge_before).await.unwrap();

        assert_eq!(deleted_count, 0);
    }

    #[tokio::test]
    async fn should_preserve_attributes_through_roundtrip() {
        let (repo, entity_id) = setup().await;
        let timestamp = now();
        let history = EntityHistory::builder()
            .entity_id(entity_id)
            .state(EntityState::On)
            .attribute("temperature", AttributeValue::Float(22.5))
            .attribute("humidity", AttributeValue::Int(65))
            .attribute("unit", AttributeValue::String("°C".to_string()))
            .recorded_at(timestamp)
            .build();

        repo.record(history.clone()).await.unwrap();

        let from = timestamp - Duration::hours(1);
        let to = timestamp + Duration::hours(1);
        let found = repo
            .find_by_entity_in_range(entity_id, from, to, None)
            .await
            .unwrap();

        assert_eq!(found.len(), 1);
        assert_eq!(
            found[0].get_attribute("temperature"),
            Some(&AttributeValue::Float(22.5))
        );
        assert_eq!(
            found[0].get_attribute("humidity"),
            Some(&AttributeValue::Int(65))
        );
        assert_eq!(
            found[0].get_attribute("unit"),
            Some(&AttributeValue::String("°C".to_string()))
        );
    }

    #[tokio::test]
    async fn should_filter_by_entity_id() {
        let (repo, entity_id1) = setup().await;

        // Create a second entity
        let entity_id2 = EntityId::new();
        let device_id = uuid::Uuid::new_v4();
        let pool = &repo.pool;
        sqlx::query("INSERT INTO devices (id, name, integration, unique_id) VALUES (?, ?, ?, ?)")
            .bind(device_id)
            .bind("Device 2")
            .bind("test")
            .bind("device2")
            .execute(pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO entities (id, device_id, entity_id, friendly_name, state, attributes, last_changed, last_updated) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(entity_id2.as_uuid())
            .bind(device_id)
            .bind("sensor.test2")
            .bind("Test Sensor 2")
            .bind("unknown")
            .bind("{}")
            .bind(now().to_rfc3339())
            .bind(now().to_rfc3339())
            .execute(pool)
            .await
            .unwrap();

        let timestamp = now();
        let history1 = test_history(entity_id1, timestamp);
        let history2 = test_history(entity_id2, timestamp);

        repo.record(history1).await.unwrap();
        repo.record(history2).await.unwrap();

        let from = timestamp - Duration::hours(1);
        let to = timestamp + Duration::hours(1);

        let found1 = repo
            .find_by_entity_in_range(entity_id1, from, to, None)
            .await
            .unwrap();
        assert_eq!(found1.len(), 1);
        assert_eq!(found1[0].entity_id, entity_id1);

        let found2 = repo
            .find_by_entity_in_range(entity_id2, from, to, None)
            .await
            .unwrap();
        assert_eq!(found2.len(), 1);
        assert_eq!(found2[0].entity_id, entity_id2);
    }
}
