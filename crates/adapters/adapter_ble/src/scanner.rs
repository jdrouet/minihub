//! BLE scanner — discovers BLE sensors and persists them in real time.
//!
//! [`BleScanner`] wraps BLE scanning, parsing, and real-time persistence.
//! Each advertisement is persisted via the [`IntegrationContext`] as soon as
//! it is received.

use std::time::Duration;

use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;
use tokio::task::JoinHandle;
use tokio_stream::StreamExt as _;

use minihub_app::ports::integration::{DiscoveredDevice, IntegrationContext};
use minihub_domain::device::Device;
use minihub_domain::entity::{AttributeValue, Entity, EntityState};
use minihub_domain::error::MiniHubError;
use minihub_domain::event::{Event, EventType};

use crate::error::BleError;
use crate::parser::{self, SensorReading, ServiceUuid};

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

/// Passive BLE scanner that discovers sensors and persists them in real time.
///
/// Each received advertisement is immediately persisted via the
/// [`IntegrationContext`] — there is no batching or post-scan persistence step.
pub struct BleScanner<C> {
    context: C,
    scan_duration: Duration,
    interval: Duration,
    device_filter: Vec<String>,
}

impl<C: IntegrationContext + Clone + 'static> BleScanner<C> {
    /// Create a new scanner with the given context and configuration.
    pub fn start(
        context: C,
        scan_duration: Duration,
        interval: Duration,
        device_filter: Vec<String>,
    ) -> JoinHandle<()> {
        let scanner = Self {
            context,
            scan_duration,
            interval,
            device_filter,
        };

        tokio::spawn(scanner.run())
    }

    /// Continuous scan loop — runs a scan, waits for the interval, repeats.
    async fn run(self) {
        loop {
            if let Err(err) = self.iterate().await {
                tracing::warn!(%err, "BLE background scan failed, retrying next interval");
            }
            tokio::time::sleep(self.interval).await;
        }
    }

    /// Check whether the given MAC address passes the device filter.
    fn passes_filter(&self, mac: &str) -> bool {
        if self.device_filter.is_empty() {
            return true;
        }
        self.device_filter
            .iter()
            .any(|f| f.eq_ignore_ascii_case(mac))
    }

    /// Run a single BLE scan for the given duration.
    ///
    /// Each advertisement is persisted immediately via the context — no
    /// results are collected and returned.
    ///
    /// # Errors
    ///
    /// Returns [`BleError`] when the BLE adapter is unavailable or the scan
    /// cannot be started.
    async fn iterate(&self) -> Result<(), BleError> {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        let central = adapters.into_iter().next().ok_or(BleError::NotAvailable)?;

        let mut events = central.events().await?;

        central
            .start_scan(ScanFilter {
                services: ServiceUuid::all(),
            })
            .await?;

        let deadline = tokio::time::Instant::now() + self.scan_duration;

        while tokio::time::Instant::now() < deadline {
            let remaining = deadline - tokio::time::Instant::now();
            match tokio::time::timeout(remaining, events.next()).await {
                Ok(Some(CentralEvent::ServiceDataAdvertisement { id, service_data })) => {
                    for (uuid, data) in &service_data {
                        let Ok(reading) = parser::parse_service_data(*uuid, data) else {
                            continue;
                        };

                        let mac_str = parser::format_mac(reading.mac);
                        if !self.passes_filter(&mac_str) {
                            tracing::debug!(mac = %mac_str, "filtered out by device_filter");
                            continue;
                        }

                        // Verify the peripheral exists before persisting
                        if central.peripheral(&id).await.is_err() {
                            continue;
                        }

                        let dd = build_discovered(&reading).map_err(BleError::Domain)?;

                        tracing::debug!(mac = %mac_str, "persisting BLE sensor reading");
                        if let Err(err) = self.context.persist_discovered(dd).await {
                            tracing::warn!(%err, mac = %mac_str, "failed to persist BLE discovery");
                        }
                    }
                }
                Ok(Some(CentralEvent::DeviceDiscovered(id))) => {
                    if let Ok(peripheral) = central.peripheral(&id).await {
                        if let Ok(Some(props)) = peripheral.properties().await {
                            let mac = props.address.to_string();
                            tracing::trace!(%mac, name = ?props.local_name, "BLE device detected");
                            let event = Event::new(
                                EventType::DeviceDetected,
                                None,
                                serde_json::json!({
                                    "integration": "ble",
                                    "mac": mac,
                                    "name": props.local_name,
                                    "rssi": props.rssi,
                                }),
                            );
                            if let Err(err) = self.context.publish(event).await {
                                tracing::warn!(%err, %mac, "failed to publish device_detected event");
                            }
                        }
                    }
                }
                Ok(Some(_)) => {}
                Ok(None) | Err(_) => break,
            }
        }

        central.stop_scan().await?;

        Ok(())
    }
}
