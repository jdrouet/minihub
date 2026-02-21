//! Axum router assembly.

use std::path::Path;

use axum::Router;
use axum::routing::get;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use minihub_app::ports::{
    AreaRepository, AutomationRepository, DeviceRepository, EntityHistoryRepository,
    EntityRepository, EventPublisher, EventStore,
};

use crate::state::AppState;

/// Build the top-level axum [`Router`].
///
/// Mounts API routes under `/api` and a health-check at `/health`.
/// Includes a [`TraceLayer`] that logs each HTTP request/response at the
/// `DEBUG` level using the `tracing` ecosystem.
///
/// If `dashboard_dir` is provided, serves static files from that directory
/// at `/` with a fallback to `index.html` for client-side routing.
pub fn build<ER, DR, AR, EP, ES, AUR, EHR>(
    state: AppState<ER, DR, AR, EP, ES, AUR, EHR>,
    dashboard_dir: Option<&Path>,
) -> Router
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
    EHR: EntityHistoryRepository + Send + Sync + 'static,
{
    let router = Router::new()
        .route("/health", get(health_check))
        .nest("/api", crate::api::routes())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    if let Some(dir) = dashboard_dir {
        let index = dir.join("index.html");
        router.fallback_service(ServeDir::new(dir).fallback(ServeFile::new(index)))
    } else {
        router
    }
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
    use minihub_domain::entity_history::EntityHistory;
    use minihub_domain::error::MiniHubError;
    use minihub_domain::event::Event;
    use minihub_domain::id::{AreaId, AutomationId, DeviceId, EntityId, EventId};
    use minihub_domain::time::Timestamp;
    use tower::ServiceExt;

    struct StubEntityRepo;
    struct StubDeviceRepo;
    struct StubAreaRepo;
    struct StubPublisher;
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

    fn test_state() -> AppState<
        StubEntityRepo,
        StubDeviceRepo,
        StubAreaRepo,
        StubPublisher,
        StubEventStore,
        StubAutomationRepo,
        StubEntityHistoryRepo,
    > {
        AppState::new(
            EntityService::new(StubEntityRepo, StubPublisher),
            DeviceService::new(StubDeviceRepo),
            AreaService::new(StubAreaRepo),
            StubEventStore,
            AutomationService::new(StubAutomationRepo),
            StubEntityHistoryRepo,
        )
    }

    #[tokio::test]
    async fn should_return_ok_when_health_check_called() {
        let app = build(test_state(), None);

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

    #[tokio::test]
    async fn should_serve_index_html_from_dashboard_dir_at_root() {
        let temp_dir = std::env::temp_dir().join("minihub_test_dashboard_root");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let index_path = temp_dir.join("index.html");
        std::fs::write(&index_path, "<html><body>Dashboard</body></html>").unwrap();

        let app = build(test_state(), Some(&temp_dir));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("Dashboard"));

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn should_fallback_to_index_html_for_unknown_routes_when_dashboard_dir_set() {
        let temp_dir = std::env::temp_dir().join("minihub_test_dashboard_fallback");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let index_path = temp_dir.join("index.html");
        std::fs::write(
            &index_path,
            "<html><body>SPA Fallback for Client-Side Routing</body></html>",
        )
        .unwrap();

        let app = build(test_state(), Some(&temp_dir));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/devices/123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("SPA Fallback"));

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn should_serve_static_files_from_dashboard_dir() {
        let temp_dir = std::env::temp_dir().join("minihub_test_dashboard_static");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let index_path = temp_dir.join("index.html");
        std::fs::write(&index_path, "<html>Index</html>").unwrap();
        let css_path = temp_dir.join("style.css");
        std::fs::write(&css_path, "body { color: red; }").unwrap();

        let app = build(test_state(), Some(&temp_dir));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/style.css")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("color: red"));

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn should_prioritize_api_routes_over_static_fallback() {
        let temp_dir = std::env::temp_dir().join("minihub_test_dashboard_api_precedence");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let index_path = temp_dir.join("index.html");
        std::fs::write(&index_path, "<html>Static</html>").unwrap();

        let app = build(test_state(), Some(&temp_dir));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/entities")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok());
        assert!(
            content_type.is_some() && content_type.unwrap().contains("json"),
            "API route should return JSON, not static HTML"
        );

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn should_not_serve_static_files_when_dashboard_dir_is_none() {
        let app = build(test_state(), None);

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
