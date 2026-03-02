//! JSON REST handlers for entities.

use std::str::FromStr;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use minihub_app::ports::{
    AreaRepository, AutomationRepository, DeviceRepository, EntityHistoryRepository,
    EntityRepository, EventPublisher, EventStore,
};
use minihub_domain::entity::{Entity, EntityState};
use minihub_domain::error::MiniHubError;
use minihub_domain::event::{Event, EventType};
use minihub_domain::id::{DeviceId, EntityId};

use crate::error::ApiError;
use crate::state::AppState;

/// Request body for creating an entity.
#[derive(Deserialize)]
pub struct CreateEntityRequest {
    pub device_id: String,
    pub entity_id: String,
    pub friendly_name: String,
}

/// Request body for updating entity state.
#[derive(Deserialize)]
pub struct UpdateStateRequest {
    pub state: EntityState,
}

/// Request body for calling a service on an entity.
#[derive(Deserialize)]
pub struct ServiceCallRequest {
    pub service: String,
    #[serde(default)]
    pub data: serde_json::Value,
}

/// Possible responses from the list endpoint.
pub enum ListResponse {
    Ok(Json<Vec<Entity>>),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(json) => json.into_response(),
        }
    }
}

/// Possible responses from the get endpoint.
pub enum GetResponse {
    Ok(Json<Entity>),
}

impl IntoResponse for GetResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(json) => json.into_response(),
        }
    }
}

/// Possible responses from the create endpoint.
pub enum CreateResponse {
    Created(Json<Entity>),
}

impl IntoResponse for CreateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Created(json) => (StatusCode::CREATED, json).into_response(),
        }
    }
}

/// Possible responses from the delete endpoint.
pub enum DeleteResponse {
    NoContent,
}

impl IntoResponse for DeleteResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NoContent => StatusCode::NO_CONTENT.into_response(),
        }
    }
}

/// Possible responses from the service call endpoint.
pub enum ServiceCallResponse {
    Accepted,
}

impl IntoResponse for ServiceCallResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Accepted => StatusCode::ACCEPTED.into_response(),
        }
    }
}

/// `GET /api/entities`
pub async fn list<ER, DR, AR, EP, ES, AUR, EHR>(
    State(state): State<AppState<ER, DR, AR, EP, ES, AUR, EHR>>,
) -> Result<ListResponse, ApiError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
    EHR: EntityHistoryRepository + Send + Sync + 'static,
{
    let entities = state.entity_service.list_entities().await?;
    Ok(ListResponse::Ok(Json(entities)))
}

/// `GET /api/entities/:id`
pub async fn get<ER, DR, AR, EP, ES, AUR, EHR>(
    State(state): State<AppState<ER, DR, AR, EP, ES, AUR, EHR>>,
    Path(id): Path<String>,
) -> Result<GetResponse, ApiError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
    EHR: EntityHistoryRepository + Send + Sync + 'static,
{
    let entity_id = EntityId::from_str(&id).map_err(|_| {
        ApiError::from(MiniHubError::Validation(
            minihub_domain::error::ValidationError::EmptyEntityId,
        ))
    })?;
    let entity = state.entity_service.get_entity(entity_id).await?;
    Ok(GetResponse::Ok(Json(entity)))
}

/// `POST /api/entities`
pub async fn create<ER, DR, AR, EP, ES, AUR, EHR>(
    State(state): State<AppState<ER, DR, AR, EP, ES, AUR, EHR>>,
    Json(req): Json<CreateEntityRequest>,
) -> Result<CreateResponse, ApiError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
    EHR: EntityHistoryRepository + Send + Sync + 'static,
{
    let device_id = DeviceId::from_str(&req.device_id).map_err(|_| {
        ApiError::from(MiniHubError::Validation(
            minihub_domain::error::ValidationError::EmptyEntityId,
        ))
    })?;

    let entity = Entity::builder()
        .device_id(device_id)
        .entity_id(req.entity_id)
        .friendly_name(req.friendly_name)
        .build()?;

    let created = state.entity_service.create_entity(entity).await?;
    Ok(CreateResponse::Created(Json(created)))
}

/// `PUT /api/entities/:id/state`
pub async fn update_state<ER, DR, AR, EP, ES, AUR, EHR>(
    State(state): State<AppState<ER, DR, AR, EP, ES, AUR, EHR>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateStateRequest>,
) -> Result<GetResponse, ApiError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
    EHR: EntityHistoryRepository + Send + Sync + 'static,
{
    let entity_id = EntityId::from_str(&id).map_err(|_| {
        ApiError::from(MiniHubError::Validation(
            minihub_domain::error::ValidationError::EmptyEntityId,
        ))
    })?;
    let updated = state
        .entity_service
        .update_entity_state(entity_id, req.state)
        .await?;
    Ok(GetResponse::Ok(Json(updated)))
}

