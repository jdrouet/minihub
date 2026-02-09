//! Dashboard page for the event log.

use askama::Template;
use axum::extract::State;
use axum::response::{Html, IntoResponse, Response};

use minihub_app::ports::{
    AreaRepository, AutomationRepository, DeviceRepository, EntityRepository, EventPublisher,
    EventStore,
};
use minihub_domain::event::Event;

use crate::state::AppState;

/// Event log page template.
#[derive(Template)]
#[template(path = "event_list.html")]
pub struct EventListTemplate {
    refresh_seconds: u32,
    events: Vec<Event>,
}

impl IntoResponse for EventListTemplate {
    fn into_response(self) -> Response {
        Html(self.to_string()).into_response()
    }
}

/// `GET /events` — list recent events.
pub async fn list<ER, DR, AR, EP, ES, AUR>(
    State(state): State<AppState<ER, DR, AR, EP, ES, AUR>>,
) -> EventListTemplate
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
{
    let events = state.event_store.get_recent(50).await.unwrap_or_default();

    EventListTemplate {
        refresh_seconds: 5,
        events,
    }
}
