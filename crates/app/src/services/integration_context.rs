//! Concrete [`IntegrationContext`] backed by application services.

use std::sync::Arc;

use tokio::sync::broadcast;

use minihub_domain::device::Device;
use minihub_domain::entity::Entity;
use minihub_domain::error::MiniHubError;
use minihub_domain::event::Event;

use crate::event_bus::InProcessEventBus;
use crate::ports::{DeviceRepository, EntityRepository, EventPublisher, IntegrationContext};
use crate::services::device_service::DeviceService;
use crate::services::entity_service::EntityService;

/// [`IntegrationContext`] implementation that delegates to `DeviceService`,
/// `EntityService`, and an `EventPublisher`.
///
/// Wraps `Arc`-ed services so it is cheaply cloneable and `Send + Sync`.
/// The generic parameters are confined to this struct — integrations see
/// only the [`IntegrationContext`] trait.
pub struct ServiceContext<DR, ER, EP> {
    device_service: Arc<DeviceService<DR>>,
    entity_service: Arc<EntityService<ER, EP>>,
    event_publisher: EP,
    event_bus: Arc<InProcessEventBus>,
}

impl<DR, ER, EP> ServiceContext<DR, ER, EP> {
    /// Create a new context backed by the given services, event publisher,
    /// and event bus (for subscriptions).
    pub fn new(
        device_service: Arc<DeviceService<DR>>,
        entity_service: Arc<EntityService<ER, EP>>,
        event_publisher: EP,
        event_bus: Arc<InProcessEventBus>,
    ) -> Self {
        Self {
            device_service,
            entity_service,
            event_publisher,
            event_bus,
        }
    }
}

impl<DR, ER, EP: Clone> Clone for ServiceContext<DR, ER, EP> {
    fn clone(&self) -> Self {
        Self {
            device_service: Arc::clone(&self.device_service),
            entity_service: Arc::clone(&self.entity_service),
            event_publisher: self.event_publisher.clone(),
            event_bus: Arc::clone(&self.event_bus),
        }
    }
}

impl<DR, ER, EP> IntegrationContext for ServiceContext<DR, ER, EP>
where
    DR: DeviceRepository + Send + Sync + 'static,
    ER: EntityRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
{
    async fn upsert_device(&self, device: Device) -> Result<Device, MiniHubError> {
        self.device_service.upsert_device(device).await
    }

    async fn upsert_entity(&self, entity: Entity) -> Result<Entity, MiniHubError> {
        self.entity_service.upsert_entity(entity).await
    }

    async fn publish(&self, event: Event) -> Result<(), MiniHubError> {
        self.event_publisher.publish(event).await
    }

    fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.event_bus.subscribe()
    }
}
