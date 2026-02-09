//! MQTT adapter error types.

use minihub_domain::error::MiniHubError;

/// Errors specific to the MQTT adapter.
#[derive(Debug, thiserror::Error)]
pub enum MqttError {
    /// The MQTT client has not been initialised yet.
    #[error("MQTT client not connected")]
    NotConnected,

    /// The rumqttc client returned an error.
    #[error("MQTT client error")]
    Client(#[source] rumqttc::ClientError),

    /// Failed to parse an incoming MQTT payload as JSON.
    #[error("failed to parse MQTT payload")]
    PayloadParse(#[source] serde_json::Error),

    /// A domain-level error (validation, not-found, etc.).
    #[error("domain error")]
    Domain(#[source] MiniHubError),
}

impl MqttError {
    /// Convert into a [`MiniHubError::Storage`] for propagation across port
    /// boundaries.
    pub fn into_domain(self) -> MiniHubError {
        match self {
            Self::Domain(err) => err,
            other => MiniHubError::Storage(Box::new(other)),
        }
    }
}

impl From<MqttError> for MiniHubError {
    fn from(err: MqttError) -> Self {
        err.into_domain()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_display_not_connected_error() {
        let err = MqttError::NotConnected;
        assert_eq!(err.to_string(), "MQTT client not connected");
    }

    #[test]
    fn should_convert_not_connected_to_storage_error() {
        let err: MiniHubError = MqttError::NotConnected.into();
        assert!(matches!(err, MiniHubError::Storage(_)));
    }

    #[test]
    fn should_convert_domain_error_back_to_domain() {
        let domain_err =
            MiniHubError::Validation(minihub_domain::error::ValidationError::EmptyName);
        let mqtt_err = MqttError::Domain(domain_err);
        let back: MiniHubError = mqtt_err.into();
        assert!(matches!(back, MiniHubError::Validation(_)));
    }

    #[test]
    fn should_display_payload_parse_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("{{bad").unwrap_err();
        let err = MqttError::PayloadParse(json_err);
        assert_eq!(err.to_string(), "failed to parse MQTT payload");
    }
}
