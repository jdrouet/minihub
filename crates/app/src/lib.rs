//! # minihub-app
//!
//! Application layer — use-cases and **port definitions** (traits).
//!
//! ## Responsibilities
//! - Define **port traits** that adapters must implement (driven/outbound ports):
//!   - `EntityRepository` — CRUD for entities
//!   - `DeviceRepository` — CRUD for devices
//!   - `AreaRepository` — CRUD for areas
//!   - `EventStore` — append & query events
//!   - `AutomationRepository` — CRUD for automations
//! - Define **driving/inbound ports** as use-case structs/traits:
//!   - `EntityService` — register, update state, list, get
//!   - `DeviceService` — register, list, get
//!   - `AutomationEngine` — evaluate triggers, run actions
//! - Provide **in-process infrastructure** (event bus) that doesn't need IO
//! - Orchestrate domain objects without knowing *how* persistence or IO works
//!
//! ## Dependency rule
//! Depends on `minihub-domain` only (plus `tokio::sync` for channels).
//! Never imports adapter crates. Adapters depend on *this* crate, not the reverse.

pub mod automation_engine;
pub mod event_bus;
pub mod ports;
pub mod services;
