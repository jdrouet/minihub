//! BLE advertisement payload parsers for ATC/PVVX firmware formats.
//!
//! Pure functions operating on raw `&[u8]` slices — no BLE dependency needed.
//! The two supported formats share UUID `0x181A` and are disambiguated by
//! payload length:
//!
//! - **PVVX custom** (19 bytes, little-endian)
//! - **ATC1441 original** (16 bytes, big-endian)

use crate::error::{BleError, PayloadParseError};

/// UUID used by both PVVX and ATC1441 advertisement formats.
pub const SERVICE_UUID_181A: uuid::Uuid =
    uuid::Uuid::from_u128(0x0000_181A_0000_1000_8000_0080_5F9B_34FB);

const PVVX_LEN: usize = 19;
const ATC1441_LEN: usize = 13;

/// Parsed sensor reading from a BLE advertisement.
#[derive(Debug, Clone, PartialEq)]
pub struct SensorReading {
    /// Device MAC address (6 bytes).
    pub mac: [u8; 6],
    /// Temperature in degrees Celsius.
    pub temperature: f64,
    /// Relative humidity in percent.
    pub humidity: f64,
    /// Battery level (0–100 %).
    pub battery_level: u8,
    /// Battery voltage in volts.
    pub battery_voltage: f64,
}

/// Dispatch to the correct parser based on UUID and payload length.
///
/// # Errors
///
/// Returns [`BleError::PayloadParse`] when the UUID is unrecognised or the
/// payload length does not match any known format.
pub fn parse_service_data(uuid: uuid::Uuid, data: &[u8]) -> Result<SensorReading, BleError> {
    if uuid != SERVICE_UUID_181A {
        return Err(BleError::PayloadParse(PayloadParseError::UnsupportedUuid(
            uuid,
        )));
    }

    match data.len() {
        PVVX_LEN => parse_pvvx(data),
        ATC1441_LEN => parse_atc1441(data),
        other => Err(BleError::PayloadParse(
            PayloadParseError::UnexpectedLength { actual: other },
        )),
    }
}

/// Parse a 19-byte PVVX custom-format payload (little-endian).
///
/// | Offset | Field | Type |
/// |--------|-------|------|
/// | 0–5 | MAC | 6 bytes LE |
/// | 6–7 | Temperature | i16 LE, x0.01 C |
/// | 8–9 | Humidity | u16 LE, x0.01 % |
/// | 10–11 | Battery voltage | u16 LE, mV |
/// | 12 | Battery level | u8, 0–100 % |
/// | 13 | Counter | u8 |
/// | 14 | Flags | u8 |
///
/// Bytes 15-18 are reserved/padding in the 19-byte frame.
///
/// # Errors
///
/// Returns [`BleError::PayloadParse`] when the slice length is not 19.
pub fn parse_pvvx(data: &[u8]) -> Result<SensorReading, BleError> {
    if data.len() != PVVX_LEN {
        return Err(BleError::PayloadParse(PayloadParseError::WrongLength {
            format: "PVVX",
            expected: PVVX_LEN,
            actual: data.len(),
        }));
    }

    let mut mac = [0u8; 6];
    mac.copy_from_slice(&data[0..6]);

    let temp_raw = i16::from_le_bytes([data[6], data[7]]);
    let hum_raw = u16::from_le_bytes([data[8], data[9]]);
    let batt_mv = u16::from_le_bytes([data[10], data[11]]);
    let batt_level = data[12];

    Ok(SensorReading {
        mac,
        temperature: f64::from(temp_raw) * 0.01,
        humidity: f64::from(hum_raw) * 0.01,
        battery_level: batt_level,
        battery_voltage: f64::from(batt_mv) * 0.001,
    })
}

/// Parse a 13-byte ATC1441 original-format payload (big-endian).
///
/// | Offset | Field | Type |
/// |--------|-------|------|
/// | 0–5 | MAC | 6 bytes BE |
/// | 6–7 | Temperature | i16 BE, x0.1 C |
/// | 8 | Humidity | u8, % |
/// | 9 | Battery level | u8, % |
/// | 10–11 | Battery voltage | u16 BE, mV |
/// | 12 | Counter | u8 |
///
/// # Errors
///
/// Returns [`BleError::PayloadParse`] when the slice length is not 13.
pub fn parse_atc1441(data: &[u8]) -> Result<SensorReading, BleError> {
    if data.len() != ATC1441_LEN {
        return Err(BleError::PayloadParse(PayloadParseError::WrongLength {
            format: "ATC1441",
            expected: ATC1441_LEN,
            actual: data.len(),
        }));
    }

    let mut mac = [0u8; 6];
    mac.copy_from_slice(&data[0..6]);

    let temp_raw = i16::from_be_bytes([data[6], data[7]]);
    let humidity = data[8];
    let batt_level = data[9];
    let batt_mv = u16::from_be_bytes([data[10], data[11]]);

    Ok(SensorReading {
        mac,
        temperature: f64::from(temp_raw) * 0.1,
        humidity: f64::from(humidity),
        battery_level: batt_level,
        battery_voltage: f64::from(batt_mv) * 0.001,
    })
}

