//! # minihub-domain
//!
//! Pure domain model for the minihub home automation system.
//!
//! ## Responsibilities
//! - Foundational types: typed identifiers, error conventions, timestamps
//! - Define **Entities** (state holders with identity: lights, sensors, switches, …)
//! - Define **Devices** (physical or virtual things that expose one or more entities)
//! - Define **Areas** (logical groupings such as rooms)
//! - Define **Services** (commands: `turn_on`, `turn_off`, `toggle`, …)
//! - Define **Events** (state-change records)
//! - Define **Automations** (trigger → condition → action rules)
//! - Contain all invariant enforcement and domain logic
//!
//! ## Dependency rule
//! This crate has **no internal dependencies**.
//! It must never import anything from `app`, adapters, or external IO crates.
//! All IO boundaries are expressed as traits in the `app` crate (ports).

// --- Foundational types (IDs, errors, time) ---

// TODO(M1): Define typed newtypes for IDs.
// TODO(M1): Define a base error enum.
// TODO(M1): Define timestamp helpers.
// TODO(M2): Define configuration primitives.

pub mod error;
pub mod id;
pub mod time;

// --- Domain model ---

// TODO(M1): Implement `entity` module — Entity, `EntityState`, `EntityAttributes`.
// TODO(M1): Implement `device` module — Device, `DeviceInfo`.
// TODO(M1): Implement `area` module — Area.
// TODO(M2): Implement `service` module — `ServiceDefinition`, `ServiceCall`.
// TODO(M2): Implement `event` module — Event, `EventPayload`.
// TODO(M3): Implement `automation` module — Automation, Trigger, Condition, Action.

pub mod area;
pub mod automation;
pub mod device;
pub mod entity;
pub mod event;
pub mod service;
