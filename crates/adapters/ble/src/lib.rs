//! # minihub-adapter-ble
//!
//! BLE adapter — scans for BLE sensor advertisements and reads active
//! GATT devices, exposing them as minihub devices and entities.
//!
//! ## How it works
//!
//! The adapter runs a repeating scan loop with two phases:
//!
//! 1. **Passive phase** — collects service-data advertisements (no
//!    connection needed) and parses them into sensor entities.
//! 2. **Active GATT phase** (optional) — after the passive scan stops,
//!    connects to discovered Mi Flora plant sensors, reads sensor data
//!    and firmware info via GATT, then disconnects.
//!
//! ## Supported formats
//!
//! | Format | Mode | UUID | Payload | Endianness |
//! |--------|------|------|---------|------------|
//! | PVVX custom | Passive | `0x181A` | 19 bytes | Little-endian |
//! | ATC1441 original | Passive | `0x181A` | 13 bytes | Big-endian |
//! | Mi Flora (HHCCJCY01) | Active GATT | `0xFE95` | 16 + 7 bytes | Little-endian |
//!
//! ## Dependency rule
//!
//! Same as other adapters: depends on `minihub-app` and `minihub-domain`.

mod config;
mod devices;
mod error;
pub mod parser;
mod scanner;

pub use config::BleConfig;
pub use error::BleError;

use std::collections::HashMap;
use std::time::Duration;

use btleplug::api::{BDAddr, Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;
use tokio::task::JoinHandle;
use tokio_stream::StreamExt as _;

use minihub_app::ports::integration::{Integration, IntegrationContext};
use minihub_domain::entity::Entity;
use minihub_domain::error::{MiniHubError, NotFoundError};
use minihub_domain::event::{Event, EventType};
use minihub_domain::id::EntityId;

use crate::devices::{Lywsd03mmcHandler, MifloraHandler};
use crate::parser::ServiceUuid;
use crate::scanner::BleScanner;

/// BLE integration — scans for BLE sensor advertisements and handles
/// service calls via the event bus.
///
/// Holds handles to a background [`BleScanner`] and an event subscriber
/// task. MAC addresses are stored in the database on each entity and
/// looked up at service-call time via [`IntegrationContext::find_entity_by_id`].
pub struct BleIntegration {
    config: BleConfig,
    scan_handle: Option<JoinHandle<()>>,
    subscriber_handle: Option<JoinHandle<()>>,
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
            subscriber_handle: None,
            entities: HashMap::new(),
        }
    }
}

impl Integration for BleIntegration {
    fn name(&self) -> &'static str {
        "ble"
    }

    async fn setup(&mut self, _ctx: &impl IntegrationContext) -> Result<(), MiniHubError> {
        tracing::info!("BLE integration initialised");
        Ok(())
    }

    async fn start_background(
        &mut self,
        ctx: impl IntegrationContext + Clone + 'static,
    ) -> Result<(), MiniHubError> {
        let scan_duration = Duration::from_secs(u64::from(self.config.scan_duration_secs));
        let interval = Duration::from_secs(u64::from(self.config.update_interval_secs));

        let lywsd = Lywsd03mmcHandler::new(self.config.device_filter.clone());
        let miflora = self.config.miflora_enabled.then(|| {
            MifloraHandler::new(
                self.config.miflora_filter.clone(),
                Duration::from_secs(u64::from(self.config.miflora_connect_timeout_secs)),
            )
        });

        let manager = Manager::new().await.map_err(BleError::Scan)?;
        let adapter = scanner::acquire_default_adapter(&manager).await?;

        self.scan_handle = Some(BleScanner::start(
            ctx.clone(),
            manager,
            adapter,
            scan_duration,
            interval,
            lywsd,
            miflora,
        ));

        let subscriber_ctx = ctx;
        self.subscriber_handle = Some(tokio::spawn(run_event_subscriber(subscriber_ctx)));

        tracing::info!(
            interval_secs = self.config.update_interval_secs,
            "BLE background scan loop started"
        );
        Ok(())
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
        if let Some(handle) = self.subscriber_handle.take() {
            handle.abort();
            tracing::debug!("BLE event subscriber task aborted");
        }
        if let Some(handle) = self.scan_handle.take() {
            handle.abort();
            tracing::debug!("BLE scan task aborted");
        }
        self.entities.clear();
        tracing::info!("BLE integration stopped");
        Ok(())
    }
}

