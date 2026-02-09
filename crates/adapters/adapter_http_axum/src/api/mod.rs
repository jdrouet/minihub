//! JSON REST API handler modules.

#[allow(clippy::missing_errors_doc)]
pub mod areas;
#[allow(clippy::missing_errors_doc)]
pub mod devices;
#[allow(clippy::missing_errors_doc)]
pub mod entities;

use axum::Router;
use axum::routing::{get, put};

use minihub_app::ports::{AreaRepository, DeviceRepository, EntityRepository};

use crate::state::AppState;

/// Build the `/api` sub-router.
pub fn routes<ER, DR, AR>() -> Router<AppState<ER, DR, AR>>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
{
    Router::new()
        .route(
            "/entities",
            get(entities::list::<ER, DR, AR>).post(entities::create::<ER, DR, AR>),
        )
        .route(
            "/entities/{id}",
            get(entities::get::<ER, DR, AR>).delete(entities::delete::<ER, DR, AR>),
        )
        .route(
            "/entities/{id}/state",
            put(entities::update_state::<ER, DR, AR>),
        )
        .route(
            "/devices",
            get(devices::list::<ER, DR, AR>).post(devices::create::<ER, DR, AR>),
        )
        .route(
            "/devices/{id}",
            get(devices::get::<ER, DR, AR>).delete(devices::delete::<ER, DR, AR>),
        )
        .route(
            "/areas",
            get(areas::list::<ER, DR, AR>).post(areas::create::<ER, DR, AR>),
        )
        .route(
            "/areas/{id}",
            get(areas::get::<ER, DR, AR>).delete(areas::delete::<ER, DR, AR>),
        )
}
