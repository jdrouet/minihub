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

pub mod error;
pub mod id;
pub mod time;

pub mod area;
pub mod automation;
pub mod device;
pub mod entity;
pub mod entity_history;
pub mod event;
pub mod service;
