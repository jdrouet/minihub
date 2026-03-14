use leptos::prelude::*;

/// 404 page displayed when no route matches.
#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <div class="not-found">
            <h1>"404 - Page Not Found"</h1>
            <p>"The page you are looking for does not exist."</p>
            <p>
                <a href="/">"Go back to home"</a>
            </p>
        </div>
    }
}
