//! In-process event bus backed by a tokio broadcast channel.

use tokio::sync::broadcast;

use minihub_domain::error::MiniHubError;
use minihub_domain::event::Event;

use crate::ports::EventPublisher;

/// In-process event bus using a tokio [`broadcast`] channel.
///
/// Publishing succeeds even when there are no active subscribers
/// (the event is simply dropped).
pub struct InProcessEventBus {
    sender: broadcast::Sender<Event>,
}

impl InProcessEventBus {
    /// Create a new event bus with the given channel capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Subscribe to events on this bus.
    ///
    /// Returns a receiver that will get all events published *after*
    /// the subscription is created.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }
}

impl EventPublisher for InProcessEventBus {
    fn publish(&self, event: Event) -> impl Future<Output = Result<(), MiniHubError>> + Send {
        // broadcast::send fails only when there are zero receivers,
        // which is fine — we simply ignore the error.
        let _ = self.sender.send(event);
        async { Ok(()) }
    }
}

use std::future::Future;

#[cfg(test)]
mod tests {
    use super::*;
    use minihub_domain::event::EventType;
    use minihub_domain::id::EntityId;

    #[tokio::test]
    async fn should_deliver_event_to_subscriber() {
        let bus = InProcessEventBus::new(16);
        let mut rx = bus.subscribe();

        let event = Event::new(
            EventType::StateChanged,
            Some(EntityId::new()),
            serde_json::json!({"from": "off", "to": "on"}),
        );
        let event_id = event.id;

        bus.publish(event).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.id, event_id);
    }

    #[tokio::test]
    async fn should_deliver_event_to_multiple_subscribers() {
        let bus = InProcessEventBus::new(16);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        let event = Event::new(EventType::EntityCreated, None, serde_json::json!({}));
        let event_id = event.id;

        bus.publish(event).await.unwrap();

        let r1 = rx1.recv().await.unwrap();
        let r2 = rx2.recv().await.unwrap();
        assert_eq!(r1.id, event_id);
        assert_eq!(r2.id, event_id);
    }

    #[tokio::test]
    async fn should_succeed_when_no_subscribers() {
        let bus = InProcessEventBus::new(16);
        let event = Event::new(EventType::StateChanged, None, serde_json::json!({}));
        let result = bus.publish(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_not_deliver_events_published_before_subscription() {
        let bus = InProcessEventBus::new(16);

        let event = Event::new(EventType::StateChanged, None, serde_json::json!({}));
        bus.publish(event).await.unwrap();

        let mut rx = bus.subscribe();

        let later = Event::new(EventType::EntityCreated, None, serde_json::json!({}));
        let later_id = later.id;
        bus.publish(later).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.id, later_id);
    }
}
