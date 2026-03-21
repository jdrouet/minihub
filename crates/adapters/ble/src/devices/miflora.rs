//! Xiaomi Mi Flora (HHCCJCY01) plant sensor handler.
//!
//! Mi Flora sensors require active GATT connections to read data. This module
//! provides payload parsers, GATT readout logic, and the [`MifloraHandler`]
//! that implements [`BleDeviceHandler`].

use std::time::Duration;

use btleplug::api::{Central, Characteristic, Peripheral as _, WriteType};
use btleplug::platform::{Adapter, Peripheral};

use minihub_app::ports::integration::DiscoveredDevice;
use minihub_domain::device::Device;
use minihub_domain::entity::{AttributeValue, Entity, EntityState};
use minihub_domain::error::MiniHubError;

use crate::error::{BleError, PayloadParseError};
use crate::parser::{self, ServiceUuid};

use super::BleDeviceHandler;

// GATT characteristic UUIDs

/// GATT characteristic UUID for the Mi Flora CMD register (write `[0xa0, 0x1f]` to activate).
const CMD_CHAR: uuid::Uuid = uuid::Uuid::from_u128(0x0000_1a00_0000_1000_8000_0080_5f9b_34fb);

/// GATT characteristic UUID for the Mi Flora DATA register (16-byte sensor payload).
const DATA_CHAR: uuid::Uuid = uuid::Uuid::from_u128(0x0000_1a01_0000_1000_8000_0080_5f9b_34fb);

/// GATT characteristic UUID for the Mi Flora FIRMWARE register (7-byte battery + version).
const FIRMWARE_CHAR: uuid::Uuid = uuid::Uuid::from_u128(0x0000_1a02_0000_1000_8000_0080_5f9b_34fb);

/// Command bytes to write to the CMD characteristic to activate sensor mode.
const ACTIVATE_CMD: &[u8] = &[0xa0, 0x1f];

/// Command bytes to write to the CMD characteristic to blink the LED.
const BLINK_CMD: &[u8] = &[0xfd, 0xff];

/// The local name advertised by Mi Flora peripherals.
const MIFLORA_LOCAL_NAME: &str = "Flower care";

// Payload constants

const MIBEACON_MAC_OFFSET: usize = 5;
const MIBEACON_MIN_LEN: usize = MIBEACON_MAC_OFFSET + 6;
const MIBEACON_FC_MAC_INCLUDED: u16 = 0x0010;
const DATA_LEN: usize = 16;
const FIRMWARE_LEN: usize = 7;

// Data types

/// Parsed sensor data from the 16-byte DATA characteristic.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MifloraSensorData {
    pub temperature: f64,
    pub light: u32,
    pub moisture: u8,
    pub conductivity: u16,
}

/// Parsed firmware info from the 7-byte FIRMWARE characteristic.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MifloraFirmware {
    pub battery_level: u8,
    pub firmware_version: String,
}

/// Combined reading from a single Mi Flora device.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MifloraReading {
    pub mac: [u8; 6],
    pub sensor: MifloraSensorData,
    pub firmware: MifloraFirmware,
}

// Handler

/// Handler for Xiaomi Mi Flora plant sensors.
pub(crate) struct MifloraHandler {
    filter: Vec<String>,
    connect_timeout: Duration,
}

impl MifloraHandler {
    pub(crate) fn new(filter: Vec<String>, connect_timeout: Duration) -> Self {
        Self {
            filter,
            connect_timeout,
        }
    }

    fn passes_filter(&self, mac: &str) -> bool {
        if self.filter.is_empty() {
            return true;
        }
        self.filter.iter().any(|f| f.eq_ignore_ascii_case(mac))
    }
}

