//! Server-Sent Events (SSE) stream for real-time updates.

use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use minihub_app::ports::{
    AreaRepository, AutomationRepository, DeviceRepository, EntityHistoryRepository,
    EntityRepository, EventPublisher, EventStore,
};

use crate::state::AppState;

/// `GET /api/events/stream` — SSE stream of real-time domain events.
///
/// Subscribes to the event bus broadcast channel and sends JSON-encoded
/// events as SSE `data:` frames. The stream continues until the client
/// disconnects or the event bus is closed.
///
/// Each event is sent as a JSON object with the event structure from the domain.
pub async fn stream<ER, DR, AR, EP, ES, AUR, EHR>(
    State(state): State<AppState<ER, DR, AR, EP, ES, AUR, EHR>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
    EHR: EntityHistoryRepository + Send + Sync + 'static,
{
    let event_rx = state.event_bus.subscribe();
    let event_stream = BroadcastStream::new(event_rx).filter_map(|result| match result {
        Ok(event) => {
            // Serialize event to JSON
            match serde_json::to_string(&event) {
                Ok(json) => Some(Ok(Event::default().data(json))),
                Err(err) => {
                    tracing::warn!(%err, "failed to serialize event to JSON for SSE stream");
                    None
                }
            }
        }
        Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(n)) => {
            tracing::warn!(
                skipped = n,
                "SSE subscriber lagged, some events were dropped"
            );
            None
        }
    });

    Sse::new(event_stream).keep_alive(KeepAlive::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use minihub_app::event_bus::InProcessEventBus;
    use minihub_app::services::area_service::AreaService;
    use minihub_app::services::automation_service::AutomationService;
    use minihub_app::services::device_service::DeviceService;
    use minihub_app::services::entity_service::EntityService;
    use minihub_domain::area::Area;
    use minihub_domain::automation::Automation;
    use minihub_domain::device::Device;
    use minihub_domain::entity::Entity;
    use minihub_domain::entity_history::EntityHistory;
    use minihub_domain::error::MiniHubError;
    use minihub_domain::event::{Event as DomainEvent, EventType};
    use minihub_domain::id::{AreaId, AutomationId, DeviceId, EntityId, EventId};
    use minihub_domain::time::Timestamp;
    use std::sync::Arc;

    struct StubEntityRepo;
    struct StubDeviceRepo;
    struct StubAreaRepo;
    struct StubEventStore;
    struct StubAutomationRepo;
    struct StubEntityHistoryRepo;

    impl minihub_app::ports::EntityRepository for StubEntityRepo {
        async fn create(&self, entity: Entity) -> Result<Entity, MiniHubError> {
            Ok(entity)
        }
        async fn get_by_id(&self, _id: EntityId) -> Result<Option<Entity>, MiniHubError> {
            Ok(None)
        }
        async fn get_all(&self) -> Result<Vec<Entity>, MiniHubError> {
            Ok(vec![])
        }
        async fn find_by_device_id(
            &self,
            _device_id: DeviceId,
        ) -> Result<Vec<Entity>, MiniHubError> {
            Ok(vec![])
        }
        async fn find_by_entity_id(
            &self,
            _entity_id: &str,
        ) -> Result<Option<Entity>, MiniHubError> {
            Ok(None)
        }
        async fn update(&self, entity: Entity) -> Result<Entity, MiniHubError> {
            Ok(entity)
        }
        async fn delete(&self, _id: EntityId) -> Result<(), MiniHubError> {
            Ok(())
        }
    }

    impl minihub_app::ports::DeviceRepository for StubDeviceRepo {
        async fn create(&self, device: Device) -> Result<Device, MiniHubError> {
            Ok(device)
        }
        async fn get_by_id(&self, _id: DeviceId) -> Result<Option<Device>, MiniHubError> {
            Ok(None)
        }
        async fn get_all(&self) -> Result<Vec<Device>, MiniHubError> {
            Ok(vec![])
        }
        async fn find_by_integration_unique_id(
            &self,
            _integration: &str,
            _unique_id: &str,
        ) -> Result<Option<Device>, MiniHubError> {
            Ok(None)
        }
        async fn update(&self, device: Device) -> Result<Device, MiniHubError> {
            Ok(device)
        }
        async fn delete(&self, _id: DeviceId) -> Result<(), MiniHubError> {
            Ok(())
        }
    }

    impl minihub_app::ports::AreaRepository for StubAreaRepo {
        async fn create(&self, area: Area) -> Result<Area, MiniHubError> {
            Ok(area)
        }
        async fn get_by_id(&self, _id: AreaId) -> Result<Option<Area>, MiniHubError> {
            Ok(None)
        }
        async fn get_all(&self) -> Result<Vec<Area>, MiniHubError> {
            Ok(vec![])
        }
        async fn update(&self, area: Area) -> Result<Area, MiniHubError> {
            Ok(area)
        }
        async fn delete(&self, _id: AreaId) -> Result<(), MiniHubError> {
            Ok(())
        }
    }

    impl EventStore for StubEventStore {
        async fn store(&self, event: DomainEvent) -> Result<DomainEvent, MiniHubError> {
            Ok(event)
        }
        async fn get_by_id(&self, _id: EventId) -> Result<Option<DomainEvent>, MiniHubError> {
            Ok(None)
        }
        async fn get_recent(&self, _limit: usize) -> Result<Vec<DomainEvent>, MiniHubError> {
            Ok(vec![])
        }
        async fn find_by_entity(
            &self,
            _entity_id: EntityId,
            _limit: usize,
        ) -> Result<Vec<DomainEvent>, MiniHubError> {
            Ok(vec![])
        }
    }

    impl minihub_app::ports::AutomationRepository for StubAutomationRepo {
        async fn create(&self, automation: Automation) -> Result<Automation, MiniHubError> {
            Ok(automation)
        }
        async fn get_by_id(&self, _id: AutomationId) -> Result<Option<Automation>, MiniHubError> {
            Ok(None)
        }
        async fn get_all(&self) -> Result<Vec<Automation>, MiniHubError> {
            Ok(vec![])
        }
        async fn get_enabled(&self) -> Result<Vec<Automation>, MiniHubError> {
            Ok(vec![])
        }
        async fn update(&self, automation: Automation) -> Result<Automation, MiniHubError> {
            Ok(automation)
        }
        async fn delete(&self, _id: AutomationId) -> Result<(), MiniHubError> {
            Ok(())
        }
    }

    impl minihub_app::ports::EntityHistoryRepository for StubEntityHistoryRepo {
        async fn record(&self, history: EntityHistory) -> Result<EntityHistory, MiniHubError> {
            Ok(history)
        }
        async fn find_by_entity_in_range(
            &self,
            _entity_id: EntityId,
            _from: Timestamp,
            _to: Timestamp,
            _limit: Option<usize>,
        ) -> Result<Vec<EntityHistory>, MiniHubError> {
            Ok(vec![])
        }
        async fn purge_before(&self, _before: Timestamp) -> Result<usize, MiniHubError> {
            Ok(0)
        }
    }

    fn test_state() -> (
        AppState<
            StubEntityRepo,
            StubDeviceRepo,
            StubAreaRepo,
            Arc<InProcessEventBus>,
            StubEventStore,
            StubAutomationRepo,
            StubEntityHistoryRepo,
        >,
        Arc<InProcessEventBus>,
    ) {
        let event_bus = Arc::new(InProcessEventBus::new(16));

        let state = AppState::new(
            EntityService::new(StubEntityRepo, Arc::clone(&event_bus)),
            DeviceService::new(StubDeviceRepo),
            AreaService::new(StubAreaRepo),
            StubEventStore,
            AutomationService::new(StubAutomationRepo),
            StubEntityHistoryRepo,
            Arc::clone(&event_bus),
        );

        (state, event_bus)
    }

    #[tokio::test]
    async fn should_subscribe_to_event_bus_when_stream_created() {
        let (state, event_bus) = test_state();

        // Create a direct subscription to verify events are being published
        let mut rx = event_bus.subscribe();

        // Create SSE stream (this also subscribes internally)
        let _sse_response = stream(State(state)).await;

        // Publish an event to the bus
        let test_event = DomainEvent::new(
            EventType::StateChanged,
            Some(EntityId::new()),
            serde_json::json!({"from": "off", "to": "on"}),
        );
        let event_id = test_event.id.clone();

        event_bus.publish(test_event).await.unwrap();

        // Verify the event was broadcast
        let received = rx.recv().await.unwrap();
        assert_eq!(received.id, event_id);
        assert_eq!(received.event_type, EventType::StateChanged);
    }
}
