//! Entity — the central state-holding concept in minihub.
//!
//! An entity represents a single observable/controllable aspect of a device
//! (e.g., a light's on/off state, a temperature sensor's reading).
//!
//! TODO(M1): Define `Entity` struct with fields:
//!   - `id: EntityId`
//!   - `device_id: Option<DeviceId>`
//!   - `area_id: Option<AreaId>`
//!   - `name: String`
//!   - `state: EntityState`
//!   - `attributes: HashMap<String, AttributeValue>`
//!   - `last_changed: Timestamp`
//!   - `last_updated: Timestamp`
//!
//! TODO(M1): Define `EntityState` enum (On, Off, Unavailable, Unknown, Custom(String)).
//! TODO(M1): Define `AttributeValue` enum for typed attribute storage.
