use leptos::prelude::*;

use crate::api;
use crate::components::EventTable;

/// Events page displaying recent events.
#[component]
pub fn Events() -> impl IntoView {
    let events = LocalResource::new(api::fetch_events);

    view! {
        <div>
            <h1>"Events"</h1>
            <p class="hint">"Showing most recent events (up to 100)"</p>
            <Suspense fallback=move || view! { <p>"Loading events…"</p> }>
                {move || {
                    events.read().as_deref().map(|result| match result {
                        Ok(events_list) => view! {
                            <EventTable events=events_list.clone()/>
                        }.into_any(),
                        Err(err) => view! {
                            <p class="error">{"Failed to load events: "} {err.to_string()}</p>
                        }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}
