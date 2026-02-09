//! JSON REST handlers for areas.

use std::str::FromStr;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use minihub_app::ports::{AreaRepository, DeviceRepository, EntityRepository};
use minihub_domain::area::Area;
use minihub_domain::error::MiniHubError;
use minihub_domain::id::AreaId;

use crate::error::ApiError;
use crate::state::AppState;

/// Request body for creating an area.
#[derive(Deserialize)]
pub struct CreateAreaRequest {
    pub name: String,
    pub parent_id: Option<String>,
}

/// Possible responses from the list endpoint.
pub enum ListResponse {
    Ok(Json<Vec<Area>>),
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
    Ok(Json<Area>),
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
    Created(Json<Area>),
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

/// `GET /api/areas`
pub async fn list<ER, DR, AR>(
    State(state): State<AppState<ER, DR, AR>>,
) -> Result<ListResponse, ApiError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
{
    let areas = state.area_service.list_areas().await?;
    Ok(ListResponse::Ok(Json(areas)))
}

/// `GET /api/areas/:id`
pub async fn get<ER, DR, AR>(
    State(state): State<AppState<ER, DR, AR>>,
    Path(id): Path<String>,
) -> Result<GetResponse, ApiError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
{
    let area_id = AreaId::from_str(&id).map_err(|_| {
        ApiError::from(MiniHubError::Validation(
            minihub_domain::error::ValidationError::EmptyName,
        ))
    })?;
    let area = state.area_service.get_area(area_id).await?;
    Ok(GetResponse::Ok(Json(area)))
}

/// `POST /api/areas`
pub async fn create<ER, DR, AR>(
    State(state): State<AppState<ER, DR, AR>>,
    Json(req): Json<CreateAreaRequest>,
) -> Result<CreateResponse, ApiError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
{
    let parent_id = req
        .parent_id
        .map(|s| AreaId::from_str(&s))
        .transpose()
        .map_err(|_| {
            ApiError::from(MiniHubError::Validation(
                minihub_domain::error::ValidationError::EmptyName,
            ))
        })?;

    let mut builder = Area::builder().name(req.name);
    if let Some(parent_id) = parent_id {
        builder = builder.parent_id(parent_id);
    }

    let area = builder.build()?;
    let created = state.area_service.create_area(area).await?;
    Ok(CreateResponse::Created(Json(created)))
}

/// `DELETE /api/areas/:id`
pub async fn delete<ER, DR, AR>(
    State(state): State<AppState<ER, DR, AR>>,
    Path(id): Path<String>,
) -> Result<DeleteResponse, ApiError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
{
    let area_id = AreaId::from_str(&id).map_err(|_| {
        ApiError::from(MiniHubError::Validation(
            minihub_domain::error::ValidationError::EmptyName,
        ))
    })?;
    state.area_service.delete_area(area_id).await?;
    Ok(DeleteResponse::NoContent)
}
