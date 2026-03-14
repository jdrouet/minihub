//! Device detail page showing device information and its entities.

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;
use minihub_domain::id::DeviceId;

use crate::api::{self, ApiError};
use crate::components::{EntityTable, Loading};

/// Device detail page.
#[component]
pub fn DeviceDetail() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.read().get("id").unwrap_or_default();

    let device = LocalResource::new(move || {
        let device_id = id();
        async move { api::fetch_device(&device_id).await }
    });

    let entities: LocalResource<Result<Vec<minihub_domain::entity::Entity>, ApiError>> =
        LocalResource::new(move || {
            let device_id = id();
            async move {
                let all = api::fetch_entities().await?;
                let did: DeviceId = device_id.parse().map_err(|err: uuid::Error| ApiError {
                    message: err.to_string(),
                })?;
                Ok(all
                    .into_iter()
                    .filter(|e| e.device_id == did)
                    .collect::<Vec<_>>())
            }
        });

    view! {
        <div>
            <h1>"Device Detail"</h1>
            <Suspense fallback=move || view! { <Loading/> }>
                {move || {
                    device.read().as_ref().map(|result| match result {
                        Ok(dev) => {
                            let manufacturer = dev.manufacturer.clone().unwrap_or_else(|| "\u{2014}".to_string());
                            let model = dev.model.clone().unwrap_or_else(|| "\u{2014}".to_string());
                            let area = dev.area_id.as_ref().map_or("\u{2014}".to_string(), |a| a.to_string());

                            view! {
                                <div class="card">
                                    <h2>{dev.name.clone()}</h2>
                                    <p><strong>"Manufacturer: "</strong> {manufacturer}</p>
                                    <p><strong>"Model: "</strong> {model}</p>
                                    <p><strong>"Integration: "</strong> {dev.integration.clone()}</p>
                                    <p><strong>"Area: "</strong> {area}</p>
                                    <p><strong>"Unique ID: "</strong> {dev.unique_id.clone()}</p>
                                </div>
                            }.into_any()
                        }
                        Err(err) => view! {
                            <p class="error">{"Failed to load device: "} {err.to_string()}</p>
                        }.into_any(),
                    })
                }}
            </Suspense>

            <h2>"Entities"</h2>
            <Suspense fallback=move || view! { <Loading message="Loading entities\u{2026}"/> }>
                {move || {
                    entities.read().as_ref().map(|result: &Result<Vec<minihub_domain::entity::Entity>, _>| match result {
                        Ok(entity_list) => view! {
                            <EntityTable entities=entity_list.clone()/>
                        }.into_any(),
                        Err(err) => view! {
                            <p class="error">{"Failed to load entities: "} {err.to_string()}</p>
                        }.into_any(),
                    })
                }}
            </Suspense>

            <p><A href="/devices">"\u{2190} Back to Devices"</A></p>
        </div>
    }
}
