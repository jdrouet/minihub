//! # minihub-adapter-storage-sqlite-sqlx
//!
//! `SQLite` persistence adapter using [sqlx](https://docs.rs/sqlx).
//!
//! ## Responsibilities
//! - Implement the repository port traits defined in `minihub-app::ports::storage`
//! - Manage `SQLite` connection pool lifecycle
//! - Run database migrations
//! - Map between domain types and database rows
//!
//! ## Dependency rule
//! Depends on `minihub-app` (for port traits) and `minihub-domain` (for domain types).
//! The `app` and `domain` crates must never reference this adapter.

mod area_repo;
mod automation_repo;
mod device_repo;
mod entity_repo;
mod error;
mod event_store;
mod pool;

pub use area_repo::SqliteAreaRepository;
pub use automation_repo::SqliteAutomationRepository;
pub use device_repo::SqliteDeviceRepository;
pub use entity_repo::SqliteEntityRepository;
pub use error::StorageError;
pub use event_store::SqliteEventStore;
pub use pool::{Config, Database};
