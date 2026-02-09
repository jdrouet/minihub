//! JSON REST API handler modules.

#[allow(clippy::missing_errors_doc)]
pub mod areas;
#[allow(clippy::missing_errors_doc)]
pub mod devices;
#[allow(clippy::missing_errors_doc)]
pub mod entities;

use axum::Router;
use axum::routing::{get, put};

use minihub_app::ports::{AreaRepository, DeviceRepository, EntityRepository, EventPublisher};

use crate::state::AppState;

/// Build the `/api` sub-router.
pub fn routes<ER, DR, AR, EP>() -> Router<AppState<ER, DR, AR, EP>>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
{
    Router::new()
        .route(
            "/entities",
            get(entities::list::<ER, DR, AR, EP>).post(entities::create::<ER, DR, AR, EP>),
        )
        .route(
            "/entities/{id}",
            get(entities::get::<ER, DR, AR, EP>).delete(entities::delete::<ER, DR, AR, EP>),
        )
        .route(
            "/entities/{id}/state",
            put(entities::update_state::<ER, DR, AR, EP>),
        )
        .route(
            "/devices",
            get(devices::list::<ER, DR, AR, EP>).post(devices::create::<ER, DR, AR, EP>),
        )
        .route(
            "/devices/{id}",
            get(devices::get::<ER, DR, AR, EP>).delete(devices::delete::<ER, DR, AR, EP>),
        )
        .route(
            "/areas",
            get(areas::list::<ER, DR, AR, EP>).post(areas::create::<ER, DR, AR, EP>),
        )
        .route(
            "/areas/{id}",
            get(areas::get::<ER, DR, AR, EP>).delete(areas::delete::<ER, DR, AR, EP>),
        )
}
