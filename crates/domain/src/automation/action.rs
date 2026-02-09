//! Action — the effect performed when an automation fires.

use serde::{Deserialize, Serialize};

use crate::id::EntityId;

/// An operation to execute when the automation's trigger fires and
/// all conditions are satisfied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    /// Invoke a service on a target entity (e.g. `"turn_on"`, `"toggle"`).
    CallService {
        entity_id: EntityId,
        /// Service name, e.g. `"turn_on"`, `"turn_off"`, `"toggle"`.
        service: String,
        /// Additional parameters for the service call.
        #[serde(default)]
        data: serde_json::Value,
    },
    /// Wait for a specified duration before continuing to the next action.
    Delay {
        /// Number of seconds to wait.
        seconds: u64,
    },
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CallService {
                entity_id, service, ..
            } => write!(f, "call_service({service}, {entity_id})"),
            Self::Delay { seconds } => write!(f, "delay({seconds}s)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_display_call_service_action() {
        let eid = EntityId::new();
        let a = Action::CallService {
            entity_id: eid,
            service: "turn_on".to_string(),
            data: serde_json::json!({}),
        };
        let display = a.to_string();
        assert!(display.starts_with("call_service(turn_on,"));
    }

    #[test]
    fn should_display_delay_action() {
        let a = Action::Delay { seconds: 30 };
        assert_eq!(a.to_string(), "delay(30s)");
    }

    #[test]
    fn should_roundtrip_actions_through_serde_json() {
        let eid = EntityId::new();
        let actions = vec![
            Action::CallService {
                entity_id: eid,
                service: "turn_on".to_string(),
                data: serde_json::json!({"brightness": 255}),
            },
            Action::Delay { seconds: 5 },
        ];

        for action in &actions {
            let json = serde_json::to_string(action).unwrap();
            let parsed: Action = serde_json::from_str(&json).unwrap();
            assert_eq!(&parsed, action);
        }
    }

    #[test]
    fn should_deserialize_call_service_from_tagged_json() {
        let eid = EntityId::new();
        let json = serde_json::json!({
            "type": "call_service",
            "entity_id": eid,
            "service": "toggle",
            "data": {"brightness": 128}
        });
        let a: Action = serde_json::from_value(json).unwrap();
        assert!(matches!(a, Action::CallService { service, .. } if service == "toggle"));
    }

    #[test]
    fn should_deserialize_call_service_with_default_data() {
        let eid = EntityId::new();
        let json = serde_json::json!({
            "type": "call_service",
            "entity_id": eid,
            "service": "turn_off"
        });
        let a: Action = serde_json::from_value(json).unwrap();
        match a {
            Action::CallService { data, .. } => assert!(data.is_null()),
            Action::Delay { .. } => panic!("expected CallService"),
        }
    }

    #[test]
    fn should_deserialize_delay_from_tagged_json() {
        let json = serde_json::json!({
            "type": "delay",
            "seconds": 10
        });
        let a: Action = serde_json::from_value(json).unwrap();
        assert!(matches!(a, Action::Delay { seconds: 10 }));
    }
}
