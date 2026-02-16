//! `SQLite` implementation of [`EventStore`].

use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqlitePool};

use minihub_app::ports::EventStore;
use minihub_domain::error::MiniHubError;
use minihub_domain::event::{Event, EventType};
use minihub_domain::id::{EntityId, EventId};

use crate::error::StorageError;

struct Wrapper(Event);

impl Wrapper {
    fn maybe(value: Option<Self>) -> Option<Event> {
        value.map(|w| w.0)
    }
}

impl<'r> FromRow<'r, SqliteRow> for Wrapper {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let id: uuid::Uuid = row.try_get("id")?;
        let event_type: String = row.try_get("event_type")?;
        let entity_id: Option<uuid::Uuid> = row.try_get("entity_id")?;
        let timestamp_str: String = row.try_get("timestamp")?;
        let data_json: String = row.try_get("data")?;

        let id = EventId::from_uuid(id);
        let event_type: EventType = serde_json::from_str(&format!("\"{event_type}\""))
            .map_err(|err| sqlx::Error::Decode(Box::new(err)))?;
        let entity_id = entity_id.map(EntityId::from_uuid);
        let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp_str)
            .map_err(|err| sqlx::Error::Decode(Box::new(err)))?
            .to_utc();
        let data: serde_json::Value =
            serde_json::from_str(&data_json).map_err(|err| sqlx::Error::Decode(Box::new(err)))?;

        Ok(Self(Event {
            id,
            event_type,
            entity_id,
            timestamp,
            data,
        }))
    }
}

const INSERT: &str = r"
    INSERT INTO events (id, event_type, entity_id, timestamp, data)
    VALUES (?, ?, ?, ?, ?)
";

const SELECT_BY_ID: &str = "SELECT * FROM events WHERE id = ?";
const SELECT_RECENT: &str = "SELECT * FROM events ORDER BY timestamp DESC LIMIT ?";
const SELECT_BY_ENTITY: &str =
    "SELECT * FROM events WHERE entity_id = ? ORDER BY timestamp DESC LIMIT ?";

/// `SQLite`-backed event store.
pub struct SqliteEventStore {
    pool: SqlitePool,
}

impl SqliteEventStore {
    /// Create a new event store using the given connection pool.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl EventStore for SqliteEventStore {
    async fn store(&self, event: Event) -> Result<Event, MiniHubError> {
        let data_json = serde_json::to_string(&event.data).map_err(StorageError::from)?;

        sqlx::query(INSERT)
            .bind(event.id.as_uuid())
            .bind(event.event_type.as_str())
            .bind(event.entity_id.map(EntityId::as_uuid))
            .bind(event.timestamp.to_rfc3339())
            .bind(&data_json)
            .execute(&self.pool)
            .await
            .map_err(StorageError::from)?;

        Ok(event)
    }

    async fn get_by_id(&self, id: EventId) -> Result<Option<Event>, MiniHubError> {
        let row: Option<Wrapper> = sqlx::query_as(SELECT_BY_ID)
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(StorageError::from)?;

        Ok(Wrapper::maybe(row))
    }

    async fn get_recent(&self, limit: usize) -> Result<Vec<Event>, MiniHubError> {
        let limit = i32::try_from(limit).unwrap_or(i32::MAX);
        let rows: Vec<Wrapper> = sqlx::query_as(SELECT_RECENT)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::from)?;

