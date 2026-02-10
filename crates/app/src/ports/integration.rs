//! Integration port — lifecycle and service-call handling for device integrations.
//!
//! An integration bridges an external protocol (virtual, MQTT, Zigbee, …) into
//! the minihub system. It discovers devices/entities on startup and handles
//! service calls directed at entities it owns.

use std::future::Future;

use minihub_domain::device::Device;
use minihub_domain::entity::Entity;
use minihub_domain::error::MiniHubError;
use minihub_domain::event::Event;
use minihub_domain::id::EntityId;

/// Context provided to integrations for persisting discoveries.
///
/// This is a **port** — adapters call it to persist devices and entities
/// they discover. The binary crate provides a concrete implementation
/// backed by `DeviceService` and `EntityService`.
pub trait IntegrationContext: Send + Sync {
    /// Persist a discovered device (create or update by `integration`+`unique_id`).
    fn upsert_device(
        &self,
        device: Device,
    ) -> impl Future<Output = Result<Device, MiniHubError>> + Send;

    /// Persist a discovered entity (create or update by `entity_id` string).
    ///
    /// Also publishes `StateChanged` / `EntityCreated` events through the
    /// event bus when appropriate (delegated to `EntityService`).
    fn upsert_entity(
        &self,
        entity: Entity,
    ) -> impl Future<Output = Result<Entity, MiniHubError>> + Send;

    /// Publish a domain event to the event bus.
    fn publish(&self, event: Event) -> impl Future<Output = Result<(), MiniHubError>> + Send;

    /// Convenience: persist a full [`DiscoveredDevice`] (device + all entities).
    fn persist_discovered(
        &self,
        dd: DiscoveredDevice,
    ) -> impl Future<Output = Result<(), MiniHubError>> + Send {
        async move {
            self.upsert_device(dd.device).await?;
            for entity in dd.entities {
                self.upsert_entity(entity).await?;
            }
            Ok(())
        }
    }
}

/// A pluggable device integration.
///
/// Implementations live in adapter crates (e.g. `adapter_virtual`).
/// The binary crate calls the lifecycle methods in order:
///
/// 1. [`setup`](Self::setup) — initialise and persist instant discoveries
/// 2. [`start_background`](Self::start_background) — spawn long-running tasks
/// 3. (the server runs, forwarding service calls via [`handle_service_call`](Self::handle_service_call))
/// 4. [`teardown`](Self::teardown) — clean up resources
pub trait Integration {
    /// Unique name identifying this integration (e.g. `"virtual"`).
    fn name(&self) -> &'static str;

    /// Fast, non-blocking initialisation.
    ///
    /// Integrations that discover devices instantly (e.g. virtual) persist
    /// them via `ctx` here. Background integrations (BLE, MQTT) should
    /// connect and configure but **not** block for discovery — do the work
    /// in [`start_background`](Self::start_background) instead.
    fn setup(
        &mut self,
        ctx: &impl IntegrationContext,
    ) -> impl Future<Output = Result<(), MiniHubError>> + Send;

    /// Start long-running background discovery.
    ///
    /// Spawns internal tasks that persist discoveries via `ctx` and returns
    /// immediately. The default implementation is a no-op (suitable for
    /// integrations that discover everything in [`setup`](Self::setup)).
    fn start_background(
        &mut self,
        _ctx: impl IntegrationContext + Clone + 'static,
    ) -> impl Future<Output = Result<(), MiniHubError>> + Send {
        async { Ok(()) }
    }

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

/// A device and its associated entities discovered during integration setup.
#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub device: Device,
    pub entities: Vec<Entity>,
}
