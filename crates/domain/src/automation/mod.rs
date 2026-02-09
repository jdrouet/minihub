//! Automation — trigger → condition → action rules.
//!
//! Automations allow the system to react to events or state changes
//! without manual intervention. Each automation has a [`Trigger`] that
//! determines when it activates, optional [`Condition`]s that must hold,
//! and one or more [`Action`]s to execute.

mod action;
mod condition;
mod trigger;

pub use action::Action;
pub use condition::Condition;
pub use trigger::Trigger;

use serde::{Deserialize, Serialize};

use crate::error::{MiniHubError, ValidationError};
use crate::id::AutomationId;
use crate::time::Timestamp;

/// A rule that reacts to events by executing actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Automation {
    pub id: AutomationId,
    pub name: String,
    pub enabled: bool,
    pub trigger: Trigger,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
    pub last_triggered: Option<Timestamp>,
}

impl Automation {
    /// Create a builder for constructing an [`Automation`].
    #[must_use]
    pub fn builder() -> AutomationBuilder {
        AutomationBuilder::default()
    }

    /// Check domain invariants.
    ///
    /// # Errors
    ///
    /// Returns [`MiniHubError::Validation`] when:
    /// - `name` is empty ([`ValidationError::EmptyName`])
    /// - `actions` is empty ([`ValidationError::NoActions`])
    pub fn validate(&self) -> Result<(), MiniHubError> {
        if self.name.is_empty() {
            return Err(ValidationError::EmptyName.into());
        }
        if self.actions.is_empty() {
            return Err(ValidationError::NoActions.into());
        }
        Ok(())
    }
}

/// Step-by-step builder for [`Automation`].
#[derive(Debug, Default)]
pub struct AutomationBuilder {
    id: Option<AutomationId>,
    name: Option<String>,
    enabled: Option<bool>,
    trigger: Option<Trigger>,
    conditions: Vec<Condition>,
    actions: Vec<Action>,
    last_triggered: Option<Timestamp>,
}

impl AutomationBuilder {
    #[must_use]
    pub fn id(mut self, id: AutomationId) -> Self {
        self.id = Some(id);
        self
    }

    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    #[must_use]
    pub fn trigger(mut self, trigger: Trigger) -> Self {
        self.trigger = Some(trigger);
        self
    }

