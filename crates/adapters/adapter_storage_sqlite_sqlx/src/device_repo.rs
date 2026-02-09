//! `SQLite` implementation of [`DeviceRepository`].

use std::future::Future;
use std::str::FromStr;

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
        let id: String = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        let manufacturer: Option<String> = row.try_get("manufacturer")?;
        let model: Option<String> = row.try_get("model")?;
        let area_id: Option<String> = row.try_get("area_id")?;

        let id = DeviceId::from_str(&id).map_err(|err| sqlx::Error::Decode(Box::new(err)))?;
        let area_id = area_id
            .map(|s| AreaId::from_str(&s))
            .transpose()
            .map_err(|err| sqlx::Error::Decode(Box::new(err)))?;

        Ok(Self(Device {
            id,
            name,
            manufacturer,
            model,
            area_id,
        }))
    }
}

const INSERT: &str =
    "INSERT INTO devices (id, name, manufacturer, model, area_id) VALUES (?, ?, ?, ?, ?)";
const SELECT_BY_ID: &str = "SELECT * FROM devices WHERE id = ?";
const SELECT_ALL: &str = "SELECT * FROM devices";
const UPDATE: &str =
    "UPDATE devices SET name = ?, manufacturer = ?, model = ?, area_id = ? WHERE id = ?";
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
    fn create(&self, device: Device) -> impl Future<Output = Result<Device, MiniHubError>> + Send {
        let pool = self.pool.clone();
        async move {
            sqlx::query(INSERT)
                .bind(device.id.to_string())
                .bind(&device.name)
                .bind(&device.manufacturer)
                .bind(&device.model)
                .bind(device.area_id.map(|id| id.to_string()))
                .execute(&pool)
                .await
                .map_err(StorageError::from)?;

            Ok(device)
        }
    }

    fn get_by_id(
        &self,
        id: DeviceId,
    ) -> impl Future<Output = Result<Option<Device>, MiniHubError>> + Send {
        let pool = self.pool.clone();
        async move {
            let row: Option<Wrapper> = sqlx::query_as(SELECT_BY_ID)
                .bind(id.to_string())
                .fetch_optional(&pool)
                .await
                .map_err(StorageError::from)?;

            Ok(Wrapper::maybe(row))
        }
    }

    fn get_all(&self) -> impl Future<Output = Result<Vec<Device>, MiniHubError>> + Send {
        let pool = self.pool.clone();
        async move {
            let rows: Vec<Wrapper> = sqlx::query_as(SELECT_ALL)
                .fetch_all(&pool)
                .await
                .map_err(StorageError::from)?;

            Ok(rows.into_iter().map(|w| w.0).collect())
        }
    }

    fn update(&self, device: Device) -> impl Future<Output = Result<Device, MiniHubError>> + Send {
        let pool = self.pool.clone();
        async move {
            sqlx::query(UPDATE)
                .bind(&device.name)
                .bind(&device.manufacturer)
                .bind(&device.model)
                .bind(device.area_id.map(|id| id.to_string()))
                .bind(device.id.to_string())
                .execute(&pool)
                .await
                .map_err(StorageError::from)?;

            Ok(device)
        }
    }

    fn delete(&self, id: DeviceId) -> impl Future<Output = Result<(), MiniHubError>> + Send {
        let pool = self.pool.clone();
        async move {
            sqlx::query(DELETE_BY_ID)
                .bind(id.to_string())
                .execute(&pool)
                .await
                .map_err(StorageError::from)?;

            Ok(())
        }
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
        Device::builder().name("Hue Bridge").build().unwrap()
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
        repo.create(Device::builder().name("Motion Sensor").build().unwrap())
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
            .build()
            .unwrap();
        let id = device.id;
        repo.create(device).await.unwrap();

        let fetched = repo.get_by_id(id).await.unwrap().unwrap();
        assert_eq!(fetched.manufacturer.as_deref(), Some("Aqara"));
        assert_eq!(fetched.model.as_deref(), Some("RTCGQ11LM"));
        assert!(fetched.area_id.is_none());
    }
}
