//! Entity table component for displaying a list of entities.

use leptos::prelude::*;
use leptos_router::components::A;
use minihub_domain::entity::{Entity, EntityState};

/// A table displaying a list of entities.
#[component]
pub fn EntityTable(
    /// The list of entities to display.
    entities: Vec<Entity>,
) -> impl IntoView {
    if entities.is_empty() {
        view! {
            <p>"No entities found."</p>
        }
        .into_any()
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
                    {entities.into_iter().map(|entity| {
                        view! {
                            <EntityRow entity/>
                        }
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>
        }
        .into_any()
    }
}

/// A single row in the entity table.
#[component]
fn EntityRow(
    /// The entity to display.
    entity: Entity,
) -> impl IntoView {
    let entity_id = entity.id.to_string();
    let entity_id_str = entity.entity_id;
    let friendly_name = entity.friendly_name;
    let state = entity.state;

    view! {
        <tr>
            <td>
                <A href=format!("/entities/{}", entity_id)>
                    {entity_id_str}
                </A>
            </td>
            <td>{friendly_name}</td>
            <td>
                <StateBadge state/>
            </td>
        </tr>
    }
}

/// A badge displaying an entity state with appropriate styling.
#[component]
fn StateBadge(
    /// The entity state to display.
    state: EntityState,
) -> impl IntoView {
    let state_class = match state {
        EntityState::On => "badge-on",
        EntityState::Off => "badge-off",
        EntityState::Unknown => "badge-unknown",
        EntityState::Unavailable => "badge-unavailable",
    };
    let state_str = state.to_string();

    view! {
        <span class=format!("badge {}", state_class)>
            {state_str}
        </span>
    }
}
