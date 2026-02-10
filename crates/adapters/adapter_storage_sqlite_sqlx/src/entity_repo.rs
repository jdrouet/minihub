//! `SQLite` implementation of [`EntityRepository`].

use std::collections::HashMap;
use std::str::FromStr;

use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqlitePool};

use minihub_app::ports::EntityRepository;
use minihub_domain::entity::{AttributeValue, Entity, EntityState};
use minihub_domain::error::MiniHubError;
use minihub_domain::id::{DeviceId, EntityId};

use crate::error::StorageError;

/// Wrapper for converting database rows into domain types without polluting
/// domain structs with database concerns.
struct Wrapper(Entity);

impl Wrapper {
    fn maybe(value: Option<Self>) -> Option<Entity> {
        value.map(|w| w.0)
    }
}

impl<'r> FromRow<'r, SqliteRow> for Wrapper {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let id: String = row.try_get("id")?;
        let device_id: String = row.try_get("device_id")?;
        let entity_id: String = row.try_get("entity_id")?;
        let friendly_name: String = row.try_get("friendly_name")?;
        let state_str: String = row.try_get("state")?;
        let attributes_json: String = row.try_get("attributes")?;
        let last_changed_str: String = row.try_get("last_changed")?;
        let last_updated_str: String = row.try_get("last_updated")?;

        let id = EntityId::from_str(&id).map_err(|err| sqlx::Error::Decode(Box::new(err)))?;
        let device_id =
            DeviceId::from_str(&device_id).map_err(|err| sqlx::Error::Decode(Box::new(err)))?;
        let state: EntityState = serde_json::from_str(&format!("\"{state_str}\""))
            .map_err(|err| sqlx::Error::Decode(Box::new(err)))?;
        let attributes: HashMap<String, AttributeValue> = serde_json::from_str(&attributes_json)
            .map_err(|err| sqlx::Error::Decode(Box::new(err)))?;
        let last_changed = chrono::DateTime::parse_from_rfc3339(&last_changed_str)
            .map_err(|err| sqlx::Error::Decode(Box::new(err)))?
            .to_utc();
        let last_updated = chrono::DateTime::parse_from_rfc3339(&last_updated_str)
            .map_err(|err| sqlx::Error::Decode(Box::new(err)))?
            .to_utc();

        Ok(Self(Entity {
            id,
            device_id,
            entity_id,
            friendly_name,
            state,
            attributes,
            last_changed,
            last_updated,
        }))
    }
}

const INSERT: &str = r"
    INSERT INTO entities (id, device_id, entity_id, friendly_name, state, attributes, last_changed, last_updated)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
";

const SELECT_BY_ID: &str = "SELECT * FROM entities WHERE id = ?";
const SELECT_ALL: &str = "SELECT * FROM entities";
const SELECT_BY_DEVICE: &str = "SELECT * FROM entities WHERE device_id = ?";
const SELECT_BY_ENTITY_ID: &str = "SELECT * FROM entities WHERE entity_id = ?";

const UPDATE: &str = r"
    UPDATE entities
    SET device_id = ?, entity_id = ?, friendly_name = ?, state = ?, attributes = ?,
        last_changed = ?, last_updated = ?
    WHERE id = ?
";

const DELETE_BY_ID: &str = "DELETE FROM entities WHERE id = ?";

/// `SQLite`-backed entity repository.
pub struct SqliteEntityRepository {
    pool: SqlitePool,
}

impl SqliteEntityRepository {
    /// Create a new repository using the given connection pool.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl EntityRepository for SqliteEntityRepository {
    async fn create(&self, entity: Entity) -> Result<Entity, MiniHubError> {
        let attributes_json =
            serde_json::to_string(&entity.attributes).map_err(StorageError::from)?;

        sqlx::query(INSERT)
            .bind(entity.id.as_uuid())
            .bind(entity.device_id.as_uuid())
            .bind(&entity.entity_id)
            .bind(&entity.friendly_name)
            .bind(entity.state.to_string())
            .bind(&attributes_json)
            .bind(entity.last_changed.to_rfc3339())
            .bind(entity.last_updated.to_rfc3339())
            .execute(&self.pool)
            .await
            .map_err(StorageError::from)?;

        Ok(entity)
    }

    async fn get_by_id(&self, id: EntityId) -> Result<Option<Entity>, MiniHubError> {
        let row: Option<Wrapper> = sqlx::query_as(SELECT_BY_ID)
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(StorageError::from)?;

        Ok(Wrapper::maybe(row))
    }

    async fn get_all(&self) -> Result<Vec<Entity>, MiniHubError> {
        let rows: Vec<Wrapper> = sqlx::query_as(SELECT_ALL)
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::from)?;

