//! Reusable BLE scanning functions.
//!
//! Provides [`run_scan`] for a single bounded scan and
//! [`background_scan_loop`] for continuous periodic scanning.

use std::collections::HashMap;
use std::time::Duration;

use btleplug::api::{BDAddr, Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;
use tokio::sync::mpsc;
use tokio_stream::StreamExt as _;

use minihub_app::ports::integration::DiscoveredDevice;
use minihub_domain::device::Device;
use minihub_domain::entity::{AttributeValue, Entity, EntityState};
use minihub_domain::error::MiniHubError;

use crate::error::BleError;
use crate::parser::{self, SERVICE_UUID_181A, SensorReading};

/// Build a [`DiscoveredDevice`] from a [`SensorReading`].
pub(crate) fn build_discovered(reading: &SensorReading) -> Result<DiscoveredDevice, MiniHubError> {
    let mac_str = parser::format_mac(reading.mac);
    let slug = parser::mac_slug(reading.mac);

    let device = Device::builder()
        .name(format!("LYWSD03MMC {mac_str}"))
        .manufacturer("Xiaomi")
        .model("LYWSD03MMC")
        .integration("ble")
        .unique_id(&mac_str)
        .build()?;

    let entity = Entity::builder()
        .device_id(device.id)
        .entity_id(format!("sensor.ble_{slug}"))
        .friendly_name(format!("BLE Temp/Humidity {mac_str}"))
        .state(EntityState::On)
        .attribute("temperature", AttributeValue::Float(reading.temperature))
        .attribute("humidity", AttributeValue::Float(reading.humidity))
        .attribute(
            "battery_level",
            AttributeValue::Int(i64::from(reading.battery_level)),
        )
        .attribute(
            "battery_voltage",
            AttributeValue::Float(reading.battery_voltage),
        )
        .build()?;

    Ok(DiscoveredDevice {
        device,
        entities: vec![entity],
    })
}

/// Check whether the given MAC address passes the device filter.
fn passes_filter(device_filter: &[String], mac: &str) -> bool {
    if device_filter.is_empty() {
        return true;
    }
    device_filter.iter().any(|f| f.eq_ignore_ascii_case(mac))
}

/// Run a single BLE scan for the given duration.
///
/// Returns discovered devices keyed by BLE address, filtered by the given
/// MAC allowlist (empty = accept all).
///
/// # Errors
///
/// Returns [`BleError`] when the BLE adapter is unavailable or the scan
/// cannot be started.
pub async fn run_scan(
    duration: Duration,
    device_filter: &[String],
) -> Result<HashMap<BDAddr, DiscoveredDevice>, BleError> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().next().ok_or(BleError::NotAvailable)?;

    let mut events = central.events().await?;

    central
        .start_scan(ScanFilter {
            services: vec![SERVICE_UUID_181A],
        })
        .await?;

    let deadline = tokio::time::Instant::now() + duration;
    let mut discovered: HashMap<BDAddr, DiscoveredDevice> = HashMap::new();

    while tokio::time::Instant::now() < deadline {
        let remaining = deadline - tokio::time::Instant::now();
        match tokio::time::timeout(remaining, events.next()).await {
            Ok(Some(CentralEvent::ServiceDataAdvertisement { id, service_data })) => {
                for (uuid, data) in &service_data {
                    let Ok(reading) = parser::parse_service_data(*uuid, data) else {
                        continue;
                    };

                    let mac_str = parser::format_mac(reading.mac);
                    if !passes_filter(device_filter, &mac_str) {
                        tracing::debug!(mac = %mac_str, "filtered out by device_filter");
                        continue;
                    }

                    let addr = match central.peripheral(&id).await {
                        Ok(p) => p.address(),
                        Err(_) => continue,
                    };

                    let is_new = !discovered.contains_key(&addr);
                    let dd = build_discovered(&reading).map_err(BleError::Domain)?;
                    discovered.insert(addr, dd);

                    if is_new {
                        tracing::info!(mac = %mac_str, "discovered BLE sensor");
                    }
                }
            }
            Ok(Some(_)) => {}
            Ok(None) | Err(_) => break,
        }
    }

    central.stop_scan().await?;

    Ok(discovered)
}

/// Run continuous BLE scanning in the background.
///
/// Calls [`run_scan`] every `interval`, sending each discovered device
/// through `tx`. Exits when the channel is closed or the BLE adapter
/// becomes unavailable.
pub async fn background_scan_loop(
    scan_duration: Duration,
    interval: Duration,
    device_filter: Vec<String>,
    tx: mpsc::Sender<DiscoveredDevice>,
) {
    loop {
        tokio::time::sleep(interval).await;

        let results = match run_scan(scan_duration, &device_filter).await {
            Ok(r) => r,
            Err(err) => {
                tracing::warn!(%err, "BLE background scan failed, retrying next interval");
                continue;
            }
        };

        tracing::debug!(count = results.len(), "BLE background scan complete");

        for dd in results.into_values() {
            if tx.send(dd).await.is_err() {
                tracing::debug!("BLE discovery channel closed, stopping background loop");
                return;
            }
        }
    }
}
