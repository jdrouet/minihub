//! Integration port — lifecycle and service-call handling for device integrations.
//!
//! An integration bridges an external protocol (virtual, MQTT, Zigbee, …) into
//! the minihub system. It discovers devices/entities on startup and handles
//! service calls directed at entities it owns.

use std::future::Future;

use minihub_domain::device::Device;
use minihub_domain::entity::Entity;
use minihub_domain::error::MiniHubError;
use minihub_domain::id::EntityId;

/// A pluggable device integration.
///
/// Implementations live in adapter crates (e.g. `adapter_virtual`).
/// The binary crate calls the lifecycle methods in order:
///
/// 1. [`setup`](Self::setup) — register devices and entities
/// 2. (the server runs, forwarding service calls via [`handle_service_call`](Self::handle_service_call))
/// 3. [`teardown`](Self::teardown) — clean up resources
pub trait Integration {
    /// Unique name identifying this integration (e.g. `"virtual"`).
    fn name(&self) -> &'static str;

    /// Called once after the composition root is fully wired.
    ///
    /// The integration should discover its devices and entities and return
    /// them so the caller can persist them via the appropriate services.
    fn setup(&mut self)
    -> impl Future<Output = Result<Vec<DiscoveredDevice>, MiniHubError>> + Send;

    /// Handle a service call (e.g. `turn_on`, `turn_off`, `toggle`) for an
    /// entity owned by this integration.
    ///
    /// Returns the new [`Entity`] state after handling the call.
    fn handle_service_call(
        &self,
        entity_id: EntityId,
        service: &str,
        data: serde_json::Value,
    ) -> impl Future<Output = Result<Entity, MiniHubError>> + Send;

    /// Called on graceful shutdown. Clean up any background tasks or connections.
    fn teardown(&mut self) -> impl Future<Output = Result<(), MiniHubError>> + Send;
}

/// A device and its associated entities discovered during [`Integration::setup`].
#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub device: Device,
    pub entities: Vec<Entity>,
}
