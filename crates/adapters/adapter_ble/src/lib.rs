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
mod scanner;

pub use config::BleConfig;
pub use error::BleError;

use std::collections::HashMap;
use std::time::Duration;

use btleplug::api::BDAddr;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use minihub_app::ports::integration::{DiscoveredDevice, Integration};
use minihub_domain::entity::Entity;
use minihub_domain::error::{MiniHubError, NotFoundError};
use minihub_domain::id::EntityId;

/// Passive BLE integration that scans for BLE sensor advertisements.
pub struct BleIntegration {
    config: BleConfig,
    scan_handle: Option<JoinHandle<()>>,
    /// Optional channel for sending newly discovered devices to a receiver
    /// task that persists them.
    discovery_tx: Option<mpsc::Sender<DiscoveredDevice>>,
    /// Maps BLE MAC address to the most recent entity snapshot.
    entities: HashMap<BDAddr, Entity>,
}

impl BleIntegration {
    /// Create a new BLE integration with the given configuration.
    ///
    /// If `discovery_tx` is `Some`, a background scan loop will be spawned
    /// after the initial scan, sending newly discovered or updated devices
    /// through the channel.
    #[must_use]
    pub fn new(config: BleConfig, discovery_tx: Option<mpsc::Sender<DiscoveredDevice>>) -> Self {
        Self {
            config,
            scan_handle: None,
            discovery_tx,
            entities: HashMap::new(),
        }
    }
}

impl Integration for BleIntegration {
    fn name(&self) -> &'static str {
        "ble"
    }

    async fn setup(&mut self) -> Result<Vec<DiscoveredDevice>, MiniHubError> {
        let duration = Duration::from_secs(u64::from(self.config.scan_duration_secs));

        tracing::info!(
            duration_secs = self.config.scan_duration_secs,
            "BLE scan started"
        );

        let discovered = scanner::run_scan(duration, &self.config.device_filter).await?;

        tracing::info!(count = discovered.len(), "BLE discovery complete");

        // Store entity snapshots for handle_service_call lookups.
        for dd in discovered.values() {
            for entity in &dd.entities {
                // Use a zero BDAddr as key — we only need entity lookup by id.
                self.entities.insert(BDAddr::default(), entity.clone());
            }
        }

        // Spawn background scan loop if a discovery channel was provided.
        if let Some(tx) = self.discovery_tx.take() {
            let scan_duration = duration;
            let interval = Duration::from_secs(u64::from(self.config.update_interval_secs));
            let device_filter = self.config.device_filter.clone();

            let handle = tokio::spawn(scanner::background_scan_loop(
                scan_duration,
                interval,
                device_filter,
                tx,
            ));
            self.scan_handle = Some(handle);

            tracing::info!(
                interval_secs = self.config.update_interval_secs,
                "BLE background scan loop started"
            );
        }

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
    use minihub_domain::entity::{AttributeValue, EntityState};

    #[test]
    fn should_create_integration_with_config() {
        let config = BleConfig::default();
        let integration = BleIntegration::new(config, None);
        assert_eq!(integration.name(), "ble");
        assert!(integration.entities.is_empty());
        assert!(integration.scan_handle.is_none());
    }

    #[test]
    fn should_build_discovered_device_from_reading() {
        let reading = parser::SensorReading {
            mac: [0xA4, 0xC1, 0x38, 0x5B, 0x0E, 0xDF],
            temperature: 23.1,
            humidity: 45.0,
            battery_level: 87,
            battery_voltage: 3.05,
        };

        let dd = scanner::build_discovered(&reading).unwrap();
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
        let integration = BleIntegration::new(BleConfig::default(), None);
        let result = integration
            .handle_service_call(EntityId::new(), "read", serde_json::json!({}))
            .await;
        assert!(matches!(result, Err(MiniHubError::NotFound(_))));
    }

    #[tokio::test]
    async fn should_teardown_without_error_when_not_scanning() {
        let mut integration = BleIntegration::new(BleConfig::default(), None);
        let result = integration.teardown().await;
        assert!(result.is_ok());
        assert!(integration.entities.is_empty());
    }

    #[tokio::test]
    async fn should_store_discovery_tx_when_provided() {
        let (tx, _rx) = mpsc::channel(1);
        let integration = BleIntegration::new(BleConfig::default(), Some(tx));
        assert!(integration.discovery_tx.is_some());
    }

    #[tokio::test]
    async fn should_teardown_abort_background_task() {
        let (tx, _rx) = mpsc::channel(1);
        let mut integration = BleIntegration::new(BleConfig::default(), Some(tx));
        // Simulate a background task by spawning a long-running one.
        integration.scan_handle = Some(tokio::spawn(async {
            tokio::time::sleep(Duration::from_secs(3600)).await;
        }));
        assert!(integration.scan_handle.is_some());

        integration.teardown().await.unwrap();
        assert!(integration.scan_handle.is_none());
    }
}