        Ok(rows.into_iter().map(|w| w.0).collect())
    }

    async fn find_by_device_id(&self, device_id: DeviceId) -> Result<Vec<Entity>, MiniHubError> {
        let rows: Vec<Wrapper> = sqlx::query_as(SELECT_BY_DEVICE)
            .bind(device_id.as_uuid())
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::from)?;

        Ok(rows.into_iter().map(|w| w.0).collect())
    }

    async fn find_by_entity_id(&self, entity_id: &str) -> Result<Option<Entity>, MiniHubError> {
        let row: Option<Wrapper> = sqlx::query_as(SELECT_BY_ENTITY_ID)
            .bind(entity_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(StorageError::from)?;

        Ok(Wrapper::maybe(row))
    }

    async fn update(&self, entity: Entity) -> Result<Entity, MiniHubError> {
        let attributes_json =
            serde_json::to_string(&entity.attributes).map_err(StorageError::from)?;

        sqlx::query(UPDATE)
            .bind(entity.device_id.as_uuid())
            .bind(&entity.entity_id)
            .bind(&entity.friendly_name)
            .bind(entity.state.to_string())
            .bind(&attributes_json)
            .bind(entity.last_changed.to_rfc3339())
            .bind(entity.last_updated.to_rfc3339())
            .bind(entity.id.to_string())
            .execute(&self.pool)
            .await
            .map_err(StorageError::from)?;

        Ok(entity)
    }

    async fn delete(&self, id: EntityId) -> Result<(), MiniHubError> {
        sqlx::query(DELETE_BY_ID)
            .bind(id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(StorageError::from)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::Config;
    use minihub_domain::entity::EntityState;
    use minihub_domain::id::DeviceId;

    async fn setup() -> (SqliteEntityRepository, DeviceId) {
        let db = Config {
            database_url: "sqlite::memory:".to_string(),
        }
        .build()
        .await
        .unwrap();
        let pool = db.pool().clone();
        let device_id = DeviceId::new();

        sqlx::query("INSERT INTO devices (id, name) VALUES (?, ?)")
            .bind(device_id.to_string())
            .bind("Test Device")
            .execute(&pool)
            .await
            .unwrap();

        (SqliteEntityRepository::new(pool), device_id)
    }

    fn test_entity(device_id: DeviceId) -> Entity {
        Entity::builder()
            .device_id(device_id)
            .entity_id("light.living_room")
            .friendly_name("Living Room Light")
            .state(EntityState::Off)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn should_create_and_retrieve_entity_when_valid() {
        let (repo, device_id) = setup().await;
        let entity = test_entity(device_id);
        let id = entity.id;

        repo.create(entity).await.unwrap();

        let fetched = repo.get_by_id(id).await.unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.id, id);
        assert_eq!(fetched.entity_id, "light.living_room");
        assert_eq!(fetched.friendly_name, "Living Room Light");
        assert_eq!(fetched.state, EntityState::Off);
    }

    #[tokio::test]
    async fn should_return_none_when_entity_not_found() {
        let (repo, _device_id) = setup().await;
        let result = repo.get_by_id(EntityId::new()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn should_list_all_entities() {
        let (repo, device_id) = setup().await;
        repo.create(test_entity(device_id)).await.unwrap();

        let mut entity2 = test_entity(device_id);
        entity2.entity_id = "sensor.temperature".to_string();
        repo.create(entity2).await.unwrap();

        let all = repo.get_all().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn should_find_entities_by_device_id() {
        let (repo, device_id) = setup().await;
        let entity = test_entity(device_id);
        repo.create(entity).await.unwrap();

        let found = repo.find_by_device_id(device_id).await.unwrap();
        assert_eq!(found.len(), 1);

        let not_found = repo.find_by_device_id(DeviceId::new()).await.unwrap();
        assert!(not_found.is_empty());
    }

    #[tokio::test]
    async fn should_update_entity_when_exists() {
        let (repo, device_id) = setup().await;
        let mut entity = test_entity(device_id);
        let id = entity.id;
        repo.create(entity.clone()).await.unwrap();

        entity.state = EntityState::On;
        entity.friendly_name = "Updated Name".to_string();
        repo.update(entity).await.unwrap();

        let fetched = repo.get_by_id(id).await.unwrap().unwrap();
        assert_eq!(fetched.state, EntityState::On);
        assert_eq!(fetched.friendly_name, "Updated Name");
    }

    #[tokio::test]
    async fn should_delete_entity_when_exists() {
        let (repo, device_id) = setup().await;
        let entity = test_entity(device_id);
        let id = entity.id;
        repo.create(entity).await.unwrap();

        repo.delete(id).await.unwrap();

        let result = repo.get_by_id(id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn should_find_entity_by_entity_id_string() {
        let (repo, device_id) = setup().await;
        let entity = test_entity(device_id);
        repo.create(entity).await.unwrap();

        let found = repo.find_by_entity_id("light.living_room").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().entity_id, "light.living_room");
    }

    #[tokio::test]
    async fn should_return_none_when_entity_id_string_not_found() {
        let (repo, _device_id) = setup().await;
        let found = repo.find_by_entity_id("sensor.nonexistent").await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn should_preserve_attributes_through_roundtrip() {
        let (repo, device_id) = setup().await;
        let entity = Entity::builder()
            .device_id(device_id)
            .entity_id("sensor.temp")
            .friendly_name("Temperature")
            .attribute("unit", AttributeValue::String("°C".to_string()))
            .attribute("precision", AttributeValue::Int(2))
            .build()
            .unwrap();
        let id = entity.id;
        repo.create(entity).await.unwrap();

        let fetched = repo.get_by_id(id).await.unwrap().unwrap();
        assert_eq!(
            fetched.get_attribute("unit"),
            Some(&AttributeValue::String("°C".to_string()))
        );
        assert_eq!(
            fetched.get_attribute("precision"),
            Some(&AttributeValue::Int(2))
        );
    }
}
