use leptos::prelude::*;

use crate::api;
use crate::components::{DeviceTable, Loading};

/// Devices page displaying all devices in a table.
#[component]
pub fn Devices() -> impl IntoView {
    let devices = LocalResource::new(api::fetch_devices);

    view! {
        <div>
            <h1>"Devices"</h1>
            <Suspense fallback=move || view! { <Loading message="Loading devices\u{2026}"/> }>
                {move || {
                    devices.read().as_ref().map(|result| match result {
                        Ok(devices_list) => view! {
                            <DeviceTable devices=devices_list.clone()/>
                        }.into_any(),
                        Err(err) => view! {
                            <p class="error">{"Failed to load devices: "} {err.to_string()}</p>
                        }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}
