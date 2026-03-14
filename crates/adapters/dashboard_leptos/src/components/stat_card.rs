//! Stat card component for displaying a labelled numeric value.

use leptos::prelude::*;

/// A card displaying a label and a numeric value.
#[component]
pub fn StatCard(
    /// The label shown above the value.
    #[prop(into)]
    label: String,
    /// The numeric value to display.
    value: usize,
) -> impl IntoView {
    view! {
        <div class="stat-card">
            <span class="stat-label">{label}</span>
            <span class="stat-value">{value}</span>
        </div>
    }
}
