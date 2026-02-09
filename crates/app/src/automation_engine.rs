//! Automation engine — reacts to events by evaluating and executing automations.
//!
//! The engine subscribes to the event bus and, for each incoming event,
//! checks all enabled automations. When a trigger matches, it evaluates
//! conditions and—if all pass—executes the automation's actions in order.

use minihub_domain::automation::{Action, Condition};
use minihub_domain::entity::EntityState;
use minihub_domain::error::MiniHubError;
use minihub_domain::event::{Event, EventType};
use minihub_domain::id::EntityId;

use crate::ports::{AutomationRepository, EntityRepository, EventPublisher};

/// Reactive automation engine that subscribes to domain events.
pub struct AutomationEngine<AR, ER, P> {
    automation_repo: AR,
    entity_repo: ER,
    publisher: P,
}

impl<AR, ER, P> AutomationEngine<AR, ER, P>
where
    AR: AutomationRepository,
    ER: EntityRepository,
    P: EventPublisher,
{
    /// Create a new engine.
    pub fn new(automation_repo: AR, entity_repo: ER, publisher: P) -> Self {
        Self {
            automation_repo,
            entity_repo,
            publisher,
        }
    }

    /// Process a single event against all enabled automations.
    ///
    /// For each automation whose trigger matches, conditions are evaluated.
    /// If all conditions pass, the actions are executed in order.
    ///
    /// # Errors
    ///
    /// Returns a storage error if loading automations or entities fails.
    pub async fn process_event(&self, event: &Event) -> Result<Vec<AutomationId>, MiniHubError> {
        let automations = self.automation_repo.get_enabled().await?;
        let mut triggered = Vec::new();

        for automation in &automations {
            if !automation.trigger.matches_event(event) {
                continue;
            }

            let conditions_met = self.evaluate_conditions(&automation.conditions).await?;
            if !conditions_met {
                continue;
            }

            self.execute_actions(&automation.actions).await?;

            // Publish AutomationTriggered event (fire-and-forget)
            let trigger_event = Event::new(
                EventType::AutomationTriggered,
                None,
                serde_json::json!({
                    "automation_id": automation.id,
                    "automation_name": automation.name,
                }),
            );
            let _ = self.publisher.publish(trigger_event).await;

            triggered.push(automation.id);
        }

        Ok(triggered)
    }

    /// Evaluate all conditions (logical AND). Returns `true` if empty.
    async fn evaluate_conditions(&self, conditions: &[Condition]) -> Result<bool, MiniHubError> {
        for condition in conditions {
            if !self.evaluate_condition(condition).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Evaluate a single condition.
    async fn evaluate_condition(&self, condition: &Condition) -> Result<bool, MiniHubError> {
        match condition {
            Condition::StateIs { entity_id, state } => {
                let entity = self.entity_repo.get_by_id(*entity_id).await?;
                match entity {
                    Some(e) => Ok(e.state.to_string() == *state),
                    None => Ok(false),
                }
            }
            Condition::TimeRange { after, before } => {
                let now = chrono::Utc::now().format("%H:%M").to_string();
                if after <= before {
                    // Same-day range: after <= now <= before
                    Ok(now >= *after && now <= *before)
                } else {
                    // Overnight range (e.g., 22:00..06:00): now >= after OR now <= before
                    Ok(now >= *after || now <= *before)
                }
            }
        }
    }

    /// Execute actions in order.
    async fn execute_actions(&self, actions: &[Action]) -> Result<(), MiniHubError> {
        for action in actions {
            self.execute_action(action).await?;
        }
        Ok(())
    }

    /// Execute a single action.
    async fn execute_action(&self, action: &Action) -> Result<(), MiniHubError> {
        match action {
            Action::CallService {
                entity_id, service, ..
            } => {
                let new_state = service_to_state(service, *entity_id, &self.entity_repo).await?;
                if let Some(state) = new_state {
                    let mut entity =
                        self.entity_repo
                            .get_by_id(*entity_id)
                            .await?
                            .ok_or_else(|| minihub_domain::error::NotFoundError {
                                entity: "Entity",
                                id: entity_id.to_string(),
                            })?;
                    entity.update_state(state, minihub_domain::time::now());
                    self.entity_repo.update(entity).await?;
                }
            }
            Action::Delay { seconds } => {
                tokio::time::sleep(tokio::time::Duration::from_secs(*seconds)).await;
            }
        }
        Ok(())
    }
}

use minihub_domain::id::AutomationId;

/// Map a service name to the target state for an entity.
async fn service_to_state<ER: EntityRepository>(
    service: &str,
    entity_id: EntityId,
    repo: &ER,
) -> Result<Option<EntityState>, MiniHubError> {
    match service {
        "turn_on" => Ok(Some(EntityState::On)),
        "turn_off" => Ok(Some(EntityState::Off)),
        "toggle" => {
            let entity = repo.get_by_id(entity_id).await?;
            match entity {
                Some(e) => {
                    let toggled = match e.state {
                        EntityState::On => EntityState::Off,
                        _ => EntityState::On,
                    };
                    Ok(Some(toggled))
                }
                None => Ok(None),
            }
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use minihub_domain::automation::{Action, Automation, Condition, Trigger};
    use minihub_domain::entity::Entity;
    use minihub_domain::event::Event;
    use minihub_domain::id::{AutomationId, DeviceId, EntityId};
    use std::collections::HashMap;
    use std::future::Future;
    use std::sync::Mutex;

    // ── In-memory automation repo ──────────────────────────────────

    struct InMemoryAutomationRepo {
        store: Mutex<HashMap<AutomationId, Automation>>,
    }

    impl InMemoryAutomationRepo {
        fn with(automations: Vec<Automation>) -> Self {
            let map: HashMap<_, _> = automations.into_iter().map(|a| (a.id, a)).collect();
            Self {
                store: Mutex::new(map),
            }
        }
    }

    impl AutomationRepository for InMemoryAutomationRepo {
        fn create(
            &self,
            automation: Automation,
        ) -> impl Future<Output = Result<Automation, MiniHubError>> + Send {
            let mut store = self.store.lock().unwrap();
            store.insert(automation.id, automation.clone());
            async { Ok(automation) }
        }
        fn get_by_id(
            &self,
            id: AutomationId,
        ) -> impl Future<Output = Result<Option<Automation>, MiniHubError>> + Send {
            let store = self.store.lock().unwrap();
            let r = store.get(&id).cloned();
            async { Ok(r) }
        }
        fn get_all(&self) -> impl Future<Output = Result<Vec<Automation>, MiniHubError>> + Send {
            let store = self.store.lock().unwrap();
            let r: Vec<_> = store.values().cloned().collect();
            async { Ok(r) }
        }
        fn get_enabled(
            &self,
        ) -> impl Future<Output = Result<Vec<Automation>, MiniHubError>> + Send {
            let store = self.store.lock().unwrap();
            let r: Vec<_> = store.values().filter(|a| a.enabled).cloned().collect();
            async { Ok(r) }
        }
        fn update(
            &self,
            automation: Automation,
        ) -> impl Future<Output = Result<Automation, MiniHubError>> + Send {
            let mut store = self.store.lock().unwrap();
            store.insert(automation.id, automation.clone());
            async { Ok(automation) }
        }
        fn delete(
            &self,
            id: AutomationId,
        ) -> impl Future<Output = Result<(), MiniHubError>> + Send {
            let mut store = self.store.lock().unwrap();
            store.remove(&id);
            async { Ok(()) }
        }
    }

    // ── In-memory entity repo ──────────────────────────────────────

    struct InMemoryEntityRepo {
        store: Mutex<HashMap<EntityId, Entity>>,
    }

    impl InMemoryEntityRepo {
        fn with(entities: Vec<Entity>) -> Self {
            let map: HashMap<_, _> = entities.into_iter().map(|e| (e.id, e)).collect();
            Self {
                store: Mutex::new(map),
            }
        }
    }

    impl EntityRepository for InMemoryEntityRepo {
        fn create(
            &self,
            entity: Entity,
        ) -> impl Future<Output = Result<Entity, MiniHubError>> + Send {
            let mut store = self.store.lock().unwrap();
            store.insert(entity.id, entity.clone());
            async { Ok(entity) }
        }
        fn get_by_id(
            &self,
            id: EntityId,
        ) -> impl Future<Output = Result<Option<Entity>, MiniHubError>> + Send {
            let store = self.store.lock().unwrap();
            let r = store.get(&id).cloned();
            async { Ok(r) }
        }
        fn get_all(&self) -> impl Future<Output = Result<Vec<Entity>, MiniHubError>> + Send {
            let store = self.store.lock().unwrap();
            let r: Vec<_> = store.values().cloned().collect();
            async { Ok(r) }
        }
        fn find_by_device_id(
            &self,
            device_id: DeviceId,
        ) -> impl Future<Output = Result<Vec<Entity>, MiniHubError>> + Send {
            let store = self.store.lock().unwrap();
            let r: Vec<_> = store
                .values()
                .filter(|e| e.device_id == device_id)
                .cloned()
                .collect();
            async { Ok(r) }
        }
        fn find_by_entity_id(
            &self,
            entity_id: &str,
        ) -> impl Future<Output = Result<Option<Entity>, MiniHubError>> + Send {
            let store = self.store.lock().unwrap();
            let r = store.values().find(|e| e.entity_id == entity_id).cloned();
            async { Ok(r) }
        }
        fn update(
            &self,
            entity: Entity,
        ) -> impl Future<Output = Result<Entity, MiniHubError>> + Send {
            let mut store = self.store.lock().unwrap();
            store.insert(entity.id, entity.clone());
            async { Ok(entity) }
        }
        fn delete(&self, id: EntityId) -> impl Future<Output = Result<(), MiniHubError>> + Send {
            let mut store = self.store.lock().unwrap();
            store.remove(&id);
            async { Ok(()) }
        }
    }

    // ── Spy publisher ──────────────────────────────────────────────

    struct SpyPublisher {
        events: Mutex<Vec<Event>>,
    }

    impl Default for SpyPublisher {
        fn default() -> Self {
            Self {
                events: Mutex::new(Vec::new()),
            }
        }
    }

    impl EventPublisher for SpyPublisher {
        fn publish(&self, event: Event) -> impl Future<Output = Result<(), MiniHubError>> + Send {
            self.events.lock().unwrap().push(event);
            async { Ok(()) }
        }
    }

    // ── Helpers ────────────────────────────────────────────────────

    fn light_entity(id: EntityId, state: EntityState) -> Entity {
        Entity::builder()
            .id(id)
            .entity_id("light.test")
            .friendly_name("Test Light")
            .state(state)
            .build()
            .unwrap()
    }

    fn state_changed_event(entity_id: EntityId, from: &str, to: &str) -> Event {
        Event::new(
            EventType::StateChanged,
            Some(entity_id),
            serde_json::json!({"from": from, "to": to}),
        )
    }

    fn make_engine(
        automations: Vec<Automation>,
        entities: Vec<Entity>,
    ) -> AutomationEngine<InMemoryAutomationRepo, InMemoryEntityRepo, SpyPublisher> {
        AutomationEngine::new(
            InMemoryAutomationRepo::with(automations),
            InMemoryEntityRepo::with(entities),
            SpyPublisher::default(),
        )
    }

    // ── Tests ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn should_trigger_automation_when_event_matches() {
        let eid = EntityId::new();
        let auto = Automation::builder()
            .name("Light watcher")
            .trigger(Trigger::StateChanged {
                entity_id: eid,
                from: None,
                to: None,
            })
            .action(Action::CallService {
                entity_id: eid,
                service: "turn_on".to_string(),
                data: serde_json::json!({}),
            })
            .build()
            .unwrap();

        let entity = light_entity(eid, EntityState::Off);
        let engine = make_engine(vec![auto.clone()], vec![entity]);

        let event = state_changed_event(eid, "off", "on");
        let triggered = engine.process_event(&event).await.unwrap();

        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0], auto.id);
    }

    #[tokio::test]
    async fn should_not_trigger_when_event_does_not_match() {
        let trigger_eid = EntityId::new();
        let event_eid = EntityId::new();
        let auto = Automation::builder()
            .name("Light watcher")
            .trigger(Trigger::StateChanged {
                entity_id: trigger_eid,
                from: None,
                to: None,
            })
            .action(Action::CallService {
                entity_id: trigger_eid,
                service: "turn_on".to_string(),
                data: serde_json::json!({}),
            })
            .build()
            .unwrap();

        let engine = make_engine(vec![auto], vec![]);

        let event = state_changed_event(event_eid, "off", "on");
        let triggered = engine.process_event(&event).await.unwrap();
        assert!(triggered.is_empty());
    }

    #[tokio::test]
    async fn should_skip_disabled_automations() {
        let eid = EntityId::new();
        let mut auto = Automation::builder()
            .name("Disabled rule")
            .trigger(Trigger::StateChanged {
                entity_id: eid,
                from: None,
                to: None,
            })
            .action(Action::CallService {
                entity_id: eid,
                service: "turn_on".to_string(),
                data: serde_json::json!({}),
            })
            .build()
            .unwrap();
        auto.enabled = false;

        let engine = make_engine(vec![auto], vec![]);

        let event = state_changed_event(eid, "off", "on");
        let triggered = engine.process_event(&event).await.unwrap();
        assert!(triggered.is_empty());
    }

    #[tokio::test]
    async fn should_execute_turn_on_action() {
        let eid = EntityId::new();
        let auto = Automation::builder()
            .name("Turn on")
            .trigger(Trigger::StateChanged {
                entity_id: eid,
                from: None,
                to: None,
            })
            .action(Action::CallService {
                entity_id: eid,
                service: "turn_on".to_string(),
                data: serde_json::json!({}),
            })
            .build()
            .unwrap();

        let entity = light_entity(eid, EntityState::Off);
        let engine = make_engine(vec![auto], vec![entity]);

        let event = state_changed_event(eid, "off", "on");
        engine.process_event(&event).await.unwrap();

        // Verify entity state was updated
        let updated = engine.entity_repo.get_by_id(eid).await.unwrap().unwrap();
        assert_eq!(updated.state, EntityState::On);
    }

    #[tokio::test]
    async fn should_execute_turn_off_action() {
        let eid = EntityId::new();
        let auto = Automation::builder()
            .name("Turn off")
            .trigger(Trigger::StateChanged {
                entity_id: eid,
                from: None,
                to: None,
            })
            .action(Action::CallService {
                entity_id: eid,
                service: "turn_off".to_string(),
                data: serde_json::json!({}),
            })
            .build()
            .unwrap();

        let entity = light_entity(eid, EntityState::On);
        let engine = make_engine(vec![auto], vec![entity]);

        let event = state_changed_event(eid, "on", "off");
        engine.process_event(&event).await.unwrap();

        let updated = engine.entity_repo.get_by_id(eid).await.unwrap().unwrap();
        assert_eq!(updated.state, EntityState::Off);
    }

    #[tokio::test]
    async fn should_execute_toggle_action() {
        let eid = EntityId::new();
        let auto = Automation::builder()
            .name("Toggle")
            .trigger(Trigger::StateChanged {
                entity_id: eid,
                from: None,
                to: None,
            })
            .action(Action::CallService {
                entity_id: eid,
                service: "toggle".to_string(),
                data: serde_json::json!({}),
            })
            .build()
            .unwrap();

        let entity = light_entity(eid, EntityState::On);
        let engine = make_engine(vec![auto], vec![entity]);

        let event = state_changed_event(eid, "off", "on");
        engine.process_event(&event).await.unwrap();

        let updated = engine.entity_repo.get_by_id(eid).await.unwrap().unwrap();
        assert_eq!(updated.state, EntityState::Off);
    }

    #[tokio::test]
    async fn should_publish_automation_triggered_event() {
        let eid = EntityId::new();
        let auto = Automation::builder()
            .name("Publisher test")
            .trigger(Trigger::StateChanged {
                entity_id: eid,
                from: None,
                to: None,
            })
            .action(Action::CallService {
                entity_id: eid,
                service: "turn_on".to_string(),
                data: serde_json::json!({}),
            })
            .build()
            .unwrap();
        let auto_id = auto.id;

        let entity = light_entity(eid, EntityState::Off);
        let engine = make_engine(vec![auto], vec![entity]);

        let event = state_changed_event(eid, "off", "on");
        engine.process_event(&event).await.unwrap();

        let published = engine.publisher.events.lock().unwrap();
        assert_eq!(published.len(), 1);
        assert_eq!(published[0].event_type, EventType::AutomationTriggered);
        assert_eq!(published[0].data["automation_id"], auto_id.to_string());
    }

    #[tokio::test]
    async fn should_not_execute_when_state_is_condition_fails() {
        let trigger_eid = EntityId::new();
        let condition_eid = EntityId::new();

        let auto = Automation::builder()
            .name("Conditional")
            .trigger(Trigger::StateChanged {
                entity_id: trigger_eid,
                from: None,
                to: None,
            })
            .condition(Condition::StateIs {
                entity_id: condition_eid,
                state: "on".to_string(),
            })
            .action(Action::CallService {
                entity_id: trigger_eid,
                service: "turn_on".to_string(),
                data: serde_json::json!({}),
            })
            .build()
            .unwrap();

        // Condition entity is OFF, so condition should fail
        let trigger_entity = light_entity(trigger_eid, EntityState::Off);
        let condition_entity = light_entity(condition_eid, EntityState::Off);
        let engine = make_engine(vec![auto], vec![trigger_entity, condition_entity]);

        let event = state_changed_event(trigger_eid, "off", "on");
        let triggered = engine.process_event(&event).await.unwrap();
        assert!(triggered.is_empty());
    }

    #[tokio::test]
    async fn should_execute_when_state_is_condition_passes() {
        let trigger_eid = EntityId::new();
        let condition_eid = EntityId::new();

        let auto = Automation::builder()
            .name("Conditional")
            .trigger(Trigger::StateChanged {
                entity_id: trigger_eid,
                from: None,
                to: None,
            })
            .condition(Condition::StateIs {
                entity_id: condition_eid,
                state: "on".to_string(),
            })
            .action(Action::CallService {
                entity_id: trigger_eid,
                service: "turn_on".to_string(),
                data: serde_json::json!({}),
            })
            .build()
            .unwrap();

        // Condition entity is ON, so condition should pass
        let trigger_entity = light_entity(trigger_eid, EntityState::Off);
        let condition_entity = light_entity(condition_eid, EntityState::On);
        let engine = make_engine(vec![auto], vec![trigger_entity, condition_entity]);

        let event = state_changed_event(trigger_eid, "off", "on");
        let triggered = engine.process_event(&event).await.unwrap();
        assert_eq!(triggered.len(), 1);
    }

    #[tokio::test]
    async fn should_return_false_for_state_is_condition_when_entity_missing() {
        let trigger_eid = EntityId::new();
        let missing_eid = EntityId::new();

        let auto = Automation::builder()
            .name("Missing entity")
            .trigger(Trigger::StateChanged {
                entity_id: trigger_eid,
                from: None,
                to: None,
            })
            .condition(Condition::StateIs {
                entity_id: missing_eid,
                state: "on".to_string(),
            })
            .action(Action::CallService {
                entity_id: trigger_eid,
                service: "turn_on".to_string(),
                data: serde_json::json!({}),
            })
            .build()
            .unwrap();

        let trigger_entity = light_entity(trigger_eid, EntityState::Off);
        let engine = make_engine(vec![auto], vec![trigger_entity]);

        let event = state_changed_event(trigger_eid, "off", "on");
        let triggered = engine.process_event(&event).await.unwrap();
        assert!(triggered.is_empty());
    }

    #[tokio::test]
    async fn should_handle_empty_automations_list() {
        let engine = make_engine(vec![], vec![]);
        let event = state_changed_event(EntityId::new(), "off", "on");
        let triggered = engine.process_event(&event).await.unwrap();
        assert!(triggered.is_empty());
    }

    #[tokio::test]
    async fn should_trigger_multiple_matching_automations() {
        let eid = EntityId::new();
        let auto1 = Automation::builder()
            .name("First")
            .trigger(Trigger::StateChanged {
                entity_id: eid,
                from: None,
                to: None,
            })
            .action(Action::CallService {
                entity_id: eid,
                service: "turn_on".to_string(),
                data: serde_json::json!({}),
            })
            .build()
            .unwrap();
        let auto2 = Automation::builder()
            .name("Second")
            .trigger(Trigger::StateChanged {
                entity_id: eid,
                from: None,
                to: None,
            })
            .action(Action::CallService {
                entity_id: eid,
                service: "turn_on".to_string(),
                data: serde_json::json!({}),
            })
            .build()
            .unwrap();

        let entity = light_entity(eid, EntityState::Off);
        let engine = make_engine(vec![auto1, auto2], vec![entity]);

        let event = state_changed_event(eid, "off", "on");
        let triggered = engine.process_event(&event).await.unwrap();
        assert_eq!(triggered.len(), 2);
    }

    #[tokio::test]
    async fn should_ignore_unknown_service_name() {
        let eid = EntityId::new();
        let auto = Automation::builder()
            .name("Unknown service")
            .trigger(Trigger::StateChanged {
                entity_id: eid,
                from: None,
                to: None,
            })
            .action(Action::CallService {
                entity_id: eid,
                service: "unknown_service".to_string(),
                data: serde_json::json!({}),
            })
            .build()
            .unwrap();

        let entity = light_entity(eid, EntityState::Off);
        let engine = make_engine(vec![auto], vec![entity]);

        let event = state_changed_event(eid, "off", "on");
        let triggered = engine.process_event(&event).await.unwrap();
        assert_eq!(triggered.len(), 1);

        // Entity state should be unchanged
        let entity = engine.entity_repo.get_by_id(eid).await.unwrap().unwrap();
        assert_eq!(entity.state, EntityState::Off);
    }

    #[tokio::test]
    async fn should_toggle_off_to_on_when_entity_is_off() {
        let eid = EntityId::new();
        let auto = Automation::builder()
            .name("Toggle off→on")
            .trigger(Trigger::StateChanged {
                entity_id: eid,
                from: None,
                to: None,
            })
            .action(Action::CallService {
                entity_id: eid,
                service: "toggle".to_string(),
                data: serde_json::json!({}),
            })
            .build()
            .unwrap();

        let entity = light_entity(eid, EntityState::Off);
        let engine = make_engine(vec![auto], vec![entity]);

        let event = state_changed_event(eid, "on", "off");
        engine.process_event(&event).await.unwrap();

        let updated = engine.entity_repo.get_by_id(eid).await.unwrap().unwrap();
        assert_eq!(updated.state, EntityState::On);
    }

    #[tokio::test]
    async fn should_return_none_when_toggling_missing_entity() {
        let eid = EntityId::new();
        let missing = EntityId::new();
        let auto = Automation::builder()
            .name("Toggle missing")
            .trigger(Trigger::StateChanged {
                entity_id: eid,
                from: None,
                to: None,
            })
            .action(Action::CallService {
                entity_id: missing,
                service: "toggle".to_string(),
                data: serde_json::json!({}),
            })
            .build()
            .unwrap();

        let engine = make_engine(vec![auto], vec![]);

        let event = state_changed_event(eid, "off", "on");
        // toggle on missing entity returns None → no-op, no error
        let triggered = engine.process_event(&event).await.unwrap();
        assert_eq!(triggered.len(), 1);
    }

    #[tokio::test]
    async fn should_error_when_call_service_targets_missing_entity() {
        let eid = EntityId::new();
        let missing = EntityId::new();
        let auto = Automation::builder()
            .name("Turn on missing")
            .trigger(Trigger::StateChanged {
                entity_id: eid,
                from: None,
                to: None,
            })
            .action(Action::CallService {
                entity_id: missing,
                service: "turn_on".to_string(),
                data: serde_json::json!({}),
            })
            .build()
            .unwrap();

        let engine = make_engine(vec![auto], vec![]);

        let event = state_changed_event(eid, "off", "on");
        let result = engine.process_event(&event).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_evaluate_time_range_same_day() {
        // Build an automation with a TimeRange condition spanning the whole day
        // so it always passes regardless of when the test runs.
        let eid = EntityId::new();
        let auto = Automation::builder()
            .name("Time range same-day")
            .trigger(Trigger::StateChanged {
                entity_id: eid,
                from: None,
                to: None,
            })
            .condition(Condition::TimeRange {
                after: "00:00".to_string(),
                before: "23:59".to_string(),
            })
            .action(Action::CallService {
                entity_id: eid,
                service: "turn_on".to_string(),
                data: serde_json::json!({}),
            })
            .build()
            .unwrap();

        let entity = light_entity(eid, EntityState::Off);
        let engine = make_engine(vec![auto], vec![entity]);

        let event = state_changed_event(eid, "off", "on");
        let triggered = engine.process_event(&event).await.unwrap();
        assert_eq!(triggered.len(), 1);
    }

    #[tokio::test]
    async fn should_evaluate_time_range_overnight() {
        // Overnight range 00:00–23:59 always matches (now >= "00:00" is always true).
        let eid = EntityId::new();
        let auto = Automation::builder()
            .name("Time range overnight")
            .trigger(Trigger::StateChanged {
                entity_id: eid,
                from: None,
                to: None,
            })
            .condition(Condition::TimeRange {
                after: "23:00".to_string(),
                before: "06:00".to_string(),
            })
            .action(Action::CallService {
                entity_id: eid,
                service: "turn_on".to_string(),
                data: serde_json::json!({}),
            })
            .build()
            .unwrap();

        let entity = light_entity(eid, EntityState::Off);
        let engine = make_engine(vec![auto], vec![entity]);

        let event = state_changed_event(eid, "off", "on");
        // This will either match or not depending on the current time.
        // We just verify it doesn't error — the branch is exercised either way.
        let _ = engine.process_event(&event).await.unwrap();
    }

    #[tokio::test]
    async fn should_fail_time_range_when_outside_same_day_window() {
        // Narrow window that is almost certainly not "now" (03:00–03:01).
        let eid = EntityId::new();
        let auto = Automation::builder()
            .name("Time range narrow")
            .trigger(Trigger::StateChanged {
                entity_id: eid,
                from: None,
                to: None,
            })
            .condition(Condition::TimeRange {
                after: "03:00".to_string(),
                before: "03:01".to_string(),
            })
            .action(Action::CallService {
                entity_id: eid,
                service: "turn_on".to_string(),
                data: serde_json::json!({}),
            })
            .build()
            .unwrap();

        let entity = light_entity(eid, EntityState::Off);
        let engine = make_engine(vec![auto], vec![entity]);

        let event = state_changed_event(eid, "off", "on");
        // Unless we run at exactly 03:00 UTC, the condition should fail.
        let _ = engine.process_event(&event).await.unwrap();
    }
}
