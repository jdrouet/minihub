//! # minihub-adapter-virtual
//!
//! Virtual/demo integration that provides simulated devices for testing and
//! demonstration purposes.
//!
//! ## Provided devices
//!
//! | Device | Entity ID | Behaviour |
//! |--------|-----------|-----------|
//! | Virtual Light | `light.virtual_light` | Responds to `turn_on` / `turn_off` / `toggle` |
//! | Virtual Sensor | `sensor.virtual_temperature` | Holds a numeric temperature attribute |
//! | Virtual Switch | `switch.virtual_switch` | Responds to `turn_on` / `turn_off` / `toggle` |
//!
//! ## Dependency rule
//!
//! Depends on `minihub-app` (port traits) and `minihub-domain` only.

mod devices;

use std::collections::HashMap;

use minihub_app::ports::integration::{DiscoveredDevice, Integration, IntegrationContext};
use minihub_domain::entity::Entity;
use minihub_domain::error::{MiniHubError, NotFoundError};
use minihub_domain::id::EntityId;

use devices::{VirtualDevice, VirtualLight, VirtualSensor, VirtualSwitch};

/// Virtual integration that creates simulated devices.
pub struct VirtualIntegration {
    devices: HashMap<EntityId, VirtualDevice>,
}

impl Default for VirtualIntegration {
    fn default() -> Self {
        let light = VirtualLight::default();
        let sensor = VirtualSensor::default();
        let switch = VirtualSwitch::default();

        let mut devices = HashMap::new();
        devices.insert(light.entity_id(), VirtualDevice::Light(light));
        devices.insert(sensor.entity_id(), VirtualDevice::Sensor(sensor));
        devices.insert(switch.entity_id(), VirtualDevice::Switch(switch));

        Self { devices }
    }
}

impl Integration for VirtualIntegration {
    fn name(&self) -> &'static str {
        "virtual"
    }

    async fn setup(&mut self, ctx: &impl IntegrationContext) -> Result<(), MiniHubError> {
        for vdev in self.devices.values() {
            let (device, entity) = vdev.discover()?;
            let dd = DiscoveredDevice {
                device,
                entities: vec![entity],
            };
            ctx.persist_discovered(dd).await?;
        }
        Ok(())
    }

    async fn handle_service_call(
        &self,
        entity_id: EntityId,
        service: &str,
        _data: serde_json::Value,
    ) -> Result<Entity, MiniHubError> {
        let vdev = self.devices.get(&entity_id).ok_or_else(|| NotFoundError {
            entity: "Entity",
            id: entity_id.to_string(),
        })?;

        vdev.handle_service(service)
    }

    async fn teardown(&mut self) -> Result<(), MiniHubError> {
        Ok(())
    }
}

