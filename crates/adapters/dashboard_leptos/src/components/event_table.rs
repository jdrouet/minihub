//! Event table component for displaying a list of events.

use leptos::prelude::*;
use minihub_domain::event::Event;

/// A table displaying a list of events.
#[component]
pub fn EventTable(
    /// The list of events to display.
    events: Vec<Event>,
) -> impl IntoView {
    if events.is_empty() {
        view! {
            <p>"No events found."</p>
        }
        .into_any()
    } else {
        view! {
            <table>
                <thead>
                    <tr>
                        <th>"Timestamp"</th>
                        <th>"Event Type"</th>
                        <th>"Entity"</th>
                        <th>"Data"</th>
                    </tr>
                </thead>
                <tbody>
                    {events.iter().map(|event| {
                        view! {
                            <EventRow event=event.clone()/>
                        }
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>
        }
        .into_any()
    }
}

/// A single row in the event table.
#[component]
fn EventRow(
    /// The event to display.
    event: Event,
) -> impl IntoView {
    let timestamp = event.timestamp.to_string();
    let event_type = format!("{:?}", event.event_type);
    let entity = event.entity_id.map_or("—".to_string(), |e| e.to_string());
    let data = event.data.to_string();

    view! {
        <tr>
            <td>{timestamp}</td>
            <td>{event_type}</td>
            <td>{entity}</td>
            <td>
                <code class="json-data">{data}</code>
            </td>
        </tr>
    }
}
