//! Server-side rendered HTML dashboard (no JavaScript).

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
pub mod home;

use axum::Router;
use axum::routing::{get, post};

use minihub_app::ports::{
    AreaRepository, AutomationRepository, DeviceRepository, EntityRepository, EventPublisher,
    EventStore,
};

use crate::state::AppState;

/// Build the dashboard sub-router for SSR HTML pages.
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
        .route("/", get(home::index::<ER, DR, AR, EP, ES, AUR>))
        .route("/entities", get(entities::list::<ER, DR, AR, EP, ES, AUR>))
        .route(
            "/entities/{id}",
            get(entities::detail::<ER, DR, AR, EP, ES, AUR>),
        )
        .route(
            "/entities/{id}/state",
            post(entities::update_state::<ER, DR, AR, EP, ES, AUR>),
        )
        .route("/devices", get(devices::list::<ER, DR, AR, EP, ES, AUR>))
        .route("/areas", get(areas::list::<ER, DR, AR, EP, ES, AUR>))
        .route("/events", get(events::list::<ER, DR, AR, EP, ES, AUR>))
        .route(
            "/automations",
            get(automations::list::<ER, DR, AR, EP, ES, AUR>),
        )
        .route(
            "/automations/{id}",
            get(automations::detail::<ER, DR, AR, EP, ES, AUR>),
        )
        .route(
            "/automations/{id}/toggle",
            post(automations::toggle_enabled::<ER, DR, AR, EP, ES, AUR>),
        )
}
