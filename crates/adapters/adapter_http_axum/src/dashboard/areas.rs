//! Dashboard page for areas.

use askama::Template;
use axum::extract::State;
use axum::response::{Html, IntoResponse, Response};

use minihub_app::ports::{AreaRepository, DeviceRepository, EntityRepository, EventPublisher};
use minihub_domain::area::Area;

use crate::state::AppState;

/// Area list page template.
#[derive(Template)]
#[template(path = "area_list.html")]
pub struct AreaListTemplate {
    refresh_seconds: u32,
    areas: Vec<Area>,
}

impl IntoResponse for AreaListTemplate {
    fn into_response(self) -> Response {
        Html(self.to_string()).into_response()
    }
}

/// `GET /areas` — list all areas.
pub async fn list<ER, DR, AR, EP>(State(state): State<AppState<ER, DR, AR, EP>>) -> AreaListTemplate
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
{
    let areas = state.area_service.list_areas().await.unwrap_or_default();

    AreaListTemplate {
        refresh_seconds: 10,
        areas,
    }
}
