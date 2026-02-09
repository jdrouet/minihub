//! MQTT integration configuration.

use serde::Deserialize;

/// Configuration for the MQTT integration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct MqttConfig {
    /// MQTT broker hostname or IP address.
    pub broker_host: String,
    /// MQTT broker port.
    pub broker_port: u16,
    /// MQTT client identifier.
    pub client_id: String,
    /// Base topic prefix for all minihub MQTT communication.
    pub base_topic: String,
    /// Keep-alive interval in seconds.
    pub keep_alive_secs: u16,
    /// How long to wait for discovery messages during setup, in seconds.
    pub discovery_timeout_secs: u16,
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            broker_host: "localhost".to_string(),
            broker_port: 1883,
            client_id: "minihub".to_string(),
            base_topic: "minihub".to_string(),
            keep_alive_secs: 30,
            discovery_timeout_secs: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_have_sensible_defaults() {
        let config = MqttConfig::default();
        assert_eq!(config.broker_host, "localhost");
        assert_eq!(config.broker_port, 1883);
        assert_eq!(config.client_id, "minihub");
        assert_eq!(config.base_topic, "minihub");
        assert_eq!(config.keep_alive_secs, 30);
        assert_eq!(config.discovery_timeout_secs, 3);
    }

    #[test]
    fn should_deserialize_from_toml() {
        let toml = r#"
            broker_host = "mqtt.example.com"
            broker_port = 8883
            client_id = "my-hub"
            base_topic = "home"
            keep_alive_secs = 60
            discovery_timeout_secs = 10
        "#;
        let config: MqttConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.broker_host, "mqtt.example.com");
        assert_eq!(config.broker_port, 8883);
        assert_eq!(config.client_id, "my-hub");
        assert_eq!(config.base_topic, "home");
        assert_eq!(config.keep_alive_secs, 60);
        assert_eq!(config.discovery_timeout_secs, 10);
    }

    #[test]
    fn should_use_defaults_for_missing_fields() {
        let toml = r#"broker_host = "192.168.1.100""#;
        let config: MqttConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.broker_host, "192.168.1.100");
        assert_eq!(config.broker_port, 1883);
        assert_eq!(config.client_id, "minihub");
    }
}
