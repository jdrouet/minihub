//! Event bus port — publish/subscribe for domain events.

use std::future::Future;

use minihub_domain::error::MiniHubError;
use minihub_domain::event::Event;

/// Publishes domain events to interested subscribers.
pub trait EventPublisher {
    /// Publish an event to all current subscribers.
    fn publish(&self, event: Event) -> impl Future<Output = Result<(), MiniHubError>> + Send;
}
