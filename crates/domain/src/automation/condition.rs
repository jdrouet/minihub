//! Condition — a guard that must be true for the automation to proceed.

use serde::{Deserialize, Serialize};

use crate::id::EntityId;

/// A predicate that must hold for the automation actions to execute.
///
/// Conditions are evaluated *after* the trigger fires. All conditions
/// in an automation must be satisfied (logical AND).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Condition {
    /// Requires a specific entity to be in a given state.
    StateIs {
        entity_id: EntityId,
        /// Expected state value, e.g. `"on"`, `"off"`.
        state: String,
    },
    /// Requires the current time to be within a window.
    TimeRange {
        /// Start of the window, `HH:MM` in 24-hour format.
        after: String,
        /// End of the window, `HH:MM` in 24-hour format.
        before: String,
    },
}

impl std::fmt::Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StateIs { entity_id, state } => {
                write!(f, "state_is({entity_id}, {state})")
            }
            Self::TimeRange { after, before } => {
                write!(f, "time_range({after}..{before})")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_display_state_is_condition() {
        let eid = EntityId::new();
        let c = Condition::StateIs {
            entity_id: eid,
            state: "on".to_string(),
        };
        let display = c.to_string();
        assert!(display.starts_with("state_is("));
        assert!(display.contains("on"));
    }

    #[test]
    fn should_display_time_range_condition() {
        let c = Condition::TimeRange {
            after: "08:00".to_string(),
            before: "22:00".to_string(),
        };
        assert_eq!(c.to_string(), "time_range(08:00..22:00)");
    }

    #[test]
    fn should_roundtrip_conditions_through_serde_json() {
        let eid = EntityId::new();
        let conditions = vec![
            Condition::StateIs {
                entity_id: eid,
                state: "on".to_string(),
            },
            Condition::TimeRange {
                after: "08:00".to_string(),
                before: "22:00".to_string(),
            },
        ];

        for condition in &conditions {
            let json = serde_json::to_string(condition).unwrap();
            let parsed: Condition = serde_json::from_str(&json).unwrap();
            assert_eq!(&parsed, condition);
        }
    }

    #[test]
    fn should_deserialize_state_is_from_tagged_json() {
        let eid = EntityId::new();
        let json = serde_json::json!({
            "type": "state_is",
            "entity_id": eid,
            "state": "off"
        });
        let c: Condition = serde_json::from_value(json).unwrap();
        assert!(matches!(c, Condition::StateIs { state, .. } if state == "off"));
    }

    #[test]
    fn should_deserialize_time_range_from_tagged_json() {
        let json = serde_json::json!({
            "type": "time_range",
            "after": "06:00",
            "before": "09:00"
        });
        let c: Condition = serde_json::from_value(json).unwrap();
        assert!(
            matches!(c, Condition::TimeRange { after, before } if after == "06:00" && before == "09:00")
        );
    }
}
