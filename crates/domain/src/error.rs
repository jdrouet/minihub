//! Common error types used across the workspace.
//!
//! TODO(M1): Implement a base `MiniHubError` enum using `thiserror` with
//! typed source errors and `#[from]` conversion (no `String` variants).
//! Each layer defines its own typed errors and converts via `#[from]`.
//!
//! Example:
//! ```ignore
//! #[derive(Debug, thiserror::Error)]
//! pub enum MiniHubError {
//!     #[error("Validation error")]
//!     Validation(#[from] ValidationError),
//!
//!     #[error("Entity not found")]
//!     NotFound(#[from] NotFoundError),
//!
//!     #[error("Storage error")]
//!     Storage(#[from] StorageError),
//! }
//! ```
