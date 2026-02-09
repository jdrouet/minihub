//! Virtual temperature sensor — holds a numeric reading as an attribute.

use minihub_domain::device::Device;
use minihub_domain::entity::AttributeValue;
use minihub_domain::entity::{Entity, EntityState};
use minihub_domain::id::{DeviceId, EntityId};

/// A simulated temperature sensor.
///
/// Sensors do not respond to service calls — they only expose read-only
/// attributes (`temperature`, `unit`).
pub struct VirtualSensor {
    device_id: DeviceId,
    entity_id: EntityId,
}

impl Default for VirtualSensor {
    fn default() -> Self {
        Self {
            device_id: DeviceId::new(),
            entity_id: EntityId::new(),
        }
    }
}

impl VirtualSensor {
    /// The fixed entity id for this sensor.
    #[must_use]
    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    /// Produce the [`Device`] and [`Entity`] descriptors.
    #[must_use]
    pub fn discover(&self) -> (Device, Entity) {
        let device = Device::builder()
            .id(self.device_id)
            .name("Virtual Sensor")
            .manufacturer("minihub")
            .model("VSensor-1")
            .build()
            .unwrap();

        let entity = Entity::builder()
            .id(self.entity_id)
            .device_id(self.device_id)
            .entity_id("sensor.virtual_temperature")
            .friendly_name("Virtual Temperature")
            .state(EntityState::Unknown)
            .attribute("temperature", AttributeValue::Float(21.5))
            .attribute("unit", AttributeValue::String("\u{b0}C".to_string()))
            .build()
            .unwrap();

        (device, entity)
    }

    /// Sensors are read-only — service calls return the current entity unchanged.
    pub fn handle_service(&self, _service: &str) -> Entity {
        self.discover().1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_default_to_unknown_state() {
        let sensor = VirtualSensor::default();
        let (_, entity) = sensor.discover();
        assert_eq!(entity.state, EntityState::Unknown);
    }

    #[test]
    fn should_have_temperature_attribute() {
        let sensor = VirtualSensor::default();
        let (_, entity) = sensor.discover();
        assert_eq!(
            entity.get_attribute("temperature"),
            Some(&AttributeValue::Float(21.5))
        );
    }

    #[test]
    fn should_have_unit_attribute() {
        let sensor = VirtualSensor::default();
        let (_, entity) = sensor.discover();
        assert_eq!(
            entity.get_attribute("unit"),
            Some(&AttributeValue::String("\u{b0}C".to_string()))
        );
    }

    #[test]
    fn should_produce_correct_device_metadata() {
        let sensor = VirtualSensor::default();
        let (device, _) = sensor.discover();
        assert_eq!(device.name, "Virtual Sensor");
        assert_eq!(device.manufacturer.as_deref(), Some("minihub"));
        assert_eq!(device.model.as_deref(), Some("VSensor-1"));
    }

    #[test]
    fn should_return_entity_unchanged_on_service_call() {
        let sensor = VirtualSensor::default();
        let entity = sensor.handle_service("turn_on");
        assert_eq!(entity.state, EntityState::Unknown);
    }
}
