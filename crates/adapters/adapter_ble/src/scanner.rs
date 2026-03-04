//! BLE scanner — discovers BLE sensors and persists them in real time.
//!
//! [`BleScanner`] wraps BLE scanning, parsing, and real-time persistence.
//! Each advertisement is persisted via the [`IntegrationContext`] as soon as
//! it is received.

use std::collections::HashMap;
use std::time::Duration;

use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager};
use tokio::task::JoinHandle;
use tokio_stream::StreamExt as _;

use minihub_app::ports::integration::IntegrationContext;
use minihub_domain::event::{Event, EventType};

use crate::devices::{BleDeviceHandler, Lywsd03mmcHandler, MifloraHandler};
use crate::error::BleError;
use crate::parser::ServiceUuid;

/// BLE scanner that discovers sensors via passive advertisements and,
/// optionally, reads Mi Flora plant sensors via active GATT connections.
///
/// Each received advertisement is immediately persisted via the
/// [`IntegrationContext`] — there is no batching or post-scan persistence step.
/// After each passive scan, if Mi Flora is enabled, the scanner connects to
/// discovered Mi Flora peripherals to read sensor data.
pub struct BleScanner<C> {
    context: C,
    manager: Manager,
    central: Adapter,
    scan_duration: Duration,
    interval: Duration,
    lywsd: Lywsd03mmcHandler,
    miflora: Option<MifloraHandler>,
}

impl<C: IntegrationContext + Clone + 'static> BleScanner<C> {
    /// Create a new scanner and spawn it as a background task.
    pub fn start(
        context: C,
        manager: Manager,
        central: Adapter,
        scan_duration: Duration,
        interval: Duration,
        lywsd: Lywsd03mmcHandler,
        miflora: Option<MifloraHandler>,
    ) -> JoinHandle<()> {
        let scanner = Self {
            context,
            manager,
            central,
            scan_duration,
            interval,
            lywsd,
            miflora,
        };

        tokio::spawn(scanner.run())
    }

    /// Continuous scan loop — runs a scan, waits for the interval, repeats.
    async fn run(mut self) {
        loop {
            if let Err(err) = self.iterate(&self.central).await {
                tracing::warn!(%err, "BLE background scan failed, retrying next interval");
                match acquire_default_adapter(&self.manager).await {
                    Ok(adapter) => self.central = adapter,
                    Err(acquire_err) => tracing::warn!(
                        %acquire_err,
                        "BLE adapter unavailable during recovery"
                    ),
                }
            }
            tokio::time::sleep(self.interval).await;
        }
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
    async fn iterate(&self, central: &Adapter) -> Result<(), BleError> {
        let mut events = central.events().await?;

        central.start_scan(ScanFilter::default()).await?;

        let deadline = tokio::time::Instant::now() + self.scan_duration;

        while tokio::time::Instant::now() < deadline {
            let remaining = deadline - tokio::time::Instant::now();
            match tokio::time::timeout(remaining, events.next()).await {
                Ok(Some(CentralEvent::ServiceDataAdvertisement { id, service_data })) => {
                    for (uuid, data) in &service_data {
                        let dd = match self.lywsd.try_parse_advertisement(*uuid, data) {
                            Ok(Some(dd)) => dd,
                            Ok(None) => continue,
                            Err(err) => {
                                tracing::debug!(%err, handler = self.lywsd.name(), "advertisement parse error");
                                continue;
                            }
                        };

                        // Verify the peripheral exists before persisting
                        let Ok(peripheral) = central.peripheral(&id).await else {
                            continue;
                        };

                        // Skip peripherals that also advertise MiBeacon (0xFE95) —
                        // they are Mi Flora devices handled in the active GATT phase.
                        if let Ok(Some(props)) = peripheral.properties().await
                            && is_mibeacon_peripheral(&props.service_data)
                        {
                            tracing::debug!(
                                handler = self.lywsd.name(),
                                "skipping MiBeacon peripheral (handled by Mi Flora active scan)"
                            );
                            continue;
                        }

                        tracing::debug!(
                            handler = self.lywsd.name(),
                            "persisting BLE sensor reading"
                        );
                        if let Err(err) = self.context.persist_discovered(dd).await {
                            tracing::warn!(%err, "failed to persist BLE discovery");
                        }
                    }
                }
                Ok(Some(CentralEvent::DeviceDiscovered(id))) => {
                    if let Ok(peripheral) = central.peripheral(&id).await
                        && let Ok(Some(props)) = peripheral.properties().await
                    {
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
                Ok(Some(_)) => {}
                Ok(None) | Err(_) => break,
            }
        }

        central.stop_scan().await?;

        // Post-scan active phase: GATT-based device handlers.
        if let Some(ref miflora) = self.miflora {
            for dd in miflora.process_after_scan(&central).await {
                if let Err(err) = self.context.persist_discovered(dd).await {
                    tracing::warn!(%err, handler = miflora.name(), "failed to persist discovery");
                }
            }
        }

        Ok(())
    }
}

/// Acquire the first available BLE adapter from `btleplug`.
pub(crate) async fn acquire_default_adapter(manager: &Manager) -> Result<Adapter, BleError> {
    let adapters = manager.adapters().await?;
    adapters.into_iter().next().ok_or(BleError::NotAvailable)
}

/// Returns `true` if the peripheral's service data contains a `MiBeacon`
/// (`0xFE95`) entry, indicating it is a Mi Flora device that should be
/// handled by the active GATT phase rather than the passive scan.
fn is_mibeacon_peripheral(service_data: &HashMap<uuid::Uuid, Vec<u8>>) -> bool {
    service_data.contains_key(&ServiceUuid::MIFLORA)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_detect_mibeacon_peripheral_when_fe95_present() {
        let mut service_data = HashMap::new();
        service_data.insert(ServiceUuid::MIFLORA, vec![0x71, 0x20, 0x98, 0x00]);
        assert!(is_mibeacon_peripheral(&service_data));
    }

    #[test]
    fn should_not_detect_mibeacon_peripheral_when_only_181a() {
        let mut service_data = HashMap::new();
        service_data.insert(ServiceUuid::ATC1441, vec![0u8; 19]);
        assert!(!is_mibeacon_peripheral(&service_data));
    }

    #[test]
    fn should_detect_mibeacon_peripheral_when_both_uuids_present() {
        let mut service_data = HashMap::new();
        service_data.insert(ServiceUuid::ATC1441, vec![0u8; 19]);
        service_data.insert(ServiceUuid::MIFLORA, vec![0x71, 0x20, 0x98, 0x00]);
        assert!(is_mibeacon_peripheral(&service_data));
    }

    #[test]
    fn should_not_detect_mibeacon_peripheral_when_empty() {
        let service_data = HashMap::new();
        assert!(!is_mibeacon_peripheral(&service_data));
    }
}
