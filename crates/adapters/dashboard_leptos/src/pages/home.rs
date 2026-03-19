use leptos::prelude::*;
use leptos::task::spawn_local;
use minihub_domain::entity::Entity;
use minihub_domain::event::EventType;

use crate::api;
use crate::components::{Loading, SensorCardGrid, StatCard};
use crate::sse::use_sse_events;

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

/// Home page displaying counts and BLE sensor cards with live SSE updates.
#[component]
pub fn Home() -> impl IntoView {
    let (entities, set_entities) = signal(Vec::<Entity>::new());
    let (counts, set_counts) = signal(None::<(usize, usize, usize)>);
    let (error, set_error) = signal(None::<String>);
    let (loading, set_loading) = signal(true);

    Effect::new(move |_| {
        spawn_local(async move {
            set_loading.set(true);
            match fetch_dashboard_data().await {
                Ok(dd) => {
                    set_error.set(None);
                    set_counts.set(Some((dd.entity_count, dd.device_count, dd.area_count)));
                    set_entities.set(dd.entities);
                    set_loading.set(false);
                }
                Err(err) => {
                    set_error.set(Some(err.message));
                    set_loading.set(false);
                }
            }
        });
    });

    let sse_event = use_sse_events();

    Effect::new(move |_| {
        let Some(event) = sse_event.get() else {
            return;
        };

        match event.event_type {
            EventType::StateChanged | EventType::AttributeChanged => {
                spawn_local(async move {
                    if let Ok(new_entities) = api::fetch_entities().await {
                        set_error.set(None);
                        set_entities.set(new_entities);
                    }
                });
            }
            EventType::EntityCreated | EventType::EntityRemoved | EventType::DeviceDetected => {
                spawn_local(async move {
                    if let Ok(dd) = fetch_dashboard_data().await {
                        set_error.set(None);
                        set_counts.set(Some((dd.entity_count, dd.device_count, dd.area_count)));
                        set_entities.set(dd.entities);
                    }
                });
            }
            _ => {}
        }
    });

    view! {
        <div>
            <h1>"Home"</h1>
            {move || {
                if loading.get() {
                    view! { <Loading/> }.into_any()
                } else if let Some(err_msg) = error.get() {
                    view! {
                        <p class="error">{"Failed to load dashboard: "} {err_msg}</p>
                    }.into_any()
                } else {
                    let (ec, dc, ac) = counts.get().unwrap_or_default();
                    view! {
                        <div class="stat-grid">
                            <StatCard label="Entities" value=ec/>
                            <StatCard label="Devices" value=dc/>
                            <StatCard label="Areas" value=ac/>
                        </div>
                        <h2>"Sensors"</h2>
                        <SensorCardGrid entities/>
                    }.into_any()
                }
            }}
        </div>
    }
}
