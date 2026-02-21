//! Entity history — time-series records of entity state and attribute changes.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::entity::{AttributeValue, EntityState};
use crate::id::EntityId;
use crate::time::Timestamp;

/// A unique identifier for an [`EntityHistory`] record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityHistoryId(uuid::Uuid);

impl Default for EntityHistoryId {
    fn default() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl EntityHistoryId {
    /// Generate a new random identifier.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Wrap an existing UUID.
    #[must_use]
    pub fn from_uuid(uuid: uuid::Uuid) -> Self {
        Self(uuid)
    }

    /// Access the inner UUID.
    #[must_use]
    pub fn as_uuid(self) -> uuid::Uuid {
        self.0
    }
}

impl std::fmt::Display for EntityHistoryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for EntityHistoryId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        uuid::Uuid::parse_str(s).map(Self)
    }
}

/// A historical snapshot of an entity's state and attributes at a specific point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityHistory {
    pub id: EntityHistoryId,
    pub entity_id: EntityId,
    pub state: EntityState,
    pub attributes: HashMap<String, AttributeValue>,
    pub recorded_at: Timestamp,
}

impl EntityHistory {
    /// Create a builder for constructing an [`EntityHistory`].
    #[must_use]
    pub fn builder() -> EntityHistoryBuilder {
        EntityHistoryBuilder::default()
    }

    /// Look up an attribute by key.
    #[must_use]
    pub fn get_attribute(&self, key: &str) -> Option<&AttributeValue> {
        self.attributes.get(key)
    }
}

/// Step-by-step builder for [`EntityHistory`].
#[derive(Debug, Default)]
pub struct EntityHistoryBuilder {
    id: Option<EntityHistoryId>,
    entity_id: Option<EntityId>,
    state: Option<EntityState>,
    attributes: HashMap<String, AttributeValue>,
    recorded_at: Option<Timestamp>,
}

impl EntityHistoryBuilder {
    #[must_use]
    pub fn id(mut self, id: EntityHistoryId) -> Self {
        self.id = Some(id);
        self
    }

    #[must_use]
    pub fn entity_id(mut self, entity_id: EntityId) -> Self {
        self.entity_id = Some(entity_id);
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

    #[must_use]
    pub fn attributes(mut self, attributes: HashMap<String, AttributeValue>) -> Self {
        self.attributes = attributes;
        self
    }

    #[must_use]
    pub fn recorded_at(mut self, recorded_at: Timestamp) -> Self {
        self.recorded_at = Some(recorded_at);
        self
    }

    /// Consume the builder and return an [`EntityHistory`].
    #[must_use]
    pub fn build(self) -> EntityHistory {
        EntityHistory {
            id: self.id.unwrap_or_default(),
            entity_id: self.entity_id.unwrap_or_default(),
            state: self.state.unwrap_or_default(),
            attributes: self.attributes,
            recorded_at: self.recorded_at.unwrap_or_else(crate::time::now),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::now;

    #[test]
    fn should_build_entity_history_with_all_fields() {
        let entity_id = EntityId::new();
        let recorded = now();

        let history = EntityHistory::builder()
            .entity_id(entity_id)
            .state(EntityState::On)
            .attribute("brightness", AttributeValue::Int(255))
            .recorded_at(recorded)
            .build();

        assert_eq!(history.entity_id, entity_id);
        assert_eq!(history.state, EntityState::On);
        assert_eq!(history.recorded_at, recorded);
        assert_eq!(
            history.attributes.get("brightness"),
            Some(&AttributeValue::Int(255))
        );
    }

    #[test]
    fn should_use_defaults_when_fields_not_provided() {
        let history = EntityHistory::builder().build();

        assert_eq!(history.state, EntityState::Unknown);
        assert!(history.attributes.is_empty());
    }

    #[test]
    fn should_generate_unique_ids_when_called_twice() {
        let id1 = EntityHistoryId::new();
        let id2 = EntityHistoryId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn should_roundtrip_through_display_and_from_str() {
        let id = EntityHistoryId::new();
        let text = id.to_string();
        let parsed: EntityHistoryId = text.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn should_roundtrip_through_serde_json() {
        let id = EntityHistoryId::new();
        let json = serde_json::to_string(&id).unwrap();
        let parsed: EntityHistoryId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn should_wrap_existing_uuid_when_using_from_uuid() {
        let uuid = uuid::Uuid::new_v4();
        let id = EntityHistoryId::from_uuid(uuid);
        assert_eq!(id.as_uuid(), uuid);
    }

    #[test]
    fn should_build_with_multiple_attributes() {
        let mut attrs = HashMap::new();
        attrs.insert("temperature".to_string(), AttributeValue::Float(22.5));
        attrs.insert("humidity".to_string(), AttributeValue::Int(65));

        let history = EntityHistory::builder()
            .entity_id(EntityId::new())
            .state(EntityState::On)
            .attributes(attrs.clone())
            .build();

        assert_eq!(history.attributes.len(), 2);
        assert_eq!(
            history.attributes.get("temperature"),
            Some(&AttributeValue::Float(22.5))
        );
        assert_eq!(
            history.attributes.get("humidity"),
            Some(&AttributeValue::Int(65))
        );
    }

    #[test]
    fn should_serialize_and_deserialize_entity_history() {
        let history = EntityHistory::builder()
            .entity_id(EntityId::new())
            .state(EntityState::Off)
            .attribute("power", AttributeValue::Float(100.5))
            .build();

        let json = serde_json::to_string(&history).unwrap();
        let deserialized: EntityHistory = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.entity_id, history.entity_id);
        assert_eq!(deserialized.state, history.state);
        assert_eq!(deserialized.attributes, history.attributes);
    }
}
