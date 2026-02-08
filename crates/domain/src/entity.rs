//! Entity — the central state-holding concept in minihub.
//!
//! An entity represents a single observable/controllable aspect of a device
//! (e.g., a light's on/off state, a temperature sensor's reading).

mod attribute_value;
mod state;

pub use attribute_value::AttributeValue;
pub use state::EntityState;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::{MiniHubError, ValidationError};
use crate::id::{DeviceId, EntityId};
use crate::time::Timestamp;

/// An observable/controllable data point in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub device_id: DeviceId,
    /// Domain-level identifier, e.g. `"light.living_room"`.
    pub entity_id: String,
    pub friendly_name: String,
    pub state: EntityState,
    pub attributes: HashMap<String, AttributeValue>,
    pub last_changed: Timestamp,
    pub last_updated: Timestamp,
}

impl Entity {
    /// Create a builder for constructing an [`Entity`].
    #[must_use]
    pub fn builder() -> EntityBuilder {
        EntityBuilder::default()
    }

    /// Update the state, bumping `last_changed` only when the value differs.
    pub fn update_state(&mut self, new_state: EntityState, timestamp: Timestamp) {
        if self.state != new_state {
            self.state = new_state;
            self.last_changed = timestamp;
        }
        self.last_updated = timestamp;
    }

    /// Insert or overwrite an attribute.
    pub fn set_attribute(&mut self, key: String, value: AttributeValue) {
        self.attributes.insert(key, value);
    }

    /// Look up an attribute by key.
    #[must_use]
    pub fn get_attribute(&self, key: &str) -> Option<&AttributeValue> {
        self.attributes.get(key)
    }

    /// Check domain invariants.
    ///
    /// # Errors
    ///
    /// Returns [`MiniHubError::Validation`] when `entity_id` or
    /// `friendly_name` is empty.
    pub fn validate(&self) -> Result<(), MiniHubError> {
        if self.entity_id.is_empty() {
            return Err(ValidationError::EmptyEntityId.into());
        }
        if self.friendly_name.is_empty() {
            return Err(ValidationError::EmptyFriendlyName.into());
        }
        Ok(())
    }
}

/// Step-by-step builder for [`Entity`].
#[derive(Debug, Default)]
pub struct EntityBuilder {
    id: Option<EntityId>,
    device_id: Option<DeviceId>,
    entity_id: Option<String>,
    friendly_name: Option<String>,
    state: Option<EntityState>,
    attributes: HashMap<String, AttributeValue>,
}

impl EntityBuilder {
    #[must_use]
    pub fn id(mut self, id: EntityId) -> Self {
        self.id = Some(id);
        self
    }

    #[must_use]
    pub fn device_id(mut self, device_id: DeviceId) -> Self {
        self.device_id = Some(device_id);
        self
    }

    #[must_use]
    pub fn entity_id(mut self, entity_id: impl Into<String>) -> Self {
        self.entity_id = Some(entity_id.into());
        self
    }

    #[must_use]
    pub fn friendly_name(mut self, name: impl Into<String>) -> Self {
        self.friendly_name = Some(name.into());
        self
    }

    #[must_use]
    pub fn state(mut self, state: EntityState) -> Self {
        self.state = Some(state);
        self
    }

    #[must_use]
    pub fn attribute(mut self, key: impl Into<String>, value: AttributeValue) -> Self {
        self.attributes.insert(key.into(), value);
        self
    }

    /// Consume the builder, validate, and return an [`Entity`].
    ///
    /// # Errors
    ///
    /// Returns [`MiniHubError::Validation`] if required fields are missing or empty.
    pub fn build(self) -> Result<Entity, MiniHubError> {
        let now = crate::time::now();
        let entity = Entity {
            id: self.id.unwrap_or_default(),
            device_id: self.device_id.unwrap_or_default(),
            entity_id: self.entity_id.unwrap_or_default(),
            friendly_name: self.friendly_name.unwrap_or_default(),
            state: self.state.unwrap_or_default(),
            attributes: self.attributes,
            last_changed: now,
            last_updated: now,
        };
        entity.validate()?;
        Ok(entity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::now;

    fn valid_entity() -> Entity {
        Entity::builder()
            .entity_id("light.living_room")
            .friendly_name("Living Room Light")
            .state(EntityState::Off)
            .build()
            .unwrap()
    }

    #[test]
    fn should_build_valid_entity_when_required_fields_provided() {
        let entity = valid_entity();
        assert_eq!(entity.entity_id, "light.living_room");
        assert_eq!(entity.friendly_name, "Living Room Light");
        assert_eq!(entity.state, EntityState::Off);
        assert!(entity.attributes.is_empty());
    }

    #[test]
    fn should_return_validation_error_when_entity_id_is_empty() {
        let result = Entity::builder().friendly_name("A Name").build();
        assert!(matches!(
            result,
            Err(MiniHubError::Validation(ValidationError::EmptyEntityId))
        ));
    }

    #[test]
    fn should_return_validation_error_when_friendly_name_is_empty() {
        let result = Entity::builder().entity_id("sensor.temp").build();
        assert!(matches!(
            result,
            Err(MiniHubError::Validation(ValidationError::EmptyFriendlyName))
        ));
    }

    #[test]
    fn should_update_last_changed_when_state_differs() {
        let mut entity = valid_entity();
        let original_changed = entity.last_changed;
        let ts = now();

        entity.update_state(EntityState::On, ts);

        assert_eq!(entity.state, EntityState::On);
        assert_eq!(entity.last_changed, ts);
        assert_ne!(entity.last_changed, original_changed);
        assert_eq!(entity.last_updated, ts);
    }

    #[test]
    fn should_not_update_last_changed_when_state_is_same() {
        let mut entity = valid_entity();
        let original_changed = entity.last_changed;
        let ts = now();

        entity.update_state(EntityState::Off, ts);

        assert_eq!(entity.last_changed, original_changed);
        assert_eq!(entity.last_updated, ts);
    }

    #[test]
    fn should_insert_and_retrieve_attribute() {
        let mut entity = valid_entity();
        entity.set_attribute("brightness".to_string(), AttributeValue::Int(255));

        let attr = entity.get_attribute("brightness");
        assert_eq!(attr, Some(&AttributeValue::Int(255)));
    }

    #[test]
    fn should_return_none_when_attribute_missing() {
        let entity = valid_entity();
        assert_eq!(entity.get_attribute("nonexistent"), None);
    }

    #[test]
    fn should_overwrite_attribute_when_key_exists() {
        let mut entity = valid_entity();
        entity.set_attribute("brightness".to_string(), AttributeValue::Int(100));
        entity.set_attribute("brightness".to_string(), AttributeValue::Int(200));

        assert_eq!(
            entity.get_attribute("brightness"),
            Some(&AttributeValue::Int(200))
        );
    }

    #[test]
    fn should_build_entity_with_attributes_via_builder() {
        let entity = Entity::builder()
            .entity_id("sensor.temp")
            .friendly_name("Temperature")
            .attribute("unit", AttributeValue::String("°C".to_string()))
            .build()
            .unwrap();

        assert_eq!(
            entity.get_attribute("unit"),
            Some(&AttributeValue::String("°C".to_string()))
        );
    }
}
