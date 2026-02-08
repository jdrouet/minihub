//! Event bus port — publish/subscribe traits for domain events.
//!
//! TODO(M2): Define trait `EventPublisher`:
//!   - `async fn publish(&self, event: Event) -> Result<()>`
//!
//! TODO(M2): Define trait `EventSubscriber`:
//!   - `async fn subscribe(&self, event_type: &str) -> Result<Receiver<Event>>`
