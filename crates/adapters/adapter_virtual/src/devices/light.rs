//! Virtual light — responds to `turn_on`, `turn_off`, `toggle`.

use std::sync::Mutex;

use minihub_domain::device::Device;
use minihub_domain::entity::{Entity, EntityState};
use minihub_domain::error::MiniHubError;
use minihub_domain::id::{DeviceId, EntityId};
use minihub_domain::time::now;

/// A simulated light that can be turned on and off.
pub struct VirtualLight {
    device_id: DeviceId,
    entity_id: EntityId,
    state: Mutex<EntityState>,
}

impl Default for VirtualLight {
    fn default() -> Self {
        Self {
            device_id: DeviceId::new(),
            entity_id: EntityId::new(),
            state: Mutex::new(EntityState::Off),
        }
    }
}

impl VirtualLight {
    /// The fixed entity id for this light.
    #[must_use]
    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    /// Produce the [`Device`] and [`Entity`] descriptors.
    ///
    /// # Errors
    ///
    /// Returns a validation error if the builder fails (should not happen
    /// with hardcoded inputs).
    pub fn discover(&self) -> Result<(Device, Entity), MiniHubError> {
        let state = self.lock_state();
        let device = Device::builder()
            .id(self.device_id)
            .name("Virtual Light")
            .manufacturer("minihub")
            .model("VLight-1")
            .build()?;

        let entity = Entity::builder()
            .id(self.entity_id)
            .device_id(self.device_id)
            .entity_id("light.virtual_light")
            .friendly_name("Virtual Light")
            .state(state)
            .build()?;

        Ok((device, entity))
    }

    /// Handle a service call, returning the updated entity snapshot.
    ///
    /// # Errors
    ///
    /// Returns a validation error if the builder fails.
    pub fn handle_service(&self, service: &str) -> Result<Entity, MiniHubError> {
        let new_state = {
            let mut state = self.lock_state_mut();
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

        let mut entity = self.discover()?.1;
        entity.update_state(new_state, now());
        Ok(entity)
    }

    fn lock_state(&self) -> EntityState {
        self.state
            .lock()
            .map_or_else(|poisoned| poisoned.into_inner().clone(), |g| g.clone())
    }

    fn lock_state_mut(&self) -> std::sync::MutexGuard<'_, EntityState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_default_to_off() {
        let light = VirtualLight::default();
        let (_, entity) = light.discover().unwrap();
        assert_eq!(entity.state, EntityState::Off);
    }

    #[test]
    fn should_turn_on_when_service_called() {
        let light = VirtualLight::default();
        let entity = light.handle_service("turn_on").unwrap();
        assert_eq!(entity.state, EntityState::On);
    }

    #[test]
    fn should_turn_off_when_service_called() {
        let light = VirtualLight::default();
        light.handle_service("turn_on").unwrap();
        let entity = light.handle_service("turn_off").unwrap();
        assert_eq!(entity.state, EntityState::Off);
    }

    #[test]
    fn should_toggle_from_off_to_on() {
        let light = VirtualLight::default();
        let entity = light.handle_service("toggle").unwrap();
        assert_eq!(entity.state, EntityState::On);
    }

    #[test]
    fn should_toggle_from_on_to_off() {
        let light = VirtualLight::default();
        light.handle_service("turn_on").unwrap();
        let entity = light.handle_service("toggle").unwrap();
        assert_eq!(entity.state, EntityState::Off);
    }

    #[test]
    fn should_ignore_unknown_service() {
        let light = VirtualLight::default();
        let entity = light.handle_service("set_color").unwrap();
        assert_eq!(entity.state, EntityState::Off);
    }

    #[test]
    fn should_produce_correct_device_metadata() {
        let light = VirtualLight::default();
        let (device, _) = light.discover().unwrap();
        assert_eq!(device.name, "Virtual Light");
        assert_eq!(device.manufacturer.as_deref(), Some("minihub"));
        assert_eq!(device.model.as_deref(), Some("VLight-1"));
    }
}
