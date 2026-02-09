//! Port definitions — traits that adapters implement.
//!
//! Ports are the boundaries between the application core and the outside world.
//! They are defined here (in `app`) so that both the use-case layer and the
//! adapter layer can depend on them without creating circular dependencies.

pub mod event_bus;
pub mod event_store;
pub mod storage;

pub use event_bus::EventPublisher;
pub use event_store::EventStore;
pub use storage::{AreaRepository, DeviceRepository, EntityRepository};
