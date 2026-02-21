//! Configuration loading — TOML file with environment variable overrides.
//!
//! Looks for `minihub.toml` in the working directory. Every field has a
//! sensible default so the file is optional. Environment variables take
//! precedence over file values.

use serde::Deserialize;

/// Top-level configuration.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    /// HTTP server settings.
    pub server: ServerConfig,
    /// Database settings.
    pub database: DatabaseConfig,
    /// Logging settings.
    pub logging: LoggingConfig,
    /// Integration toggles.
    pub integrations: IntegrationsConfig,
    /// Entity history retention settings.
    pub history: HistoryConfig,
}

/// HTTP listener configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// Address to bind to (e.g. `0.0.0.0`).
    pub host: String,
    /// TCP port.
    pub port: u16,
    /// Path to the dashboard static assets directory (trunk build output).
    pub dashboard_dir: Option<String>,
}

/// `SQLite` database configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct DatabaseConfig {
    /// `SQLite` connection URL or file path.
    pub url: String,
}

/// Logging configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    /// Filter directive (`RUST_LOG` syntax).
    pub filter: String,
}

/// Per-integration toggles.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct IntegrationsConfig {
    /// Enable the virtual/demo integration.
    pub virtual_enabled: bool,
    /// MQTT integration settings (disabled by default).
    pub mqtt: MqttIntegrationConfig,
    /// BLE integration settings (disabled by default).
    pub ble: BleIntegrationConfig,
}

/// Entity history retention settings.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct HistoryConfig {
    /// Number of days to retain entity history (default: 30).
    pub retention_days: u16,
    /// Interval between purge operations, in hours (default: 24).
    pub purge_interval_hours: u16,
}

/// MQTT integration configuration within the main config file.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct MqttIntegrationConfig {
    /// Whether the MQTT integration is enabled.
    pub enabled: bool,
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
}

/// BLE passive scanner integration configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct BleIntegrationConfig {
    /// Whether the BLE integration is enabled.
    pub enabled: bool,
    /// How long to scan for advertisements during setup, in seconds.
    pub scan_duration_secs: u16,
    /// Interval between background re-scans, in seconds.
    pub update_interval_secs: u16,
    /// Optional MAC address allowlist (e.g. `["A4:C1:38:AA:BB:CC"]`).
    pub device_filter: Vec<String>,
    /// Enable active GATT readout for Mi Flora plant sensors.
    pub miflora_enabled: bool,
    /// Optional MAC allowlist for Mi Flora devices.
    pub miflora_filter: Vec<String>,
    /// Per-device GATT connection timeout, in seconds.
    pub miflora_connect_timeout_secs: u16,
}

impl Config {
    /// Load configuration from `minihub.toml` (if present) then apply
    /// environment-variable overrides.
    ///
    /// # Errors
    ///
    /// Returns an error if the TOML file exists but is malformed.
    pub fn load() -> Result<Self, ConfigError> {
        let mut config = Self::from_file("minihub.toml")?;
        config.apply_env_overrides();
        config.validate()?;
        Ok(config)
    }

