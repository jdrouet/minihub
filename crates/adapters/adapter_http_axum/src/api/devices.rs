//! JSON REST handlers for devices.

use std::str::FromStr;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use minihub_app::ports::{AreaRepository, DeviceRepository, EntityRepository};
use minihub_domain::device::Device;
use minihub_domain::error::MiniHubError;
use minihub_domain::id::{AreaId, DeviceId};

use crate::error::ApiError;
use crate::state::AppState;

/// Request body for creating a device.
#[derive(Deserialize)]
pub struct CreateDeviceRequest {
    pub name: String,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub area_id: Option<String>,
}

/// Possible responses from the list endpoint.
pub enum ListResponse {
    Ok(Json<Vec<Device>>),
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
    Ok(Json<Device>),
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
    Created(Json<Device>),
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

/// `GET /api/devices`
pub async fn list<ER, DR, AR>(
    State(state): State<AppState<ER, DR, AR>>,
) -> Result<ListResponse, ApiError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
{
    let devices = state.device_service.list_devices().await?;
    Ok(ListResponse::Ok(Json(devices)))
}

/// `GET /api/devices/:id`
pub async fn get<ER, DR, AR>(
    State(state): State<AppState<ER, DR, AR>>,
    Path(id): Path<String>,
) -> Result<GetResponse, ApiError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
{
    let device_id = DeviceId::from_str(&id).map_err(|_| {
        ApiError::from(MiniHubError::Validation(
            minihub_domain::error::ValidationError::EmptyName,
        ))
    })?;
    let device = state.device_service.get_device(device_id).await?;
    Ok(GetResponse::Ok(Json(device)))
}

/// `POST /api/devices`
pub async fn create<ER, DR, AR>(
    State(state): State<AppState<ER, DR, AR>>,
    Json(req): Json<CreateDeviceRequest>,
) -> Result<CreateResponse, ApiError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
{
    let area_id = req
        .area_id
        .map(|s| AreaId::from_str(&s))
        .transpose()
        .map_err(|_| {
            ApiError::from(MiniHubError::Validation(
                minihub_domain::error::ValidationError::EmptyName,
            ))
        })?;

    let mut builder = Device::builder().name(req.name);
    if let Some(manufacturer) = req.manufacturer {
        builder = builder.manufacturer(manufacturer);
    }
    if let Some(model) = req.model {
        builder = builder.model(model);
    }
    if let Some(area_id) = area_id {
        builder = builder.area_id(area_id);
    }

    let device = builder.build()?;
    let created = state.device_service.create_device(device).await?;
    Ok(CreateResponse::Created(Json(created)))
}

/// `DELETE /api/devices/:id`
pub async fn delete<ER, DR, AR>(
    State(state): State<AppState<ER, DR, AR>>,
    Path(id): Path<String>,
) -> Result<DeleteResponse, ApiError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
{
    let device_id = DeviceId::from_str(&id).map_err(|_| {
        ApiError::from(MiniHubError::Validation(
            minihub_domain::error::ValidationError::EmptyName,
        ))
    })?;
    state.device_service.delete_device(device_id).await?;
    Ok(DeleteResponse::NoContent)
}