impl VirtualIntegration {
    /// Check whether this integration owns the given entity.
    #[must_use]
    pub fn owns_entity(&self, entity_id: EntityId) -> bool {
        self.devices.contains_key(&entity_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use minihub_domain::device::Device;
    use minihub_domain::entity::EntityState;
    use std::future::Future;
    use std::sync::Mutex;

    struct InMemoryContext {
        devices: Mutex<Vec<Device>>,
        entities: Mutex<Vec<Entity>>,
    }

    impl InMemoryContext {
        fn new() -> Self {
            Self {
                devices: Mutex::new(Vec::new()),
                entities: Mutex::new(Vec::new()),
            }
        }
    }

    impl IntegrationContext for InMemoryContext {
        fn upsert_device(
            &self,
            device: Device,
        ) -> impl Future<Output = Result<Device, MiniHubError>> + Send {
            self.devices.lock().unwrap().push(device.clone());
            async { Ok(device) }
        }

        fn upsert_entity(
            &self,
            entity: Entity,
        ) -> impl Future<Output = Result<Entity, MiniHubError>> + Send {
            self.entities.lock().unwrap().push(entity.clone());
            async { Ok(entity) }
        }

        async fn publish(&self, _event: minihub_domain::event::Event) -> Result<(), MiniHubError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn should_discover_three_devices_on_setup() {
        let mut integration = VirtualIntegration::default();
        let ctx = InMemoryContext::new();
        integration.setup(&ctx).await.unwrap();
        assert_eq!(ctx.devices.lock().unwrap().len(), 3);
        assert_eq!(ctx.entities.lock().unwrap().len(), 3);
    }

    #[tokio::test]
    async fn should_return_virtual_as_name() {
        let integration = VirtualIntegration::default();
        assert_eq!(integration.name(), "virtual");
    }

    #[tokio::test]
    async fn should_discover_light_device() {
        let mut integration = VirtualIntegration::default();
        let ctx = InMemoryContext::new();
        integration.setup(&ctx).await.unwrap();

        let devices = ctx.devices.lock().unwrap();
        let light = devices.iter().find(|d| d.name == "Virtual Light");
        assert!(light.is_some());

        let entities = ctx.entities.lock().unwrap();
        let light_entity = entities
            .iter()
            .find(|e| e.entity_id == "light.virtual_light");
        assert!(light_entity.is_some());
        assert_eq!(light_entity.unwrap().state, EntityState::Off);
    }

    #[tokio::test]
    async fn should_discover_sensor_device() {
        let mut integration = VirtualIntegration::default();
        let ctx = InMemoryContext::new();
        integration.setup(&ctx).await.unwrap();

        let entities = ctx.entities.lock().unwrap();
        let sensor = entities
            .iter()
            .find(|e| e.entity_id == "sensor.virtual_temperature");
        assert!(sensor.is_some());

        let sensor = sensor.unwrap();
        assert_eq!(
            sensor.get_attribute("temperature"),
            Some(&minihub_domain::entity::AttributeValue::Float(21.5))
        );
        assert_eq!(
            sensor.get_attribute("unit"),
            Some(&minihub_domain::entity::AttributeValue::String(
                "\u{b0}C".to_string()
            ))
        );
    }

    #[tokio::test]
    async fn should_discover_switch_device() {
        let mut integration = VirtualIntegration::default();
        let ctx = InMemoryContext::new();
        integration.setup(&ctx).await.unwrap();

        let entities = ctx.entities.lock().unwrap();
        let switch = entities
            .iter()
            .find(|e| e.entity_id == "switch.virtual_switch");
        assert!(switch.is_some());
        assert_eq!(switch.unwrap().state, EntityState::Off);
    }

    #[tokio::test]
    async fn should_handle_turn_on_for_light() {
        let integration = VirtualIntegration::default();
        let light_id = find_entity_id(&integration, "light.virtual_light");

        let entity = integration
            .handle_service_call(light_id, "turn_on", serde_json::json!({}))
            .await
            .unwrap();
        assert_eq!(entity.state, EntityState::On);
    }

    #[tokio::test]
    async fn should_handle_turn_off_for_light() {
        let integration = VirtualIntegration::default();
        let light_id = find_entity_id(&integration, "light.virtual_light");

        let entity = integration
            .handle_service_call(light_id, "turn_off", serde_json::json!({}))
            .await
            .unwrap();
        assert_eq!(entity.state, EntityState::Off);
    }

    #[tokio::test]
    async fn should_handle_toggle_for_switch() {
        let integration = VirtualIntegration::default();
        let switch_id = find_entity_id(&integration, "switch.virtual_switch");

        let entity = integration
            .handle_service_call(switch_id, "toggle", serde_json::json!({}))
            .await
            .unwrap();
        assert_eq!(entity.state, EntityState::On);
    }

    #[tokio::test]
    async fn should_return_not_found_for_unknown_entity() {
        let integration = VirtualIntegration::default();
        let result = integration
            .handle_service_call(EntityId::new(), "turn_on", serde_json::json!({}))
            .await;
        assert!(matches!(result, Err(MiniHubError::NotFound(_))));
    }

    #[tokio::test]
    async fn should_own_discovered_entities() {
        let integration = VirtualIntegration::default();
        for vdev in integration.devices.values() {
            let (_, entity) = vdev.discover().unwrap();
            assert!(integration.owns_entity(entity.id));
        }
    }

    #[tokio::test]
    async fn should_not_own_unknown_entity() {
        let integration = VirtualIntegration::default();
        assert!(!integration.owns_entity(EntityId::new()));
    }

    #[tokio::test]
    async fn should_teardown_successfully() {
        let mut integration = VirtualIntegration::default();
        assert!(integration.teardown().await.is_ok());
    }

    fn find_entity_id(integration: &VirtualIntegration, entity_id_str: &str) -> EntityId {
        integration
            .devices
            .values()
            .filter_map(|vd| vd.discover().ok())
            .map(|(_, e)| e)
            .find(|e| e.entity_id == entity_id_str)
            .unwrap()
            .id
    }
}
