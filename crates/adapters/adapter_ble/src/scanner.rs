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

use minihub_app::ports::integration::IntegrationContext;
use minihub_domain::event::{Event, EventType};

use crate::devices::{BleDeviceHandler, Lywsd03mmcHandler, MifloraHandler};
use crate::error::BleError;

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
    lywsd: Lywsd03mmcHandler,
    miflora: Option<MifloraHandler>,
}

impl<C: IntegrationContext + Clone + 'static> BleScanner<C> {
    /// Create a new scanner and spawn it as a background task.
    pub fn start(
        context: C,
        scan_duration: Duration,
        interval: Duration,
        lywsd: Lywsd03mmcHandler,
        miflora: Option<MifloraHandler>,
    ) -> JoinHandle<()> {
        let scanner = Self {
            context,
            scan_duration,
            interval,
            lywsd,
            miflora,
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
                        if central.peripheral(&id).await.is_err() {
                            continue;
                        }

                        tracing::debug!(handler = self.lywsd.name(), "persisting BLE sensor reading");
                        if let Err(err) = self.context.persist_discovered(dd).await {
                            tracing::warn!(%err, "failed to persist BLE discovery");
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
