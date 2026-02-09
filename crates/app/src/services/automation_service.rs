//! Automation service — use-cases for managing automations.

use minihub_domain::automation::Automation;
use minihub_domain::error::{MiniHubError, NotFoundError};
use minihub_domain::id::AutomationId;

use crate::ports::AutomationRepository;

/// Application service for automation CRUD operations.
pub struct AutomationService<R> {
    repo: R,
}

impl<R: AutomationRepository> AutomationService<R> {
    /// Create a new service backed by the given repository.
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Create a new automation after validating domain invariants.
    ///
    /// # Errors
    ///
    /// Returns [`MiniHubError::Validation`] if invariants fail, or a
    /// storage error propagated from the repository.
    #[tracing::instrument(skip(self, automation), fields(automation_name = %automation.name))]
    pub async fn create_automation(
        &self,
        automation: Automation,
    ) -> Result<Automation, MiniHubError> {
        automation.validate()?;
        self.repo.create(automation).await
    }

    /// Look up an automation by id, returning an error if not found.
    ///
    /// # Errors
    ///
    /// Returns [`MiniHubError::NotFound`] when no automation with `id` exists,
    /// or a storage error from the repository.
    #[tracing::instrument(skip(self))]
    pub async fn get_automation(&self, id: AutomationId) -> Result<Automation, MiniHubError> {
        self.repo.get_by_id(id).await?.ok_or_else(|| {
            NotFoundError {
                entity: "Automation",
                id: id.to_string(),
            }
            .into()
        })
    }

    /// List all automations.
    ///
    /// # Errors
    ///
    /// Returns a storage error propagated from the repository.
    pub async fn list_automations(&self) -> Result<Vec<Automation>, MiniHubError> {
        self.repo.get_all().await
    }

    /// Get all enabled automations.
    ///
    /// # Errors
    ///
    /// Returns a storage error propagated from the repository.
    pub async fn list_enabled(&self) -> Result<Vec<Automation>, MiniHubError> {
        self.repo.get_enabled().await
    }

    /// Update an existing automation.
    ///
    /// # Errors
    ///
    /// Returns [`MiniHubError::Validation`] if invariants fail, or a
    /// storage error from the repository.
    #[tracing::instrument(skip(self, automation))]
    pub async fn update_automation(
        &self,
        automation: Automation,
    ) -> Result<Automation, MiniHubError> {
        automation.validate()?;
        self.repo.update(automation).await
    }

    /// Delete an automation by id.
    ///
    /// # Errors
    ///
    /// Returns a storage error propagated from the repository.
    #[tracing::instrument(skip(self))]
    pub async fn delete_automation(&self, id: AutomationId) -> Result<(), MiniHubError> {
        self.repo.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use minihub_domain::automation::{Action, Trigger};
    use minihub_domain::error::ValidationError;
    use minihub_domain::id::EntityId;
    use std::collections::HashMap;
    use std::future::Future;
    use std::sync::Mutex;

    struct InMemoryAutomationRepo {
        store: Mutex<HashMap<AutomationId, Automation>>,
    }

    impl Default for InMemoryAutomationRepo {
        fn default() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
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
            let result = store.get(&id).cloned();
            async { Ok(result) }
        }

        fn get_all(&self) -> impl Future<Output = Result<Vec<Automation>, MiniHubError>> + Send {
            let store = self.store.lock().unwrap();
            let result: Vec<Automation> = store.values().cloned().collect();
            async { Ok(result) }
        }

        fn get_enabled(
            &self,
        ) -> impl Future<Output = Result<Vec<Automation>, MiniHubError>> + Send {
            let store = self.store.lock().unwrap();
            let result: Vec<Automation> = store.values().filter(|a| a.enabled).cloned().collect();
            async { Ok(result) }
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

    fn make_service() -> AutomationService<InMemoryAutomationRepo> {
        AutomationService::new(InMemoryAutomationRepo::default())
    }

    fn valid_automation() -> Automation {
        Automation::builder()
            .name("Test automation")
            .trigger(Trigger::Manual)
            .action(Action::CallService {
                entity_id: EntityId::new(),
                service: "turn_on".to_string(),
                data: serde_json::json!({}),
            })
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn should_create_automation_when_valid() {
        let svc = make_service();
        let auto = valid_automation();
        let id = auto.id;

        let created = svc.create_automation(auto).await.unwrap();
        assert_eq!(created.id, id);

        let fetched = svc.get_automation(id).await.unwrap();
        assert_eq!(fetched.name, "Test automation");
    }

    #[tokio::test]
    async fn should_reject_create_when_name_is_empty() {
        let svc = make_service();
        let mut auto = valid_automation();
        auto.name = String::new();

        let result = svc.create_automation(auto).await;
        assert!(matches!(
            result,
            Err(MiniHubError::Validation(ValidationError::EmptyName))
        ));
    }

    #[tokio::test]
    async fn should_return_not_found_when_automation_missing() {
        let svc = make_service();
        let result = svc.get_automation(AutomationId::new()).await;
        assert!(matches!(result, Err(MiniHubError::NotFound(_))));
    }

    #[tokio::test]
    async fn should_list_all_automations() {
        let svc = make_service();
        svc.create_automation(valid_automation()).await.unwrap();
        let mut auto2 = valid_automation();
        auto2.name = "Second".to_string();
        svc.create_automation(auto2).await.unwrap();

        let all = svc.list_automations().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn should_list_only_enabled_automations() {
        let svc = make_service();
        svc.create_automation(valid_automation()).await.unwrap();

        let mut disabled = valid_automation();
        disabled.name = "Disabled".to_string();
        disabled.enabled = false;
        svc.create_automation(disabled).await.unwrap();

        let enabled = svc.list_enabled().await.unwrap();
        assert_eq!(enabled.len(), 1);
        assert!(enabled[0].enabled);
    }

    #[tokio::test]
    async fn should_update_automation() {
        let svc = make_service();
        let auto = valid_automation();
        let id = auto.id;
        svc.create_automation(auto).await.unwrap();

        let mut updated = svc.get_automation(id).await.unwrap();
        updated.name = "Updated name".to_string();
        let saved = svc.update_automation(updated).await.unwrap();
        assert_eq!(saved.name, "Updated name");
    }

    #[tokio::test]
    async fn should_delete_automation() {
        let svc = make_service();
        let auto = valid_automation();
        let id = auto.id;
        svc.create_automation(auto).await.unwrap();

        svc.delete_automation(id).await.unwrap();

        let result = svc.get_automation(id).await;
        assert!(matches!(result, Err(MiniHubError::NotFound(_))));
    }
}