/// Format a 6-byte MAC as a colon-separated hex string (e.g. `"A4:C1:38:5B:0E:DF"`).
#[must_use]
pub fn format_mac(mac: [u8; 6]) -> String {
    format!(
        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    )
}

/// Normalise a MAC to a lowercase slug suitable for entity IDs (e.g. `"a4c1385b0edf"`).
#[must_use]
pub fn mac_slug(mac: [u8; 6]) -> String {
    format!(
        "{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── PVVX tests ──────────────────────────────────────────────────────

    #[test]
    fn should_parse_pvvx_positive_temperature() {
        // MAC A4:C1:38:5B:0E:DF, temp 23.10 C, hum 40.00%, batt 100%, 3130 mV
        let data: [u8; 19] = [
            0xA4, 0xC1, 0x38, 0x5B, 0x0E, 0xDF, // MAC
            0x06, 0x09, // temp: 2310 (0x0906) LE → 23.10 C
            0xA0, 0x0F, // hum: 4000 (0x0FA0) LE → 40.00%
            0x3A, 0x0C, // voltage: 3130 mV (0x0C3A) LE
            0x64, // battery: 100%
            0x03, // counter
            0x00, // flags
            0x00, 0x00, 0x00, 0x00, // padding
        ];

        let reading = parse_pvvx(&data).unwrap();
        assert_eq!(reading.mac, [0xA4, 0xC1, 0x38, 0x5B, 0x0E, 0xDF]);
        assert!((reading.temperature - 23.10).abs() < 0.001);
        assert!((reading.humidity - 40.0).abs() < 0.001);
        assert_eq!(reading.battery_level, 100);
        assert!((reading.battery_voltage - 3.130).abs() < 0.001);
    }

    #[test]
    fn should_parse_pvvx_negative_temperature() {
        // temp = -5.50 C → raw = -550 = 0xFDDA LE → [0xDA, 0xFD]
        let mut data = [0u8; 19];
        data[6] = 0xDA;
        data[7] = 0xFD;
        data[8] = 0xE8; // hum: 1000 → 10.00%
        data[9] = 0x03;
        data[10] = 0xC8; // voltage: 2760 mV (0x0AC8) LE
        data[11] = 0x0A;
        data[12] = 50; // battery: 50%

        let reading = parse_pvvx(&data).unwrap();
        assert!((reading.temperature - (-5.50)).abs() < 0.001);
        assert!((reading.humidity - 10.0).abs() < 0.001);
        assert_eq!(reading.battery_level, 50);
        assert!((reading.battery_voltage - 2.760).abs() < 0.001);
    }

    #[test]
    fn should_reject_pvvx_wrong_length() {
        let data = [0u8; 10];
        let err = parse_pvvx(&data).unwrap_err();
        let source = std::error::Error::source(&err).unwrap();
        assert!(source.to_string().contains("19 bytes"));
    }

    // ── ATC1441 tests ───────────────────────────────────────────────────

    #[test]
    fn should_parse_atc1441_positive_temperature() {
        // MAC A4:C1:38:5B:0E:DF, temp 23.1 C, hum 40%, batt 100%, 3130 mV
        let data: [u8; 13] = [
            0xA4, 0xC1, 0x38, 0x5B, 0x0E, 0xDF, // MAC
            0x00, 0xE7, // temp: 231 (0x00E7) BE → 23.1 C
            0x28, // hum: 40%
            0x64, // battery: 100%
            0x0C, 0x3A, // voltage: 3130 mV (0x0C3A) BE
            0x03, // counter
        ];

        let reading = parse_atc1441(&data).unwrap();
        assert_eq!(reading.mac, [0xA4, 0xC1, 0x38, 0x5B, 0x0E, 0xDF]);
        assert!((reading.temperature - 23.1).abs() < 0.01);
        assert!((reading.humidity - 40.0).abs() < 0.01);
        assert_eq!(reading.battery_level, 100);
        assert!((reading.battery_voltage - 3.130).abs() < 0.001);
    }

    #[test]
    fn should_parse_atc1441_negative_temperature() {
        // temp = -3.2 C → raw = -32 = 0xFFE0 BE → [0xFF, 0xE0]
        let data: [u8; 13] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // MAC
            0xFF, 0xE0, // temp: -32 BE → -3.2 C
            0x50, // hum: 80%
            0x30, // battery: 48%
            0x0A, 0x8C, // voltage: 2700 mV
            0x01, // counter
        ];

        let reading = parse_atc1441(&data).unwrap();
        assert!((reading.temperature - (-3.2)).abs() < 0.01);
        assert!((reading.humidity - 80.0).abs() < 0.01);
        assert_eq!(reading.battery_level, 48);
        assert!((reading.battery_voltage - 2.700).abs() < 0.001);
    }

    #[test]
    fn should_reject_atc1441_wrong_length() {
        let data = [0u8; 10];
        let err = parse_atc1441(&data).unwrap_err();
        let source = std::error::Error::source(&err).unwrap();
        assert!(source.to_string().contains("13 bytes"));
    }

    // ── Dispatch tests ──────────────────────────────────────────────────

    #[test]
    fn should_dispatch_pvvx_by_length() {
        let data = [0u8; 19];
        let result = parse_service_data(SERVICE_UUID_181A, &data);
        assert!(result.is_ok());
    }

    #[test]
    fn should_dispatch_atc1441_by_length() {
        let data = [0u8; 13];
        let result = parse_service_data(SERVICE_UUID_181A, &data);
        assert!(result.is_ok());
    }

    #[test]
    fn should_reject_unknown_uuid() {
        let unknown = uuid::Uuid::from_u128(0x0000_FFFF_0000_1000_8000_0080_5F9B_34FB);
        let data = [0u8; 19];
        let err = parse_service_data(unknown, &data).unwrap_err();
        let source = std::error::Error::source(&err).unwrap();
        assert!(source.to_string().contains("unsupported service UUID"));
    }

    #[test]
    fn should_reject_unknown_length_for_181a() {
        let data = [0u8; 10];
        let err = parse_service_data(SERVICE_UUID_181A, &data).unwrap_err();
        let source = std::error::Error::source(&err).unwrap();
        assert!(source.to_string().contains("unexpected payload length 10"));
    }

    // ── MAC formatting ──────────────────────────────────────────────────

    #[test]
    fn should_format_mac_with_colons() {
        let mac = [0xA4, 0xC1, 0x38, 0x5B, 0x0E, 0xDF];
        assert_eq!(format_mac(mac), "A4:C1:38:5B:0E:DF");
    }

    #[test]
    fn should_format_mac_slug() {
        let mac = [0xA4, 0xC1, 0x38, 0x5B, 0x0E, 0xDF];
        assert_eq!(mac_slug(mac), "a4c1385b0edf");
    }

    #[test]
    fn should_format_mac_with_leading_zeros() {
        let mac = [0x00, 0x01, 0x02, 0x0A, 0x0B, 0x0C];
        assert_eq!(format_mac(mac), "00:01:02:0A:0B:0C");
        assert_eq!(mac_slug(mac), "0001020a0b0c");
    }

    // ── Boundary values ─────────────────────────────────────────────────

    #[test]
    fn should_parse_pvvx_zero_values() {
        let data = [0u8; 19];
        let reading = parse_pvvx(&data).unwrap();
        assert!((reading.temperature).abs() < 0.001);
        assert!((reading.humidity).abs() < 0.001);
        assert_eq!(reading.battery_level, 0);
        assert!((reading.battery_voltage).abs() < 0.001);
    }

    #[test]
    fn should_parse_pvvx_max_temperature() {
        // i16::MAX = 32767 → 327.67 C
        let mut data = [0u8; 19];
        data[6] = 0xFF;
        data[7] = 0x7F; // 32767 LE
        let reading = parse_pvvx(&data).unwrap();
        assert!((reading.temperature - 327.67).abs() < 0.001);
    }

    #[test]
    fn should_parse_pvvx_min_temperature() {
        // i16::MIN = -32768 → -327.68 C
        let mut data = [0u8; 19];
        data[6] = 0x00;
        data[7] = 0x80; // -32768 LE
        let reading = parse_pvvx(&data).unwrap();
        assert!((reading.temperature - (-327.68)).abs() < 0.001);
    }
}
