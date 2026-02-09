//! Shared application state for axum handlers.

use std::sync::Arc;

use minihub_app::ports::{
    AreaRepository, AutomationRepository, DeviceRepository, EntityRepository, EventPublisher,
    EventStore,
};
use minihub_app::services::area_service::AreaService;
use minihub_app::services::automation_service::AutomationService;
use minihub_app::services::device_service::DeviceService;
use minihub_app::services::entity_service::EntityService;

/// Application state shared across all axum handlers.
///
/// Generic over the repository types, event publisher, event store, and
/// automation repository to avoid dynamic dispatch.
/// `Clone` is implemented manually so the underlying types themselves do not
/// need to be `Clone` — only the `Arc` wrappers are cloned.
pub struct AppState<ER, DR, AR, EP, ES, AUR> {
    /// Entity CRUD service.
    pub entity_service: Arc<EntityService<ER, EP>>,
    /// Device CRUD service.
    pub device_service: Arc<DeviceService<DR>>,
    /// Area CRUD service.
    pub area_service: Arc<AreaService<AR>>,
    /// Event store for querying persisted events.
    pub event_store: Arc<ES>,
    /// Automation CRUD service.
    pub automation_service: Arc<AutomationService<AUR>>,
}

impl<ER, DR, AR, EP, ES, AUR> Clone for AppState<ER, DR, AR, EP, ES, AUR> {
    fn clone(&self) -> Self {
        Self {
            entity_service: Arc::clone(&self.entity_service),
            device_service: Arc::clone(&self.device_service),
            area_service: Arc::clone(&self.area_service),
            event_store: Arc::clone(&self.event_store),
            automation_service: Arc::clone(&self.automation_service),
        }
    }
}

impl<ER, DR, AR, EP, ES, AUR> AppState<ER, DR, AR, EP, ES, AUR>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
{
    /// Create a new application state from service instances.
    pub fn new(
        entity_service: EntityService<ER, EP>,
        device_service: DeviceService<DR>,
        area_service: AreaService<AR>,
        event_store: ES,
        automation_service: AutomationService<AUR>,
    ) -> Self {
        Self {
            entity_service: Arc::new(entity_service),
            device_service: Arc::new(device_service),
            area_service: Arc::new(area_service),
            event_store: Arc::new(event_store),
            automation_service: Arc::new(automation_service),
        }
    }
}