/// `DELETE /api/entities/:id`
pub async fn delete<ER, DR, AR, EP, ES, AUR, EHR>(
    State(state): State<AppState<ER, DR, AR, EP, ES, AUR, EHR>>,
    Path(id): Path<String>,
) -> Result<DeleteResponse, ApiError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
    EHR: EntityHistoryRepository + Send + Sync + 'static,
{
    let entity_id = EntityId::from_str(&id).map_err(|_| {
        ApiError::from(MiniHubError::Validation(
            minihub_domain::error::ValidationError::EmptyEntityId,
        ))
    })?;
    state.entity_service.delete_entity(entity_id).await?;
    Ok(DeleteResponse::NoContent)
}

/// `POST /api/entities/:id/service`
pub async fn service_call<ER, DR, AR, EP, ES, AUR, EHR>(
    State(state): State<AppState<ER, DR, AR, EP, ES, AUR, EHR>>,
    Path(id): Path<String>,
    Json(req): Json<ServiceCallRequest>,
) -> Result<ServiceCallResponse, ApiError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
    EHR: EntityHistoryRepository + Send + Sync + 'static,
{
    let entity_id = EntityId::from_str(&id).map_err(|_| {
        ApiError::from(MiniHubError::Validation(
            minihub_domain::error::ValidationError::EmptyEntityId,
        ))
    })?;

    state.entity_service.get_entity(entity_id).await?;

    let event = Event::new(
        EventType::ServiceCallRequested,
        Some(entity_id),
        serde_json::json!({ "service": req.service, "data": req.data }),
    );
    state.event_bus.publish(event).await?;

    Ok(ServiceCallResponse::Accepted)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    use minihub_app::event_bus::InProcessEventBus;
    use minihub_app::ports::EventPublisher;
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

    use crate::state::AppState;

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
        async fn get_by_id(&self, id: EntityId) -> Result<Option<Entity>, MiniHubError> {
            Ok(Some(
                Entity::builder()
                    .device_id(DeviceId::new())
                    .entity_id("light.test")
                    .friendly_name("Test Light")
                    .id(id)
                    .build()
                    .unwrap(),
            ))
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

    struct NotFoundEntityRepo;

    impl minihub_app::ports::EntityRepository for NotFoundEntityRepo {
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

    impl minihub_app::ports::EventStore for StubEventStore {
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

    fn build_app_with_entity_repo<
        ER: minihub_app::ports::EntityRepository + Send + Sync + 'static,
    >(
        entity_repo: ER,
    ) -> axum::Router {
        let event_bus = Arc::new(InProcessEventBus::new(16));
        let state = AppState::new(
            EntityService::new(entity_repo, StubPublisher),
            DeviceService::new(StubDeviceRepo),
            AreaService::new(StubAreaRepo),
            StubEventStore,
            AutomationService::new(StubAutomationRepo),
            StubEntityHistoryRepo,
            event_bus,
        );
        crate::router::build(state, None)
    }

    #[tokio::test]
    async fn should_return_accepted_when_service_called_on_existing_entity() {
        let app = build_app_with_entity_repo(StubEntityRepo);
        let entity_id = EntityId::new();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/entities/{entity_id}/service"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "service": "blink", "data": { "times": 3 } })
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn should_return_accepted_when_data_field_is_omitted() {
        let app = build_app_with_entity_repo(StubEntityRepo);
        let entity_id = EntityId::new();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/entities/{entity_id}/service"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "service": "turn_on" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn should_publish_service_call_requested_event() {
        let event_bus = Arc::new(InProcessEventBus::new(16));
        let mut rx = event_bus.subscribe();

        let state = AppState::new(
            EntityService::new(StubEntityRepo, StubPublisher),
            DeviceService::new(StubDeviceRepo),
            AreaService::new(StubAreaRepo),
            StubEventStore,
            AutomationService::new(StubAutomationRepo),
            StubEntityHistoryRepo,
            Arc::clone(&event_bus),
        );
        let app = crate::router::build(state, None);
        let entity_id = EntityId::new();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/entities/{entity_id}/service"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "service": "blink", "data": { "times": 3 } })
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);

        let event = rx.try_recv().unwrap();
        assert_eq!(
            event.event_type,
            minihub_domain::event::EventType::ServiceCallRequested
        );
        assert_eq!(event.entity_id, Some(entity_id));
        assert_eq!(event.data["service"], "blink");
        assert_eq!(event.data["data"]["times"], 3);
    }

    #[tokio::test]
    async fn should_return_not_found_when_entity_does_not_exist() {
        let app = build_app_with_entity_repo(NotFoundEntityRepo);
        let entity_id = EntityId::new();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/entities/{entity_id}/service"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "service": "blink" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn should_return_bad_request_when_entity_id_is_invalid() {
        let app = build_app_with_entity_repo(StubEntityRepo);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/entities/not-a-uuid/service")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "service": "blink" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn should_reject_when_body_is_invalid_json() {
        let app = build_app_with_entity_repo(StubEntityRepo);
        let entity_id = EntityId::new();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/entities/{entity_id}/service"))
                    .header("content-type", "application/json")
                    .body(Body::from("not json"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status().is_client_error());
    }

    #[tokio::test]
    async fn should_reject_when_service_field_is_missing() {
        let app = build_app_with_entity_repo(StubEntityRepo);
        let entity_id = EntityId::new();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/entities/{entity_id}/service"))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::json!({ "data": {} }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status().is_client_error());
    }
}
