//! Shared application state for axum handlers.

use std::sync::Arc;

use minihub_app::ports::{AreaRepository, DeviceRepository, EntityRepository};
use minihub_app::services::area_service::AreaService;
use minihub_app::services::device_service::DeviceService;
use minihub_app::services::entity_service::EntityService;

/// Application state shared across all axum handlers.
///
/// Generic over the three repository types to avoid dynamic dispatch.
/// `Clone` is implemented manually so the repository types themselves do not
/// need to be `Clone` — only the `Arc` wrappers are cloned.
pub struct AppState<ER, DR, AR> {
    /// Entity CRUD service.
    pub entity_service: Arc<EntityService<ER>>,
    /// Device CRUD service.
    pub device_service: Arc<DeviceService<DR>>,
    /// Area CRUD service.
    pub area_service: Arc<AreaService<AR>>,
}

impl<ER, DR, AR> Clone for AppState<ER, DR, AR> {
    fn clone(&self) -> Self {
        Self {
            entity_service: Arc::clone(&self.entity_service),
            device_service: Arc::clone(&self.device_service),
            area_service: Arc::clone(&self.area_service),
        }
    }
}

impl<ER, DR, AR> AppState<ER, DR, AR>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
{
    /// Create a new application state from service instances.
    pub fn new(
        entity_service: EntityService<ER>,
        device_service: DeviceService<DR>,
        area_service: AreaService<AR>,
    ) -> Self {
        Self {
            entity_service: Arc::new(entity_service),
            device_service: Arc::new(device_service),
            area_service: Arc::new(area_service),
        }
    }
}
