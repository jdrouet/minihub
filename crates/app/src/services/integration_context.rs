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

    async fn find_entity_by_id(
        &self,
        id: minihub_domain::id::EntityId,
    ) -> Result<Option<Entity>, MiniHubError> {
        match self.entity_service.get_entity(id).await {
            Ok(entity) => Ok(Some(entity)),
            Err(MiniHubError::NotFound(_)) => Ok(None),
            Err(err) => Err(err),
        }
    }

    async fn publish(&self, event: Event) -> Result<(), MiniHubError> {
        self.event_publisher.publish(event).await
    }

    fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.event_bus.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::future::Future;
    use std::sync::Mutex;

    use minihub_domain::entity::EntityState;
    use minihub_domain::event::EventType;
    use minihub_domain::id::{DeviceId, EntityId};

    struct StubDeviceRepo {
        store: Mutex<HashMap<DeviceId, Device>>,
    }

    impl Default for StubDeviceRepo {
        fn default() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }
    }

    impl DeviceRepository for StubDeviceRepo {
        fn create(
            &self,
            device: Device,
        ) -> impl Future<Output = Result<Device, MiniHubError>> + Send {
            self.store.lock().unwrap().insert(device.id, device.clone());
            async { Ok(device) }
        }

        fn get_by_id(
            &self,
            id: DeviceId,
        ) -> impl Future<Output = Result<Option<Device>, MiniHubError>> + Send {
            let result = self.store.lock().unwrap().get(&id).cloned();
            async { Ok(result) }
        }

        fn get_all(&self) -> impl Future<Output = Result<Vec<Device>, MiniHubError>> + Send {
            let result: Vec<Device> = self.store.lock().unwrap().values().cloned().collect();
            async { Ok(result) }
        }

        async fn find_by_integration_unique_id(
            &self,
            _integration: &str,
            _unique_id: &str,
        ) -> Result<Option<Device>, MiniHubError> {
            Ok(None)
        }

        fn update(
            &self,
            device: Device,
        ) -> impl Future<Output = Result<Device, MiniHubError>> + Send {
            self.store.lock().unwrap().insert(device.id, device.clone());
            async { Ok(device) }
        }

        fn delete(&self, id: DeviceId) -> impl Future<Output = Result<(), MiniHubError>> + Send {
            self.store.lock().unwrap().remove(&id);
            async { Ok(()) }
        }
    }

    struct StubEntityRepo {
        store: Mutex<HashMap<EntityId, Entity>>,
    }

    impl Default for StubEntityRepo {
        fn default() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }
    }

    impl EntityRepository for StubEntityRepo {
        fn create(
            &self,
            entity: Entity,
        ) -> impl Future<Output = Result<Entity, MiniHubError>> + Send {
            self.store.lock().unwrap().insert(entity.id, entity.clone());
            async { Ok(entity) }
        }

        fn get_by_id(
            &self,
            id: EntityId,
        ) -> impl Future<Output = Result<Option<Entity>, MiniHubError>> + Send {
            let result = self.store.lock().unwrap().get(&id).cloned();
            async { Ok(result) }
        }

        fn get_all(&self) -> impl Future<Output = Result<Vec<Entity>, MiniHubError>> + Send {
            let result: Vec<Entity> = self.store.lock().unwrap().values().cloned().collect();
            async { Ok(result) }
        }

        fn find_by_device_id(
            &self,
            device_id: DeviceId,
        ) -> impl Future<Output = Result<Vec<Entity>, MiniHubError>> + Send {
            let result: Vec<Entity> = self
                .store
                .lock()
                .unwrap()
                .values()
                .filter(|ent| ent.device_id == device_id)
                .cloned()
                .collect();
            async { Ok(result) }
        }

        fn find_by_entity_id(
            &self,
            entity_id: &str,
        ) -> impl Future<Output = Result<Option<Entity>, MiniHubError>> + Send {
            let result = self
                .store
                .lock()
                .unwrap()
                .values()
                .find(|ent| ent.entity_id == entity_id)
                .cloned();
            async { Ok(result) }
        }

        fn update(
            &self,
            entity: Entity,
        ) -> impl Future<Output = Result<Entity, MiniHubError>> + Send {
            self.store.lock().unwrap().insert(entity.id, entity.clone());
            async { Ok(entity) }
        }

        fn delete(&self, id: EntityId) -> impl Future<Output = Result<(), MiniHubError>> + Send {
            self.store.lock().unwrap().remove(&id);
            async { Ok(()) }
        }
    }

    fn make_context() -> ServiceContext<StubDeviceRepo, StubEntityRepo, Arc<InProcessEventBus>> {
        let event_bus = Arc::new(InProcessEventBus::new(16));
        ServiceContext::new(
            Arc::new(DeviceService::new(StubDeviceRepo::default())),
            Arc::new(EntityService::new(
                StubEntityRepo::default(),
                Arc::clone(&event_bus),
            )),
            Arc::clone(&event_bus),
            event_bus,
        )
    }

    #[tokio::test]
    async fn should_receive_event_when_subscribed() {
        let ctx = make_context();
        let mut rx = ctx.subscribe();

        let event = Event::new(
            EventType::StateChanged,
            Some(EntityId::new()),
            serde_json::json!({"from": "off", "to": "on"}),
        );
        let event_id = event.id;

        ctx.publish(event).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.id, event_id);
    }

    #[tokio::test]
    async fn should_receive_event_when_subscribed_from_clone() {
        let ctx = make_context();
        let ctx2 = ctx.clone();
        let mut rx = ctx2.subscribe();

        let event = Event::new(EventType::EntityCreated, None, serde_json::json!({}));
        let event_id = event.id;

        ctx.publish(event).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.id, event_id);
    }

    #[tokio::test]
    async fn should_upsert_device_when_called() {
        let ctx = make_context();
        let device = Device {
            id: DeviceId::new(),
            name: "test".into(),
            manufacturer: None,
            model: None,
            integration: "virtual".into(),
            unique_id: "dev-1".into(),
            area_id: None,
        };
        let result = ctx.upsert_device(device.clone()).await.unwrap();
        assert_eq!(result.id, device.id);
    }

    #[tokio::test]
    async fn should_upsert_entity_when_called() {
        let ctx = make_context();
        let entity = Entity {
            id: EntityId::new(),
            entity_id: "light.test".into(),
            device_id: DeviceId::new(),
            friendly_name: "Test light".into(),
            state: EntityState::default(),
            attributes: HashMap::new(),
            mac_address: None,
            last_changed: minihub_domain::time::now(),
            last_updated: minihub_domain::time::now(),
        };
        let result = ctx.upsert_entity(entity.clone()).await.unwrap();
        assert_eq!(result.id, entity.id);
    }
}
