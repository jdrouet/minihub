//! JSON REST handler for entity history.

use std::str::FromStr;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use chrono::Duration;
use serde::Deserialize;

use minihub_app::ports::{
    AreaRepository, AutomationRepository, DeviceRepository, EntityHistoryRepository,
    EntityRepository, EventPublisher, EventStore,
};
use minihub_domain::entity_history::EntityHistory;
use minihub_domain::error::MiniHubError;
use minihub_domain::id::EntityId;
use minihub_domain::time::{Timestamp, now};

use crate::error::ApiError;
use crate::state::AppState;

/// Default limit for history records.
const DEFAULT_LIMIT: usize = 1000;

/// Default time range: last 24 hours.
const DEFAULT_HOURS: i64 = 24;

/// Query parameters for the history endpoint.
#[derive(Deserialize)]
pub struct HistoryQuery {
    /// Start of time range (RFC 3339). Defaults to 24 hours ago.
    pub from: Option<String>,
    /// End of time range (RFC 3339). Defaults to now.
    pub to: Option<String>,
    /// Maximum number of records. Defaults to 1000.
    pub limit: Option<usize>,
}

/// Possible responses from the history list endpoint.
pub enum ListResponse {
    /// 200 OK with a JSON array of history records.
    Ok(Json<Vec<EntityHistory>>),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(json) => json.into_response(),
        }
    }
}

/// Parse an optional RFC 3339 timestamp string, returning a validation error on failure.
fn parse_timestamp(value: &str) -> Result<Timestamp, ApiError> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.to_utc())
        .map_err(|_| {
            ApiError::from(MiniHubError::Validation(
                minihub_domain::error::ValidationError::InvalidTimestamp(value.to_owned()),
            ))
        })
}

/// `GET /api/entities/:id/history?from=&to=&limit=`
pub async fn list<ER, DR, AR, EP, ES, AUR, EHR>(
    State(state): State<AppState<ER, DR, AR, EP, ES, AUR, EHR>>,
    Path(id): Path<String>,
    Query(params): Query<HistoryQuery>,
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
    let entity_id = EntityId::from_str(&id).map_err(|_| {
        ApiError::from(MiniHubError::Validation(
            minihub_domain::error::ValidationError::EmptyEntityId,
        ))
    })?;

    let current = now();
    let from = params
        .from
        .as_deref()
        .map(parse_timestamp)
        .transpose()?
        .unwrap_or_else(|| current - Duration::hours(DEFAULT_HOURS));
    let to = params
        .to
        .as_deref()
        .map(parse_timestamp)
        .transpose()?
        .unwrap_or(current);
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT);

    let records = state
        .entity_history_repo
        .find_by_entity_in_range(entity_id, from, to, Some(limit))
        .await?;

    Ok(ListResponse::Ok(Json(records)))
}