        Ok(rows.into_iter().map(|w| w.0).collect())
    }

    async fn find_by_entity(
        &self,
        entity_id: EntityId,
        limit: usize,
    ) -> Result<Vec<Event>, MiniHubError> {
        let limit = i32::try_from(limit).unwrap_or(i32::MAX);
        let rows: Vec<Wrapper> = sqlx::query_as(SELECT_BY_ENTITY)
            .bind(entity_id.as_uuid())
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::from)?;

        Ok(rows.into_iter().map(|w| w.0).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::Config;
    use minihub_domain::event::EventType;
    use minihub_domain::id::{DeviceId, EntityId};

    async fn setup() -> (SqliteEventStore, EntityId) {
        let db = Config {
            database_url: "sqlite::memory:".to_string(),
        }
        .build()
        .await
        .unwrap();
        let pool = db.pool().clone();

        let device_id = DeviceId::new();
        sqlx::query("INSERT INTO devices (id, name) VALUES (?, ?)")
            .bind(device_id.as_uuid())
            .bind("Test Device")
            .execute(&pool)
            .await
            .unwrap();

        let entity_id = EntityId::new();
        let now = chrono::Utc::now();
        sqlx::query("INSERT INTO entities (id, device_id, entity_id, friendly_name, state, attributes, last_changed, last_updated) VALUES (?, ?, ?, ?, ?, '{}', ?, ?)")
            .bind(entity_id.as_uuid())
            .bind(device_id.as_uuid())
            .bind("light.test")
            .bind("Test Light")
            .bind("off")
            .bind(now.to_rfc3339())
            .bind(now.to_rfc3339())
            .execute(&pool)
            .await
            .unwrap();

        (SqliteEventStore::new(pool), entity_id)
    }

    fn test_event(entity_id: Option<EntityId>) -> Event {
        Event::new(
            EventType::StateChanged,
            entity_id,
            serde_json::json!({"from": "off", "to": "on"}),
        )
    }

    #[tokio::test]
    async fn should_store_and_retrieve_event_by_id() {
        let (store, entity_id) = setup().await;
        let event = test_event(Some(entity_id));
        let id = event.id;

        store.store(event).await.unwrap();

        let fetched = store.get_by_id(id).await.unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.id, id);
        assert_eq!(fetched.event_type, EventType::StateChanged);
        assert_eq!(fetched.entity_id, Some(entity_id));
        assert_eq!(fetched.data["from"], "off");
    }

    #[tokio::test]
    async fn should_return_none_when_event_not_found() {
        let (store, _) = setup().await;
        let result = store.get_by_id(EventId::new()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn should_store_event_without_entity_id() {
        let (store, _) = setup().await;
        let event = test_event(None);
        let id = event.id;

        store.store(event).await.unwrap();

        let fetched = store.get_by_id(id).await.unwrap().unwrap();
        assert!(fetched.entity_id.is_none());
    }

    #[tokio::test]
    async fn should_get_recent_events_ordered_newest_first() {
        let (store, entity_id) = setup().await;

        let e1 = test_event(Some(entity_id));
        let e1_id = e1.id;
        store.store(e1).await.unwrap();

        let e2 = Event::new(
            EventType::EntityCreated,
            None,
            serde_json::json!({"name": "test"}),
        );
        let e2_id = e2.id;
        store.store(e2).await.unwrap();

        let recent = store.get_recent(10).await.unwrap();
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].id, e2_id);
        assert_eq!(recent[1].id, e1_id);
    }

    #[tokio::test]
    async fn should_respect_limit_on_get_recent() {
        let (store, _) = setup().await;

        for _ in 0..5 {
            store.store(test_event(None)).await.unwrap();
        }

        let recent = store.get_recent(3).await.unwrap();
        assert_eq!(recent.len(), 3);
    }

    #[tokio::test]
    async fn should_find_events_by_entity_id() {
        let (store, entity_id) = setup().await;

        store.store(test_event(Some(entity_id))).await.unwrap();
        store.store(test_event(Some(entity_id))).await.unwrap();
        store.store(test_event(None)).await.unwrap();

        let by_entity = store.find_by_entity(entity_id, 10).await.unwrap();
        assert_eq!(by_entity.len(), 2);

        let other = store.find_by_entity(EntityId::new(), 10).await.unwrap();
        assert!(other.is_empty());
    }

    #[tokio::test]
    async fn should_preserve_event_data_through_roundtrip() {
        let (store, _) = setup().await;
        let event = Event::new(
            EventType::AttributeChanged,
            None,
            serde_json::json!({"key": "brightness", "old": 100, "new": 200}),
        );
        let id = event.id;

        store.store(event).await.unwrap();

        let fetched = store.get_by_id(id).await.unwrap().unwrap();
        assert_eq!(fetched.data["key"], "brightness");
        assert_eq!(fetched.data["old"], 100);
        assert_eq!(fetched.data["new"], 200);
    }
}
