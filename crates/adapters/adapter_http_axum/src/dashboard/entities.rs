//! Dashboard pages for entities.

use std::str::FromStr;

use askama::Template;
use axum::extract::{Form, Path, State};
use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::Deserialize;

use minihub_app::ports::{AreaRepository, DeviceRepository, EntityRepository, EventPublisher};
use minihub_domain::entity::{Entity, EntityState};
use minihub_domain::id::EntityId;

use crate::error::ApiError;
use crate::state::AppState;

/// Entity list page template.
#[derive(Template)]
#[template(path = "entity_list.html")]
pub struct EntityListTemplate {
    refresh_seconds: u32,
    entities: Vec<Entity>,
}

impl IntoResponse for EntityListTemplate {
    fn into_response(self) -> Response {
        Html(self.to_string()).into_response()
    }
}

/// Entity detail page template.
#[derive(Template)]
#[template(path = "entity_detail.html")]
pub struct EntityDetailTemplate {
    refresh_seconds: u32,
    entity: Entity,
}

impl IntoResponse for EntityDetailTemplate {
    fn into_response(self) -> Response {
        Html(self.to_string()).into_response()
    }
}

/// Response from the state update form handler (PRG pattern).
pub enum UpdateStateResponse {
    /// Redirect back to entity detail page.
    Redirect(Redirect),
}

impl IntoResponse for UpdateStateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Redirect(redirect) => redirect.into_response(),
        }
    }
}

/// `GET /entities` — list all entities.
pub async fn list<ER, DR, AR, EP>(
    State(state): State<AppState<ER, DR, AR, EP>>,
) -> EntityListTemplate
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
{
    let entities = state
        .entity_service
        .list_entities()
        .await
        .unwrap_or_default();

    EntityListTemplate {
        refresh_seconds: 5,
        entities,
    }
}

/// `GET /entities/:id` — entity detail + control form.
pub async fn detail<ER, DR, AR, EP>(
    State(state): State<AppState<ER, DR, AR, EP>>,
    Path(id): Path<String>,
) -> Result<EntityDetailTemplate, ApiError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
{
    let entity_id = EntityId::from_str(&id).map_err(|_| {
        ApiError::from(minihub_domain::error::MiniHubError::Validation(
            minihub_domain::error::ValidationError::EmptyEntityId,
        ))
    })?;
    let entity = state.entity_service.get_entity(entity_id).await?;

    Ok(EntityDetailTemplate {
        refresh_seconds: 5,
        entity,
    })
}

/// Form data for state update.
#[derive(Deserialize)]
pub struct StateForm {
    pub state: EntityState,
}

/// `POST /entities/:id/state` — update entity state (PRG).
pub async fn update_state<ER, DR, AR, EP>(
    State(state): State<AppState<ER, DR, AR, EP>>,
    Path(id): Path<String>,
    Form(form): Form<StateForm>,
) -> Result<UpdateStateResponse, ApiError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
{
    let entity_id = EntityId::from_str(&id).map_err(|_| {
        ApiError::from(minihub_domain::error::MiniHubError::Validation(
            minihub_domain::error::ValidationError::EmptyEntityId,
        ))
    })?;
    state
        .entity_service
        .update_entity_state(entity_id, form.state)
        .await?;

    Ok(UpdateStateResponse::Redirect(Redirect::to(&format!(
        "/entities/{id}"
    ))))
}
