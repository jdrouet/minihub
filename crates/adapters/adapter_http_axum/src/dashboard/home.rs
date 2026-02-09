//! Dashboard home page — overview of the system.

use askama::Template;
use axum::extract::State;
use axum::response::{Html, IntoResponse, Response};

use minihub_app::ports::{AreaRepository, DeviceRepository, EntityRepository};
use minihub_domain::entity::EntityState;

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
pub async fn index<ER, DR, AR>(State(state): State<AppState<ER, DR, AR>>) -> HomeTemplate
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
{
    let entities = state
        .entity_service
        .list_entities()
        .await
        .unwrap_or_default();
    let devices = state
        .device_service
        .list_devices()
        .await
        .unwrap_or_default();
    let areas = state.area_service.list_areas().await.unwrap_or_default();

    let on_count = entities
        .iter()
        .filter(|ent| ent.state == EntityState::On)
        .count();

    HomeTemplate {
        refresh_seconds: 10,
        entity_count: entities.len(),
        on_count,
        device_count: devices.len(),
        area_count: areas.len(),
    }
}
