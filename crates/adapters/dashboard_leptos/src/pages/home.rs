use leptos::prelude::*;

use crate::api::{self, DashboardCounts};
use crate::components::{Loading, StatCard};

/// Home page displaying entity, device, and area counts.
#[component]
pub fn Home() -> impl IntoView {
    let counts = LocalResource::new(api::fetch_dashboard_counts);

    view! {
        <div>
            <h1>"Home"</h1>
            <Suspense fallback=move || view! { <Loading/> }>
                {move || {
                    counts.read().as_ref().map(|result| match result {
                        Ok(DashboardCounts { entities, devices, areas }) => view! {
                            <div class="stat-grid">
                                <StatCard label="Entities" value=*entities/>
                                <StatCard label="Devices" value=*devices/>
                                <StatCard label="Areas" value=*areas/>
                            </div>
                        }.into_any(),
                        Err(err) => view! {
                            <p class="error">{"Failed to load dashboard: "} {err.to_string()}</p>
                        }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}
