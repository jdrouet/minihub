//! BLE adapter error types.

use minihub_domain::error::MiniHubError;

/// Errors specific to the BLE adapter.
#[derive(Debug, thiserror::Error)]
pub enum BleError {
    /// No BLE adapter found on the host.
    #[error("no BLE adapter available")]
    NotAvailable,

    /// BLE scan or adapter operation failed.
    #[error("BLE scan error: {0}")]
    Scan(String),

    /// Failed to parse a BLE advertisement payload.
    #[error("failed to parse BLE payload: {0}")]
    PayloadParse(String),

    /// A domain-level error (validation, not-found, etc.).
    #[error("domain error")]
    Domain(#[source] MiniHubError),
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
        let err = BleError::Scan("adapter reset".to_string());
        assert_eq!(err.to_string(), "BLE scan error: adapter reset");
    }

    #[test]
    fn should_display_payload_parse_error() {
        let err = BleError::PayloadParse("too short".to_string());
        assert_eq!(err.to_string(), "failed to parse BLE payload: too short");
    }

    #[test]
    fn should_convert_not_available_to_storage_error() {
        let err: MiniHubError = BleError::NotAvailable.into();
        assert!(matches!(err, MiniHubError::Storage(_)));
    }

    #[test]
    fn should_convert_scan_error_to_storage_error() {
        let err: MiniHubError = BleError::Scan("fail".to_string()).into();
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
