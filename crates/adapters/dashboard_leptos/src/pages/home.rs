use leptos::prelude::*;
use minihub_domain::entity::Entity;

use crate::api;
use crate::components::{Loading, SensorCardGrid, StatCard};

/// Dashboard data loaded on the home page.
#[derive(Debug, Clone)]
struct DashboardData {
    entity_count: usize,
    device_count: usize,
    area_count: usize,
    entities: Vec<Entity>,
}

/// Fetch counts and full entity list in one pass.
async fn fetch_dashboard_data() -> Result<DashboardData, crate::api::ApiError> {
    let entities = api::fetch_entities().await?;
    let devices = api::fetch_devices().await?;
    let areas = api::fetch_areas().await?;

    Ok(DashboardData {
        entity_count: entities.len(),
        device_count: devices.len(),
        area_count: areas.len(),
        entities,
    })
}

/// Home page displaying counts and BLE sensor cards.
#[component]
pub fn Home() -> impl IntoView {
    let data = LocalResource::new(fetch_dashboard_data);

    view! {
        <div>
            <h1>"Home"</h1>
            <Suspense fallback=move || view! { <Loading/> }>
                {move || {
                    data.read().as_ref().map(|result| match result {
                        Ok(dd) => view! {
                            <div class="stat-grid">
                                <StatCard label="Entities" value=dd.entity_count/>
                                <StatCard label="Devices" value=dd.device_count/>
                                <StatCard label="Areas" value=dd.area_count/>
                            </div>
                            <h2>"Sensors"</h2>
                            <SensorCardGrid entities=dd.entities.clone()/>
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
