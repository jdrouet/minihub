//! Mi Flora (HHCCJCY01) payload parsers and domain mapping.
//!
//! Mi Flora plant sensors require active GATT connections to read data.
//! This module provides:
//!
//! - GATT characteristic UUID constants (`CMD_CHAR`, `DATA_CHAR`, `FIRMWARE_CHAR`)
//! - Pure parser functions for the 16-byte sensor DATA and 7-byte FIRMWARE payloads
//! - A [`build_discovered`] function to map a [`MifloraReading`] into domain types

use minihub_app::ports::integration::DiscoveredDevice;
use minihub_domain::device::Device;
use minihub_domain::entity::{AttributeValue, Entity, EntityState};
use minihub_domain::error::MiniHubError;

use crate::error::{BleError, PayloadParseError};
use crate::parser;

/// GATT characteristic UUID for the Mi Flora CMD register (write `[0xa0, 0x1f]` to activate).
pub const CMD_CHAR: uuid::Uuid = uuid::Uuid::from_u128(0x0000_1a00_0000_1000_8000_0080_5f9b_34fb);

/// GATT characteristic UUID for the Mi Flora DATA register (16-byte sensor payload).
pub const DATA_CHAR: uuid::Uuid = uuid::Uuid::from_u128(0x0000_1a01_0000_1000_8000_0080_5f9b_34fb);

/// GATT characteristic UUID for the Mi Flora FIRMWARE register (7-byte battery + version).
pub const FIRMWARE_CHAR: uuid::Uuid =
    uuid::Uuid::from_u128(0x0000_1a02_0000_1000_8000_0080_5f9b_34fb);

const MIBEACON_MIN_LEN: usize = 13;
const MIBEACON_MAC_OFFSET: usize = 7;
const DATA_LEN: usize = 16;
const FIRMWARE_LEN: usize = 7;

/// Parsed sensor data from the 16-byte DATA characteristic.
#[derive(Debug, Clone, PartialEq)]
pub struct MifloraSensorData {
    /// Temperature in degrees Celsius.
    pub temperature: f64,
    /// Light intensity in lux.
    pub light: u32,
    /// Soil moisture percentage (0–100).
    pub moisture: u8,
    /// Soil conductivity in µS/cm.
    pub conductivity: u16,
}

/// Parsed firmware info from the 7-byte FIRMWARE characteristic.
#[derive(Debug, Clone, PartialEq)]
pub struct MifloraFirmware {
    /// Battery level percentage (0–100).
    pub battery_level: u8,
    /// Firmware version string (e.g. `"3.1.8"`).
    pub firmware_version: String,
}

/// Combined reading from a single Mi Flora device.
#[derive(Debug, Clone, PartialEq)]
pub struct MifloraReading {
    /// Device MAC address (6 bytes).
    pub mac: [u8; 6],
    /// Sensor data from the DATA characteristic.
    pub sensor: MifloraSensorData,
    /// Firmware info from the FIRMWARE characteristic.
    pub firmware: MifloraFirmware,
}

/// Extract the 6-byte MAC address from a `MiBeacon` `0xFE95` service data payload.
///
/// The `MiBeacon` protocol stores the MAC at bytes 7–12 in reverse order.
/// On macOS, `peripheral.address()` returns a zeroed address, so this
/// function provides a reliable cross-platform alternative.
///
/// # Errors
///
/// Returns [`BleError::PayloadParse`] when the payload is shorter than 13 bytes.
pub fn parse_mibeacon_mac(data: &[u8]) -> Result<[u8; 6], BleError> {
    if data.len() < MIBEACON_MIN_LEN {
        return Err(BleError::PayloadParse(PayloadParseError::WrongLength {
            format: "MiBeacon",
            expected: MIBEACON_MIN_LEN,
            actual: data.len(),
        }));
    }

    let raw = &data[MIBEACON_MAC_OFFSET..MIBEACON_MAC_OFFSET + 6];
    Ok([raw[5], raw[4], raw[3], raw[2], raw[1], raw[0]])
}

