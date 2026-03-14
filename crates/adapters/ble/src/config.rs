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
    /// Enable active GATT readout for Mi Flora plant sensors.
    pub miflora_enabled: bool,
    /// Optional MAC allowlist for Mi Flora devices.
    ///
    /// When empty, all detected Mi Flora sensors are accepted.
    pub miflora_filter: Vec<String>,
    /// Per-device GATT connection timeout, in seconds.
    pub miflora_connect_timeout_secs: u16,
}

impl Default for BleConfig {
    fn default() -> Self {
        Self {
            scan_duration_secs: 10,
            update_interval_secs: 60,
            device_filter: Vec::new(),
            miflora_enabled: false,
            miflora_filter: Vec::new(),
            miflora_connect_timeout_secs: 10,
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
        assert!(!config.miflora_enabled);
        assert!(config.miflora_filter.is_empty());
        assert_eq!(config.miflora_connect_timeout_secs, 10);
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
    fn should_deserialize_miflora_fields_from_toml() {
        let toml = r#"
            miflora_enabled = true
            miflora_filter = ["C4:7C:8D:6A:XX:YY"]
            miflora_connect_timeout_secs = 15
        "#;
        let config: BleConfig = toml::from_str(toml).unwrap();
        assert!(config.miflora_enabled);
        assert_eq!(config.miflora_filter, vec!["C4:7C:8D:6A:XX:YY"]);
        assert_eq!(config.miflora_connect_timeout_secs, 15);
    }

    #[test]
    fn should_use_defaults_for_missing_fields() {
        let toml = r"scan_duration_secs = 5";
        let config: BleConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.scan_duration_secs, 5);
        assert_eq!(config.update_interval_secs, 60);
        assert!(config.device_filter.is_empty());
        assert!(!config.miflora_enabled);
        assert!(config.miflora_filter.is_empty());
        assert_eq!(config.miflora_connect_timeout_secs, 10);
    }

    #[test]
    fn should_deserialize_empty_toml() {
        let config: BleConfig = toml::from_str("").unwrap();
        assert_eq!(config.scan_duration_secs, 10);
        assert!(!config.miflora_enabled);
    }
}
