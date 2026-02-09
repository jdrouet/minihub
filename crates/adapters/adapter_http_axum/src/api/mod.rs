//! JSON REST API handler modules.

#[allow(clippy::missing_errors_doc)]
pub mod areas;
#[allow(clippy::missing_errors_doc)]
pub mod automations;
#[allow(clippy::missing_errors_doc)]
pub mod devices;
#[allow(clippy::missing_errors_doc)]
pub mod entities;
#[allow(clippy::missing_errors_doc)]
pub mod events;

use axum::Router;
use axum::routing::{get, put};

use minihub_app::ports::{
    AreaRepository, AutomationRepository, DeviceRepository, EntityRepository, EventPublisher,
    EventStore,
};

use crate::state::AppState;

/// Build the `/api` sub-router.
pub fn routes<ER, DR, AR, EP, ES, AUR>() -> Router<AppState<ER, DR, AR, EP, ES, AUR>>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
{
    Router::new()
        // Entities
        .route(
            "/entities",
            get(entities::list::<ER, DR, AR, EP, ES, AUR>)
                .post(entities::create::<ER, DR, AR, EP, ES, AUR>),
        )
        .route(
            "/entities/{id}",
            get(entities::get::<ER, DR, AR, EP, ES, AUR>)
                .delete(entities::delete::<ER, DR, AR, EP, ES, AUR>),
        )
        .route(
            "/entities/{id}/state",
            put(entities::update_state::<ER, DR, AR, EP, ES, AUR>),
        )
        // Devices
        .route(
            "/devices",
            get(devices::list::<ER, DR, AR, EP, ES, AUR>)
                .post(devices::create::<ER, DR, AR, EP, ES, AUR>),
        )
        .route(
            "/devices/{id}",
            get(devices::get::<ER, DR, AR, EP, ES, AUR>)
                .delete(devices::delete::<ER, DR, AR, EP, ES, AUR>),
        )
        // Areas
        .route(
            "/areas",
            get(areas::list::<ER, DR, AR, EP, ES, AUR>)
                .post(areas::create::<ER, DR, AR, EP, ES, AUR>),
        )
        .route(
            "/areas/{id}",
            get(areas::get::<ER, DR, AR, EP, ES, AUR>)
                .delete(areas::delete::<ER, DR, AR, EP, ES, AUR>),
        )
        // Events
        .route("/events", get(events::list::<ER, DR, AR, EP, ES, AUR>))
        .route("/events/{id}", get(events::get::<ER, DR, AR, EP, ES, AUR>))
        // Automations
        .route(
            "/automations",
            get(automations::list::<ER, DR, AR, EP, ES, AUR>)
                .post(automations::create::<ER, DR, AR, EP, ES, AUR>),
        )
        .route(
            "/automations/{id}",
            get(automations::get::<ER, DR, AR, EP, ES, AUR>)
                .put(automations::update::<ER, DR, AR, EP, ES, AUR>)
                .delete(automations::delete::<ER, DR, AR, EP, ES, AUR>),
        )
}