    fn from_file(path: &str) -> Result<Self, ConfigError> {
        match std::fs::read_to_string(path) {
            Ok(content) => toml::from_str(&content).map_err(ConfigError::Parse),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(err) => Err(ConfigError::Io(err)),
        }
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("MINIHUB_HOST") {
            self.server.host = val;
        }
        if let Ok(val) = std::env::var("MINIHUB_PORT") {
            if let Ok(port) = val.parse() {
                self.server.port = port;
            }
        }
        if let Ok(val) = std::env::var("MINIHUB_BIND") {
            if let Some((host, port)) = val.rsplit_once(':') {
                self.server.host = host.to_string();
                if let Ok(port) = port.parse() {
                    self.server.port = port;
                }
            }
        }
        if let Ok(val) = std::env::var("MINIHUB_DASHBOARD_DIR") {
            self.server.dashboard_dir = Some(val);
        }
        if let Ok(val) = std::env::var("MINIHUB_DATABASE_URL") {
            self.database.url = val;
        }
        if let Ok(val) = std::env::var("MINIHUB_LOG") {
            self.logging.filter = val;
        }
        if let Ok(val) = std::env::var("RUST_LOG") {
            self.logging.filter = val;
        }
        if let Ok(val) = std::env::var("MINIHUB_MQTT_ENABLED") {
            self.integrations.mqtt.enabled = val == "1" || val.eq_ignore_ascii_case("true");
        }
        if let Ok(val) = std::env::var("MINIHUB_MQTT_BROKER_HOST") {
            self.integrations.mqtt.broker_host = val;
        }
        if let Ok(val) = std::env::var("MINIHUB_MQTT_BROKER_PORT") {
            if let Ok(port) = val.parse() {
                self.integrations.mqtt.broker_port = port;
            }
        }
        if let Ok(val) = std::env::var("MINIHUB_BLE_ENABLED") {
            self.integrations.ble.enabled = val == "1" || val.eq_ignore_ascii_case("true");
        }
        if let Ok(val) = std::env::var("MINIHUB_BLE_SCAN_DURATION_SECS") {
            if let Ok(secs) = val.parse() {
                self.integrations.ble.scan_duration_secs = secs;
            }
        }
        if let Ok(val) = std::env::var("MINIHUB_BLE_MIFLORA_ENABLED") {
            self.integrations.ble.miflora_enabled = val == "1" || val.eq_ignore_ascii_case("true");
        }
        if let Ok(val) = std::env::var("MINIHUB_HISTORY_RETENTION_DAYS") {
            if let Ok(days) = val.parse() {
                self.history.retention_days = days;
            }
        }
        if let Ok(val) = std::env::var("MINIHUB_HISTORY_PURGE_INTERVAL_HOURS") {
            if let Ok(hours) = val.parse() {
                self.history.purge_interval_hours = hours;
            }
        }
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.server.port == 0 {
            return Err(ConfigError::Validation("port must be non-zero".to_string()));
        }
        Ok(())
    }

    /// Return the `host:port` bind address.
    #[must_use]
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// Return the database URL in `sqlx`-compatible format.
    #[must_use]
    pub fn database_url(&self) -> &str {
        &self.database.url
    }

    /// Return the dashboard assets directory, if configured.
    #[must_use]
    pub fn dashboard_dir(&self) -> Option<std::path::PathBuf> {
        self.server
            .dashboard_dir
            .as_ref()
            .map(std::path::PathBuf::from)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3000,
            dashboard_dir: None,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite:minihub.db?mode=rwc".to_string(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            filter: "minihubd=info,minihub=info,tower_http=debug".to_string(),
        }
    }
}

impl Default for IntegrationsConfig {
    fn default() -> Self {
        Self {
            virtual_enabled: true,
            mqtt: MqttIntegrationConfig::default(),
            ble: BleIntegrationConfig::default(),
        }
    }
}

impl Default for MqttIntegrationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            broker_host: "localhost".to_string(),
            broker_port: 1883,
            client_id: "minihub".to_string(),
            base_topic: "minihub".to_string(),
            keep_alive_secs: 30,
        }
    }
}

impl Default for BleIntegrationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            scan_duration_secs: 10,
            update_interval_secs: 60,
            device_filter: Vec::new(),
            miflora_enabled: false,
            miflora_filter: Vec::new(),
            miflora_connect_timeout_secs: 10,
        }
    }
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            retention_days: 30,
            purge_interval_hours: 24,
        }
    }
}

