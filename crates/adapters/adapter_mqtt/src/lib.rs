//! # minihub-adapter-mqtt
//!
//! MQTT adapter — bridges MQTT-based devices into minihub.
//!
//! **Status: placeholder only.** This adapter is planned for a later milestone
//! and exists here to show the intended integration boundary.
//!
//! ## Responsibilities (future)
//! - Connect to an MQTT broker
//! - Subscribe to device topics
//! - Translate MQTT messages into entity state updates
//! - Publish service call results back to MQTT topics
//!
//! ## Dependency rule
//! Same as other adapters: depends on `minihub-app` and `minihub-domain`.

// TODO(M4+): Implement MQTT client, topic mapping, and integration lifecycle.
