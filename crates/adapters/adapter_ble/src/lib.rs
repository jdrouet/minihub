//! # minihub-adapter-ble
//!
//! Passive BLE adapter — scans for BLE sensor advertisements and exposes
//! them as minihub devices and entities.
//!
//! ## How it works
//!
//! Some BLE sensors broadcast readings as service-data advertisements
//! (no connection needed). This adapter passively scans for those
//! advertisements and parses them into sensor entities.
//!
//! ## Currently supported formats
//!
//! | Format | UUID | Payload length | Endianness |
//! |--------|------|----------------|------------|
//! | PVVX custom | `0x181A` | 19 bytes | Little-endian |
//! | ATC1441 original | `0x181A` | 13 bytes | Big-endian |
//!
//! ## Dependency rule
//!
//! Same as other adapters: depends on `minihub-app` and `minihub-domain`.

mod config;
mod error;
pub mod parser;

pub use config::BleConfig;
pub use error::BleError;

use std::collections::HashMap;
use std::time::Duration;

use btleplug::api::{BDAddr, Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;
use tokio::task::JoinHandle;
use tokio_stream::StreamExt as _;

use minihub_app::ports::integration::{DiscoveredDevice, Integration};
use minihub_domain::device::Device;
use minihub_domain::entity::{AttributeValue, Entity, EntityState};
use minihub_domain::error::{MiniHubError, NotFoundError};
use minihub_domain::id::EntityId;

use parser::{SERVICE_UUID_181A, SensorReading};

/// Passive BLE integration that scans for LYWSD03MMC (ATC firmware) sensors.
pub struct BleIntegration {
    config: BleConfig,
    scan_handle: Option<JoinHandle<()>>,
    /// Maps BLE MAC address to the most recent entity snapshot.
    entities: HashMap<BDAddr, Entity>,
}

impl BleIntegration {
    /// Create a new BLE integration with the given configuration.
    #[must_use]
    pub fn new(config: BleConfig) -> Self {
        Self {
            config,
            scan_handle: None,
            entities: HashMap::new(),
        }
    }

    /// Check whether the given MAC address passes the device filter.
    fn passes_filter(&self, mac: &str) -> bool {
        if self.config.device_filter.is_empty() {
            return true;
        }
        self.config
            .device_filter
            .iter()
            .any(|f| f.eq_ignore_ascii_case(mac))
    }

    /// Build a [`DiscoveredDevice`] from a [`SensorReading`].
    fn build_discovered(reading: &SensorReading) -> Result<DiscoveredDevice, MiniHubError> {
        let mac_str = parser::format_mac(reading.mac);
        let slug = parser::mac_slug(reading.mac);

        let device = Device::builder()
            .name(format!("LYWSD03MMC {mac_str}"))
            .manufacturer("Xiaomi")
            .model("LYWSD03MMC")
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
}

impl Integration for BleIntegration {
    fn name(&self) -> &'static str {
        "ble"
    }

    async fn setup(&mut self) -> Result<Vec<DiscoveredDevice>, MiniHubError> {
        let manager = Manager::new().await.map_err(BleError::from)?;

        let adapters = manager.adapters().await.map_err(BleError::from)?;

        let central = adapters.into_iter().next().ok_or(BleError::NotAvailable)?;

        let mut events = central.events().await.map_err(BleError::from)?;

        central
            .start_scan(ScanFilter {
                services: vec![SERVICE_UUID_181A],
            })
            .await
            .map_err(BleError::from)?;

        tracing::info!(
            duration_secs = self.config.scan_duration_secs,
            "BLE scan started"
        );

        let timeout = Duration::from_secs(u64::from(self.config.scan_duration_secs));
        let deadline = tokio::time::Instant::now() + timeout;
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
                        if !self.passes_filter(&mac_str) {
                            tracing::debug!(mac = %mac_str, "filtered out by device_filter");
                            continue;
                        }

                        let addr = match central.peripheral(&id).await {
                            Ok(p) => p.address(),
                            Err(_) => continue,
                        };

                        let is_new = !discovered.contains_key(&addr);
                        let dd = Self::build_discovered(&reading).map_err(BleError::Domain)?;

                        if let Some(entity) = dd.entities.first() {
                            self.entities.insert(addr, entity.clone());
                        }
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

        central.stop_scan().await.map_err(BleError::from)?;

        tracing::info!(count = discovered.len(), "BLE discovery complete");

        Ok(discovered.into_values().collect())
    }

    async fn handle_service_call(
        &self,
        entity_id: EntityId,
        _service: &str,
        _data: serde_json::Value,
    ) -> Result<Entity, MiniHubError> {
        let entity = self
            .entities
            .values()
            .find(|ent| ent.id == entity_id)
            .ok_or_else(|| NotFoundError {
                entity: "Entity",
                id: entity_id.to_string(),
            })?;

        Ok(entity.clone())
    }

    async fn teardown(&mut self) -> Result<(), MiniHubError> {
        if let Some(handle) = self.scan_handle.take() {
            handle.abort();
            tracing::debug!("BLE scan task aborted");
        }
        self.entities.clear();
        tracing::info!("BLE integration stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_integration_with_config() {
        let config = BleConfig::default();
        let integration = BleIntegration::new(config);
        assert_eq!(integration.name(), "ble");
        assert!(integration.entities.is_empty());
        assert!(integration.scan_handle.is_none());
    }

    #[test]
    fn should_pass_filter_when_empty() {
        let integration = BleIntegration::new(BleConfig::default());
        assert!(integration.passes_filter("A4:C1:38:5B:0E:DF"));
    }

    #[test]
    fn should_pass_filter_when_mac_matches() {
        let config = BleConfig {
            device_filter: vec!["A4:C1:38:5B:0E:DF".to_string()],
            ..BleConfig::default()
        };
        let integration = BleIntegration::new(config);
        assert!(integration.passes_filter("A4:C1:38:5B:0E:DF"));
        assert!(integration.passes_filter("a4:c1:38:5b:0e:df"));
    }

    #[test]
    fn should_reject_filter_when_mac_not_listed() {
        let config = BleConfig {
            device_filter: vec!["A4:C1:38:AA:BB:CC".to_string()],
            ..BleConfig::default()
        };
        let integration = BleIntegration::new(config);
        assert!(!integration.passes_filter("A4:C1:38:5B:0E:DF"));
    }

    #[test]
    fn should_build_discovered_device_from_reading() {
        let reading = SensorReading {
            mac: [0xA4, 0xC1, 0x38, 0x5B, 0x0E, 0xDF],
            temperature: 23.1,
            humidity: 45.0,
            battery_level: 87,
            battery_voltage: 3.05,
        };

        let dd = BleIntegration::build_discovered(&reading).unwrap();
        assert_eq!(dd.device.name, "LYWSD03MMC A4:C1:38:5B:0E:DF");
        assert_eq!(dd.device.manufacturer.as_deref(), Some("Xiaomi"));
        assert_eq!(dd.device.model.as_deref(), Some("LYWSD03MMC"));

        assert_eq!(dd.entities.len(), 1);
        let entity = &dd.entities[0];
        assert_eq!(entity.entity_id, "sensor.ble_a4c1385b0edf");
        assert_eq!(entity.friendly_name, "BLE Temp/Humidity A4:C1:38:5B:0E:DF");
        assert_eq!(entity.state, EntityState::On);
        assert_eq!(
            entity.get_attribute("temperature"),
            Some(&AttributeValue::Float(23.1))
        );
        assert_eq!(
            entity.get_attribute("humidity"),
            Some(&AttributeValue::Float(45.0))
        );
        assert_eq!(
            entity.get_attribute("battery_level"),
            Some(&AttributeValue::Int(87))
        );
        assert_eq!(
            entity.get_attribute("battery_voltage"),
            Some(&AttributeValue::Float(3.05))
        );
    }

    #[tokio::test]
    async fn should_return_not_found_for_unknown_entity() {
        let integration = BleIntegration::new(BleConfig::default());
        let result = integration
            .handle_service_call(EntityId::new(), "read", serde_json::json!({}))
            .await;
        assert!(matches!(result, Err(MiniHubError::NotFound(_))));
    }

    #[tokio::test]
    async fn should_teardown_without_error_when_not_scanning() {
        let mut integration = BleIntegration::new(BleConfig::default());
        let result = integration.teardown().await;
        assert!(result.is_ok());
        assert!(integration.entities.is_empty());
    }
}
