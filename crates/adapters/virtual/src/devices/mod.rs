//! Virtual device implementations — light, sensor, switch.
//!
//! Each virtual device holds a fixed [`EntityId`] and [`DeviceId`] so they
//! remain stable across restarts of the integration.

mod light;
mod sensor;
mod switch;

pub use light::VirtualLight;
pub use sensor::VirtualSensor;
pub use switch::VirtualSwitch;

use minihub_domain::device::Device;
use minihub_domain::entity::Entity;
use minihub_domain::error::MiniHubError;

/// Wrapper enum for the concrete virtual device types.
pub enum VirtualDevice {
    Light(VirtualLight),
    Sensor(VirtualSensor),
    Switch(VirtualSwitch),
}

impl VirtualDevice {
    /// Create the [`Device`] and [`Entity`] descriptors for registration.
    ///
    /// # Errors
    ///
    /// Returns a validation error if the builder fails.
    pub fn discover(&self) -> Result<(Device, Entity), MiniHubError> {
        match self {
            Self::Light(d) => d.discover(),
            Self::Sensor(d) => d.discover(),
            Self::Switch(d) => d.discover(),
        }
    }

    /// Handle a service call, returning the resulting entity snapshot.
    ///
    /// # Errors
    ///
    /// Returns a validation error if the builder fails.
    pub fn handle_service(&self, service: &str) -> Result<Entity, MiniHubError> {
        match self {
            Self::Light(d) => d.handle_service(service),
            Self::Sensor(d) => d.handle_service(service),
            Self::Switch(d) => d.handle_service(service),
        }
    }
}