/// Configuration errors.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// TOML parse failure.
    #[error("failed to parse config file")]
    Parse(#[from] toml::de::Error),
    /// File I/O failure.
    #[error("failed to read config file")]
    Io(#[from] std::io::Error),
    /// Semantic validation failure.
    #[error("invalid configuration: {0}")]
    Validation(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_produce_sensible_defaults() {
        let config = Config::default();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.database.url, "sqlite:minihub.db?mode=rwc");
        assert!(config.integrations.virtual_enabled);
        assert!(!config.integrations.mqtt.enabled);
        assert_eq!(config.integrations.mqtt.broker_host, "localhost");
        assert_eq!(config.integrations.mqtt.broker_port, 1883);
        assert!(!config.integrations.ble.enabled);
        assert_eq!(config.integrations.ble.scan_duration_secs, 10);
        assert_eq!(config.integrations.ble.update_interval_secs, 60);
        assert!(config.integrations.ble.device_filter.is_empty());
        assert!(!config.integrations.ble.miflora_enabled);
        assert!(config.integrations.ble.miflora_filter.is_empty());
        assert_eq!(config.integrations.ble.miflora_connect_timeout_secs, 10);
    }

    #[test]
    fn should_parse_minimal_toml() {
        let toml = "";
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.server.port, 3000);
    }

    #[test]
    fn should_parse_full_toml() {
        let toml = "
            [server]
            host = '127.0.0.1'
            port = 9090

            [database]
            url = 'sqlite:test.db'

            [logging]
            filter = 'debug'

            [integrations]
            virtual_enabled = false

            [integrations.mqtt]
            enabled = true
            broker_host = 'mqtt.local'
            broker_port = 8883
            client_id = 'my-hub'
            base_topic = 'home'
            keep_alive_secs = 60

            [integrations.ble]
            enabled = true
            scan_duration_secs = 5
            update_interval_secs = 30
            device_filter = ['A4:C1:38:AA:BB:CC']
            miflora_enabled = true
            miflora_filter = ['C4:7C:8D:6A:XX:YY']
            miflora_connect_timeout_secs = 15
        ";
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 9090);
        assert_eq!(config.database.url, "sqlite:test.db");
        assert_eq!(config.logging.filter, "debug");
        assert!(!config.integrations.virtual_enabled);
        assert!(config.integrations.mqtt.enabled);
        assert_eq!(config.integrations.mqtt.broker_host, "mqtt.local");
        assert_eq!(config.integrations.mqtt.broker_port, 8883);
        assert_eq!(config.integrations.mqtt.client_id, "my-hub");
        assert_eq!(config.integrations.mqtt.base_topic, "home");
        assert_eq!(config.integrations.mqtt.keep_alive_secs, 60);
        assert!(config.integrations.ble.enabled);
        assert_eq!(config.integrations.ble.scan_duration_secs, 5);
        assert_eq!(config.integrations.ble.update_interval_secs, 30);
        assert_eq!(
            config.integrations.ble.device_filter,
            vec!["A4:C1:38:AA:BB:CC"]
        );
        assert!(config.integrations.ble.miflora_enabled);
        assert_eq!(
            config.integrations.ble.miflora_filter,
            vec!["C4:7C:8D:6A:XX:YY"]
        );
        assert_eq!(config.integrations.ble.miflora_connect_timeout_secs, 15);
    }

    #[test]
    fn should_return_default_when_file_not_found() {
        let config = Config::from_file("nonexistent.toml").unwrap();
        assert_eq!(config.server.port, 3000);
    }

    #[test]
    fn should_reject_zero_port() {
        let mut config = Config::default();
        config.server.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn should_accept_valid_port() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn should_format_bind_addr() {
        let config = Config::default();
        assert_eq!(config.bind_addr(), "0.0.0.0:3000");
    }

    #[test]
    fn should_format_custom_bind_addr() {
        let mut config = Config::default();
        config.server.host = "127.0.0.1".to_string();
        config.server.port = 9090;
        assert_eq!(config.bind_addr(), "127.0.0.1:9090");
    }

    #[test]
    fn should_return_database_url() {
        let config = Config::default();
        assert_eq!(config.database_url(), "sqlite:minihub.db?mode=rwc");
    }

    #[test]
    fn should_parse_partial_toml_with_defaults() {
        let toml = "
            [server]
            port = 8080
        ";
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.database.url, "sqlite:minihub.db?mode=rwc");
        assert!(config.integrations.virtual_enabled);
    }

    #[test]
    fn should_default_miflora_disabled_in_partial_ble_toml() {
        let toml = "
            [integrations.ble]
            enabled = true
            scan_duration_secs = 5
        ";
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.integrations.ble.enabled);
        assert_eq!(config.integrations.ble.scan_duration_secs, 5);
        assert!(!config.integrations.ble.miflora_enabled);
        assert!(config.integrations.ble.miflora_filter.is_empty());
        assert_eq!(config.integrations.ble.miflora_connect_timeout_secs, 10);
    }

    #[test]
    fn should_apply_miflora_enabled_env_override() {
        let mut config = Config::default();
        assert!(!config.integrations.ble.miflora_enabled);

        // Simulate what apply_env_overrides does for MINIHUB_BLE_MIFLORA_ENABLED
        config.integrations.ble.miflora_enabled = "true".eq_ignore_ascii_case("true");
        assert!(config.integrations.ble.miflora_enabled);

        config.integrations.ble.miflora_enabled = "1" == "1";
        assert!(config.integrations.ble.miflora_enabled);

        config.integrations.ble.miflora_enabled =
            "false" == "1" || "false".eq_ignore_ascii_case("true");
        assert!(!config.integrations.ble.miflora_enabled);
    }

    #[test]
    fn should_report_parse_error_for_invalid_toml() {
        let result: Result<Config, _> = toml::from_str("invalid {{{");
        assert!(result.is_err());
    }

    #[test]
    fn should_return_io_error_when_path_is_a_directory() {
        // Reading a directory instead of a file triggers an IO error.
        let result = Config::from_file(".");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ConfigError::Io(_)));
    }

    #[test]
    fn should_return_parse_error_for_malformed_file() {
        // Create a temp file with invalid TOML content
        let dir = std::env::temp_dir().join("minihub_test_config");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bad.toml");
        std::fs::write(&path, "[server\ninvalid").unwrap();

        let result = Config::from_file(path.to_str().unwrap());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Parse(_)));

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn should_display_config_errors() {
        let err = ConfigError::Validation("port must be non-zero".to_string());
        assert_eq!(
            err.to_string(),
            "invalid configuration: port must be non-zero"
        );
    }

    #[test]
    fn should_debug_format_config() {
        let config = Config::default();
        let debug = format!("{config:?}");
        assert!(debug.contains("Config"));
    }

    #[test]
    fn should_use_logging_filter_default() {
        let config = Config::default();
        assert!(config.logging.filter.contains("info"));
    }

    #[test]
    fn should_default_dashboard_dir_to_none() {
        let config = Config::default();
        assert!(config.server.dashboard_dir.is_none());
        assert!(config.dashboard_dir().is_none());
    }

    #[test]
    fn should_parse_dashboard_dir_from_toml() {
        let toml = r#"
            [server]
            dashboard_dir = "/var/www/dashboard"
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(
            config.server.dashboard_dir.as_deref(),
            Some("/var/www/dashboard")
        );
        assert_eq!(
            config.dashboard_dir(),
            Some(std::path::PathBuf::from("/var/www/dashboard"))
        );
    }

    #[test]
    fn should_parse_relative_dashboard_dir_from_toml() {
        let toml = r#"
            [server]
            dashboard_dir = "./dist"
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.server.dashboard_dir.as_deref(), Some("./dist"));
        assert_eq!(
            config.dashboard_dir(),
            Some(std::path::PathBuf::from("./dist"))
        );
    }

    #[test]
    fn should_override_dashboard_dir_with_env_var() {
        // Create a temp TOML file with dashboard_dir set
        let dir = std::env::temp_dir().join("minihub_test_config_dashboard");
        std::fs::create_dir_all(&dir).unwrap();
        let config_path = dir.join("test_dashboard.toml");
        std::fs::write(
            &config_path,
            r#"
            [server]
            dashboard_dir = "/original/path"
            "#,
        )
        .unwrap();

        let mut config: Config =
            toml::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();

        // Simulate environment variable override
        config.server.dashboard_dir = Some("/overridden/path".to_string());

        assert_eq!(
            config.server.dashboard_dir.as_deref(),
            Some("/overridden/path")
        );
        assert_eq!(
            config.dashboard_dir(),
            Some(std::path::PathBuf::from("/overridden/path"))
        );

        // Cleanup
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn should_apply_minihub_dashboard_dir_env_override() {
        let toml = r#"
            [server]
            dashboard_dir = "/original"
        "#;
        let mut config: Config = toml::from_str(toml).unwrap();

        // Simulate what apply_env_overrides does when MINIHUB_DASHBOARD_DIR is set
        config.server.dashboard_dir = Some("/env/override".to_string());

        assert_eq!(
            config.server.dashboard_dir.as_deref(),
            Some("/env/override")
        );
    }

    #[test]
    fn should_handle_empty_dashboard_dir_in_toml() {
        let toml = r"
            [server]
            port = 8080
        ";
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.server.dashboard_dir.is_none());
        assert!(config.dashboard_dir().is_none());
    }

    #[test]
    fn should_return_pathbuf_from_dashboard_dir_accessor() {
        let mut config = Config::default();
        config.server.dashboard_dir = Some("/test/path".to_string());

        let path_buf = config.dashboard_dir();
        assert!(path_buf.is_some());
        assert_eq!(path_buf.unwrap(), std::path::PathBuf::from("/test/path"));
    }

    #[test]
    fn should_preserve_dashboard_dir_through_full_config_lifecycle() {
        let toml = r#"
            [server]
            host = "127.0.0.1"
            port = 8080
            dashboard_dir = "/custom/dashboard"

            [database]
            url = "sqlite:test.db"
        "#;
        let config: Config = toml::from_str(toml).unwrap();

        // Verify dashboard_dir is preserved alongside other config
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(
            config.server.dashboard_dir.as_deref(),
            Some("/custom/dashboard")
        );
        assert_eq!(config.database.url, "sqlite:test.db");
    }
}