/// Duration for the short BLE scan used to find a peripheral for service calls.
const SERVICE_CALL_SCAN_SECS: u64 = 3;

/// Subscribe to the event bus, filter for [`EventType::ServiceCallRequested`]
/// events that target BLE entities (those with a stored `mac_address`), and
/// handle them.
async fn run_event_subscriber(ctx: impl IntegrationContext + 'static) {
    let mut rx = ctx.subscribe();

    loop {
        let event = match rx.recv().await {
            Ok(event) => event,
            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                tracing::warn!(
                    skipped,
                    "BLE event subscriber lagged, some events were missed"
                );
                continue;
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                tracing::info!("BLE event subscriber channel closed, stopping");
                break;
            }
        };

        if event.event_type != EventType::ServiceCallRequested {
            continue;
        }

        let Some(entity_id) = event.entity_id else {
            continue;
        };

        // Look up the entity in the database and extract its MAC address.
        let mac = match ctx.find_entity_by_id(entity_id).await {
            Ok(Some(entity)) => entity
                .mac_address
                .as_deref()
                .and_then(crate::parser::parse_mac),
            Ok(None) => None,
            Err(err) => {
                tracing::warn!(%err, %entity_id, "failed to look up entity for service call");
                continue;
            }
        };

        let Some(mac) = mac else {
            continue;
        };

        let service = event
            .data
            .get("service")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let mac_str = crate::parser::format_mac(mac);

        tracing::info!(
            %entity_id,
            mac = %mac_str,
            service,
            "BLE handling service call"
        );

        match service {
            "blink" => {
                let result = handle_blink(mac).await;
                let result_event = match result {
                    Ok(()) => {
                        tracing::info!(mac = %mac_str, "BLE blink completed");
                        Event::new(
                            EventType::ServiceCallCompleted,
                            Some(entity_id),
                            serde_json::json!({ "service": "blink" }),
                        )
                    }
                    Err(err) => {
                        tracing::warn!(%err, mac = %mac_str, "BLE blink failed");
                        Event::new(
                            EventType::ServiceCallFailed,
                            Some(entity_id),
                            serde_json::json!({
                                "service": "blink",
                                "error": err.to_string(),
                            }),
                        )
                    }
                };

                if let Err(err) = ctx.publish(result_event).await {
                    tracing::warn!(%err, "failed to publish service call result event");
                }
            }
            other => {
                tracing::debug!(service = other, "BLE ignoring unknown service");
            }
        }
    }
}

