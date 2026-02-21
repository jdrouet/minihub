//! Common error types used across the workspace.
//!
//! Each layer defines its own concrete error types. The domain layer provides
//! [`ValidationError`] and [`NotFoundError`]. Adapter layers define their own
//! (e.g., `StorageError` wrapping `sqlx::Error`) and wire them into
//! [`MiniHubError`] via `#[from]` conversion.

/// Validation failures raised by domain invariant checks.
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("entity_id cannot be empty")]
    EmptyEntityId,
    #[error("friendly_name cannot be empty")]
    EmptyFriendlyName,
    #[error("name cannot be empty")]
    EmptyName,
    #[error("integration cannot be empty")]
    EmptyIntegration,
    #[error("unique_id cannot be empty")]
    EmptyUniqueId,
    #[error("at least one action is required")]
    NoActions,
    #[error("invalid RFC 3339 timestamp: {0}")]
    InvalidTimestamp(String),
}

/// Returned when a lookup by identifier finds nothing.
#[derive(Debug, thiserror::Error)]
#[error("{entity} {id} not found")]
pub struct NotFoundError {
    pub entity: &'static str,
    pub id: String,
}

/// Top-level domain error.
///
/// Adapter crates may introduce additional variants by wrapping their own
/// error types via `#[from]`.
#[derive(Debug, thiserror::Error)]
pub enum MiniHubError {
    #[error("Validation error")]
    Validation(#[from] ValidationError),

    #[error("Not found")]
    NotFound(#[from] NotFoundError),

    #[error("Storage error")]
    Storage(#[source] Box<dyn std::error::Error + Send + Sync>),
}

/// Convenience alias used throughout the domain and application layers.
pub type Result<T> = std::result::Result<T, MiniHubError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_display_validation_error_message() {
        let err = ValidationError::EmptyEntityId;
        assert_eq!(err.to_string(), "entity_id cannot be empty");
    }

    #[test]
    fn should_display_not_found_error_with_entity_and_id() {
        let err = NotFoundError {
            entity: "Entity",
            id: "abc-123".to_string(),
        };
        assert_eq!(err.to_string(), "Entity abc-123 not found");
    }

    #[test]
    fn should_convert_validation_error_into_minihub_error() {
        let err: MiniHubError = ValidationError::EmptyName.into();
        assert!(matches!(err, MiniHubError::Validation(_)));
    }

    #[test]
    fn should_convert_not_found_error_into_minihub_error() {
        let err: MiniHubError = NotFoundError {
            entity: "Device",
            id: "xyz".to_string(),
        }
        .into();
        assert!(matches!(err, MiniHubError::NotFound(_)));
    }
}
