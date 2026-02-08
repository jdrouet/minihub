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
//!   - `ServiceCaller` — dispatch service calls
//!   - `AutomationEngine` — evaluate triggers, run actions
//! - Orchestrate domain objects without knowing *how* persistence or IO works
//!
//! ## Dependency rule
//! Depends on `minihub-domain` only.
//! Never imports adapter crates. Adapters depend on *this* crate, not the reverse.

// TODO(M1): Implement `ports::storage` — repository traits.
// TODO(M1): Implement `services::entity_service` — entity use-cases.
// TODO(M2): Implement `ports::event_bus` — event publishing trait.
// TODO(M2): Implement `services::service_caller` — service dispatch.
// TODO(M3): Implement `services::automation_engine` — automation evaluation.

pub mod ports;
pub mod services;
