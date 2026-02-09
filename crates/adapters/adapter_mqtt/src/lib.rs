//! # minihub-adapter-mqtt
//!
//! MQTT adapter — bridges MQTT-based devices into minihub via rumqttc.
//!
//! ## Topic conventions
//!
//! The adapter uses a configurable **base topic** (default `minihub`). Under it:
//!
//! | Topic pattern | Direction | Purpose |
//! |---------------|-----------|---------|
//! | `{base}/{device_id}/{entity_slug}/state` | Broker → minihub | State updates from devices |
//! | `{base}/{device_id}/{entity_slug}/set` | minihub → Broker | Service call commands |
//! | `{base}/{device_id}/config` | Broker → minihub | Device/entity discovery |
//!
//! ## Discovery payload
//!
//! Devices announce themselves by publishing a JSON config message:
//!
//! ```json
//! {
//!   "device": { "name": "...", "manufacturer": "...", "model": "..." },
//!   "entities": [
//!     { "entity_id": "light.kitchen", "friendly_name": "Kitchen Light", "state": "off" }
//!   ]
//! }
//! ```
//!
//! ## Dependency rule
//!
//! Same as other adapters: depends on `minihub-app` and `minihub-domain`.

mod config;
mod error;

pub use config::MqttConfig;
pub use error::MqttError;

use std::collections::HashMap;
use std::time::Duration;

use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use minihub_app::ports::integration::{DiscoveredDevice, Integration};
use minihub_domain::device::Device;
use minihub_domain::entity::{Entity, EntityState};
use minihub_domain::error::{MiniHubError, NotFoundError};
use minihub_domain::id::EntityId;

/// MQTT integration that bridges MQTT-based devices into minihub.
///
/// Connects to an MQTT broker, subscribes to discovery and state topics,
/// and translates messages into entity state updates.
pub struct MqttIntegration {
    config: MqttConfig,
    client: Option<AsyncClient>,
    eventloop_handle: Option<JoinHandle<()>>,
    /// Maps `entity_id` string (e.g. `"light.kitchen"`) to the entity snapshot.
    entities: HashMap<String, Entity>,
    /// Maps entity UUID to the MQTT command topic.
    command_topics: HashMap<EntityId, String>,
}

impl MqttIntegration {
    /// Create a new MQTT integration with the given configuration.
    #[must_use]
    pub fn new(config: MqttConfig) -> Self {
        Self {
            config,
            client: None,
            eventloop_handle: None,
            entities: HashMap::new(),
            command_topics: HashMap::new(),
        }
    }

    /// Build rumqttc options from our config.
    fn mqtt_options(&self) -> MqttOptions {
        let mut opts = MqttOptions::new(
            &self.config.client_id,
            &self.config.broker_host,
            self.config.broker_port,
        );
        opts.set_keep_alive(Duration::from_secs(u64::from(self.config.keep_alive_secs)));
        opts
    }

