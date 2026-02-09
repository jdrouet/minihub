//! Trigger — the event pattern that activates an automation.

use serde::{Deserialize, Serialize};

use crate::entity::EntityState;
use crate::event::{Event, EventType};
use crate::id::EntityId;

/// Describes what event pattern should activate an automation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Trigger {
    /// Fires when a specific entity changes state.
    StateChanged {
        entity_id: EntityId,
        /// Optional: only match if transitioning *from* this state.
        from: Option<EntityState>,
        /// Optional: only match if transitioning *to* this state.
        to: Option<EntityState>,
    },
    /// Fires on a cron-like time pattern (e.g. `"0 8 * * *"`).
    TimePattern { cron: String },
    /// Fires only when triggered manually via the API.
    Manual,
}

impl Trigger {
    /// Check whether this trigger matches a given event.
    ///
    /// `TimePattern` and `Manual` triggers never match broadcast events;
    /// they are activated through other mechanisms.
    #[must_use]
    pub fn matches_event(&self, event: &Event) -> bool {
        match self {
            Self::StateChanged {
                entity_id,
                from,
                to,
            } => {
                if event.event_type != EventType::StateChanged {
                    return false;
                }
                if event.entity_id != Some(*entity_id) {
                    return false;
                }
                if let Some(expected_from) = from {
                    let actual = event.data.get("from").and_then(|v| v.as_str());
                    if actual != Some(&expected_from.to_string()) {
                        return false;
                    }
                }
                if let Some(expected_to) = to {
                    let actual = event.data.get("to").and_then(|v| v.as_str());
                    if actual != Some(&expected_to.to_string()) {
                        return false;
                    }
                }
                true
            }
            Self::TimePattern { .. } | Self::Manual => false,
        }
    }
}

impl std::fmt::Display for Trigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StateChanged { entity_id, .. } => write!(f, "state_changed({entity_id})"),
            Self::TimePattern { cron } => write!(f, "time_pattern({cron})"),
            Self::Manual => f.write_str("manual"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_changed_event(entity_id: EntityId, from: &str, to: &str) -> Event {
        Event::new(
            EventType::StateChanged,
            Some(entity_id),
            serde_json::json!({"from": from, "to": to}),
        )
    }

    #[test]
    fn should_match_when_entity_and_type_match_without_from_to() {
        let eid = EntityId::new();
        let trigger = Trigger::StateChanged {
            entity_id: eid,
            from: None,
            to: None,
        };
        let event = state_changed_event(eid, "off", "on");
        assert!(trigger.matches_event(&event));
    }

    #[test]
    fn should_match_when_from_and_to_both_match() {
        let eid = EntityId::new();
        let trigger = Trigger::StateChanged {
            entity_id: eid,
            from: Some(EntityState::Off),
            to: Some(EntityState::On),
        };
        let event = state_changed_event(eid, "off", "on");
        assert!(trigger.matches_event(&event));
    }

    #[test]
    fn should_not_match_when_from_differs() {
        let eid = EntityId::new();
        let trigger = Trigger::StateChanged {
            entity_id: eid,
            from: Some(EntityState::On),
            to: None,
        };
        let event = state_changed_event(eid, "off", "on");
        assert!(!trigger.matches_event(&event));
    }

    #[test]
    fn should_not_match_when_to_differs() {
        let eid = EntityId::new();
        let trigger = Trigger::StateChanged {
            entity_id: eid,
            from: None,
            to: Some(EntityState::Off),
        };
        let event = state_changed_event(eid, "off", "on");
        assert!(!trigger.matches_event(&event));
    }

    #[test]
    fn should_not_match_when_entity_id_differs() {
        let trigger_eid = EntityId::new();
        let event_eid = EntityId::new();
        let trigger = Trigger::StateChanged {
            entity_id: trigger_eid,
            from: None,
            to: None,
        };
        let event = state_changed_event(event_eid, "off", "on");
        assert!(!trigger.matches_event(&event));
    }

    #[test]
    fn should_not_match_when_event_type_is_not_state_changed() {
        let eid = EntityId::new();
        let trigger = Trigger::StateChanged {
            entity_id: eid,
            from: None,
            to: None,
        };
        let event = Event::new(EventType::EntityCreated, Some(eid), serde_json::json!({}));
        assert!(!trigger.matches_event(&event));
    }

    #[test]
    fn should_not_match_time_pattern_trigger_against_events() {
        let trigger = Trigger::TimePattern {
            cron: "0 8 * * *".to_string(),
        };
        let event = state_changed_event(EntityId::new(), "off", "on");
        assert!(!trigger.matches_event(&event));
    }

    #[test]
    fn should_not_match_manual_trigger_against_events() {
        let trigger = Trigger::Manual;
        let event = state_changed_event(EntityId::new(), "off", "on");
        assert!(!trigger.matches_event(&event));
    }

    #[test]
    fn should_display_trigger_variants() {
        let eid = EntityId::new();
        let t = Trigger::StateChanged {
            entity_id: eid,
            from: None,
            to: None,
        };
        assert!(t.to_string().starts_with("state_changed("));

        let t = Trigger::TimePattern {
            cron: "0 8 * * *".to_string(),
        };
        assert_eq!(t.to_string(), "time_pattern(0 8 * * *)");

        assert_eq!(Trigger::Manual.to_string(), "manual");
    }

    #[test]
    fn should_roundtrip_trigger_through_serde_json() {
        let eid = EntityId::new();
        let triggers = vec![
            Trigger::StateChanged {
                entity_id: eid,
                from: Some(EntityState::Off),
                to: Some(EntityState::On),
            },
            Trigger::TimePattern {
                cron: "0 8 * * *".to_string(),
            },
            Trigger::Manual,
        ];

        for trigger in &triggers {
            let json = serde_json::to_string(trigger).unwrap();
            let parsed: Trigger = serde_json::from_str(&json).unwrap();
            assert_eq!(&parsed, trigger);
        }
    }
}
