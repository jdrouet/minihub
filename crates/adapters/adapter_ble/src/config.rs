//! BLE integration configuration.

use serde::Deserialize;

/// Configuration for the passive BLE integration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct BleConfig {
    /// How long to scan for advertisements during `setup()`, in seconds.
    pub scan_duration_secs: u16,
    /// Interval between background re-scans, in seconds.
    pub update_interval_secs: u16,
    /// Optional MAC address allowlist (e.g. `["A4:C1:38:AA:BB:CC"]`).
    ///
    /// When empty, all detected LYWSD03MMC sensors are accepted.
    pub device_filter: Vec<String>,
}

impl Default for BleConfig {
    fn default() -> Self {
        Self {
            scan_duration_secs: 10,
            update_interval_secs: 60,
            device_filter: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_have_sensible_defaults() {
        let config = BleConfig::default();
        assert_eq!(config.scan_duration_secs, 10);
        assert_eq!(config.update_interval_secs, 60);
        assert!(config.device_filter.is_empty());
    }

    #[test]
    fn should_deserialize_from_toml() {
        let toml = r#"
            scan_duration_secs = 20
            update_interval_secs = 120
            device_filter = ["A4:C1:38:AA:BB:CC", "A4:C1:38:DD:EE:FF"]
        "#;
        let config: BleConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.scan_duration_secs, 20);
        assert_eq!(config.update_interval_secs, 120);
        assert_eq!(config.device_filter.len(), 2);
        assert_eq!(config.device_filter[0], "A4:C1:38:AA:BB:CC");
    }

    #[test]
    fn should_use_defaults_for_missing_fields() {
        let toml = r"scan_duration_secs = 5";
        let config: BleConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.scan_duration_secs, 5);
        assert_eq!(config.update_interval_secs, 60);
        assert!(config.device_filter.is_empty());
    }

    #[test]
    fn should_deserialize_empty_toml() {
        let config: BleConfig = toml::from_str("").unwrap();
        assert_eq!(config.scan_duration_secs, 10);
    }
}
