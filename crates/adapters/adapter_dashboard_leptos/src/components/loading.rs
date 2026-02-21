//! Loading spinner component.

use leptos::prelude::*;

/// A loading indicator with an animated spinner and optional message.
#[component]
pub fn Loading(
    /// Text shown next to the spinner.
    #[prop(default = "Loading\u{2026}".into(), into)]
    message: String,
) -> impl IntoView {
    view! {
        <div class="loading">
            <span class="spinner"></span>
            <span>{message}</span>
        </div>
    }
}
