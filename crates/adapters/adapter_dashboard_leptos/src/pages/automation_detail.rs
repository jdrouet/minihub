use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn AutomationDetail() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.read().get("id").unwrap_or_default();

    view! {
        <div>
            <h1>"Automation Detail"</h1>
            <p>"Automation ID: " {id}</p>
        </div>
    }
}
