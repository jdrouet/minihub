//! HTTP API client wrapping `gloo-net` for calls to `/api/*`.

use gloo_net::http::{Request, Response};
use minihub_domain::{
    area::Area, automation::Automation, device::Device, entity::Entity,
    entity_history::EntityHistory, event::Event,
};
use serde::Deserialize;

/// Error returned by API client methods.
#[derive(Debug, Clone)]
pub struct ApiError {
    pub message: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<gloo_net::Error> for ApiError {
    fn from(err: gloo_net::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

/// JSON error body returned by the server on non-2xx responses.
#[derive(Deserialize)]
struct ErrorBody {
    error: String,
}

/// Check the HTTP response status and extract an error if non-2xx.
async fn check_response(resp: Response) -> Result<Response, ApiError> {
    if resp.ok() {
        return Ok(resp);
    }
    let message = match resp.json::<ErrorBody>().await {
        Ok(body) => body.error,
        Err(_) => format!("HTTP {}", resp.status()),
    };
    Err(ApiError { message })
}

/// Percent-encode a query parameter value (handles `+`, `&`, `=`, spaces, etc.).
fn encode_query_value(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace('+', "%2B")
        .replace('&', "%26")
        .replace('=', "%3D")
        .replace(' ', "%20")
}

/// Summary counts for the home page dashboard.
#[derive(Debug, Clone)]
pub struct DashboardCounts {
    pub entities: usize,
    pub devices: usize,
    pub areas: usize,
}

/// Fetch all entities from the API.
pub async fn fetch_entities() -> Result<Vec<Entity>, ApiError> {
    let resp = check_response(Request::get("/api/entities").send().await?).await?;
    let entities: Vec<Entity> = resp.json().await?;
    Ok(entities)
}

/// Fetch all devices from the API.
pub async fn fetch_devices() -> Result<Vec<Device>, ApiError> {
    let resp = check_response(Request::get("/api/devices").send().await?).await?;
    let devices: Vec<Device> = resp.json().await?;
    Ok(devices)
}

/// Fetch all areas from the API.
pub async fn fetch_areas() -> Result<Vec<Area>, ApiError> {
    let resp = check_response(Request::get("/api/areas").send().await?).await?;
    let areas: Vec<Area> = resp.json().await?;
    Ok(areas)
}

/// Fetch entity, device, and area counts for the dashboard.
pub async fn fetch_dashboard_counts() -> Result<DashboardCounts, ApiError> {
    let entities = fetch_entities().await?.len();
    let devices = fetch_devices().await?.len();
    let areas = fetch_areas().await?.len();

    Ok(DashboardCounts {
        entities,
        devices,
        areas,
    })
}

/// Fetch a single entity by ID from the API.
pub async fn fetch_entity(id: &str) -> Result<Entity, ApiError> {
    let url = format!("/api/entities/{id}");
    let resp = check_response(Request::get(&url).send().await?).await?;
    let entity: Entity = resp.json().await?;
    Ok(entity)
}

/// Update entity state via PUT /api/entities/{id}/state.
pub async fn update_entity_state(
    id: &str,
    state: minihub_domain::entity::EntityState,
) -> Result<Entity, ApiError> {
    use serde::Serialize;

    #[derive(Serialize)]
    struct UpdateStateRequest {
        state: minihub_domain::entity::EntityState,
    }

    let url = format!("/api/entities/{id}/state");
    let resp = check_response(
        Request::put(&url)
            .json(&UpdateStateRequest { state })?
            .send()
            .await?,
    )
    .await?;
    let entity: Entity = resp.json().await?;
    Ok(entity)
}

/// Fetch all events from the API.
pub async fn fetch_events() -> Result<Vec<Event>, ApiError> {
    let resp = check_response(Request::get("/api/events").send().await?).await?;
    let events: Vec<Event> = resp.json().await?;
    Ok(events)
}

/// Fetch all automations from the API.
pub async fn fetch_automations() -> Result<Vec<Automation>, ApiError> {
    let resp = check_response(Request::get("/api/automations").send().await?).await?;
    let automations: Vec<Automation> = resp.json().await?;
    Ok(automations)
}

/// Fetch a single automation by ID from the API.
pub async fn fetch_automation(id: &str) -> Result<Automation, ApiError> {
    let url = format!("/api/automations/{id}");
    let resp = check_response(Request::get(&url).send().await?).await?;
    let automation: Automation = resp.json().await?;
    Ok(automation)
}

/// Fetch entity history for a given time range.
///
/// `from` and `to` are RFC 3339 timestamps. If omitted, the server defaults
/// to the last 24 hours with a limit of 1000 records.
pub async fn fetch_entity_history(
    id: &str,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<Vec<EntityHistory>, ApiError> {
    let mut url = format!("/api/entities/{id}/history");
    let mut params = Vec::new();
    if let Some(f) = from {
        params.push(format!("from={}", encode_query_value(f)));
    }
    if let Some(t) = to {
        params.push(format!("to={}", encode_query_value(t)));
    }
    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }
    let resp = check_response(Request::get(&url).send().await?).await?;
    let history: Vec<EntityHistory> = resp.json().await?;
    Ok(history)
}

/// Update automation (toggle enabled state).
pub async fn update_automation(automation: Automation) -> Result<Automation, ApiError> {
    use serde::Serialize;

    #[derive(Serialize)]
    struct UpdateAutomationRequest {
        name: String,
        enabled: bool,
        trigger: minihub_domain::automation::Trigger,
        conditions: Vec<minihub_domain::automation::Condition>,
        actions: Vec<minihub_domain::automation::Action>,
    }

    let url = format!("/api/automations/{}", automation.id);
    let resp = check_response(
        Request::put(&url)
            .json(&UpdateAutomationRequest {
                name: automation.name,
                enabled: automation.enabled,
                trigger: automation.trigger,
                conditions: automation.conditions,
                actions: automation.actions,
            })?
            .send()
            .await?,
    )
    .await?;
    let updated: Automation = resp.json().await?;
    Ok(updated)
}
