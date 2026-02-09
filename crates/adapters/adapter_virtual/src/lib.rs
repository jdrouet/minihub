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

use minihub_app::ports::integration::{DiscoveredDevice, Integration};
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

    async fn setup(&mut self) -> Result<Vec<DiscoveredDevice>, MiniHubError> {
        let mut discovered = Vec::new();

        for vdev in self.devices.values() {
            let (device, entity) = vdev.discover();
            discovered.push(DiscoveredDevice {
                device,
                entities: vec![entity],
            });
        }

        Ok(discovered)
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

        Ok(vdev.handle_service(service))
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
    use minihub_domain::entity::{AttributeValue, EntityState};

    #[tokio::test]
    async fn should_discover_three_devices_on_setup() {
        let mut integration = VirtualIntegration::default();
        let discovered = integration.setup().await.unwrap();
        assert_eq!(discovered.len(), 3);
    }

    #[tokio::test]
    async fn should_return_virtual_as_name() {
        let integration = VirtualIntegration::default();
        assert_eq!(integration.name(), "virtual");
    }

    #[tokio::test]
    async fn should_discover_light_device() {
        let mut integration = VirtualIntegration::default();
        let discovered = integration.setup().await.unwrap();

        let light = discovered.iter().find(|d| d.device.name == "Virtual Light");
        assert!(light.is_some());

        let light = light.unwrap();
        assert_eq!(light.entities.len(), 1);
        assert_eq!(light.entities[0].entity_id, "light.virtual_light");
        assert_eq!(light.entities[0].state, EntityState::Off);
    }

    #[tokio::test]
    async fn should_discover_sensor_device() {
        let mut integration = VirtualIntegration::default();
        let discovered = integration.setup().await.unwrap();

        let sensor = discovered
            .iter()
            .find(|d| d.device.name == "Virtual Sensor");
        assert!(sensor.is_some());

        let sensor = sensor.unwrap();
        assert_eq!(sensor.entities.len(), 1);
        assert_eq!(sensor.entities[0].entity_id, "sensor.virtual_temperature");
        assert_eq!(
            sensor.entities[0].get_attribute("temperature"),
            Some(&AttributeValue::Float(21.5))
        );
        assert_eq!(
            sensor.entities[0].get_attribute("unit"),
            Some(&AttributeValue::String("\u{b0}C".to_string()))
        );
    }

    #[tokio::test]
    async fn should_discover_switch_device() {
        let mut integration = VirtualIntegration::default();
        let discovered = integration.setup().await.unwrap();

        let switch = discovered
            .iter()
            .find(|d| d.device.name == "Virtual Switch");
        assert!(switch.is_some());

        let switch = switch.unwrap();
        assert_eq!(switch.entities.len(), 1);
        assert_eq!(switch.entities[0].entity_id, "switch.virtual_switch");
        assert_eq!(switch.entities[0].state, EntityState::Off);
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
        let mut integration = VirtualIntegration::default();
        let discovered = integration.setup().await.unwrap();

        for dd in &discovered {
            for entity in &dd.entities {
                assert!(integration.owns_entity(entity.id));
            }
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
            .map(|vd| {
                let (_, e) = vd.discover();
                e
            })
            .find(|e| e.entity_id == entity_id_str)
            .unwrap()
            .id
    }
}
