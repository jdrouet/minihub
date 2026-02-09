//! Dashboard page for devices.

use askama::Template;
use axum::extract::State;
use axum::response::{Html, IntoResponse, Response};

use minihub_app::ports::{
    AreaRepository, AutomationRepository, DeviceRepository, EntityRepository, EventPublisher,
    EventStore,
};
use minihub_domain::device::Device;

use crate::state::AppState;

/// Device list page template.
#[derive(Template)]
#[template(path = "device_list.html")]
pub struct DeviceListTemplate {
    refresh_seconds: u32,
    devices: Vec<Device>,
}

impl IntoResponse for DeviceListTemplate {
    fn into_response(self) -> Response {
        Html(self.to_string()).into_response()
    }
}

/// `GET /devices` — list all devices.
pub async fn list<ER, DR, AR, EP, ES, AUR>(
    State(state): State<AppState<ER, DR, AR, EP, ES, AUR>>,
) -> DeviceListTemplate
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
{
    let devices = state
        .device_service
        .list_devices()
        .await
        .unwrap_or_default();

    DeviceListTemplate {
        refresh_seconds: 10,
        devices,
    }
}
