use leptos::prelude::*;
use leptos_router::components::A;

use super::ThemeToggle;

/// Top navigation bar with page links and theme toggle.
#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav>
            <ul>
                <li><A href="/">"Home"</A></li>
                <li><A href="/devices">"Devices"</A></li>
                <li><A href="/entities">"Entities"</A></li>
                <li><A href="/areas">"Areas"</A></li>
                <li><A href="/events">"Events"</A></li>
                <li><A href="/automations">"Automations"</A></li>
            </ul>
            <ThemeToggle/>
        </nav>
    }
}
