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
