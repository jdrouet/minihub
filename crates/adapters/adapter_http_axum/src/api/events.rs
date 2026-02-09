//! JSON REST handlers for events.

use std::str::FromStr;

use axum::Json;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};

use minihub_app::ports::{
    AreaRepository, AutomationRepository, DeviceRepository, EntityRepository, EventPublisher,
    EventStore,
};
use minihub_domain::event::Event;
use minihub_domain::id::EventId;

use crate::error::ApiError;
use crate::state::AppState;

/// Possible responses from the list endpoint.
pub enum ListResponse {
    Ok(Json<Vec<Event>>),
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
    Ok(Json<Event>),
}

impl IntoResponse for GetResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(json) => json.into_response(),
        }
    }
}

/// `GET /api/events` — list recent events.
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
    let events = state.event_store.get_recent(100).await?;
    Ok(ListResponse::Ok(Json(events)))
}

/// `GET /api/events/:id` — get event by ID.
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
    let event_id = EventId::from_str(&id).map_err(|_| {
        ApiError::from(minihub_domain::error::MiniHubError::Validation(
            minihub_domain::error::ValidationError::EmptyName,
        ))
    })?;
    let event = state
        .event_store
        .get_by_id(event_id)
        .await?
        .ok_or_else(|| {
            ApiError::from(minihub_domain::error::MiniHubError::NotFound(
                minihub_domain::error::NotFoundError {
                    entity: "Event",
                    id,
                },
            ))
        })?;
    Ok(GetResponse::Ok(Json(event)))
}