/// Perform a short BLE scan to find the peripheral with the given MAC,
/// then call [`devices::miflora::blink_miflora`] on it.
async fn handle_blink(mac: [u8; 6]) -> Result<(), BleError> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().next().ok_or(BleError::NotAvailable)?;

    let mut events = central.events().await?;

    central
        .start_scan(ScanFilter {
            services: vec![ServiceUuid::MIFLORA],
        })
        .await?;

    let deadline = tokio::time::Instant::now() + Duration::from_secs(SERVICE_CALL_SCAN_SECS);

    while tokio::time::Instant::now() < deadline {
        let remaining = deadline - tokio::time::Instant::now();
        match tokio::time::timeout(remaining, events.next()).await {
            Ok(Some(_)) => {}
            Ok(None) | Err(_) => break,
        }
    }

    central.stop_scan().await?;

    let peripherals = central.peripherals().await?;
    for peripheral in &peripherals {
        let Ok(Some(props)) = peripheral.properties().await else {
            continue;
        };

        let Some(mibeacon_data) = props.service_data.get(&ServiceUuid::MIFLORA) else {
            continue;
        };

        let Ok(parsed_mac) = devices::parse_mibeacon_mac(mibeacon_data) else {
            continue;
        };

        if parsed_mac == mac {
            return devices::blink_miflora(peripheral).await;
        }
    }

    let mac_str = crate::parser::format_mac(mac);
    Err(BleError::Domain(MiniHubError::NotFound(NotFoundError {
        entity: "BLE peripheral",
        id: mac_str,
    })))
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use minihub_domain::entity::{AttributeValue, EntityState};
    use minihub_domain::id::DeviceId;
    use std::sync::Arc;
    use tokio::sync::broadcast;

    use crate::devices::lywsd03mmc::{SensorReading, build_discovered};

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

        async fn find_entity_by_id(&self, _id: EntityId) -> Result<Option<Entity>, MiniHubError> {
            Ok(None)
        }

        async fn find_entity_by_entity_id(
            &self,
            _entity_id: &str,
        ) -> Result<Option<Entity>, MiniHubError> {
            Ok(None)
        }

        async fn publish(&self, _event: Event) -> Result<(), MiniHubError> {
            Ok(())
        }

        fn subscribe(&self) -> broadcast::Receiver<Event> {
            let (tx, rx) = broadcast::channel(1);
            drop(tx);
            rx
        }
    }

    /// Test context backed by a real broadcast channel for subscriber tests.
    ///
    /// Stores entities in a `HashMap` so that `find_entity_by_id` can resolve MAC
    /// addresses during service-call handling.
    #[derive(Clone)]
    struct BroadcastContext {
        tx: broadcast::Sender<Event>,
        published: Arc<Mutex<Vec<Event>>>,
        entities: Arc<Mutex<HashMap<EntityId, Entity>>>,
    }

    impl BroadcastContext {
        fn new() -> Self {
            let (tx, _) = broadcast::channel(16);
            Self {
                tx,
                published: Arc::new(Mutex::new(Vec::new())),
                entities: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        fn send(&self, event: Event) {
            let _ = self.tx.send(event);
        }

        /// Register an entity with a MAC address for `find_entity_by_id` lookups.
        fn insert_entity(&self, entity_id: EntityId, mac: &str) {
            let entity = Entity {
                id: entity_id,
                entity_id: format!("sensor.test_{entity_id}"),
                device_id: DeviceId::new(),
                friendly_name: "Test entity".into(),
                state: EntityState::On,
                attributes: HashMap::new(),
                mac_address: Some(mac.to_owned()),
                last_changed: minihub_domain::time::now(),
                last_updated: minihub_domain::time::now(),
            };
            self.entities.lock().unwrap().insert(entity_id, entity);
        }
    }

    impl IntegrationContext for BroadcastContext {
        async fn upsert_device(
            &self,
            device: minihub_domain::device::Device,
        ) -> Result<minihub_domain::device::Device, MiniHubError> {
            Ok(device)
        }

        async fn upsert_entity(&self, entity: Entity) -> Result<Entity, MiniHubError> {
            Ok(entity)
        }

        async fn find_entity_by_id(&self, id: EntityId) -> Result<Option<Entity>, MiniHubError> {
            Ok(self.entities.lock().unwrap().get(&id).cloned())
        }

        async fn find_entity_by_entity_id(
            &self,
            _entity_id: &str,
        ) -> Result<Option<Entity>, MiniHubError> {
            Ok(None)
        }

        async fn publish(&self, event: Event) -> Result<(), MiniHubError> {
            self.published
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(event);
            Ok(())
        }

        fn subscribe(&self) -> broadcast::Receiver<Event> {
            self.tx.subscribe()
        }
    }

    #[test]
    fn should_create_integration_with_config() {
        let config = BleConfig::default();
        let integration = BleIntegration::new(config);
        assert_eq!(integration.name(), "ble");
        assert!(integration.entities.is_empty());
        assert!(integration.scan_handle.is_none());
        assert!(integration.subscriber_handle.is_none());
    }

    #[tokio::test]
    async fn should_return_ok_on_setup() {
        let mut integration = BleIntegration::new(BleConfig::default());
        let ctx = NoOpContext;
        let result = integration.setup(&ctx).await;
        assert!(result.is_ok());
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

        let dd = build_discovered(&reading).unwrap();
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

    #[tokio::test]
    async fn should_teardown_abort_background_tasks() {
        let mut integration = BleIntegration::new(BleConfig::default());
        integration.scan_handle = Some(tokio::spawn(async {
            tokio::time::sleep(Duration::from_secs(3600)).await;
        }));
        integration.subscriber_handle = Some(tokio::spawn(async {
            tokio::time::sleep(Duration::from_secs(3600)).await;
        }));
        assert!(integration.scan_handle.is_some());
        assert!(integration.subscriber_handle.is_some());

        integration.teardown().await.unwrap();
        assert!(integration.scan_handle.is_none());
        assert!(integration.subscriber_handle.is_none());
    }

    #[tokio::test]
    async fn should_ignore_non_service_call_events() {
        let ctx = BroadcastContext::new();
        let entity_id = EntityId::new();
        ctx.insert_entity(entity_id, "C4:7C:8D:6A:12:34");

        let handle = tokio::spawn(run_event_subscriber(ctx.clone()));

        ctx.send(Event::new(
            EventType::StateChanged,
            Some(entity_id),
            serde_json::json!({}),
        ));

        tokio::time::sleep(Duration::from_millis(50)).await;

        let published = ctx
            .published
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(published.is_empty());

        handle.abort();
    }

    #[tokio::test]
    async fn should_ignore_service_call_for_unknown_entity() {
        let ctx = BroadcastContext::new();

        let handle = tokio::spawn(run_event_subscriber(ctx.clone()));

        ctx.send(Event::new(
            EventType::ServiceCallRequested,
            Some(EntityId::new()),
            serde_json::json!({ "service": "blink" }),
        ));

        tokio::time::sleep(Duration::from_millis(50)).await;

        let published = ctx
            .published
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(published.is_empty());

        handle.abort();
    }

    #[tokio::test]
    async fn should_ignore_service_call_without_entity_id() {
        let ctx = BroadcastContext::new();

        let handle = tokio::spawn(run_event_subscriber(ctx.clone()));

        ctx.send(Event::new(
            EventType::ServiceCallRequested,
            None,
            serde_json::json!({ "service": "blink" }),
        ));

        tokio::time::sleep(Duration::from_millis(50)).await;

        let published = ctx
            .published
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(published.is_empty());

        handle.abort();
    }

    #[tokio::test]
    async fn should_ignore_unknown_service_for_known_entity() {
        let ctx = BroadcastContext::new();
        let entity_id = EntityId::new();
        ctx.insert_entity(entity_id, "C4:7C:8D:6A:12:34");

        let handle = tokio::spawn(run_event_subscriber(ctx.clone()));

        ctx.send(Event::new(
            EventType::ServiceCallRequested,
            Some(entity_id),
            serde_json::json!({ "service": "turn_on" }),
        ));

        tokio::time::sleep(Duration::from_millis(50)).await;

        let published = ctx
            .published
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(published.is_empty());

        handle.abort();
    }

    #[tokio::test]
    async fn should_publish_service_call_failed_when_blink_fails() {
        let ctx = BroadcastContext::new();
        let entity_id = EntityId::new();
        ctx.insert_entity(entity_id, "C4:7C:8D:6A:12:34");

        let handle = tokio::spawn(run_event_subscriber(ctx.clone()));

        // Yield to let the subscriber task start and call subscribe()/recv()
        tokio::task::yield_now().await;

        ctx.send(Event::new(
            EventType::ServiceCallRequested,
            Some(entity_id),
            serde_json::json!({ "service": "blink" }),
        ));

        // handle_blink will either fail immediately (no adapter) or after a
        // 3-second scan (adapter present but peripheral not found).
        tokio::time::sleep(Duration::from_secs(5)).await;

        let published = ctx
            .published
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert_eq!(published.len(), 1);
        assert_eq!(published[0].event_type, EventType::ServiceCallFailed);
        assert_eq!(published[0].entity_id, Some(entity_id));
        assert_eq!(published[0].data["service"], "blink");
        assert!(published[0].data["error"].is_string());

        handle.abort();
    }

    /// Context that subscribes from an external sender (not owned by self).
    ///
    /// This allows the test to close the channel externally, causing the
    /// subscriber's receiver to get `RecvError::Closed`.
    struct ExternalSenderContext {
        rx_factory: Mutex<Option<broadcast::Receiver<Event>>>,
    }

    impl ExternalSenderContext {
        fn new(rx: broadcast::Receiver<Event>) -> Self {
            Self {
                rx_factory: Mutex::new(Some(rx)),
            }
        }
    }

    impl IntegrationContext for ExternalSenderContext {
        async fn upsert_device(
            &self,
            device: minihub_domain::device::Device,
        ) -> Result<minihub_domain::device::Device, MiniHubError> {
            Ok(device)
        }

        async fn upsert_entity(&self, entity: Entity) -> Result<Entity, MiniHubError> {
            Ok(entity)
        }

        async fn find_entity_by_id(&self, _id: EntityId) -> Result<Option<Entity>, MiniHubError> {
            Ok(None)
        }

        async fn find_entity_by_entity_id(
            &self,
            _entity_id: &str,
        ) -> Result<Option<Entity>, MiniHubError> {
            Ok(None)
        }

        async fn publish(&self, _event: Event) -> Result<(), MiniHubError> {
            Ok(())
        }

        fn subscribe(&self) -> broadcast::Receiver<Event> {
            self.rx_factory
                .lock()
                .unwrap()
                .take()
                .expect("subscribe called more than once")
        }
    }

    #[tokio::test]
    async fn should_stop_subscriber_when_channel_closed() {
        let (tx, rx) = broadcast::channel::<Event>(16);
        let ctx = ExternalSenderContext::new(rx);

        let handle = tokio::spawn(run_event_subscriber(ctx));

        // Drop the only sender so the receiver gets Closed
        drop(tx);

        let result = tokio::time::timeout(Duration::from_secs(1), handle).await;
        assert!(result.is_ok(), "subscriber should stop when channel closes");
    }

    #[tokio::test]
    async fn should_ignore_entity_without_mac_address() {
        let ctx = BroadcastContext::new();
        let entity_id = EntityId::new();

        // Insert entity WITHOUT a MAC address
        let entity = Entity {
            id: entity_id,
            entity_id: format!("sensor.test_{entity_id}"),
            device_id: DeviceId::new(),
            friendly_name: "No-MAC entity".into(),
            state: EntityState::On,
            attributes: HashMap::new(),
            mac_address: None,
            last_changed: minihub_domain::time::now(),
            last_updated: minihub_domain::time::now(),
        };
        ctx.entities.lock().unwrap().insert(entity_id, entity);

        let handle = tokio::spawn(run_event_subscriber(ctx.clone()));

        tokio::task::yield_now().await;

        ctx.send(Event::new(
            EventType::ServiceCallRequested,
            Some(entity_id),
            serde_json::json!({ "service": "blink" }),
        ));

        tokio::time::sleep(Duration::from_millis(50)).await;

        let published = ctx
            .published
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(published.is_empty(), "should skip entities without MAC");

        handle.abort();
    }
}
