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
pub mod entity_history;
#[allow(clippy::missing_errors_doc)]
pub mod events;

use axum::Router;
use axum::routing::{get, put};

use minihub_app::ports::{
    AreaRepository, AutomationRepository, DeviceRepository, EntityHistoryRepository,
    EntityRepository, EventPublisher, EventStore,
};

use crate::state::AppState;

/// Build the `/api` sub-router.
pub fn routes<ER, DR, AR, EP, ES, AUR, EHR>() -> Router<AppState<ER, DR, AR, EP, ES, AUR, EHR>>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
    EHR: EntityHistoryRepository + Send + Sync + 'static,
{
    Router::new()
        // Entities
        .route(
            "/entities",
            get(entities::list::<ER, DR, AR, EP, ES, AUR, EHR>)
                .post(entities::create::<ER, DR, AR, EP, ES, AUR, EHR>),
        )
        .route(
            "/entities/{id}",
            get(entities::get::<ER, DR, AR, EP, ES, AUR, EHR>)
                .delete(entities::delete::<ER, DR, AR, EP, ES, AUR, EHR>),
        )
        .route(
            "/entities/{id}/state",
            put(entities::update_state::<ER, DR, AR, EP, ES, AUR, EHR>),
        )
        .route(
            "/entities/{id}/history",
            get(entity_history::list::<ER, DR, AR, EP, ES, AUR, EHR>),
        )
        // Devices
        .route(
            "/devices",
            get(devices::list::<ER, DR, AR, EP, ES, AUR, EHR>)
                .post(devices::create::<ER, DR, AR, EP, ES, AUR, EHR>),
        )
        .route(
            "/devices/{id}",
            get(devices::get::<ER, DR, AR, EP, ES, AUR, EHR>)
                .delete(devices::delete::<ER, DR, AR, EP, ES, AUR, EHR>),
        )
        // Areas
        .route(
            "/areas",
            get(areas::list::<ER, DR, AR, EP, ES, AUR, EHR>)
                .post(areas::create::<ER, DR, AR, EP, ES, AUR, EHR>),
        )
        .route(
            "/areas/{id}",
            get(areas::get::<ER, DR, AR, EP, ES, AUR, EHR>)
                .delete(areas::delete::<ER, DR, AR, EP, ES, AUR, EHR>),
        )
        // Events
        .route("/events", get(events::list::<ER, DR, AR, EP, ES, AUR, EHR>))
        .route(
            "/events/{id}",
            get(events::get::<ER, DR, AR, EP, ES, AUR, EHR>),
        )
        // Automations
        .route(
            "/automations",
            get(automations::list::<ER, DR, AR, EP, ES, AUR, EHR>)
                .post(automations::create::<ER, DR, AR, EP, ES, AUR, EHR>),
        )
        .route(
            "/automations/{id}",
            get(automations::get::<ER, DR, AR, EP, ES, AUR, EHR>)
                .put(automations::update::<ER, DR, AR, EP, ES, AUR, EHR>)
                .delete(automations::delete::<ER, DR, AR, EP, ES, AUR, EHR>),
        )
}