impl BleDeviceHandler for MifloraHandler {
    fn name(&self) -> &'static str {
        "Mi Flora"
    }

    fn try_parse_advertisement(
        &self,
        _uuid: uuid::Uuid,
        _data: &[u8],
    ) -> Result<Option<DiscoveredDevice>, BleError> {
        // Mi Flora uses active GATT, not passive advertisements.
        Ok(None)
    }

    async fn process_after_scan(&self, adapter: &Adapter) -> Vec<DiscoveredDevice> {
        let peripherals = match adapter.peripherals().await {
            Ok(list) => list,
            Err(err) => {
                tracing::warn!(%err, "failed to list peripherals for Mi Flora readout");
                return Vec::new();
            }
        };

        let mut discovered = Vec::new();

        for peripheral in &peripherals {
            let Ok(Some(props)) = peripheral.properties().await else {
                continue;
            };

            let name_matches = props
                .local_name
                .as_deref()
                .is_some_and(|name| name == MIFLORA_LOCAL_NAME);
            if !name_matches {
                continue;
            }

            let Some(mibeacon_data) = props.service_data.get(&ServiceUuid::MIFLORA) else {
                tracing::debug!("Mi Flora peripheral has no 0xFE95 service data, skipping");
                continue;
            };

            let mac_bytes = match parse_mibeacon_mac(mibeacon_data) {
                Ok(mac) => mac,
                Err(err) => {
                    tracing::warn!(%err, "failed to parse Mi Flora MAC from MiBeacon payload");
                    continue;
                }
            };

            let mac_str = parser::format_mac(mac_bytes);
            if !self.passes_filter(&mac_str) {
                tracing::debug!(mac = %mac_str, "Mi Flora filtered out by miflora_filter");
                continue;
            }

            tracing::debug!(mac = %mac_str, "reading Mi Flora sensor via GATT");

            let result =
                tokio::time::timeout(self.connect_timeout, read_miflora(peripheral, mac_bytes))
                    .await;

            let reading = match result {
                Ok(Ok(reading)) => reading,
                Ok(Err(err)) => {
                    tracing::warn!(%err, mac = %mac_str, "failed to read Mi Flora device");
                    continue;
                }
                Err(_) => {
                    tracing::warn!(mac = %mac_str, "Mi Flora GATT readout timed out");
                    continue;
                }
            };

            match build_discovered(&reading) {
                Ok(dd) => discovered.push(dd),
                Err(err) => {
                    tracing::warn!(%err, mac = %mac_str, "failed to build Mi Flora discovered device");
                }
            }
        }

        discovered
    }
}

// Payload parsers

