//! Application services — use-case implementations.
//!
//! Each service struct accepts port trait objects (or generics) via constructor
//! injection, keeping this layer decoupled from concrete adapters.

// TODO(M1): Implement `entity_service` module.
// TODO(M2): Implement `device_service` module.
// TODO(M2): Implement `service_caller` module.
// TODO(M3): Implement `automation_engine` module.

pub mod entity_service;
