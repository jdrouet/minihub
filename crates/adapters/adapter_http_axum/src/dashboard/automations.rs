//! Dashboard pages for automations.

use std::str::FromStr;

use askama::Template;
use axum::extract::{Form, Path, State};
use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::Deserialize;

use minihub_app::ports::{
    AreaRepository, AutomationRepository, DeviceRepository, EntityRepository, EventPublisher,
    EventStore,
};
use minihub_domain::automation::Automation;
use minihub_domain::id::AutomationId;

use super::DashboardError;
use crate::state::AppState;

/// Automation list page template.
#[derive(Template)]
#[template(path = "automation_list.html")]
pub struct AutomationListTemplate {
    refresh_seconds: u32,
    automations: Vec<Automation>,
}

impl IntoResponse for AutomationListTemplate {
    fn into_response(self) -> Response {
        Html(self.to_string()).into_response()
    }
}

/// Automation detail page template.
#[derive(Template)]
#[template(path = "automation_detail.html")]
pub struct AutomationDetailTemplate {
    refresh_seconds: u32,
    automation: Automation,
}

impl IntoResponse for AutomationDetailTemplate {
    fn into_response(self) -> Response {
        Html(self.to_string()).into_response()
    }
}

/// Response from the toggle-enabled form handler (PRG pattern).
pub enum ToggleEnabledResponse {
    Redirect(Redirect),
}

impl IntoResponse for ToggleEnabledResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Redirect(redirect) => redirect.into_response(),
        }
    }
}

/// `GET /automations` — list all automations.
pub async fn list<ER, DR, AR, EP, ES, AUR>(
    State(state): State<AppState<ER, DR, AR, EP, ES, AUR>>,
) -> Result<AutomationListTemplate, DashboardError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
{
    let automations = state.automation_service.list_automations().await?;

    Ok(AutomationListTemplate {
        refresh_seconds: 10,
        automations,
    })
}

/// `GET /automations/:id` — automation detail page.
pub async fn detail<ER, DR, AR, EP, ES, AUR>(
    State(state): State<AppState<ER, DR, AR, EP, ES, AUR>>,
    Path(id): Path<String>,
) -> Result<AutomationDetailTemplate, DashboardError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
{
    let automation_id = AutomationId::from_str(&id).map_err(|_| {
        DashboardError::from(minihub_domain::error::MiniHubError::Validation(
            minihub_domain::error::ValidationError::EmptyName,
        ))
    })?;
    let automation = state
        .automation_service
        .get_automation(automation_id)
        .await?;

    Ok(AutomationDetailTemplate {
        refresh_seconds: 10,
        automation,
    })
}

/// Form data for toggling enabled status.
#[derive(Deserialize)]
pub struct ToggleForm {
    pub enabled: String,
}

/// `POST /automations/:id/toggle` — enable/disable an automation (PRG).
pub async fn toggle_enabled<ER, DR, AR, EP, ES, AUR>(
    State(state): State<AppState<ER, DR, AR, EP, ES, AUR>>,
    Path(id): Path<String>,
    Form(form): Form<ToggleForm>,
) -> Result<ToggleEnabledResponse, DashboardError>
where
    ER: EntityRepository + Send + Sync + 'static,
    DR: DeviceRepository + Send + Sync + 'static,
    AR: AreaRepository + Send + Sync + 'static,
    EP: EventPublisher + Send + Sync + 'static,
    ES: EventStore + Send + Sync + 'static,
    AUR: AutomationRepository + Send + Sync + 'static,
{
    let automation_id = AutomationId::from_str(&id).map_err(|_| {
        DashboardError::from(minihub_domain::error::MiniHubError::Validation(
            minihub_domain::error::ValidationError::EmptyName,
        ))
    })?;
    let mut automation = state
        .automation_service
        .get_automation(automation_id)
        .await?;

    automation.enabled = form.enabled == "true";
    state
        .automation_service
        .update_automation(automation)
        .await?;

    Ok(ToggleEnabledResponse::Redirect(Redirect::to(&format!(
        "/automations/{id}"
    ))))
}
