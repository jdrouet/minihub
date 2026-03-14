//! Supported BLE device handlers.
//!
//! Each supported sensor type implements [`BleDeviceHandler`], which defines
//! how to parse passive advertisements and optionally perform post-scan
//! active GATT reads.

pub(crate) mod lywsd03mmc;
mod miflora;

pub(crate) use lywsd03mmc::Lywsd03mmcHandler;
pub(crate) use miflora::MifloraHandler;

// Re-exports used by lib.rs for service-call handling.
pub(crate) use miflora::{blink_miflora, parse_mibeacon_mac};

use std::future::Future;

use minihub_app::ports::integration::DiscoveredDevice;

use crate::error::BleError;

/// A supported BLE device type that the scanner knows how to handle.
///
/// Implementors define how to identify their devices from advertisements
/// and optionally how to perform active GATT reads after the scan completes.
pub(crate) trait BleDeviceHandler: Send + Sync {
    /// Human-readable name for logging (e.g. `"LYWSD03MMC"`, `"Mi Flora"`).
    fn name(&self) -> &'static str;

    /// Try to parse a passive service-data advertisement.
    ///
    /// Returns `Ok(Some(dd))` when the advertisement is recognised and parsed
    /// (including passing the MAC filter), `Ok(None)` when the advertisement
    /// does not belong to this handler, or `Err` on a parse failure for a
    /// claimed advertisement.
    fn try_parse_advertisement(
        &self,
        uuid: uuid::Uuid,
        data: &[u8],
    ) -> Result<Option<DiscoveredDevice>, BleError>;

    /// Perform post-scan active work (e.g. GATT connections).
    ///
    /// Called once after the passive scan completes. Returns discovered
    /// devices; errors are logged internally. The default implementation
    /// is a no-op (suitable for passive-only devices).
    fn process_after_scan(
        &self,
        _adapter: &btleplug::platform::Adapter,
    ) -> impl Future<Output = Vec<DiscoveredDevice>> + Send {
        async { Vec::new() }
    }
}
