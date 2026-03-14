//! `SQLite` implementation of [`DeviceRepository`].

use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqlitePool};

use minihub_app::ports::DeviceRepository;
use minihub_domain::device::Device;
use minihub_domain::error::MiniHubError;
use minihub_domain::id::{AreaId, DeviceId};

use crate::error::StorageError;

/// Wrapper for converting database rows into domain [`Device`].
struct Wrapper(Device);

impl Wrapper {
    fn maybe(value: Option<Self>) -> Option<Device> {
        value.map(|w| w.0)
    }
}

impl<'r> FromRow<'r, SqliteRow> for Wrapper {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let id: uuid::Uuid = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        let manufacturer: Option<String> = row.try_get("manufacturer")?;
        let model: Option<String> = row.try_get("model")?;
        let area_id: Option<uuid::Uuid> = row.try_get("area_id")?;

        let id = DeviceId::from_uuid(id);
        let area_id = area_id.map(AreaId::from_uuid);

        let integration: String = row.try_get("integration")?;
        let unique_id: String = row.try_get("unique_id")?;

        Ok(Self(Device {
            id,
            name,
            manufacturer,
            model,
            area_id,
            integration,
            unique_id,
        }))
    }
}

const INSERT: &str = "INSERT INTO devices (id, name, manufacturer, model, area_id, integration, unique_id) VALUES (?, ?, ?, ?, ?, ?, ?)";
const SELECT_BY_ID: &str = "SELECT * FROM devices WHERE id = ?";
const SELECT_ALL: &str = "SELECT * FROM devices";
const SELECT_BY_INTEGRATION_UNIQUE_ID: &str =
    "SELECT * FROM devices WHERE integration = ? AND unique_id = ?";
const UPDATE: &str = "UPDATE devices SET name = ?, manufacturer = ?, model = ?, area_id = ?, integration = ?, unique_id = ? WHERE id = ?";
const DELETE_BY_ID: &str = "DELETE FROM devices WHERE id = ?";

/// `SQLite`-backed device repository.
pub struct SqliteDeviceRepository {
    pool: SqlitePool,
}

impl SqliteDeviceRepository {
    /// Create a new repository using the given connection pool.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl DeviceRepository for SqliteDeviceRepository {
    async fn create(&self, device: Device) -> Result<Device, MiniHubError> {
        sqlx::query(INSERT)
            .bind(device.id.as_uuid())
            .bind(&device.name)
            .bind(&device.manufacturer)
            .bind(&device.model)
            .bind(device.area_id.map(AreaId::as_uuid))
            .bind(&device.integration)
            .bind(&device.unique_id)
            .execute(&self.pool)
            .await
            .map_err(StorageError::from)?;

        Ok(device)
    }

    async fn get_by_id(&self, id: DeviceId) -> Result<Option<Device>, MiniHubError> {
        let row: Option<Wrapper> = sqlx::query_as(SELECT_BY_ID)
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(StorageError::from)?;

        Ok(Wrapper::maybe(row))
    }

    async fn get_all(&self) -> Result<Vec<Device>, MiniHubError> {
        let rows: Vec<Wrapper> = sqlx::query_as(SELECT_ALL)
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::from)?;

