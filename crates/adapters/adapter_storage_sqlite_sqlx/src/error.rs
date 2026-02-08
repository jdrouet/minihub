//! Storage-specific error type wrapping sqlx errors.

use minihub_domain::error::MiniHubError;

/// Errors originating from the `SQLite` storage layer.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// A query or connection failed.
    #[error("database error")]
    Database(#[from] sqlx::Error),

    /// Failed to deserialize a stored JSON value.
    #[error("JSON deserialization error")]
    Json(#[from] serde_json::Error),

    /// Failed to run migrations.
    #[error("migration error")]
    Migration(#[from] sqlx::migrate::MigrateError),
}

impl From<StorageError> for MiniHubError {
    fn from(err: StorageError) -> Self {
        Self::Storage(Box::new(err))
    }
}
