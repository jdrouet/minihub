//! Device service — use-cases for managing devices.

use minihub_domain::device::Device;
use minihub_domain::error::{MiniHubError, NotFoundError};
use minihub_domain::id::DeviceId;

use crate::ports::DeviceRepository;

/// Application service for device CRUD operations.
pub struct DeviceService<R> {
    repo: R,
}

impl<R: DeviceRepository> DeviceService<R> {
    /// Create a new service backed by the given repository.
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Create a new device after validating domain invariants.
    ///
    /// # Errors
    ///
    /// Returns [`MiniHubError::Validation`] if invariants fail, or a
    /// storage error propagated from the repository.
    #[tracing::instrument(skip(self, device), fields(device_name = %device.name))]
    pub async fn create_device(&self, device: Device) -> Result<Device, MiniHubError> {
        device.validate()?;
        self.repo.create(device).await
    }

    /// Look up a device by id, returning an error if not found.
    ///
    /// # Errors
    ///
    /// Returns [`MiniHubError::NotFound`] when no device with `id` exists,
    /// or a storage error from the repository.
    #[tracing::instrument(skip(self))]
    pub async fn get_device(&self, id: DeviceId) -> Result<Device, MiniHubError> {
        self.repo.get_by_id(id).await?.ok_or_else(|| {
            NotFoundError {
                entity: "Device",
                id: id.to_string(),
            }
            .into()
        })
    }

    /// List all devices.
    ///
    /// # Errors
    ///
    /// Returns a storage error propagated from the repository.
    pub async fn list_devices(&self) -> Result<Vec<Device>, MiniHubError> {
        self.repo.get_all().await
    }

    /// Update an existing device.
    ///
    /// # Errors
    ///
    /// Returns [`MiniHubError::Validation`] if invariants fail, or a
    /// storage error from the repository.
    #[tracing::instrument(skip(self, device))]
    pub async fn update_device(&self, device: Device) -> Result<Device, MiniHubError> {
        device.validate()?;
        self.repo.update(device).await
    }

    /// Create or update a device by its `(integration, unique_id)` pair.
    ///
    /// If a device with the same integration and unique id already exists, its
    /// name, manufacturer, model, and area are updated (preserving the original
    /// UUID). Otherwise a new device is created.
    ///
    /// # Errors
    ///
    /// Returns [`MiniHubError::Validation`] if invariants fail, or a
    /// storage error propagated from the repository.
    #[tracing::instrument(skip(self, device), fields(device_name = %device.name))]
    pub async fn upsert_device(&self, device: Device) -> Result<Device, MiniHubError> {
        if let Some(existing) = self
            .repo
            .find_by_integration_unique_id(&device.integration, &device.unique_id)
            .await?
        {
            let updated = Device {
                id: existing.id,
                name: device.name,
                manufacturer: device.manufacturer,
                model: device.model,
                area_id: existing.area_id,
                integration: device.integration,
                unique_id: device.unique_id,
            };
            return self.update_device(updated).await;
        }
        self.create_device(device).await
    }

    /// Delete a device by id.
    ///
    /// # Errors
    ///
    /// Returns a storage error propagated from the repository.
    #[tracing::instrument(skip(self))]
    pub async fn delete_device(&self, id: DeviceId) -> Result<(), MiniHubError> {
        self.repo.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use minihub_domain::error::ValidationError;
    use std::collections::HashMap;
    use std::future::Future;
    use std::sync::Mutex;

    struct InMemoryDeviceRepo {
        store: Mutex<HashMap<DeviceId, Device>>,
    }

    impl Default for InMemoryDeviceRepo {
        fn default() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }
    }

    impl DeviceRepository for InMemoryDeviceRepo {
        fn create(
            &self,
            device: Device,
        ) -> impl Future<Output = Result<Device, MiniHubError>> + Send {
            let mut store = self.store.lock().unwrap();
            store.insert(device.id, device.clone());
            async { Ok(device) }
        }

        fn get_by_id(
            &self,
            id: DeviceId,
        ) -> impl Future<Output = Result<Option<Device>, MiniHubError>> + Send {
            let store = self.store.lock().unwrap();
            let result = store.get(&id).cloned();
            async { Ok(result) }
        }

        fn get_all(&self) -> impl Future<Output = Result<Vec<Device>, MiniHubError>> + Send {
            let store = self.store.lock().unwrap();
            let result: Vec<Device> = store.values().cloned().collect();
            async { Ok(result) }
        }

        fn find_by_integration_unique_id(
            &self,
            integration: &str,
            unique_id: &str,
        ) -> impl Future<Output = Result<Option<Device>, MiniHubError>> + Send {
            let store = self.store.lock().unwrap();
            let result = store
                .values()
                .find(|d| d.integration == integration && d.unique_id == unique_id)
                .cloned();
            async { Ok(result) }
        }

        fn update(
            &self,
            device: Device,
        ) -> impl Future<Output = Result<Device, MiniHubError>> + Send {
            let mut store = self.store.lock().unwrap();
            store.insert(device.id, device.clone());
            async { Ok(device) }
        }

