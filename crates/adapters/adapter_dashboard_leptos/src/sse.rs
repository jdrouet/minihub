//! SSE client module for subscribing to `/api/events/stream`.
//!
//! Provides a reactive hook that connects to the server-sent events endpoint
//! and delivers parsed domain events to Leptos signals.

use leptos::prelude::*;
use minihub_domain::event::Event;
use wasm_bindgen::prelude::*;
use web_sys::{EventSource, MessageEvent};

/// Guard that closes the `EventSource` connection on drop (if connected).
pub struct SseConnection {
    source: Option<EventSource>,
    _on_message: Option<Closure<dyn FnMut(MessageEvent)>>,
    _on_error: Option<Closure<dyn FnMut(web_sys::Event)>>,
}

impl Drop for SseConnection {
    fn drop(&mut self) {
        if let Some(source) = &self.source {
            source.close();
        }
    }
}

/// Subscribe to the SSE event stream at `/api/events/stream`.
///
/// Returns a read signal that yields each incoming [`Event`] as it arrives,
/// plus a guard that keeps the connection alive. Drop the guard to disconnect.
///
/// If the `EventSource` cannot be created (e.g. the endpoint is unreachable),
/// the signal will always be `None` and the connection guard is inert.
pub fn use_sse_events() -> (ReadSignal<Option<Event>>, SseConnection) {
    let (event_sig, set_event) = signal(None::<Event>);

    let source = match EventSource::new("/api/events/stream") {
        Ok(s) => s,
        Err(err) => {
            leptos::logging::warn!("failed to create EventSource: {err:?}");
            return (
                event_sig,
                SseConnection {
                    source: None,
                    _on_message: None,
                    _on_error: None,
                },
            );
        }
    };

    let on_message = Closure::<dyn FnMut(MessageEvent)>::new(move |msg: MessageEvent| {
        if let Some(data) = msg.data().as_string() {
            match serde_json::from_str::<Event>(&data) {
                Ok(event) => set_event.set(Some(event)),
                Err(err) => {
                    leptos::logging::warn!("failed to parse SSE event: {err}");
                }
            }
        }
    });

    let on_error = Closure::<dyn FnMut(web_sys::Event)>::new(move |_: web_sys::Event| {
        leptos::logging::warn!("SSE connection error — browser will auto-reconnect");
    });

    source
        .add_event_listener_with_callback("message", on_message.as_ref().unchecked_ref())
        .expect("failed to add message listener to EventSource");
    source
        .add_event_listener_with_callback("error", on_error.as_ref().unchecked_ref())
        .expect("failed to add error listener to EventSource");

    let conn = SseConnection {
        source: Some(source),
        _on_message: Some(on_message),
        _on_error: Some(on_error),
    };

    (event_sig, conn)
}
