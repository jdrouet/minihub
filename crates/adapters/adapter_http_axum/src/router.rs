//! Axum router assembly.

use axum::Router;
use axum::routing::get;
use tower_http::trace::TraceLayer;

use minihub_app::ports::{
    AreaRepository, AutomationRepository, DeviceRepository, EntityRepository, EventPublisher,
    EventStore,
};

use crate::state::AppState;

/// Build the top-level axum [`Router`].
///
/// Merges API routes under `/api` and dashboard routes at `/`.
/// Includes a [`TraceLayer`] that logs each HTTP request/response at the
/// `DEBUG` level using the `tracing` ecosystem.
pub fn build<ER, DR, AR, EP, ES, AUR>(state: AppState<ER, DR, AR, EP, ES, AUR>) -> Router
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
{
    Router::new()
        .route("/health", get(health_check))
        .nest("/api", crate::api::routes())
        .merge(crate::dashboard::routes())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use minihub_app::services::area_service::AreaService;
    use minihub_app::services::automation_service::AutomationService;
    use minihub_app::services::device_service::DeviceService;
    use minihub_app::services::entity_service::EntityService;
    use minihub_domain::area::Area;
    use minihub_domain::automation::Automation;
    use minihub_domain::device::Device;
    use minihub_domain::entity::Entity;
    use minihub_domain::error::MiniHubError;
    use minihub_domain::event::Event;
    use minihub_domain::id::{AreaId, AutomationId, DeviceId, EntityId, EventId};
    use tower::ServiceExt;

    struct StubEntityRepo;
    struct StubDeviceRepo;
    struct StubAreaRepo;
    struct StubPublisher;
    struct StubEventStore;
    struct StubAutomationRepo;

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

    impl EventPublisher for StubPublisher {
        async fn publish(&self, _event: Event) -> Result<(), MiniHubError> {
            Ok(())
        }
    }

    impl EventStore for StubEventStore {
        async fn store(&self, event: Event) -> Result<Event, MiniHubError> {
            Ok(event)
        }
        async fn get_by_id(&self, _id: EventId) -> Result<Option<Event>, MiniHubError> {
            Ok(None)
        }
        async fn get_recent(&self, _limit: usize) -> Result<Vec<Event>, MiniHubError> {
            Ok(vec![])
        }
        async fn find_by_entity(
            &self,
            _entity_id: EntityId,
            _limit: usize,
        ) -> Result<Vec<Event>, MiniHubError> {
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

    fn test_state() -> AppState<
        StubEntityRepo,
        StubDeviceRepo,
        StubAreaRepo,
        StubPublisher,
        StubEventStore,
        StubAutomationRepo,
    > {
        AppState::new(
            EntityService::new(StubEntityRepo, StubPublisher),
            DeviceService::new(StubDeviceRepo),
            AreaService::new(StubAreaRepo),
            StubEventStore,
            AutomationService::new(StubAutomationRepo),
        )
    }

    #[tokio::test]
    async fn should_return_ok_when_health_check_called() {
        let app = build(test_state());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
