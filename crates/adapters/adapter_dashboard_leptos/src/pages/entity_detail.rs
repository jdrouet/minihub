use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn EntityDetail() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.read().get("id").unwrap_or_default();

    view! {
        <div>
            <h1>"Entity Detail"</h1>
            <p>"Entity ID: " {id}</p>
        </div>
    }
}
