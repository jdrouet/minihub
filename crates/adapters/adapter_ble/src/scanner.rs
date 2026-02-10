//! BLE scanner — discovers BLE sensors and persists them in real time.
//!
//! [`BleScanner`] wraps BLE scanning, parsing, and real-time persistence.
//! Each advertisement is persisted via the [`IntegrationContext`] as soon as
//! it is received.

use std::time::Duration;

use btleplug::api::{Central, CentralEvent, Manager as _, ScanFilter};
use btleplug::platform::Manager;
use tokio::task::JoinHandle;
use tokio_stream::StreamExt as _;

use minihub_app::ports::integration::{DiscoveredDevice, IntegrationContext};
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

/// Passive BLE scanner that discovers sensors and persists them in real time.
///
/// Each received advertisement is immediately persisted via the
/// [`IntegrationContext`] — there is no batching or post-scan persistence step.
pub struct BleScanner<C> {
    ctx: C,
    scan_duration: Duration,
    interval: Duration,
    device_filter: Vec<String>,
}

impl<C: IntegrationContext + Clone + 'static> BleScanner<C> {
    /// Create a new scanner with the given context and configuration.
    pub fn new(
        ctx: C,
        scan_duration: Duration,
        interval: Duration,
        device_filter: Vec<String>,
    ) -> Self {
        Self {
            ctx,
            scan_duration,
            interval,
            device_filter,
        }
    }

    /// Start continuous background scanning.
    ///
    /// Spawns a tokio task that runs [`scan_loop`](Self::scan_loop) and
    /// returns the [`JoinHandle`]. Abort the handle to stop scanning.
    pub fn start(&mut self) -> JoinHandle<()> {
        let ctx = self.ctx.clone();
        let scan_duration = self.scan_duration;
        let interval = self.interval;
        let device_filter = self.device_filter.clone();

        tokio::spawn(async move {
            Self::scan_loop(ctx, scan_duration, interval, &device_filter).await;
        })
    }

    /// Continuous scan loop — runs a scan, waits for the interval, repeats.
    async fn scan_loop(
        ctx: C,
        scan_duration: Duration,
        interval: Duration,
        device_filter: &[String],
    ) {
        loop {
            if let Err(err) = Self::run_scan(&ctx, scan_duration, device_filter).await {
                tracing::warn!(%err, "BLE background scan failed, retrying next interval");
            }
            tokio::time::sleep(interval).await;
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
    async fn run_scan(
        ctx: &C,
        duration: Duration,
        device_filter: &[String],
    ) -> Result<(), BleError> {
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

                        // Verify the peripheral exists before persisting
                        if central.peripheral(&id).await.is_err() {
                            continue;
                        }

                        let dd = build_discovered(&reading).map_err(BleError::Domain)?;

                        tracing::debug!(mac = %mac_str, "persisting BLE sensor reading");
                        if let Err(err) = ctx.persist_discovered(dd).await {
                            tracing::warn!(%err, mac = %mac_str, "failed to persist BLE discovery");
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