/// Parse the 16-byte DATA characteristic payload (little-endian).
///
/// | Bytes | Type | Field |
/// |-------|------|-------|
/// | 0–1 | i16 LE (×0.1 °C) | Temperature |
/// | 2 | — | Padding |
/// | 3–6 | u32 LE (lux) | Light |
/// | 7 | u8 (%) | Moisture |
/// | 8–9 | u16 LE (µS/cm) | Conductivity |
/// | 10–15 | — | Reserved |
///
/// # Errors
///
/// Returns [`BleError::PayloadParse`] when the slice length is not 16.
pub fn parse_sensor_data(data: &[u8]) -> Result<MifloraSensorData, BleError> {
    if data.len() != DATA_LEN {
        return Err(BleError::PayloadParse(PayloadParseError::WrongLength {
            format: "Mi Flora DATA",
            expected: DATA_LEN,
            actual: data.len(),
        }));
    }

    let temp_raw = i16::from_le_bytes([data[0], data[1]]);
    let light = u32::from_le_bytes([data[3], data[4], data[5], data[6]]);
    let moisture = data[7];
    let conductivity = u16::from_le_bytes([data[8], data[9]]);

    Ok(MifloraSensorData {
        temperature: f64::from(temp_raw) * 0.1,
        light,
        moisture,
        conductivity,
    })
}

/// Parse the 7-byte FIRMWARE characteristic payload.
///
/// | Bytes | Type | Field |
/// |-------|------|-------|
/// | 0 | u8 (%) | Battery level |
/// | 1 | — | Separator |
/// | 2–6 | ASCII | Firmware version |
///
/// # Errors
///
/// Returns [`BleError::PayloadParse`] when the slice length is not 7.
pub fn parse_firmware(data: &[u8]) -> Result<MifloraFirmware, BleError> {
    if data.len() != FIRMWARE_LEN {
        return Err(BleError::PayloadParse(PayloadParseError::WrongLength {
            format: "Mi Flora FIRMWARE",
            expected: FIRMWARE_LEN,
            actual: data.len(),
        }));
    }

    let battery_level = data[0];
    let firmware_version = String::from_utf8_lossy(&data[2..7]).into_owned();

    Ok(MifloraFirmware {
        battery_level,
        firmware_version,
    })
}