    /// Spawn the eventloop driver task.
    ///
    /// Returns a receiver that yields incoming [`Publish`] packets and the
    /// join handle for the background task.
    fn spawn_eventloop(
        mut eventloop: EventLoop,
    ) -> (mpsc::Receiver<rumqttc::Publish>, JoinHandle<()>) {
        let (tx, rx) = mpsc::channel::<rumqttc::Publish>(256);

        let handle = tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(Event::Incoming(Packet::Publish(publish))) => {
                        if tx.send(publish).await.is_err() {
                            tracing::debug!("publish receiver dropped, stopping eventloop");
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(err) => {
                        tracing::warn!(%err, "MQTT connection error, reconnecting");
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        });

        (rx, handle)
    }

    /// Subscribe to the discovery and state wildcard topics.
    async fn subscribe_topics(&self) -> Result<(), MqttError> {
        let client = self.client.as_ref().ok_or(MqttError::NotConnected)?;
        let base = &self.config.base_topic;

        let config_topic = format!("{base}/+/config");
        client
            .subscribe(&config_topic, QoS::AtLeastOnce)
            .await
            .map_err(MqttError::Client)?;
        tracing::info!(topic = %config_topic, "subscribed to discovery topic");

        let state_topic = format!("{base}/+/+/state");
        client
            .subscribe(&state_topic, QoS::AtLeastOnce)
            .await
            .map_err(MqttError::Client)?;
        tracing::info!(topic = %state_topic, "subscribed to state topic");

        Ok(())
    }

    /// Process discovery messages received during setup.
    ///
    /// Waits up to `discovery_timeout` for config messages, then returns
    /// all discovered devices.
    async fn run_discovery(
        &mut self,
        rx: &mut mpsc::Receiver<rumqttc::Publish>,
    ) -> Result<Vec<DiscoveredDevice>, MqttError> {
        let timeout_dur = Duration::from_secs(u64::from(self.config.discovery_timeout_secs));
        let mut discovered = Vec::new();

        loop {
            match tokio::time::timeout(timeout_dur, rx.recv()).await {
                Ok(Some(publish)) => {
                    if let Some(dd) = self.handle_config_message(&publish)? {
                        discovered.push(dd);
                    }
                }
                Ok(None) => break,
                Err(_) => {
                    tracing::debug!("discovery timeout reached");
                    break;
                }
            }
        }

        Ok(discovered)
    }

    /// Parse a discovery config message into a [`DiscoveredDevice`].
    fn handle_config_message(
        &mut self,
        publish: &rumqttc::Publish,
    ) -> Result<Option<DiscoveredDevice>, MqttError> {
        let topic = &publish.topic;
        if !topic.ends_with("/config") {
            return Ok(None);
        }

        let payload: DiscoveryPayload =
            serde_json::from_slice(&publish.payload).map_err(MqttError::PayloadParse)?;

        let device = Device::builder()
            .name(&payload.device.name)
            .manufacturer(&payload.device.manufacturer)
            .model(&payload.device.model)
            .build()
            .map_err(MqttError::Domain)?;

        let base = &self.config.base_topic;
        let device_slug = topic
            .strip_prefix(&format!("{base}/"))
            .and_then(|rest| rest.strip_suffix("/config"))
            .unwrap_or("unknown");

        let mut entities = Vec::new();
        for ep in &payload.entities {
            let state = parse_state(&ep.state);
            let entity = Entity::builder()
                .device_id(device.id)
                .entity_id(&ep.entity_id)
                .friendly_name(&ep.friendly_name)
                .state(state)
                .build()
                .map_err(MqttError::Domain)?;

            let entity_slug = ep.entity_id.split('.').next_back().unwrap_or(&ep.entity_id);
            let cmd_topic = format!("{base}/{device_slug}/{entity_slug}/set");
            self.command_topics.insert(entity.id, cmd_topic);
            self.entities.insert(ep.entity_id.clone(), entity.clone());
            entities.push(entity);
        }

        tracing::info!(
            device = %device.name,
            entity_count = entities.len(),
            "discovered MQTT device"
        );

        Ok(Some(DiscoveredDevice { device, entities }))
    }
}

impl Integration for MqttIntegration {
    fn name(&self) -> &'static str {
        "mqtt"
    }

    async fn setup(&mut self) -> Result<Vec<DiscoveredDevice>, MiniHubError> {
        let opts = self.mqtt_options();
        let (client, eventloop) = AsyncClient::new(opts, 64);
        self.client = Some(client);

        let (mut rx, handle) = Self::spawn_eventloop(eventloop);
        self.eventloop_handle = Some(handle);

        self.subscribe_topics()
            .await
            .map_err(MqttError::into_domain)?;

        let discovered = self
            .run_discovery(&mut rx)
            .await
            .map_err(MqttError::into_domain)?;

        Self::spawn_state_listener(rx);

        Ok(discovered)
    }

    async fn handle_service_call(
        &self,
        entity_id: EntityId,
        service: &str,
        data: serde_json::Value,
    ) -> Result<Entity, MiniHubError> {
        let client = self.client.as_ref().ok_or(MqttError::NotConnected)?;
        let cmd_topic = self
            .command_topics
            .get(&entity_id)
            .ok_or_else(|| NotFoundError {
                entity: "Entity",
                id: entity_id.to_string(),
            })?;

        let payload = serde_json::json!({
            "service": service,
            "data": data,
        });
        client
            .publish(
                cmd_topic,
                QoS::AtLeastOnce,
                false,
                payload.to_string().into_bytes(),
            )
            .await
            .map_err(MqttError::Client)?;

        tracing::info!(
            entity_id = %entity_id,
            service,
            topic = %cmd_topic,
            "published MQTT service call"
        );

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
        if let Some(handle) = self.eventloop_handle.take() {
            handle.abort();
            tracing::debug!("MQTT eventloop task aborted");
        }
        self.client = None;
        tracing::info!("MQTT integration stopped");
        Ok(())
    }
}

