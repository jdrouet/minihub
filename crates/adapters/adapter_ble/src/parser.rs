//! Shared BLE utilities — service UUIDs and MAC address formatting.

/// Supported BLE service UUIDs used by this adapter.
pub struct ServiceUuid;

impl ServiceUuid {
    /// UUID used by both PVVX and ATC1441 advertisement formats (`0x181A`).
    pub const ATC1441: uuid::Uuid =
        uuid::Uuid::from_u128(0x0000_181A_0000_1000_8000_0080_5F9B_34FB);

    /// Xiaomi Mi Flora (HHCCJCY01) service UUID (`0xFE95`).
    pub const MIFLORA: uuid::Uuid =
        uuid::Uuid::from_u128(0x0000_FE95_0000_1000_8000_0080_5F9B_34FB);
}

/// Format a 6-byte MAC as a colon-separated hex string (e.g. `"A4:C1:38:5B:0E:DF"`).
#[must_use]
pub fn format_mac(mac: [u8; 6]) -> String {
    format!(
        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    )
}

/// Parse a colon-separated hex MAC string (e.g. `"A4:C1:38:5B:0E:DF"`) back
/// to 6 raw bytes.
///
/// Returns `None` if the string does not have exactly 6 colon-separated hex
/// octets.
#[must_use]
pub fn parse_mac(s: &str) -> Option<[u8; 6]> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 6 {
        return None;
    }
    let mut mac = [0u8; 6];
    for (i, part) in parts.iter().enumerate() {
        mac[i] = u8::from_str_radix(part, 16).ok()?;
    }
    Some(mac)
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

    // MAC formatting

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

    #[test]
    fn should_parse_mac_from_string() {
        let mac = parse_mac("A4:C1:38:5B:0E:DF");
        assert_eq!(mac, Some([0xA4, 0xC1, 0x38, 0x5B, 0x0E, 0xDF]));
    }

    #[test]
    fn should_reject_mac_with_wrong_part_count() {
        assert!(parse_mac("A4:C1:38").is_none());
        assert!(parse_mac("A4:C1:38:5B:0E:DF:00").is_none());
    }

    #[test]
    fn should_reject_mac_with_invalid_hex() {
        assert!(parse_mac("ZZ:C1:38:5B:0E:DF").is_none());
    }

    // UUID constants

    #[test]
    fn should_have_correct_atc1441_uuid() {
        assert!(ServiceUuid::ATC1441.to_string().contains("0000181a"));
    }

    #[test]
    fn should_have_correct_miflora_uuid() {
        assert!(ServiceUuid::MIFLORA.to_string().contains("0000fe95"));
    }
}
