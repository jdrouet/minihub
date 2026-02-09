//! # minihub-adapter-ble
//!
//! Passive BLE adapter — scans for Xiaomi LYWSD03MMC thermometer/hygrometer
//! sensors running ATC/PVVX custom firmware and exposes them as minihub
//! devices and entities.
//!
//! ## How it works
//!
//! The ATC/PVVX firmware broadcasts sensor data (temperature, humidity,
//! battery) as BLE service-data advertisements on UUID `0x181A`. This adapter
//! passively scans for those advertisements — no BLE connection is needed.
//!
//! ## Supported formats
//!
//! | Format | UUID | Payload length | Endianness |
//! |--------|------|----------------|------------|
//! | PVVX custom | `0x181A` | 19 bytes | Little-endian |
//! | ATC1441 original | `0x181A` | 16 bytes | Big-endian |
//!
//! ## Dependency rule
//!
//! Same as other adapters: depends on `minihub-app` and `minihub-domain`.

mod config;
mod error;
pub mod parser;

pub use config::BleConfig;
pub use error::BleError;