        Ok(rows.into_iter().map(|w| w.0).collect())
    }

    async fn find_by_integration_unique_id(
        &self,
        integration: &str,
        unique_id: &str,
    ) -> Result<Option<Device>, MiniHubError> {
        let row: Option<Wrapper> = sqlx::query_as(SELECT_BY_INTEGRATION_UNIQUE_ID)
            .bind(integration)
            .bind(unique_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(StorageError::from)?;

        Ok(Wrapper::maybe(row))
    }

    async fn update(&self, device: Device) -> Result<Device, MiniHubError> {
        sqlx::query(UPDATE)
            .bind(&device.name)
            .bind(&device.manufacturer)
            .bind(&device.model)
            .bind(device.area_id.map(AreaId::as_uuid))
            .bind(&device.integration)
            .bind(&device.unique_id)
            .bind(device.id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(StorageError::from)?;

        Ok(device)
    }

    async fn delete(&self, id: DeviceId) -> Result<(), MiniHubError> {
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

    async fn setup() -> SqliteDeviceRepository {
        let db = Config {
            database_url: "sqlite::memory:".to_string(),
        }
        .build()
        .await
        .unwrap();
        SqliteDeviceRepository::new(db.pool().clone())
    }

    fn test_device() -> Device {
        Device::builder()
            .name("Hue Bridge")
            .integration("test")
            .unique_id("hue_bridge_1")
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn should_create_and_retrieve_device_when_valid() {
        let repo = setup().await;
        let device = test_device();
        let id = device.id;

        repo.create(device).await.unwrap();

        let fetched = repo.get_by_id(id).await.unwrap().unwrap();
        assert_eq!(fetched.id, id);
        assert_eq!(fetched.name, "Hue Bridge");
    }

    #[tokio::test]
    async fn should_return_none_when_device_not_found() {
        let repo = setup().await;
        let result = repo.get_by_id(DeviceId::new()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn should_list_all_devices() {
        let repo = setup().await;
        repo.create(test_device()).await.unwrap();
        repo.create(
            Device::builder()
                .name("Motion Sensor")
                .integration("test")
                .unique_id("motion_sensor_1")
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

        let all = repo.get_all().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn should_update_device_when_exists() {
        let repo = setup().await;
        let mut device = test_device();
        let id = device.id;
        repo.create(device.clone()).await.unwrap();

        device.name = "Updated Bridge".to_string();
        device.manufacturer = Some("Philips".to_string());
        repo.update(device).await.unwrap();

        let fetched = repo.get_by_id(id).await.unwrap().unwrap();
        assert_eq!(fetched.name, "Updated Bridge");
        assert_eq!(fetched.manufacturer.as_deref(), Some("Philips"));
    }

    #[tokio::test]
    async fn should_delete_device_when_exists() {
        let repo = setup().await;
        let device = test_device();
        let id = device.id;
        repo.create(device).await.unwrap();

        repo.delete(id).await.unwrap();

        let result = repo.get_by_id(id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn should_preserve_optional_fields_through_roundtrip() {
        let repo = setup().await;
        let device = Device::builder()
            .name("Sensor")
            .manufacturer("Aqara")
            .model("RTCGQ11LM")
            .integration("zigbee")
            .unique_id("sensor_1")
            .build()
            .unwrap();
        let id = device.id;
        repo.create(device).await.unwrap();

        let fetched = repo.get_by_id(id).await.unwrap().unwrap();
        assert_eq!(fetched.manufacturer.as_deref(), Some("Aqara"));
        assert_eq!(fetched.model.as_deref(), Some("RTCGQ11LM"));
        assert!(fetched.area_id.is_none());
    }

    #[tokio::test]
    async fn should_find_device_by_integration_and_unique_id() {
        let repo = setup().await;
        let device = Device::builder()
            .name("BLE Sensor")
            .integration("ble")
            .unique_id("A4:C1:38:5B:0E:DF")
            .build()
            .unwrap();
        let id = device.id;
        repo.create(device).await.unwrap();

        let found = repo
            .find_by_integration_unique_id("ble", "A4:C1:38:5B:0E:DF")
            .await
            .unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, id);
    }

    #[tokio::test]
    async fn should_return_none_when_integration_unique_id_not_found() {
        let repo = setup().await;
        let found = repo
            .find_by_integration_unique_id("ble", "FF:FF:FF:FF:FF:FF")
            .await
            .unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn should_preserve_integration_and_unique_id_through_roundtrip() {
        let repo = setup().await;
        let device = Device::builder()
            .name("MQTT Device")
            .integration("mqtt")
            .unique_id("kitchen_hub")
            .build()
            .unwrap();
        let id = device.id;
        repo.create(device).await.unwrap();

        let fetched = repo.get_by_id(id).await.unwrap().unwrap();
        assert_eq!(fetched.integration, "mqtt");
        assert_eq!(fetched.unique_id, "kitchen_hub");
    }
}
