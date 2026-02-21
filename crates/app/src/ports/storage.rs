//! Storage port — repository traits for persistence.

use std::future::Future;

use minihub_domain::area::Area;
use minihub_domain::device::Device;
use minihub_domain::entity::Entity;
use minihub_domain::entity_history::EntityHistory;
use minihub_domain::error::MiniHubError;
use minihub_domain::id::{AreaId, DeviceId, EntityId};
use minihub_domain::time::Timestamp;

/// Repository for [`Entity`] persistence.
pub trait EntityRepository {
    /// Create a new entity in storage.
    fn create(&self, entity: Entity) -> impl Future<Output = Result<Entity, MiniHubError>> + Send;

    /// Get an entity by its unique identifier.
    fn get_by_id(
        &self,
        id: EntityId,
    ) -> impl Future<Output = Result<Option<Entity>, MiniHubError>> + Send;

    /// Get all entities.
    fn get_all(&self) -> impl Future<Output = Result<Vec<Entity>, MiniHubError>> + Send;

    /// Find entities belonging to a specific device.
    fn find_by_device_id(
        &self,
        device_id: DeviceId,
    ) -> impl Future<Output = Result<Vec<Entity>, MiniHubError>> + Send;

    /// Find an entity by its string identifier (e.g. `"sensor.ble_a4c1385b0edf"`).
    fn find_by_entity_id(
        &self,
        entity_id: &str,
    ) -> impl Future<Output = Result<Option<Entity>, MiniHubError>> + Send;

    /// Update an existing entity.
    fn update(&self, entity: Entity) -> impl Future<Output = Result<Entity, MiniHubError>> + Send;

    /// Delete an entity by its unique identifier.
    fn delete(&self, id: EntityId) -> impl Future<Output = Result<(), MiniHubError>> + Send;
}

/// Repository for [`Device`] persistence.
pub trait DeviceRepository {
    /// Create a new device in storage.
    fn create(&self, device: Device) -> impl Future<Output = Result<Device, MiniHubError>> + Send;

    /// Get a device by its unique identifier.
    fn get_by_id(
        &self,
        id: DeviceId,
    ) -> impl Future<Output = Result<Option<Device>, MiniHubError>> + Send;

    /// Get all devices.
    fn get_all(&self) -> impl Future<Output = Result<Vec<Device>, MiniHubError>> + Send;

    /// Find a device by its integration source and unique id pair.
    fn find_by_integration_unique_id(
        &self,
        integration: &str,
        unique_id: &str,
    ) -> impl Future<Output = Result<Option<Device>, MiniHubError>> + Send;

    /// Update an existing device.
    fn update(&self, device: Device) -> impl Future<Output = Result<Device, MiniHubError>> + Send;

    /// Delete a device by its unique identifier.
    fn delete(&self, id: DeviceId) -> impl Future<Output = Result<(), MiniHubError>> + Send;
}

/// Repository for [`Area`] persistence.
pub trait AreaRepository {
    /// Create a new area in storage.
    fn create(&self, area: Area) -> impl Future<Output = Result<Area, MiniHubError>> + Send;

    /// Get an area by its unique identifier.
    fn get_by_id(
        &self,
        id: AreaId,
    ) -> impl Future<Output = Result<Option<Area>, MiniHubError>> + Send;

    /// Get all areas.
    fn get_all(&self) -> impl Future<Output = Result<Vec<Area>, MiniHubError>> + Send;

    /// Update an existing area.
    fn update(&self, area: Area) -> impl Future<Output = Result<Area, MiniHubError>> + Send;

    /// Delete an area by its unique identifier.
    fn delete(&self, id: AreaId) -> impl Future<Output = Result<(), MiniHubError>> + Send;
}

/// Repository for [`EntityHistory`] persistence.
pub trait EntityHistoryRepository {
    /// Record a new entity history snapshot.
    fn record(
        &self,
        history: EntityHistory,
    ) -> impl Future<Output = Result<EntityHistory, MiniHubError>> + Send;

    /// Find history records for a specific entity within a time range.
    ///
    /// Results are ordered by `recorded_at` ascending (oldest first).
    fn find_by_entity_in_range(
        &self,
        entity_id: EntityId,
        from: Timestamp,
        to: Timestamp,
        limit: Option<usize>,
    ) -> impl Future<Output = Result<Vec<EntityHistory>, MiniHubError>> + Send;

    /// Purge all history records older than the given timestamp.
    fn purge_before(
        &self,
        before: Timestamp,
    ) -> impl Future<Output = Result<usize, MiniHubError>> + Send;
}
