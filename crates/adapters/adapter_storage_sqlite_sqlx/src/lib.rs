//! # minihub-adapter-storage-sqlite-sqlx
//!
//! `SQLite` persistence adapter using [sqlx](https://docs.rs/sqlx).
//!
//! ## Responsibilities
//! - Implement the repository port traits defined in `minihub-app::ports::storage`
//! - Manage `SQLite` connection pool lifecycle
//! - Run database migrations (using sqlx embedded migrations)
//! - Map between domain types and database rows
//!
//! ## Dependency rule
//! Depends on `minihub-app` (for port traits) and `minihub-domain` (for domain types).
//! The `app` and `domain` crates must never reference this adapter.

// TODO(M2): Implement `entity_repo` — `EntityRepository` trait impl for SQLite.
// TODO(M2): Implement `device_repo` — `DeviceRepository` trait impl.
// TODO(M2): Implement `area_repo` — `AreaRepository` trait impl.
// TODO(M2): Implement `event_store` — `EventStore` trait impl.
// TODO(M2): Implement `migrations` — embedded sqlx migrations.
// TODO(M2): Implement `pool` — connection pool setup helper.

pub mod migrations;
pub mod pool;
pub mod repos;
