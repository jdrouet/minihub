//! Dark/light theme toggle button using `localStorage` for persistence.

use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// Key used to persist the theme preference in `localStorage`.
const STORAGE_KEY: &str = "minihub-theme";

/// Read the stored theme from `localStorage`, defaulting to `"light"`.
fn stored_theme() -> String {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(STORAGE_KEY).ok().flatten())
        .unwrap_or_else(|| "light".to_string())
}

/// Apply the theme by setting the `data-theme` attribute on `<html>`.
fn apply_theme(theme: &str) {
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        if let Some(el) = doc.document_element() {
            let html = el.unchecked_into::<web_sys::HtmlElement>();
            let _ = html.dataset().set("theme", theme);
        }
    }
}

/// Persist the theme choice to `localStorage`.
fn save_theme(theme: &str) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item(STORAGE_KEY, theme);
    }
}

/// A toggle button that switches between light and dark themes.
///
/// Reads the initial theme from `localStorage` and applies it on mount.
/// Each click toggles and persists the new preference.
#[component]
pub fn ThemeToggle() -> impl IntoView {
    let initial = stored_theme();
    apply_theme(&initial);

    let (is_dark, set_is_dark) = signal(initial == "dark");

    let toggle = move |_| {
        let new_dark = !is_dark.get_untracked();
        let theme = if new_dark { "dark" } else { "light" };
        apply_theme(theme);
        save_theme(theme);
        set_is_dark.set(new_dark);
    };

    let label = move || {
        if is_dark.get() {
            "\u{263E}"
        } else {
            "\u{2600}"
        }
    };

    view! {
        <button class="theme-toggle" on:click=toggle title="Toggle theme">
            {label}
        </button>
    }
}
