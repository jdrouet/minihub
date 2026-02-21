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
use crate::{gatt, miflora};

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

/// The local name advertised by Mi Flora peripherals.
const MIFLORA_LOCAL_NAME: &str = "Flower care";

/// BLE scanner that discovers sensors via passive advertisements and,
/// optionally, reads Mi Flora plant sensors via active GATT connections.
///
/// Each received advertisement is immediately persisted via the
/// [`IntegrationContext`] — there is no batching or post-scan persistence step.
/// After each passive scan, if Mi Flora is enabled, the scanner connects to
/// discovered Mi Flora peripherals to read sensor data.
pub struct BleScanner<C> {
    context: C,
    scan_duration: Duration,
    interval: Duration,
    device_filter: Vec<String>,
    miflora_enabled: bool,
    miflora_filter: Vec<String>,
    miflora_connect_timeout: Duration,
}

impl<C: IntegrationContext + Clone + 'static> BleScanner<C> {
    /// Create a new scanner with the given context and configuration.
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        context: C,
        scan_duration: Duration,
        interval: Duration,
        device_filter: Vec<String>,
        miflora_enabled: bool,
        miflora_filter: Vec<String>,
        miflora_connect_timeout: Duration,
    ) -> JoinHandle<()> {
        let scanner = Self {
            context,
            scan_duration,
            interval,
            device_filter,
            miflora_enabled,
            miflora_filter,
            miflora_connect_timeout,
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

    /// Check whether the given MAC address passes the Mi Flora filter.
    fn passes_miflora_filter(&self, mac: &str) -> bool {
        if self.miflora_filter.is_empty() {
            return true;
        }
        self.miflora_filter
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

        if self.miflora_enabled {
            self.read_miflora_devices(&central).await;
        }

        Ok(())
    }

    /// Connect to each discovered Mi Flora peripheral and read sensor data.
    ///
    /// Iterates all peripherals known to the central adapter, identifies
    /// Mi Flora devices by their local name (`"Flower care"`), applies the
    /// MAC allowlist filter, then performs a GATT readout with a per-device
    /// timeout. Failures on individual devices are logged and skipped.
    async fn read_miflora_devices(&self, central: &btleplug::platform::Adapter) {
        let peripherals = match central.peripherals().await {
            Ok(list) => list,
            Err(err) => {
                tracing::warn!(%err, "failed to list peripherals for Mi Flora readout");
                return;
            }
        };

        for peripheral in &peripherals {
            let Ok(Some(props)) = peripheral.properties().await else {
                continue;
            };

            let name_matches = props
                .local_name
                .as_deref()
                .is_some_and(|name| name == MIFLORA_LOCAL_NAME);
            if !name_matches {
                continue;
            }

            let mac = props.address.to_string();
            if !self.passes_miflora_filter(&mac) {
                tracing::debug!(%mac, "Mi Flora filtered out by miflora_filter");
                continue;
            }

            tracing::debug!(%mac, "reading Mi Flora sensor via GATT");

            let result =
                tokio::time::timeout(self.miflora_connect_timeout, gatt::read_miflora(peripheral))
                    .await;

            let reading = match result {
                Ok(Ok(reading)) => reading,
                Ok(Err(err)) => {
                    tracing::warn!(%err, %mac, "failed to read Mi Flora device");
                    continue;
                }
                Err(_) => {
                    tracing::warn!(%mac, "Mi Flora GATT readout timed out");
                    continue;
                }
            };

            match miflora::build_discovered(&reading) {
                Ok(dd) => {
                    if let Err(err) = self.context.persist_discovered(dd).await {
                        tracing::warn!(%err, %mac, "failed to persist Mi Flora discovery");
                    }
                }
                Err(err) => {
                    tracing::warn!(%err, %mac, "failed to build Mi Flora discovered device");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use minihub_domain::event::Event;

    #[derive(Clone)]
    struct NoOpContext;

    impl IntegrationContext for NoOpContext {
        async fn upsert_device(
            &self,
            device: minihub_domain::device::Device,
        ) -> Result<minihub_domain::device::Device, MiniHubError> {
            Ok(device)
        }

        async fn upsert_entity(&self, entity: Entity) -> Result<Entity, MiniHubError> {
            Ok(entity)
        }

        async fn publish(&self, _event: Event) -> Result<(), MiniHubError> {
            Ok(())
        }
    }

    fn scanner_with_miflora_filter(filter: Vec<String>) -> BleScanner<NoOpContext> {
        BleScanner {
            context: NoOpContext,
            scan_duration: Duration::from_secs(1),
            interval: Duration::from_secs(1),
            device_filter: Vec::new(),
            miflora_enabled: true,
            miflora_filter: filter,
            miflora_connect_timeout: Duration::from_secs(10),
        }
    }

    #[test]
    fn should_accept_all_when_miflora_filter_is_empty() {
        let scanner = scanner_with_miflora_filter(Vec::new());
        assert!(scanner.passes_miflora_filter("C4:7C:8D:6A:12:34"));
        assert!(scanner.passes_miflora_filter("AA:BB:CC:DD:EE:FF"));
    }

    #[test]
    fn should_accept_matching_mac_in_miflora_filter() {
        let scanner = scanner_with_miflora_filter(vec!["C4:7C:8D:6A:12:34".to_owned()]);
        assert!(scanner.passes_miflora_filter("C4:7C:8D:6A:12:34"));
        assert!(!scanner.passes_miflora_filter("AA:BB:CC:DD:EE:FF"));
    }

    #[test]
    fn should_match_miflora_filter_case_insensitively() {
        let scanner = scanner_with_miflora_filter(vec!["c4:7c:8d:6a:12:34".to_owned()]);
        assert!(scanner.passes_miflora_filter("C4:7C:8D:6A:12:34"));
        assert!(scanner.passes_miflora_filter("c4:7c:8d:6a:12:34"));
    }
}