/// Extract the 6-byte MAC address from a `MiBeacon` `0xFE95` service data payload.
///
/// The `MiBeacon` v2 frame layout (after `btleplug` strips the UUID) is:
///
/// | Offset | Length | Field |
/// |--------|--------|-------|
/// | 0 | 2 | Frame Control (LE, bit 4 = MAC included) |
/// | 2 | 2 | Product ID (LE) |
/// | 4 | 1 | Frame Counter |
/// | 5 | 6 | MAC Address (reversed byte order, optional) |
/// | 11 | … | Capability / Object data (optional) |
///
/// On macOS, `peripheral.address()` returns a zeroed address, so this
/// function provides a reliable cross-platform alternative.
///
/// # Errors
///
/// Returns [`BleError::PayloadParse`] when the payload is shorter than 11
/// bytes or the Frame Control MAC-included flag (bit 4) is not set.
pub(crate) fn parse_mibeacon_mac(data: &[u8]) -> Result<[u8; 6], BleError> {
    if data.len() < MIBEACON_MIN_LEN {
        return Err(BleError::PayloadParse(PayloadParseError::WrongLength {
            format: "MiBeacon",
            expected: MIBEACON_MIN_LEN,
            actual: data.len(),
        }));
    }

    let frame_control = u16::from_le_bytes([data[0], data[1]]);
    if frame_control & MIBEACON_FC_MAC_INCLUDED == 0 {
        return Err(BleError::PayloadParse(PayloadParseError::MissingField {
            format: "MiBeacon",
            field: "MAC address",
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
fn parse_sensor_data(data: &[u8]) -> Result<MifloraSensorData, BleError> {
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
fn parse_firmware(data: &[u8]) -> Result<MifloraFirmware, BleError> {
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

// Domain mapping

/// Build a [`DiscoveredDevice`] from a [`MifloraReading`].
fn build_discovered(reading: &MifloraReading) -> Result<DiscoveredDevice, MiniHubError> {
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
        .mac_address(&mac_str)
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

// GATT operations

/// Find a GATT characteristic by UUID on a peripheral that has already
/// discovered its services.
fn find_characteristic(
    peripheral: &Peripheral,
    uuid: uuid::Uuid,
) -> Result<Characteristic, BleError> {
    peripheral
        .characteristics()
        .into_iter()
        .find(|c| c.uuid == uuid)
        .ok_or(BleError::CharacteristicNotFound { uuid })
}

/// Connect to a Mi Flora peripheral, read sensor data and firmware info,
/// and return a [`MifloraReading`].
///
/// The connection is always closed on return, even if a read fails. The
/// caller is responsible for applying a per-device timeout around this
/// function.
async fn read_miflora(peripheral: &Peripheral, mac: [u8; 6]) -> Result<MifloraReading, BleError> {
    peripheral.connect().await.map_err(BleError::GattConnect)?;

    let result = read_miflora_inner(peripheral, mac).await;

    if let Err(err) = peripheral.disconnect().await {
        tracing::warn!(%err, "failed to disconnect Mi Flora peripheral");
    }

    result
}

async fn read_miflora_inner(
    peripheral: &Peripheral,
    mac: [u8; 6],
) -> Result<MifloraReading, BleError> {
    peripheral.discover_services().await?;

    let cmd_char = find_characteristic(peripheral, CMD_CHAR)?;
    let data_char = find_characteristic(peripheral, DATA_CHAR)?;
    let firmware_char = find_characteristic(peripheral, FIRMWARE_CHAR)?;

    peripheral
        .write(&cmd_char, ACTIVATE_CMD, WriteType::WithResponse)
        .await?;

    let data_bytes = peripheral.read(&data_char).await?;
    let firmware_bytes = peripheral.read(&firmware_char).await?;

    let sensor = parse_sensor_data(&data_bytes)?;
    let firmware = parse_firmware(&firmware_bytes)?;

    Ok(MifloraReading {
        mac,
        sensor,
        firmware,
    })
}

/// Connect to a Mi Flora peripheral, write the blink LED command, and
/// disconnect.
///
/// The connection is always closed on return, even if the write fails.
pub(crate) async fn blink_miflora(peripheral: &Peripheral) -> Result<(), BleError> {
    peripheral.connect().await.map_err(BleError::GattConnect)?;

    let result = blink_miflora_inner(peripheral).await;

    if let Err(err) = peripheral.disconnect().await {
        tracing::warn!(%err, "failed to disconnect Mi Flora peripheral after blink");
    }

    result
}

async fn blink_miflora_inner(peripheral: &Peripheral) -> Result<(), BleError> {
    peripheral.discover_services().await?;

    let cmd_char = find_characteristic(peripheral, CMD_CHAR)?;

    peripheral
        .write(&cmd_char, BLINK_CMD, WriteType::WithResponse)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data_payload() -> [u8; 16] {
        let mut data = [0u8; 16];
        data[0] = 0xC9;
        data[1] = 0x00;
        data[3] = 0xD2;
        data[4] = 0x41;
        data[5] = 0x01;
        data[6] = 0x00;
        data[7] = 0x38;
        data[8] = 0x19;
        data[9] = 0x06;
        data
    }

    fn sample_firmware_payload() -> [u8; 7] {
        [
            0x63, // battery: 99%
            0x13, // separator
            b'3', b'.', b'1', b'.', b'8',
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

    // Sensor data parsing

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
        data[0] = 0xE0;
        data[1] = 0xFF;
        data[7] = 10;
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

    // Firmware parsing

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

    // UUID constants

    #[test]
    fn should_have_correct_gatt_uuids() {
        assert!(CMD_CHAR.to_string().contains("00001a00"));
        assert!(DATA_CHAR.to_string().contains("00001a01"));
        assert!(FIRMWARE_CHAR.to_string().contains("00001a02"));
    }

    #[test]
    fn should_have_correct_activate_command() {
        assert_eq!(ACTIVATE_CMD, &[0xa0, 0x1f]);
    }

    #[test]
    fn should_have_correct_blink_command() {
        assert_eq!(BLINK_CMD, &[0xfd, 0xff]);
    }

    // MiBeacon MAC parsing

    #[test]
    fn should_parse_mibeacon_mac_from_service_data() {
        // Frame Control 0x2071 (bit 4 set = MAC included), Product ID 0x0098, Counter 0x03
        // MAC reversed at offset 5: [0x34, 0x12, 0x6A, 0x8D, 0x7C, 0xC4]
        let data: [u8; 11] = [
            0x71, 0x20, 0x98, 0x00, 0x03, 0x34, 0x12, 0x6A, 0x8D, 0x7C, 0xC4,
        ];
        let mac = parse_mibeacon_mac(&data).unwrap();
        assert_eq!(mac, [0xC4, 0x7C, 0x8D, 0x6A, 0x12, 0x34]);
    }

    #[test]
    fn should_parse_mibeacon_mac_from_longer_payload() {
        // Longer payload with object data after the MAC
        let mut data = [0u8; 20];
        data[0] = 0x71; // Frame Control lo (bit 4 set)
        data[1] = 0x20; // Frame Control hi
        data[5] = 0xDF;
        data[6] = 0x0E;
        data[7] = 0x5B;
        data[8] = 0x38;
        data[9] = 0xC1;
        data[10] = 0xA4;
        let mac = parse_mibeacon_mac(&data).unwrap();
        assert_eq!(mac, [0xA4, 0xC1, 0x38, 0x5B, 0x0E, 0xDF]);
    }

    /// Real-world payload captured from a Xiaomi HHCCJCY01 (Mi Flora) device.
    ///
    /// Source: Home Assistant `xiaomi-ble` test suite (`test_Xiaomi_HHCCJCY01`).
    /// <https://github.com/Bluetooth-Devices/xiaomi-ble/blob/main/tests/test_parser.py>
    #[test]
    fn should_parse_mac_from_real_miflora_temperature_advertisement() {
        // Device MAC: C4:7C:8D:6B:4F:F3
        // Frame Control 0x2071 (MAC included), Product ID 0x0098, Counter 0x12
        // Object: temperature 19.6 °C (type 0x1004, value 0x00C4 = 196)
        #[rustfmt::skip]
        let data: [u8; 17] = [
            0x71, 0x20,                         // Frame Control
            0x98, 0x00,                         // Product ID (HHCCJCY01)
            0x12,                               // Frame Counter
            0xF3, 0x4F, 0x6B, 0x8D, 0x7C, 0xC4, // MAC (reversed)
            0x0D,                               // Capability
            0x04, 0x10,                         // Object type: temperature
            0x02,                               // Object length
            0xC4, 0x00,                         // Temperature: 196 → 19.6 °C
        ];

        let mac = parse_mibeacon_mac(&data).unwrap();
        assert_eq!(mac, [0xC4, 0x7C, 0x8D, 0x6B, 0x4F, 0xF3]);
    }

    /// Second real-world payload from the same test suite, different device.
    #[test]
    fn should_parse_mac_from_real_miflora_conductivity_advertisement() {
        // Device MAC: C4:7C:8D:6A:3E:7A
        // Object: conductivity 599 µS/cm (type 0x1009, value 0x0257 = 599)
        #[rustfmt::skip]
        let data: [u8; 17] = [
            0x71, 0x20,                         // Frame Control
            0x98, 0x00,                         // Product ID (HHCCJCY01)
            0x68,                               // Frame Counter
            0x7A, 0x3E, 0x6A, 0x8D, 0x7C, 0xC4, // MAC (reversed)
            0x0D,                               // Capability
            0x09, 0x10,                         // Object type: conductivity
            0x02,                               // Object length
            0x57, 0x02,                         // Conductivity: 599 µS/cm
        ];

        let mac = parse_mibeacon_mac(&data).unwrap();
        assert_eq!(mac, [0xC4, 0x7C, 0x8D, 0x6A, 0x3E, 0x7A]);
    }

    #[test]
    fn should_reject_mibeacon_too_short() {
        let mut data = [0u8; 10];
        data[0] = 0x71;
        data[1] = 0x20;
        let err = parse_mibeacon_mac(&data).unwrap_err();
        let source = std::error::Error::source(&err).unwrap();
        assert!(source.to_string().contains("11 bytes"));
    }

    #[test]
    fn should_reject_mibeacon_without_mac_flag() {
        // Frame Control 0x2061 — bit 4 clear (no MAC included)
        let data: [u8; 11] = [
            0x61, 0x20, 0x98, 0x00, 0x03, 0x34, 0x12, 0x6A, 0x8D, 0x7C, 0xC4,
        ];
        let err = parse_mibeacon_mac(&data).unwrap_err();
        let source = std::error::Error::source(&err).unwrap();
        assert!(source.to_string().contains("MAC address"));
    }

    // build_discovered

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

    // Handler trait tests

    #[test]
    fn should_return_none_for_any_passive_advertisement() {
        let handler = MifloraHandler::new(Vec::new(), Duration::from_secs(10));
        let data = [0u8; 19];
        let result = handler
            .try_parse_advertisement(ServiceUuid::ATC1441, &data)
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn should_accept_all_when_filter_is_empty() {
        let handler = MifloraHandler::new(Vec::new(), Duration::from_secs(10));
        assert!(handler.passes_filter("C4:7C:8D:6A:12:34"));
        assert!(handler.passes_filter("AA:BB:CC:DD:EE:FF"));
    }

    #[test]
    fn should_accept_matching_mac_in_filter() {
        let handler = MifloraHandler::new(
            vec!["C4:7C:8D:6A:12:34".to_owned()],
            Duration::from_secs(10),
        );
        assert!(handler.passes_filter("C4:7C:8D:6A:12:34"));
        assert!(!handler.passes_filter("AA:BB:CC:DD:EE:FF"));
    }

    #[test]
    fn should_match_filter_case_insensitively() {
        let handler = MifloraHandler::new(
            vec!["c4:7c:8d:6a:12:34".to_owned()],
            Duration::from_secs(10),
        );
        assert!(handler.passes_filter("C4:7C:8D:6A:12:34"));
        assert!(handler.passes_filter("c4:7c:8d:6a:12:34"));
    }
}
