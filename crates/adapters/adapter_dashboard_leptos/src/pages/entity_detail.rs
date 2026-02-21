use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use minihub_domain::entity::{Entity, EntityState};

use crate::api::{fetch_entity, update_entity_state};

#[component]
pub fn EntityDetail() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.read().get("id").unwrap_or_default();

    let (entity, set_entity) = signal(None::<Entity>);
    let (error, set_error) = signal(None::<String>);
    let (loading, set_loading) = signal(true);
    let (updating, set_updating) = signal(false);

    // Fetch entity on mount
    Effect::new(move |_| {
        let entity_id = id();
        if entity_id.is_empty() {
            return;
        }

        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            match fetch_entity(&entity_id).await {
                Ok(e) => {
                    set_entity.set(Some(e));
                    set_loading.set(false);
                }
                Err(err) => {
                    set_error.set(Some(err.message));
                    set_loading.set(false);
                }
            }
        });
    });

    // Handler for updating entity state
    let handle_update_state = move |new_state: EntityState| {
        let entity_id = id();
        spawn_local(async move {
            set_updating.set(true);
            set_error.set(None);

            match update_entity_state(&entity_id, new_state).await {
                Ok(updated) => {
                    set_entity.set(Some(updated));
                    set_updating.set(false);
                }
                Err(err) => {
                    set_error.set(Some(err.message));
                    set_updating.set(false);
                }
            }
        });
    };

    let handle_turn_on = move |_| handle_update_state(EntityState::On);
    let handle_turn_off = move |_| handle_update_state(EntityState::Off);

    view! {
        <div>
            <h1>"Entity Detail"</h1>

            {move || {
                if loading.get() {
                    view! { <p>"Loading..."</p> }.into_any()
                } else if let Some(err_msg) = error.get() {
                    view! {
                        <div class="error">
                            <p>"Error: " {err_msg}</p>
                        </div>
                    }
                        .into_any()
                } else if let Some(e) = entity.get() {
                    let state_class = match e.state {
                        EntityState::On => "state-on",
                        EntityState::Off => "state-off",
                        EntityState::Unknown => "state-unknown",
                        EntityState::Unavailable => "state-unavailable",
                    };

                    view! {
                        <div class="entity-detail">
                            <div class="card">
                                <h2>{e.friendly_name.clone()}</h2>
                                <p>
                                    <strong>"Entity ID: "</strong> {e.entity_id.clone()}
                                </p>
                                <p>
                                    <strong>"Device ID: "</strong> {e.device_id.to_string()}
                                </p>
                                <p>
                                    <strong>"State: "</strong>
                                    <span class={format!("badge {}", state_class)}>
                                        {e.state.to_string()}
                                    </span>
                                </p>
                                <p>
                                    <strong>"Last Changed: "</strong>
                                    {e.last_changed.to_rfc3339()}
                                </p>
                                <p>
                                    <strong>"Last Updated: "</strong>
                                    {e.last_updated.to_rfc3339()}
                                </p>

                                {if !e.attributes.is_empty() {
                                    view! {
                                        <div>
                                            <h3>"Attributes"</h3>
                                            <ul class="attributes">
                                                {e
                                                    .attributes
                                                    .iter()
                                                    .map(|(key, value)| {
                                                        view! {
                                                            <li>
                                                                <strong>{key.clone()}</strong> ": "
                                                                {format!("{value:?}")}
                                                            </li>
                                                        }
                                                    })
                                                    .collect_view()}
                                            </ul>
                                        </div>
                                    }
                                        .into_any()
                                } else {
                                    view! { <p><em>"No attributes"</em></p> }.into_any()
                                }}

                                <div class="controls">
                                    <button
                                        on:click=handle_turn_on
                                        disabled=move || updating.get()
                                        class="btn btn-primary"
                                    >
                                        {move || if updating.get() { "Updating..." } else { "Turn On" }}
                                    </button>
                                    <button
                                        on:click=handle_turn_off
                                        disabled=move || updating.get()
                                        class="btn btn-secondary"
                                    >
                                        {move || if updating.get() { "Updating..." } else { "Turn Off" }}
                                    </button>
                                </div>
                            </div>
                        </div>
                    }
                        .into_any()
                } else {
                    view! { <p>"No entity found"</p> }.into_any()
                }
            }}
        </div>
    }
}
