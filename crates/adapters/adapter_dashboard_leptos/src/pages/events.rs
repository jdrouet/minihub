use leptos::prelude::*;
use leptos::task::spawn_local;
use minihub_domain::event::Event;

use crate::api;
use crate::components::EventTable;
use crate::sse::use_sse_events;

const MAX_EVENTS: usize = 100;

/// Events page displaying recent events with live SSE updates.
#[component]
pub fn Events() -> impl IntoView {
    let (events, set_events) = signal(None::<Result<Vec<Event>, String>>);

    // Initial fetch
    spawn_local(async move {
        match api::fetch_events().await {
            Ok(list) => set_events.set(Some(Ok(list))),
            Err(err) => set_events.set(Some(Err(err.to_string()))),
        }
    });

    // Subscribe to SSE and prepend new events
    let (sse_event, _sse_conn) = use_sse_events();

    Effect::new(move |_| {
        let Some(event) = sse_event.get() else {
            return;
        };

        set_events.update(|current| {
            if let Some(Ok(list)) = current {
                list.insert(0, event);
                list.truncate(MAX_EVENTS);
            }
        });
    });

    view! {
        <div>
            <h1>"Events"</h1>
            <p class="hint">"Showing most recent events (up to 100) — live updates enabled"</p>
            {move || {
                match events.get() {
                    None => view! { <p>"Loading events\u{2026}"</p> }.into_any(),
                    Some(Err(err)) => view! {
                        <p class="error">{"Failed to load events: "} {err}</p>
                    }.into_any(),
                    Some(Ok(events_list)) => view! {
                        <EventTable events=events_list/>
                    }.into_any(),
                }
            }}
        </div>
    }
}
