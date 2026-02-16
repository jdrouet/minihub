//! Dashboard home page — overview of the system.

use askama::Template;
use axum::extract::State;
use axum::response::{Html, IntoResponse, Response};

use minihub_app::ports::{
    AreaRepository, AutomationRepository, DeviceRepository, EntityRepository, EventPublisher,
    EventStore,
};
use minihub_domain::entity::EntityState;

use super::DashboardError;
use crate::state::AppState;

/// Home page template.
#[derive(Template)]
#[template(path = "home.html")]
pub struct HomeTemplate {
    refresh_seconds: u32,
    entity_count: usize,
    on_count: usize,
    device_count: usize,
    area_count: usize,
}

impl IntoResponse for HomeTemplate {
    fn into_response(self) -> Response {
        Html(self.to_string()).into_response()
    }
}

/// `GET /` — system overview.
pub async fn index<ER, DR, AR, EP, ES, AUR>(
    State(state): State<AppState<ER, DR, AR, EP, ES, AUR>>,
) -> Result<HomeTemplate, DashboardError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
{
    let entities = state.entity_service.list_entities().await?;
    let devices = state.device_service.list_devices().await?;
    let areas = state.area_service.list_areas().await?;

    let on_count = entities
        .iter()
        .filter(|ent| ent.state == EntityState::On)
        .count();

    Ok(HomeTemplate {
        refresh_seconds: 10,
        entity_count: entities.len(),
        on_count,
        device_count: devices.len(),
        area_count: areas.len(),
    })
}