    #[must_use]
    pub fn condition(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    #[must_use]
    pub fn action(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }

    #[must_use]
    pub fn last_triggered(mut self, ts: Timestamp) -> Self {
        self.last_triggered = Some(ts);
        self
    }

    /// Consume the builder, validate, and return an [`Automation`].
    ///
    /// # Errors
    ///
    /// Returns [`MiniHubError::Validation`] if required fields are missing or empty.
    pub fn build(self) -> Result<Automation, MiniHubError> {
        let automation = Automation {
            id: self.id.unwrap_or_default(),
            name: self.name.unwrap_or_default(),
            enabled: self.enabled.unwrap_or(true),
            trigger: self.trigger.unwrap_or(Trigger::Manual),
            conditions: self.conditions,
            actions: self.actions,
            last_triggered: self.last_triggered,
        };
        automation.validate()?;
        Ok(automation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::EntityState;
    use crate::event::{Event, EventType};
    use crate::id::EntityId;

    fn valid_action() -> Action {
        Action::CallService {
            entity_id: EntityId::new(),
            service: "turn_on".to_string(),
            data: serde_json::json!({}),
        }
    }

    fn valid_automation() -> Automation {
        Automation::builder()
            .name("Turn on lights at sunset")
            .trigger(Trigger::StateChanged {
                entity_id: EntityId::new(),
                from: None,
                to: Some(EntityState::On),
            })
            .action(valid_action())
            .build()
            .unwrap()
    }

    #[test]
    fn should_build_valid_automation_when_required_fields_provided() {
        let auto = valid_automation();
        assert_eq!(auto.name, "Turn on lights at sunset");
        assert!(auto.enabled);
        assert!(auto.conditions.is_empty());
        assert_eq!(auto.actions.len(), 1);
        assert!(auto.last_triggered.is_none());
    }

    #[test]
    fn should_default_to_enabled_when_not_specified() {
        let auto = valid_automation();
        assert!(auto.enabled);
    }

    #[test]
    fn should_build_disabled_automation_when_enabled_is_false() {
        let auto = Automation::builder()
            .name("Disabled rule")
            .enabled(false)
            .action(valid_action())
            .build()
            .unwrap();
        assert!(!auto.enabled);
    }

    #[test]
    fn should_default_to_manual_trigger_when_not_specified() {
        let auto = Automation::builder()
            .name("Manual rule")
            .action(valid_action())
            .build()
            .unwrap();
        assert!(matches!(auto.trigger, Trigger::Manual));
    }

    #[test]
    fn should_return_validation_error_when_name_is_empty() {
        let result = Automation::builder().action(valid_action()).build();
        assert!(matches!(
            result,
            Err(MiniHubError::Validation(ValidationError::EmptyName))
        ));
    }

    #[test]
    fn should_return_validation_error_when_actions_is_empty() {
        let result = Automation::builder().name("No actions").build();
        assert!(matches!(
            result,
            Err(MiniHubError::Validation(ValidationError::NoActions))
        ));
    }

    #[test]
    fn should_accumulate_multiple_conditions() {
        let eid = EntityId::new();
        let auto = Automation::builder()
            .name("Multi-condition")
            .action(valid_action())
            .condition(Condition::StateIs {
                entity_id: eid,
                state: "on".to_string(),
            })
            .condition(Condition::TimeRange {
                after: "08:00".to_string(),
                before: "22:00".to_string(),
            })
            .build()
            .unwrap();
        assert_eq!(auto.conditions.len(), 2);
    }

    #[test]
    fn should_accumulate_multiple_actions() {
        let auto = Automation::builder()
            .name("Multi-action")
            .action(valid_action())
            .action(Action::Delay { seconds: 5 })
            .action(valid_action())
            .build()
            .unwrap();
        assert_eq!(auto.actions.len(), 3);
    }

    #[test]
    fn should_set_last_triggered_via_builder() {
        let ts = crate::time::now();
        let auto = Automation::builder()
            .name("With timestamp")
            .action(valid_action())
            .last_triggered(ts)
            .build()
            .unwrap();
        assert_eq!(auto.last_triggered, Some(ts));
    }

    #[test]
    fn should_set_custom_id_via_builder() {
        let id = AutomationId::new();
        let auto = Automation::builder()
            .id(id)
            .name("Custom ID")
            .action(valid_action())
            .build()
            .unwrap();
        assert_eq!(auto.id, id);
    }

    #[test]
    fn should_roundtrip_automation_through_serde_json() {
        let auto = valid_automation();
        let json = serde_json::to_string(&auto).unwrap();
        let parsed: Automation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, auto.id);
        assert_eq!(parsed.name, auto.name);
        assert_eq!(parsed.enabled, auto.enabled);
        assert_eq!(parsed.actions.len(), auto.actions.len());
    }

    #[test]
    fn should_match_trigger_against_matching_event() {
        let eid = EntityId::new();
        let auto = Automation::builder()
            .name("Light watcher")
            .trigger(Trigger::StateChanged {
                entity_id: eid,
                from: Some(EntityState::Off),
                to: Some(EntityState::On),
            })
            .action(valid_action())
            .build()
            .unwrap();

        let event = Event::new(
            EventType::StateChanged,
            Some(eid),
            serde_json::json!({"from": "off", "to": "on"}),
        );
        assert!(auto.trigger.matches_event(&event));
    }

    #[test]
    fn should_not_match_trigger_against_different_entity() {
        let auto = Automation::builder()
            .name("Light watcher")
            .trigger(Trigger::StateChanged {
                entity_id: EntityId::new(),
                from: None,
                to: None,
            })
            .action(valid_action())
            .build()
            .unwrap();

        let event = Event::new(
            EventType::StateChanged,
            Some(EntityId::new()),
            serde_json::json!({"from": "off", "to": "on"}),
        );
        assert!(!auto.trigger.matches_event(&event));
    }
}