/// Build a [`DiscoveredDevice`] from a [`MifloraReading`].
///
/// Creates a device named `"Mi Flora {MAC}"` with manufacturer `"Xiaomi"`,
/// model `"HHCCJCY01"`, and a single entity carrying all sensor attributes.
///
/// # Errors
///
/// Returns [`MiniHubError`] if domain validation fails.
pub fn build_discovered(reading: &MifloraReading) -> Result<DiscoveredDevice, MiniHubError> {
    let mac_str = parser::format_mac(reading.mac);
    let slug = parser::mac_slug(reading.mac);

    let device = Device::builder()
        .name(format!("Mi Flora {mac_str}"))
        .manufacturer("Xiaomi")
        .model("HHCCJCY01")
        .integration("ble")
        .unique_id(&mac_str)
        .build()?;

    let entity = Entity::builder()
        .device_id(device.id)
        .entity_id(format!("sensor.miflora_{slug}"))
        .friendly_name(format!("Mi Flora {mac_str}"))
        .state(EntityState::On)
        .attribute(
            "temperature",
            AttributeValue::Float(reading.sensor.temperature),
        )
        .attribute(
            "light",
            AttributeValue::Int(i64::from(reading.sensor.light)),
        )
        .attribute(
            "moisture",
            AttributeValue::Int(i64::from(reading.sensor.moisture)),
        )
        .attribute(
            "conductivity",
            AttributeValue::Int(i64::from(reading.sensor.conductivity)),
        )
        .attribute(
            "battery_level",
            AttributeValue::Int(i64::from(reading.firmware.battery_level)),
        )
        .attribute(
            "firmware",
            AttributeValue::String(reading.firmware.firmware_version.clone()),
        )
        .build()?;

    Ok(DiscoveredDevice {
        device,
        entities: vec![entity],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data_payload() -> [u8; 16] {
        let mut data = [0u8; 16];
        // temp: 201 (0x00C9) LE → 20.1 °C
        data[0] = 0xC9;
        data[1] = 0x00;
        // padding byte 2
        // light: 82_386 (0x000141D2) LE → bytes [0xD2, 0x41, 0x01, 0x00]
        data[3] = 0xD2;
        data[4] = 0x41;
        data[5] = 0x01;
        data[6] = 0x00;
        // moisture: 56%
        data[7] = 0x38;
        // conductivity: 1561 (0x0619) LE → [0x19, 0x06]
        data[8] = 0x19;
        data[9] = 0x06;
        data
    }

    fn sample_firmware_payload() -> [u8; 7] {
        [
            0x63, // battery: 99%
            0x13, // separator
            b'3', b'.', b'1', b'.', b'8', // firmware "3.1.8"
        ]
    }

    fn sample_reading() -> MifloraReading {
        MifloraReading {
            mac: [0xC4, 0x7C, 0x8D, 0x6A, 0x12, 0x34],
            sensor: MifloraSensorData {
                temperature: 20.1,
                light: 82_386,
                moisture: 56,
                conductivity: 1561,
            },
            firmware: MifloraFirmware {
                battery_level: 99,
                firmware_version: "3.1.8".to_owned(),
            },
        }
    }

    // ── Sensor data parsing ─────────────────────────────────────────────

    #[test]
    fn should_parse_sensor_data_with_positive_temperature() {
        let data = sample_data_payload();
        let result = parse_sensor_data(&data).unwrap();
        assert!((result.temperature - 20.1).abs() < 0.01);
        assert_eq!(result.light, 82_386);
        assert_eq!(result.moisture, 56);
        assert_eq!(result.conductivity, 1561);
    }

    #[test]
    fn should_parse_sensor_data_with_negative_temperature() {
        let mut data = [0u8; 16];
        // temp: -32 (0xFFE0) LE → -3.2 °C
        data[0] = 0xE0;
        data[1] = 0xFF;
        data[7] = 10; // moisture
        let result = parse_sensor_data(&data).unwrap();
        assert!((result.temperature - (-3.2)).abs() < 0.01);
        assert_eq!(result.moisture, 10);
    }

    #[test]
    fn should_parse_sensor_data_with_zero_values() {
        let data = [0u8; 16];
        let result = parse_sensor_data(&data).unwrap();
        assert!((result.temperature).abs() < 0.01);
        assert_eq!(result.light, 0);
        assert_eq!(result.moisture, 0);
        assert_eq!(result.conductivity, 0);
    }

    #[test]
    fn should_parse_sensor_data_with_max_light() {
        let mut data = [0u8; 16];
        // max u32 in bytes 3-6
        data[3] = 0xFF;
        data[4] = 0xFF;
        data[5] = 0xFF;
        data[6] = 0xFF;
        let result = parse_sensor_data(&data).unwrap();
        assert_eq!(result.light, u32::MAX);
    }

    #[test]
    fn should_parse_sensor_data_with_max_moisture() {
        let mut data = [0u8; 16];
        data[7] = 100;
        let result = parse_sensor_data(&data).unwrap();
        assert_eq!(result.moisture, 100);
    }

    #[test]
    fn should_parse_sensor_data_with_max_conductivity() {
        let mut data = [0u8; 16];
        data[8] = 0xFF;
        data[9] = 0xFF;
        let result = parse_sensor_data(&data).unwrap();
        assert_eq!(result.conductivity, u16::MAX);
    }

    #[test]
    fn should_reject_sensor_data_wrong_length() {
        let data = [0u8; 10];
        let err = parse_sensor_data(&data).unwrap_err();
        let source = std::error::Error::source(&err).unwrap();
        assert!(source.to_string().contains("16 bytes"));
    }

    // ── Firmware parsing ────────────────────────────────────────────────

    #[test]
    fn should_parse_firmware_payload() {
        let data = sample_firmware_payload();
        let result = parse_firmware(&data).unwrap();
        assert_eq!(result.battery_level, 99);
        assert_eq!(result.firmware_version, "3.1.8");
    }

    #[test]
    fn should_reject_firmware_wrong_length() {
        let data = [0u8; 3];
        let err = parse_firmware(&data).unwrap_err();
        let source = std::error::Error::source(&err).unwrap();
        assert!(source.to_string().contains("7 bytes"));
    }

    // ── UUID constants ──────────────────────────────────────────────────

    #[test]
    fn should_have_correct_gatt_uuids() {
        assert!(CMD_CHAR.to_string().contains("00001a00"));
        assert!(DATA_CHAR.to_string().contains("00001a01"));
        assert!(FIRMWARE_CHAR.to_string().contains("00001a02"));
    }

    // ── MiBeacon MAC parsing ───────────────────────────────────────────

    #[test]
    fn should_parse_mibeacon_mac_from_service_data() {
        // MiBeacon payload: 2 bytes frame ctrl, 2 bytes product id, 1 byte counter,
        // 2 bytes padding, then 6 bytes MAC reversed
        // MAC C4:7C:8D:6A:12:34 → reversed in payload: [0x34, 0x12, 0x6A, 0x8D, 0x7C, 0xC4]
        let data: [u8; 13] = [
            0x71, 0x20, // frame control
            0x98, 0x00, // product id
            0x03, // frame counter
            0x00, 0x00, // padding
            0x34, 0x12, 0x6A, 0x8D, 0x7C, 0xC4, // MAC reversed
        ];
        let mac = parse_mibeacon_mac(&data).unwrap();
        assert_eq!(mac, [0xC4, 0x7C, 0x8D, 0x6A, 0x12, 0x34]);
    }

    #[test]
    fn should_parse_mibeacon_mac_from_longer_payload() {
        let mut data = [0u8; 20];
        data[7] = 0xDF;
        data[8] = 0x0E;
        data[9] = 0x5B;
        data[10] = 0x38;
        data[11] = 0xC1;
        data[12] = 0xA4;
        let mac = parse_mibeacon_mac(&data).unwrap();
        assert_eq!(mac, [0xA4, 0xC1, 0x38, 0x5B, 0x0E, 0xDF]);
    }

    #[test]
    fn should_reject_mibeacon_too_short() {
        let data = [0u8; 10];
        let err = parse_mibeacon_mac(&data).unwrap_err();
        let source = std::error::Error::source(&err).unwrap();
        assert!(source.to_string().contains("13 bytes"));
    }

    // ── build_discovered ────────────────────────────────────────────────

    #[test]
    fn should_build_discovered_device_from_reading() {
        let reading = sample_reading();
        let dd = build_discovered(&reading).unwrap();

        assert_eq!(dd.device.name, "Mi Flora C4:7C:8D:6A:12:34");
        assert_eq!(dd.device.manufacturer.as_deref(), Some("Xiaomi"));
        assert_eq!(dd.device.model.as_deref(), Some("HHCCJCY01"));
        assert_eq!(dd.device.integration, "ble");
        assert_eq!(dd.device.unique_id, "C4:7C:8D:6A:12:34");

        assert_eq!(dd.entities.len(), 1);
        let entity = &dd.entities[0];
        assert_eq!(entity.entity_id, "sensor.miflora_c47c8d6a1234");
        assert_eq!(entity.friendly_name, "Mi Flora C4:7C:8D:6A:12:34");
        assert_eq!(entity.state, EntityState::On);

        assert_eq!(
            entity.get_attribute("temperature"),
            Some(&AttributeValue::Float(20.1))
        );
        assert_eq!(
            entity.get_attribute("light"),
            Some(&AttributeValue::Int(82_386))
        );
        assert_eq!(
            entity.get_attribute("moisture"),
            Some(&AttributeValue::Int(56))
        );
        assert_eq!(
            entity.get_attribute("conductivity"),
            Some(&AttributeValue::Int(1561))
        );
        assert_eq!(
            entity.get_attribute("battery_level"),
            Some(&AttributeValue::Int(99))
        );
        assert_eq!(
            entity.get_attribute("firmware"),
            Some(&AttributeValue::String("3.1.8".to_owned()))
        );
    }
}
