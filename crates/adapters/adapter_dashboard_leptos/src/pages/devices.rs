use leptos::prelude::*;
use leptos_router::components::A;

use crate::api;

/// Devices page displaying all devices in a table.
#[component]
pub fn Devices() -> impl IntoView {
    let devices = LocalResource::new(|| api::fetch_devices());

    view! {
        <div>
            <h1>"Devices"</h1>
            <Suspense fallback=move || view! { <p>"Loading devices…"</p> }>
                {move || {
                    devices.read().as_deref().map(|result| match result {
                        Ok(devices_list) => {
                            if devices_list.is_empty() {
                                view! {
                                    <p>"No devices found."</p>
                                }.into_any()
                            } else {
                                view! {
                                    <table>
                                        <thead>
                                            <tr>
                                                <th>"Name"</th>
                                                <th>"Manufacturer"</th>
                                                <th>"Model"</th>
                                                <th>"Integration"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {devices_list.iter().map(|device| {
                                                let device_id = device.id.to_string();
                                                let name = device.name.clone();
                                                let manufacturer = device.manufacturer.clone().unwrap_or_else(|| "—".to_string());
                                                let model = device.model.clone().unwrap_or_else(|| "—".to_string());
                                                let integration = device.integration.clone();
                                                view! {
                                                    <tr>
                                                        <td>
                                                            <A href=format!("/devices/{}", device_id)>
                                                                {name}
                                                            </A>
                                                        </td>
                                                        <td>{manufacturer}</td>
                                                        <td>{model}</td>
                                                        <td>{integration}</td>
                                                    </tr>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                }.into_any()
                            }
                        }
                        Err(err) => view! {
                            <p class="error">{"Failed to load devices: "} {err.to_string()}</p>
                        }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}
