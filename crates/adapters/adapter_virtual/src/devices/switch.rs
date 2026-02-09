//! Virtual switch — responds to `turn_on`, `turn_off`, `toggle`.

use std::sync::Mutex;

use minihub_domain::device::Device;
use minihub_domain::entity::{Entity, EntityState};
use minihub_domain::id::{DeviceId, EntityId};
use minihub_domain::time::now;

/// A simulated switch that can be turned on and off.
pub struct VirtualSwitch {
    device_id: DeviceId,
    entity_id: EntityId,
    state: Mutex<EntityState>,
}

impl Default for VirtualSwitch {
    fn default() -> Self {
        Self {
            device_id: DeviceId::new(),
            entity_id: EntityId::new(),
            state: Mutex::new(EntityState::Off),
        }
    }
}

impl VirtualSwitch {
    /// The fixed entity id for this switch.
    #[must_use]
    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    /// Produce the [`Device`] and [`Entity`] descriptors.
    #[must_use]
    pub fn discover(&self) -> (Device, Entity) {
        let state = self.state.lock().unwrap().clone();
        let device = Device::builder()
            .id(self.device_id)
            .name("Virtual Switch")
            .manufacturer("minihub")
            .model("VSwitch-1")
            .build()
            .unwrap();

        let entity = Entity::builder()
            .id(self.entity_id)
            .device_id(self.device_id)
            .entity_id("switch.virtual_switch")
            .friendly_name("Virtual Switch")
            .state(state)
            .build()
            .unwrap();

        (device, entity)
    }

    /// Handle a service call, returning the updated entity snapshot.
    pub fn handle_service(&self, service: &str) -> Entity {
        let new_state = {
            let mut state = self.state.lock().unwrap();
            match service {
                "turn_on" => *state = EntityState::On,
                "turn_off" => *state = EntityState::Off,
                "toggle" => {
                    *state = match *state {
                        EntityState::On => EntityState::Off,
                        _ => EntityState::On,
                    };
                }
                _ => {}
            }
            state.clone()
        };

        let mut entity = self.discover().1;
        entity.update_state(new_state, now());
        entity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_default_to_off() {
        let switch = VirtualSwitch::default();
        let (_, entity) = switch.discover();
        assert_eq!(entity.state, EntityState::Off);
    }

    #[test]
    fn should_turn_on_when_service_called() {
        let switch = VirtualSwitch::default();
        let entity = switch.handle_service("turn_on");
        assert_eq!(entity.state, EntityState::On);
    }

    #[test]
    fn should_turn_off_when_service_called() {
        let switch = VirtualSwitch::default();
        switch.handle_service("turn_on");
        let entity = switch.handle_service("turn_off");
        assert_eq!(entity.state, EntityState::Off);
    }

    #[test]
    fn should_toggle_from_off_to_on() {
        let switch = VirtualSwitch::default();
        let entity = switch.handle_service("toggle");
        assert_eq!(entity.state, EntityState::On);
    }

    #[test]
    fn should_toggle_from_on_to_off() {
        let switch = VirtualSwitch::default();
        switch.handle_service("turn_on");
        let entity = switch.handle_service("toggle");
        assert_eq!(entity.state, EntityState::Off);
    }

    #[test]
    fn should_ignore_unknown_service() {
        let switch = VirtualSwitch::default();
        let entity = switch.handle_service("reboot");
        assert_eq!(entity.state, EntityState::Off);
    }

    #[test]
    fn should_produce_correct_device_metadata() {
        let switch = VirtualSwitch::default();
        let (device, _) = switch.discover();
        assert_eq!(device.name, "Virtual Switch");
        assert_eq!(device.manufacturer.as_deref(), Some("minihub"));
        assert_eq!(device.model.as_deref(), Some("VSwitch-1"));
    }
}
