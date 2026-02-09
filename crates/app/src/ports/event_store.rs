//! Event store port — persistence for domain events.

use std::future::Future;

use minihub_domain::error::MiniHubError;
use minihub_domain::event::Event;
use minihub_domain::id::{EntityId, EventId};

/// Repository for persisting and querying [`Event`]s.
pub trait EventStore {
    /// Persist a new event.
    fn store(&self, event: Event) -> impl Future<Output = Result<Event, MiniHubError>> + Send;

    /// Get an event by its unique identifier.
    fn get_by_id(
        &self,
        id: EventId,
    ) -> impl Future<Output = Result<Option<Event>, MiniHubError>> + Send;

    /// Get the most recent events, ordered newest-first.
    fn get_recent(
        &self,
        limit: usize,
    ) -> impl Future<Output = Result<Vec<Event>, MiniHubError>> + Send;

    /// Find events for a specific entity, ordered newest-first.
    fn find_by_entity(
        &self,
        entity_id: EntityId,
        limit: usize,
    ) -> impl Future<Output = Result<Vec<Event>, MiniHubError>> + Send;
}
