//! Application services — use-case implementations.
//!
//! Each service struct accepts port trait implementations via generic parameters
//! (constructor injection), keeping this layer decoupled from concrete adapters.

pub mod area_service;
pub mod device_service;
pub mod entity_service;
