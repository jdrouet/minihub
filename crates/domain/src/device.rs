//! Device — a physical or virtual thing that exposes one or more entities.

use serde::{Deserialize, Serialize};

use crate::error::{MiniHubError, ValidationError};
use crate::id::{AreaId, DeviceId};

/// A physical or virtual thing that exposes one or more entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: DeviceId,
    pub name: String,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub area_id: Option<AreaId>,
}

impl Device {
    /// Create a builder for constructing a [`Device`].
    #[must_use]
    pub fn builder() -> DeviceBuilder {
        DeviceBuilder::default()
    }

    /// Check domain invariants.
    ///
    /// # Errors
    ///
    /// Returns [`MiniHubError::Validation`] when `name` is empty.
    pub fn validate(&self) -> Result<(), MiniHubError> {
        if self.name.is_empty() {
            return Err(ValidationError::EmptyName.into());
        }
        Ok(())
    }
}

/// Step-by-step builder for [`Device`].
#[derive(Debug, Default)]
pub struct DeviceBuilder {
    id: Option<DeviceId>,
    name: Option<String>,
    manufacturer: Option<String>,
    model: Option<String>,
    area_id: Option<AreaId>,
}

impl DeviceBuilder {
    #[must_use]
    pub fn id(mut self, id: DeviceId) -> Self {
        self.id = Some(id);
        self
    }

    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    #[must_use]
    pub fn manufacturer(mut self, manufacturer: impl Into<String>) -> Self {
        self.manufacturer = Some(manufacturer.into());
        self
    }

    #[must_use]
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    #[must_use]
    pub fn area_id(mut self, area_id: AreaId) -> Self {
        self.area_id = Some(area_id);
        self
    }

    /// Consume the builder, validate, and return a [`Device`].
    ///
    /// # Errors
    ///
    /// Returns [`MiniHubError::Validation`] if `name` is missing or empty.
    pub fn build(self) -> Result<Device, MiniHubError> {
        let device = Device {
            id: self.id.unwrap_or_default(),
            name: self.name.unwrap_or_default(),
            manufacturer: self.manufacturer,
            model: self.model,
            area_id: self.area_id,
        };
        device.validate()?;
        Ok(device)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_build_valid_device_when_name_provided() {
        let device = Device::builder().name("Hue Bridge").build().unwrap();
        assert_eq!(device.name, "Hue Bridge");
        assert!(device.manufacturer.is_none());
        assert!(device.model.is_none());
        assert!(device.area_id.is_none());
    }

    #[test]
    fn should_return_validation_error_when_name_is_empty() {
        let result = Device::builder().build();
        assert!(matches!(
            result,
            Err(MiniHubError::Validation(ValidationError::EmptyName))
        ));
    }

    #[test]
    fn should_build_device_with_all_optional_fields() {
        let area = AreaId::new();
        let device = Device::builder()
            .name("Motion Sensor")
            .manufacturer("Aqara")
            .model("RTCGQ11LM")
            .area_id(area)
            .build()
            .unwrap();

        assert_eq!(device.manufacturer.as_deref(), Some("Aqara"));
        assert_eq!(device.model.as_deref(), Some("RTCGQ11LM"));
        assert_eq!(device.area_id, Some(area));
    }

    #[test]
    fn should_roundtrip_through_serde_json() {
        let device = Device::builder().name("Lamp").build().unwrap();
        let json = serde_json::to_string(&device).unwrap();
        let parsed: Device = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, device.id);
        assert_eq!(parsed.name, device.name);
    }
}
