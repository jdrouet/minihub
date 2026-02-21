use leptos::prelude::*;

use crate::api;
use crate::components::AreaTable;

/// Areas page displaying all areas in a table.
#[component]
pub fn Areas() -> impl IntoView {
    let areas = LocalResource::new(api::fetch_areas);

    view! {
        <div>
            <h1>"Areas"</h1>
            <Suspense fallback=move || view! { <p>"Loading areas…"</p> }>
                {move || {
                    areas.read().as_ref().map(|result| match result {
                        Ok(areas_list) => view! {
                            <AreaTable areas=areas_list.clone()/>
                        }.into_any(),
                        Err(err) => view! {
                            <p class="error">{"Failed to load areas: "} {err.to_string()}</p>
                        }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}
