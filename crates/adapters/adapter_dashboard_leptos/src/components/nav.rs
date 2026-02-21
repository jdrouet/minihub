use leptos::prelude::*;

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav>
            <ul>
                <li><a href="/">"Home"</a></li>
                <li><a href="/devices">"Devices"</a></li>
                <li><a href="/entities">"Entities"</a></li>
                <li><a href="/areas">"Areas"</a></li>
                <li><a href="/events">"Events"</a></li>
                <li><a href="/automations">"Automations"</a></li>
            </ul>
        </nav>
    }
}
