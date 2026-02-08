//! Event — an immutable record of something that happened.
//!
//! Events are produced when entity state changes, services are called,
//! automations fire, etc.
//!
//! TODO(M2): Define `Event` struct with fields:
//!   - `id: EventId` (or auto-generated)
//!   - `event_type: String`
//!   - `data: serde_json::Value` (or a typed enum)
//!   - `timestamp: Timestamp`
//!   - `origin: EventOrigin` (User, Automation, Integration, …)
