//! Automation repository port — persistence for automations.

use std::future::Future;

use minihub_domain::automation::Automation;
use minihub_domain::error::MiniHubError;
use minihub_domain::id::AutomationId;

/// Repository for persisting and querying [`Automation`]s.
pub trait AutomationRepository {
    /// Create a new automation in storage.
    fn create(
        &self,
        automation: Automation,
    ) -> impl Future<Output = Result<Automation, MiniHubError>> + Send;

    /// Get an automation by its unique identifier.
    fn get_by_id(
        &self,
        id: AutomationId,
    ) -> impl Future<Output = Result<Option<Automation>, MiniHubError>> + Send;

    /// Get all automations.
    fn get_all(&self) -> impl Future<Output = Result<Vec<Automation>, MiniHubError>> + Send;

    /// Get all enabled automations.
    fn get_enabled(&self) -> impl Future<Output = Result<Vec<Automation>, MiniHubError>> + Send;

    /// Update an existing automation.
    fn update(
        &self,
        automation: Automation,
    ) -> impl Future<Output = Result<Automation, MiniHubError>> + Send;

    /// Delete an automation by its unique identifier.
    fn delete(&self, id: AutomationId) -> impl Future<Output = Result<(), MiniHubError>> + Send;
}
