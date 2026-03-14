//! Device table component for displaying a list of devices.

use leptos::prelude::*;
use leptos_router::components::A;
use minihub_domain::device::Device;

/// A table displaying a list of devices.
#[component]
pub fn DeviceTable(
    /// The list of devices to display.
    devices: Vec<Device>,
) -> impl IntoView {
    if devices.is_empty() {
        view! {
            <p>"No devices found."</p>
        }
        .into_any()
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
                    {devices.into_iter().map(|device| {
                        view! {
                            <DeviceRow device/>
                        }
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>
        }
        .into_any()
    }
}

/// A single row in the device table.
#[component]
fn DeviceRow(
    /// The device to display.
    device: Device,
) -> impl IntoView {
    let device_id = device.id.to_string();
    let name = device.name;
    let manufacturer = device.manufacturer.unwrap_or_else(|| "—".to_string());
    let model = device.model.unwrap_or_else(|| "—".to_string());
    let integration = device.integration;

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
}
