//! Server-side rendered HTML dashboard (no JavaScript).

#[allow(clippy::missing_errors_doc)]
pub mod areas;
#[allow(clippy::missing_errors_doc)]
pub mod devices;
#[allow(clippy::missing_errors_doc)]
pub mod entities;
pub mod home;

use axum::Router;
use axum::routing::{get, post};

use minihub_app::ports::{AreaRepository, DeviceRepository, EntityRepository};

use crate::state::AppState;

/// Build the dashboard sub-router for SSR HTML pages.
pub fn routes<ER, DR, AR>() -> Router<AppState<ER, DR, AR>>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
{
    Router::new()
        .route("/", get(home::index::<ER, DR, AR>))
        .route("/entities", get(entities::list::<ER, DR, AR>))
        .route("/entities/{id}", get(entities::detail::<ER, DR, AR>))
        .route(
            "/entities/{id}/state",
            post(entities::update_state::<ER, DR, AR>),
        )
        .route("/devices", get(devices::list::<ER, DR, AR>))
        .route("/areas", get(areas::list::<ER, DR, AR>))
}
