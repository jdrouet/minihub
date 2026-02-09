//! JSON REST handlers for automations.

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
use minihub_domain::automation::{Action, Automation, Condition, Trigger};
use minihub_domain::error::MiniHubError;
use minihub_domain::id::AutomationId;

use crate::error::ApiError;
use crate::state::AppState;

/// Request body for creating an automation.
#[derive(Deserialize)]
pub struct CreateAutomationRequest {
    pub name: String,
    pub enabled: Option<bool>,
    pub trigger: Trigger,
    pub conditions: Option<Vec<Condition>>,
    pub actions: Vec<Action>,
}

/// Request body for updating an automation.
#[derive(Deserialize)]
pub struct UpdateAutomationRequest {
    pub name: String,
    pub enabled: bool,
    pub trigger: Trigger,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
}

/// Possible responses from the list endpoint.
pub enum ListResponse {
    Ok(Json<Vec<Automation>>),
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
    Ok(Json<Automation>),
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
    Created(Json<Automation>),
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

/// `GET /api/automations` — list all automations.
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
    let automations = state.automation_service.list_automations().await?;
    Ok(ListResponse::Ok(Json(automations)))
}

/// `GET /api/automations/:id` — get automation by ID.
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
    let automation_id = AutomationId::from_str(&id).map_err(|_| {
        ApiError::from(MiniHubError::Validation(
            minihub_domain::error::ValidationError::EmptyName,
        ))
    })?;
    let automation = state
        .automation_service
        .get_automation(automation_id)
        .await?;
    Ok(GetResponse::Ok(Json(automation)))
}

/// `POST /api/automations` — create a new automation.
pub async fn create<ER, DR, AR, EP, ES, AUR>(
    State(state): State<AppState<ER, DR, AR, EP, ES, AUR>>,
    Json(req): Json<CreateAutomationRequest>,
) -> Result<CreateResponse, ApiError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
{
    let mut builder = Automation::builder().name(req.name).trigger(req.trigger);

    if let Some(enabled) = req.enabled {
        builder = builder.enabled(enabled);
    }

    if let Some(conditions) = req.conditions {
        for c in conditions {
            builder = builder.condition(c);
        }
    }

    for a in req.actions {
        builder = builder.action(a);
    }

    let automation = builder.build()?;
    let created = state
        .automation_service
        .create_automation(automation)
        .await?;
    Ok(CreateResponse::Created(Json(created)))
}

/// `PUT /api/automations/:id` — update an existing automation.
pub async fn update<ER, DR, AR, EP, ES, AUR>(
    State(state): State<AppState<ER, DR, AR, EP, ES, AUR>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateAutomationRequest>,
) -> Result<GetResponse, ApiError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
{
    let automation_id = AutomationId::from_str(&id).map_err(|_| {
        ApiError::from(MiniHubError::Validation(
            minihub_domain::error::ValidationError::EmptyName,
        ))
    })?;

    // Verify it exists
    state
        .automation_service
        .get_automation(automation_id)
        .await?;

    let mut builder = Automation::builder()
        .id(automation_id)
        .name(req.name)
        .enabled(req.enabled)
        .trigger(req.trigger);

    for c in req.conditions {
        builder = builder.condition(c);
    }
    for a in req.actions {
        builder = builder.action(a);
    }

    let automation = builder.build()?;
    let updated = state
        .automation_service
        .update_automation(automation)
        .await?;
    Ok(GetResponse::Ok(Json(updated)))
}

/// `DELETE /api/automations/:id` — delete an automation.
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
    let automation_id = AutomationId::from_str(&id).map_err(|_| {
        ApiError::from(MiniHubError::Validation(
            minihub_domain::error::ValidationError::EmptyName,
        ))
    })?;
    state
        .automation_service
        .delete_automation(automation_id)
        .await?;
    Ok(DeleteResponse::NoContent)
}
