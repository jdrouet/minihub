//! Automation table component for displaying a list of automations.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use minihub_domain::automation::Automation;

use crate::api;

/// A table displaying a list of automations.
#[component]
pub fn AutomationTable(
    /// The list of automations to display.
    automations: Vec<Automation>,
    /// Callback when an automation is updated.
    #[prop(into)]
    on_update: Callback<()>,
) -> impl IntoView {
    if automations.is_empty() {
        view! {
            <p>"No automations found."</p>
        }
        .into_any()
    } else {
        view! {
            <table>
                <thead>
                    <tr>
                        <th>"Name"</th>
                        <th>"Enabled"</th>
                        <th>"Trigger"</th>
                        <th>"Actions"</th>
                    </tr>
                </thead>
                <tbody>
                    {automations.into_iter().map(|automation| {
                        view! {
                            <AutomationRow automation on_update/>
                        }
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>
        }
        .into_any()
    }
}

/// A single row in the automation table.
#[component]
fn AutomationRow(
    /// The automation to display.
    automation: Automation,
    /// Callback when the automation is updated.
    #[prop(into)]
    on_update: Callback<()>,
) -> impl IntoView {
    let automation_id = automation.id.to_string();
    let name = automation.name.clone();
    let enabled = automation.enabled;
    let trigger = format!("{:?}", automation.trigger);
    let actions_count = automation.actions.len();

    let (is_updating, set_is_updating) = signal(false);
    let (error_message, set_error_message) = signal::<Option<String>>(None);

    let toggle_enabled = move |_| {
        let mut updated_automation = automation.clone();
        updated_automation.enabled = !updated_automation.enabled;

        set_is_updating.set(true);
        set_error_message.set(None);

        spawn_local(async move {
            match api::update_automation(updated_automation).await {
                Ok(_) => {
                    set_is_updating.set(false);
                    on_update.run(());
                }
                Err(err) => {
                    set_is_updating.set(false);
                    set_error_message.set(Some(err.to_string()));
                }
            }
        });
    };

    view! {
        <tr>
            <td>
                <A href=format!("/automations/{}", automation_id)>
                    {name}
                </A>
            </td>
            <td>
                <button
                    class=move || if enabled { "btn-enabled" } else { "btn-disabled" }
                    on:click=toggle_enabled
                    disabled=move || is_updating.get()
                >
                    {move || if is_updating.get() {
                        "…".to_string()
                    } else if enabled {
                        "Enabled".to_string()
                    } else {
                        "Disabled".to_string()
                    }}
                </button>
                {move || error_message.get().map(|msg| view! {
                    <span class="error-inline">{msg}</span>
                })}
            </td>
            <td>{trigger}</td>
            <td>{format!("{} action(s)", actions_count)}</td>
        </tr>
    }
}