        fn delete(&self, id: DeviceId) -> impl Future<Output = Result<(), MiniHubError>> + Send {
            let mut store = self.store.lock().unwrap();
            store.remove(&id);
            async { Ok(()) }
        }
    }

    fn make_service() -> DeviceService<InMemoryDeviceRepo> {
        DeviceService::new(InMemoryDeviceRepo::default())
    }

    fn valid_device() -> Device {
        Device::builder()
            .name("Hue Bridge")
            .integration("test")
            .unique_id("hue_bridge_1")
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn should_create_device_when_valid() {
        let svc = make_service();
        let device = valid_device();
        let id = device.id;

        let created = svc.create_device(device).await.unwrap();
        assert_eq!(created.id, id);

        let fetched = svc.get_device(id).await.unwrap();
        assert_eq!(fetched.name, "Hue Bridge");
    }

    #[tokio::test]
    async fn should_reject_create_when_name_is_empty() {
        let svc = make_service();
        let mut device = valid_device();
        device.name = String::new();

        let result = svc.create_device(device).await;
        assert!(matches!(
            result,
            Err(MiniHubError::Validation(ValidationError::EmptyName))
        ));
    }

    #[tokio::test]
    async fn should_return_not_found_when_device_missing() {
        let svc = make_service();
        let result = svc.get_device(DeviceId::new()).await;
        assert!(matches!(result, Err(MiniHubError::NotFound(_))));
    }

    #[tokio::test]
    async fn should_list_all_devices() {
        let svc = make_service();
        svc.create_device(valid_device()).await.unwrap();
        svc.create_device(
            Device::builder()
                .name("Sensor Hub")
                .integration("test")
                .unique_id("sensor_hub_1")
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

        let all = svc.list_devices().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn should_update_device() {
        let svc = make_service();
        let device = valid_device();
        let id = device.id;
        svc.create_device(device).await.unwrap();

        let mut updated = svc.get_device(id).await.unwrap();
        updated.name = "Updated Bridge".to_string();
        let saved = svc.update_device(updated).await.unwrap();
        assert_eq!(saved.name, "Updated Bridge");
    }

    #[tokio::test]
    async fn should_delete_device() {
        let svc = make_service();
        let device = valid_device();
        let id = device.id;
        svc.create_device(device).await.unwrap();

        svc.delete_device(id).await.unwrap();

        let result = svc.get_device(id).await;
        assert!(matches!(result, Err(MiniHubError::NotFound(_))));
    }

    #[tokio::test]
    async fn should_upsert_create_when_device_does_not_exist() {
        let svc = make_service();
        let device = Device::builder()
            .name("BLE Sensor")
            .integration("ble")
            .unique_id("A4:C1:38:5B:0E:DF")
            .build()
            .unwrap();
        let id = device.id;

        let result = svc.upsert_device(device).await.unwrap();
        assert_eq!(result.id, id);

        let fetched = svc.get_device(id).await.unwrap();
        assert_eq!(fetched.name, "BLE Sensor");
        assert_eq!(fetched.integration, "ble");
        assert_eq!(fetched.unique_id, "A4:C1:38:5B:0E:DF");
    }

    #[tokio::test]
    async fn should_upsert_update_when_device_already_exists() {
        let svc = make_service();
        let device = Device::builder()
            .name("BLE Sensor")
            .integration("ble")
            .unique_id("A4:C1:38:5B:0E:DF")
            .build()
            .unwrap();
        let original_id = device.id;
        svc.create_device(device).await.unwrap();

        // Build a new device with the same integration+unique_id but different name
        let updated = Device::builder()
            .name("BLE Sensor v2")
            .manufacturer("Xiaomi")
            .integration("ble")
            .unique_id("A4:C1:38:5B:0E:DF")
            .build()
            .unwrap();

        let result = svc.upsert_device(updated).await.unwrap();
        // Should preserve the original UUID
        assert_eq!(result.id, original_id);
        assert_eq!(result.name, "BLE Sensor v2");
        assert_eq!(result.manufacturer.as_deref(), Some("Xiaomi"));
    }

    #[tokio::test]
    async fn should_upsert_not_match_different_integration() {
        let svc = make_service();
        let device = Device::builder()
            .name("Sensor")
            .integration("ble")
            .unique_id("ABC")
            .build()
            .unwrap();
        let first_id = device.id;
        svc.create_device(device).await.unwrap();

        let device2 = Device::builder()
            .name("Sensor")
            .integration("mqtt")
            .unique_id("ABC")
            .build()
            .unwrap();
        let second_id = device2.id;

        let result = svc.upsert_device(device2).await.unwrap();
        // Different integration → should create new, not update
        assert_eq!(result.id, second_id);
        assert_ne!(result.id, first_id);

        let all = svc.list_devices().await.unwrap();
        assert_eq!(all.len(), 2);
    }
}
