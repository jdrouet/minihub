//! Event — an immutable record of something that happened.
//!
//! Events are produced when entity state changes, services are called,
//! automations fire, etc.

use serde::{Deserialize, Serialize};

use crate::id::{EntityId, EventId};
use crate::time::Timestamp;

/// An immutable record of something that happened in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub event_type: EventType,
    pub entity_id: Option<EntityId>,
    pub timestamp: Timestamp,
    pub data: serde_json::Value,
}

/// The kind of event that occurred.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    StateChanged,
    AttributeChanged,
    EntityCreated,
    EntityRemoved,
    AutomationTriggered,
    DeviceDetected,
}

impl Event {
    /// Create a new event with the current timestamp.
    #[must_use]
    pub fn new(
        event_type: EventType,
        entity_id: Option<EntityId>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            id: EventId::new(),
            event_type,
            entity_id,
            timestamp: crate::time::now(),
            data,
        }
    }
}

impl EventType {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::StateChanged => "state_changed",
            Self::AttributeChanged => "attribute_changed",
            Self::EntityCreated => "entity_created",
            Self::EntityRemoved => "entity_removed",
            Self::AutomationTriggered => "automation_triggered",
            Self::DeviceDetected => "device_detected",
        }
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_event_with_generated_id_and_timestamp() {
        let event = Event::new(
            EventType::StateChanged,
            Some(EntityId::new()),
            serde_json::json!({"from": "off", "to": "on"}),
        );

        assert_eq!(event.event_type, EventType::StateChanged);
        assert!(event.entity_id.is_some());
        assert_eq!(event.data["from"], "off");
        assert_eq!(event.data["to"], "on");
    }

    #[test]
    fn should_create_event_without_entity_id() {
        let event = Event::new(
            EventType::AutomationTriggered,
            None,
            serde_json::json!({"automation": "test"}),
        );

        assert_eq!(event.event_type, EventType::AutomationTriggered);
        assert!(event.entity_id.is_none());
    }

    #[test]
    fn should_generate_unique_ids_for_different_events() {
        let a = Event::new(EventType::StateChanged, None, serde_json::json!({}));
        let b = Event::new(EventType::StateChanged, None, serde_json::json!({}));
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn should_roundtrip_event_through_serde_json() {
        let event = Event::new(
            EventType::EntityCreated,
            Some(EntityId::new()),
            serde_json::json!({"name": "test"}),
        );

        let json = serde_json::to_string(&event).unwrap();
        let parsed: Event = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, event.id);
        assert_eq!(parsed.event_type, event.event_type);
        assert_eq!(parsed.entity_id, event.entity_id);
        assert_eq!(parsed.data, event.data);
    }

    #[test]
    fn should_roundtrip_event_type_through_serde_json() {
        let variants = [
            EventType::StateChanged,
            EventType::AttributeChanged,
            EventType::EntityCreated,
            EventType::EntityRemoved,
            EventType::AutomationTriggered,
            EventType::DeviceDetected,
        ];

        for variant in &variants {
            let json = serde_json::to_string(variant).unwrap();
            let parsed: EventType = serde_json::from_str(&json).unwrap();
            assert_eq!(&parsed, variant);
        }
    }

    #[test]
    fn should_display_event_type_as_snake_case() {
        assert_eq!(EventType::StateChanged.to_string(), "state_changed");
        assert_eq!(EventType::EntityCreated.to_string(), "entity_created");
        assert_eq!(
            EventType::AutomationTriggered.to_string(),
            "automation_triggered"
        );
        assert_eq!(EventType::DeviceDetected.to_string(), "device_detected");
    }
}
