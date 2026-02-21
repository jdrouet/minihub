use leptos::prelude::*;

use crate::api;
use crate::components::EntityTable;

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
                        Ok(entities_list) => view! {
                            <EntityTable entities=entities_list.clone()/>
                        }.into_any(),
                        Err(err) => view! {
                            <p class="error">{"Failed to load entities: "} {err.to_string()}</p>
                        }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}