impl MqttIntegration {
    /// Spawn a background task that listens for state update messages.
    fn spawn_state_listener(mut rx: mpsc::Receiver<rumqttc::Publish>) {
        tokio::spawn(async move {
            while let Some(publish) = rx.recv().await {
                if publish.topic.ends_with("/state") {
                    tracing::debug!(
                        topic = %publish.topic,
                        payload_len = publish.payload.len(),
                        "received state update"
                    );
                }
            }
            tracing::debug!("state listener stopped");
        });
    }
}

/// JSON payload published on `{base}/{device_id}/config` for device discovery.
#[derive(Debug, serde::Deserialize)]
struct DiscoveryPayload {
    device: DevicePayload,
    entities: Vec<EntityPayload>,
}

/// Device metadata within a discovery payload.
#[derive(Debug, serde::Deserialize)]
struct DevicePayload {
    name: String,
    #[serde(default)]
    manufacturer: String,
    #[serde(default)]
    model: String,
}

/// Entity descriptor within a discovery payload.
#[derive(Debug, serde::Deserialize)]
struct EntityPayload {
    entity_id: String,
    friendly_name: String,
    #[serde(default = "default_state")]
    state: String,
}

fn default_state() -> String {
    "unknown".to_string()
}

/// Map a string state value to [`EntityState`].
fn parse_state(s: &str) -> EntityState {
    match s.to_lowercase().as_str() {
        "on" => EntityState::On,
        "off" => EntityState::Off,
        "unavailable" => EntityState::Unavailable,
        _ => EntityState::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_on_state() {
        assert_eq!(parse_state("on"), EntityState::On);
        assert_eq!(parse_state("ON"), EntityState::On);
    }

    #[test]
    fn should_parse_off_state() {
        assert_eq!(parse_state("off"), EntityState::Off);
        assert_eq!(parse_state("OFF"), EntityState::Off);
    }

    #[test]
    fn should_parse_unavailable_state() {
        assert_eq!(parse_state("unavailable"), EntityState::Unavailable);
    }

    #[test]
    fn should_parse_unknown_state_for_unrecognised_value() {
        assert_eq!(parse_state("foo"), EntityState::Unknown);
        assert_eq!(parse_state(""), EntityState::Unknown);
    }

    #[test]
    fn should_create_integration_with_config() {
        let config = MqttConfig::default();
        let integration = MqttIntegration::new(config);
        assert_eq!(integration.name(), "mqtt");
        assert!(integration.client.is_none());
        assert!(integration.entities.is_empty());
    }

    #[test]
    fn should_build_mqtt_options_from_config() {
        let config = MqttConfig {
            broker_host: "example.com".to_string(),
            broker_port: 8883,
            client_id: "test-client".to_string(),
            ..MqttConfig::default()
        };
        let integration = MqttIntegration::new(config);
        let opts = integration.mqtt_options();
        assert_eq!(opts.broker_address().0, "example.com");
        assert_eq!(opts.broker_address().1, 8883);
    }

    #[test]
    fn should_parse_valid_discovery_payload() {
        let json = serde_json::json!({
            "device": {
                "name": "Test Device",
                "manufacturer": "TestCo",
                "model": "T-1000"
            },
            "entities": [
                {
                    "entity_id": "light.test",
                    "friendly_name": "Test Light",
                    "state": "off"
                }
            ]
        });
        let payload: DiscoveryPayload = serde_json::from_value(json).unwrap();
        assert_eq!(payload.device.name, "Test Device");
        assert_eq!(payload.entities.len(), 1);
        assert_eq!(payload.entities[0].entity_id, "light.test");
    }

    #[test]
    fn should_use_default_state_when_missing_from_payload() {
        let json = serde_json::json!({
            "device": { "name": "Dev" },
            "entities": [
                { "entity_id": "sensor.temp", "friendly_name": "Temp" }
            ]
        });
        let payload: DiscoveryPayload = serde_json::from_value(json).unwrap();
        assert_eq!(payload.entities[0].state, "unknown");
    }

    #[test]
    fn should_handle_config_message_and_populate_entities() {
        let config = MqttConfig {
            base_topic: "minihub".to_string(),
            ..MqttConfig::default()
        };
        let mut integration = MqttIntegration::new(config);

        let payload = serde_json::json!({
            "device": {
                "name": "Kitchen Hub",
                "manufacturer": "AcmeCo",
                "model": "KH-1"
            },
            "entities": [
                {
                    "entity_id": "light.kitchen",
                    "friendly_name": "Kitchen Light",
                    "state": "on"
                }
            ]
        });

        let publish = rumqttc::Publish::new(
            "minihub/kitchen_hub/config",
            QoS::AtLeastOnce,
            payload.to_string(),
        );

        let result = integration.handle_config_message(&publish).unwrap();
        assert!(result.is_some());

        let dd = result.unwrap();
        assert_eq!(dd.device.name, "Kitchen Hub");
        assert_eq!(dd.entities.len(), 1);
        assert_eq!(dd.entities[0].entity_id, "light.kitchen");
        assert_eq!(dd.entities[0].state, EntityState::On);

        assert!(integration.entities.contains_key("light.kitchen"));
        assert_eq!(integration.command_topics.len(), 1);
    }

    #[test]
    fn should_skip_non_config_messages() {
        let mut integration = MqttIntegration::new(MqttConfig::default());

        let publish = rumqttc::Publish::new("minihub/device/entity/state", QoS::AtLeastOnce, "on");

        let result = integration.handle_config_message(&publish).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn should_return_error_for_invalid_discovery_json() {
        let mut integration = MqttIntegration::new(MqttConfig::default());

        let publish = rumqttc::Publish::new(
            "minihub/device/config",
            QoS::AtLeastOnce,
            "not valid json {{",
        );

        let result = integration.handle_config_message(&publish);
        assert!(result.is_err());
    }

    #[test]
    fn should_build_correct_command_topic() {
        let config = MqttConfig {
            base_topic: "home".to_string(),
            ..MqttConfig::default()
        };
        let mut integration = MqttIntegration::new(config);

        let payload = serde_json::json!({
            "device": { "name": "Lamp", "manufacturer": "X", "model": "Y" },
            "entities": [
                { "entity_id": "light.lamp", "friendly_name": "Lamp", "state": "off" }
            ]
        });

        let publish =
            rumqttc::Publish::new("home/my_lamp/config", QoS::AtLeastOnce, payload.to_string());

        let dd = integration
            .handle_config_message(&publish)
            .unwrap()
            .unwrap();
        let entity_id = dd.entities[0].id;
        let cmd_topic = integration.command_topics.get(&entity_id).unwrap();
        assert_eq!(cmd_topic, "home/my_lamp/lamp/set");
    }

    #[test]
    fn should_discover_multiple_entities_per_device() {
        let mut integration = MqttIntegration::new(MqttConfig::default());

        let payload = serde_json::json!({
            "device": { "name": "Multi", "manufacturer": "X", "model": "M" },
            "entities": [
                { "entity_id": "light.a", "friendly_name": "A", "state": "on" },
                { "entity_id": "switch.b", "friendly_name": "B", "state": "off" }
            ]
        });

        let publish = rumqttc::Publish::new(
            "minihub/multi/config",
            QoS::AtLeastOnce,
            payload.to_string(),
        );

        let dd = integration
            .handle_config_message(&publish)
            .unwrap()
            .unwrap();
        assert_eq!(dd.entities.len(), 2);
        assert_eq!(integration.entities.len(), 2);
        assert_eq!(integration.command_topics.len(), 2);
    }

    #[tokio::test]
    async fn should_return_not_found_when_service_call_targets_unknown_entity() {
        let config = MqttConfig::default();
        let integration = MqttIntegration::new(config);

        let result = integration
            .handle_service_call(EntityId::new(), "turn_on", serde_json::json!({}))
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_teardown_without_error_when_not_connected() {
        let mut integration = MqttIntegration::new(MqttConfig::default());
        let result = integration.teardown().await;
        assert!(result.is_ok());
    }
}
