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
