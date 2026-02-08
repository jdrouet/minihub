//! Port definitions — traits that adapters implement.
//!
//! Ports are the boundaries between the application core and the outside world.
//! They are defined here (in `app`) so that both the use-case layer and the
//! adapter layer can depend on them without creating circular dependencies.

// TODO(M1): Define `storage` sub-module with repository traits.
// TODO(M2): Define `event_bus` sub-module with event publishing/subscribing traits.
// TODO(M2): Define `integration` sub-module with integration lifecycle traits.

pub mod storage;

pub mod event_bus;
