//! BLE adapter error types.

use minihub_domain::error::MiniHubError;

/// Errors specific to the BLE adapter.
#[derive(Debug, thiserror::Error)]
pub enum BleError {
    /// No BLE adapter found on the host.
    #[error("no BLE adapter available")]
    NotAvailable,

    /// BLE scan or adapter operation failed.
    #[error("BLE scan error")]
    Scan(#[from] btleplug::Error),

    /// Failed to parse a BLE advertisement payload.
    #[error("failed to parse BLE payload")]
    PayloadParse(#[source] PayloadParseError),

    /// A domain-level error (validation, not-found, etc.).
    #[error("domain error")]
    Domain(#[source] MiniHubError),
}

/// Details about why a BLE advertisement payload could not be parsed.
#[derive(Debug, thiserror::Error)]
pub enum PayloadParseError {
    /// The service UUID is not one we know how to parse.
    #[error("unsupported service UUID {0}")]
    UnsupportedUuid(uuid::Uuid),

    /// The payload length does not match any known format for the given UUID.
    #[error("unexpected payload length {actual} for UUID 0x181A")]
    UnexpectedLength {
        /// The actual length received.
        actual: usize,
    },

    /// A known format was detected but the payload is the wrong size.
    #[error("{format} payload must be {expected} bytes, got {actual}")]
    WrongLength {
        /// Format name (e.g. "PVVX", "ATC1441").
        format: &'static str,
        /// Expected byte count.
        expected: usize,
        /// Actual byte count.
        actual: usize,
    },
}

impl BleError {
    /// Convert into a [`MiniHubError::Storage`] for propagation across port
    /// boundaries.
    #[must_use]
    pub fn into_domain(self) -> MiniHubError {
        match self {
            Self::Domain(err) => err,
            other => MiniHubError::Storage(Box::new(other)),
        }
    }
}

impl From<BleError> for MiniHubError {
    fn from(err: BleError) -> Self {
        err.into_domain()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_display_not_available_error() {
        let err = BleError::NotAvailable;
        assert_eq!(err.to_string(), "no BLE adapter available");
    }

    #[test]
    fn should_display_scan_error() {
        let err = BleError::Scan(btleplug::Error::DeviceNotFound);
        assert_eq!(err.to_string(), "BLE scan error");
    }

    #[test]
    fn should_display_payload_parse_error() {
        let err = BleError::PayloadParse(PayloadParseError::UnexpectedLength { actual: 10 });
        assert_eq!(err.to_string(), "failed to parse BLE payload");
    }

    #[test]
    fn should_display_unsupported_uuid_parse_error() {
        let uuid = uuid::Uuid::from_u128(0x0000_FFFF_0000_1000_8000_0080_5F9B_34FB);
        let err = PayloadParseError::UnsupportedUuid(uuid);
        assert!(err.to_string().contains("unsupported service UUID"));
    }

    #[test]
    fn should_display_unexpected_length_parse_error() {
        let err = PayloadParseError::UnexpectedLength { actual: 10 };
        assert_eq!(
            err.to_string(),
            "unexpected payload length 10 for UUID 0x181A"
        );
    }

    #[test]
    fn should_display_wrong_length_parse_error() {
        let err = PayloadParseError::WrongLength {
            format: "PVVX",
            expected: 19,
            actual: 10,
        };
        assert_eq!(err.to_string(), "PVVX payload must be 19 bytes, got 10");
    }

    #[test]
    fn should_convert_not_available_to_storage_error() {
        let err: MiniHubError = BleError::NotAvailable.into();
        assert!(matches!(err, MiniHubError::Storage(_)));
    }

    #[test]
    fn should_convert_scan_error_to_storage_error() {
        let err: MiniHubError = BleError::Scan(btleplug::Error::DeviceNotFound).into();
        assert!(matches!(err, MiniHubError::Storage(_)));
    }

    #[test]
    fn should_convert_domain_error_back_to_domain() {
        let domain_err =
            MiniHubError::Validation(minihub_domain::error::ValidationError::EmptyName);
        let ble_err = BleError::Domain(domain_err);
        let back: MiniHubError = ble_err.into();
        assert!(matches!(back, MiniHubError::Validation(_)));
    }
}
