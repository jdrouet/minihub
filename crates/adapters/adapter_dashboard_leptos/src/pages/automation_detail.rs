use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::api;

/// Automation detail page showing trigger, conditions, and actions.
#[component]
pub fn AutomationDetail() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.read().get("id").unwrap_or_default();

    let automation = LocalResource::new(move || {
        let automation_id = id();
        async move { api::fetch_automation(&automation_id).await }
    });

    view! {
        <div>
            <h1>"Automation Detail"</h1>
            <Suspense fallback=move || view! { <p>"Loading automation…"</p> }>
                {move || {
                    automation.read().as_deref().map(|result| match result {
                        Ok(auto) => view! {
                            <div class="automation-detail">
                                <div class="detail-section">
                                    <h2>{auto.name.clone()}</h2>
                                    <p>
                                        <strong>"Status: "</strong>
                                        <span class=if auto.enabled { "status-enabled" } else { "status-disabled" }>
                                            {if auto.enabled { "Enabled" } else { "Disabled" }}
                                        </span>
                                    </p>
                                    {auto.last_triggered.as_ref().map(|ts| view! {
                                        <p><strong>"Last Triggered: "</strong> {ts.to_string()}</p>
                                    })}
                                </div>

                                <div class="detail-section">
                                    <h3>"Trigger"</h3>
                                    <code class="trigger-display">{format!("{:#?}", auto.trigger)}</code>
                                </div>

                                {if !auto.conditions.is_empty() {
                                    view! {
                                        <div class="detail-section">
                                            <h3>"Conditions"</h3>
                                            <ul class="conditions-list">
                                                {auto.conditions.iter().map(|cond| {
                                                    view! {
                                                        <li>
                                                            <code>{format!("{:#?}", cond)}</code>
                                                        </li>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </ul>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="detail-section">
                                            <h3>"Conditions"</h3>
                                            <p class="hint">"No conditions (always runs when triggered)"</p>
                                        </div>
                                    }.into_any()
                                }}

                                <div class="detail-section">
                                    <h3>"Actions"</h3>
                                    <ul class="actions-list">
                                        {auto.actions.iter().map(|action| {
                                            view! {
                                                <li>
                                                    <code>{format!("{:#?}", action)}</code>
                                                </li>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </ul>
                                </div>

                                <div class="detail-section">
                                    <A href="/automations">"← Back to Automations"</A>
                                </div>
                            </div>
                        }.into_any(),
                        Err(err) => view! {
                            <p class="error">{"Failed to load automation: "} {err.to_string()}</p>
                            <A href="/automations">"← Back to Automations"</A>
                        }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}
