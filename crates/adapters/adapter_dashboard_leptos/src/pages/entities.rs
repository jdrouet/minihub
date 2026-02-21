use leptos::prelude::*;
use leptos_router::components::A;

use crate::api;
use minihub_domain::entity::EntityState;

/// Entities page displaying all entities in a table with state badges.
#[component]
pub fn Entities() -> impl IntoView {
    let entities = LocalResource::new(|| api::fetch_entities());

    view! {
        <div>
            <h1>"Entities"</h1>
            <Suspense fallback=move || view! { <p>"Loading entities…"</p> }>
                {move || {
                    entities.read().as_deref().map(|result| match result {
                        Ok(entities_list) => {
                            if entities_list.is_empty() {
                                view! {
                                    <p>"No entities found."</p>
                                }.into_any()
                            } else {
                                view! {
                                    <table>
                                        <thead>
                                            <tr>
                                                <th>"Entity ID"</th>
                                                <th>"Name"</th>
                                                <th>"State"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {entities_list.iter().map(|entity| {
                                                let entity_id = entity.id.to_string();
                                                let entity_id_str = entity.entity_id.clone();
                                                let friendly_name = entity.friendly_name.clone();
                                                let state_class = state_badge_class(&entity.state);
                                                let state_str = entity.state.to_string();
                                                view! {
                                                    <tr>
                                                        <td>
                                                            <A href=format!("/entities/{}", entity_id)>
                                                                {entity_id_str}
                                                            </A>
                                                        </td>
                                                        <td>{friendly_name}</td>
                                                        <td>
                                                            <span class=format!("badge {}", state_class)>
                                                                {state_str}
                                                            </span>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                }.into_any()
                            }
                        }
                        Err(err) => view! {
                            <p class="error">{"Failed to load entities: "} {err.to_string()}</p>
                        }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}

/// Return CSS class name for state badge based on entity state.
fn state_badge_class(state: &EntityState) -> &'static str {
    match state {
        EntityState::On => "badge-on",
        EntityState::Off => "badge-off",
        EntityState::Unknown => "badge-unknown",
        EntityState::Unavailable => "badge-unavailable",
    }
}
