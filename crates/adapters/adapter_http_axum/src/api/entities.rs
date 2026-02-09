//! JSON REST handlers for entities.

use std::str::FromStr;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use minihub_app::ports::{
    AreaRepository, AutomationRepository, DeviceRepository, EntityRepository, EventPublisher,
    EventStore,
};
use minihub_domain::entity::{Entity, EntityState};
use minihub_domain::error::MiniHubError;
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

/// `GET /api/entities`
pub async fn list<ER, DR, AR, EP, ES, AUR>(
    State(state): State<AppState<ER, DR, AR, EP, ES, AUR>>,
) -> Result<ListResponse, ApiError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
{
    let entities = state.entity_service.list_entities().await?;
    Ok(ListResponse::Ok(Json(entities)))
}

/// `GET /api/entities/:id`
pub async fn get<ER, DR, AR, EP, ES, AUR>(
    State(state): State<AppState<ER, DR, AR, EP, ES, AUR>>,
    Path(id): Path<String>,
) -> Result<GetResponse, ApiError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
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
pub async fn create<ER, DR, AR, EP, ES, AUR>(
    State(state): State<AppState<ER, DR, AR, EP, ES, AUR>>,
    Json(req): Json<CreateEntityRequest>,
) -> Result<CreateResponse, ApiError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
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
pub async fn update_state<ER, DR, AR, EP, ES, AUR>(
    State(state): State<AppState<ER, DR, AR, EP, ES, AUR>>,
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
pub async fn delete<ER, DR, AR, EP, ES, AUR>(
    State(state): State<AppState<ER, DR, AR, EP, ES, AUR>>,
    Path(id): Path<String>,
) -> Result<DeleteResponse, ApiError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
{
    let entity_id = EntityId::from_str(&id).map_err(|_| {
        ApiError::from(MiniHubError::Validation(
            minihub_domain::error::ValidationError::EmptyEntityId,
        ))
    })?;
    state.entity_service.delete_entity(entity_id).await?;
    Ok(DeleteResponse::NoContent)
}
