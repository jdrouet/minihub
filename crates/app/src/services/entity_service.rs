//! Entity service — use-cases for managing entities.
//!
//! TODO(M1): Implement `EntityService` struct that:
//!   - Takes a `dyn EntityRepository` (or generic `R: EntityRepository`)
//!   - Provides: `register_entity`, `update_state`, `get_entity`, `list_entities`
//!   - Validates domain invariants via `minihub-domain` before persisting
//!   - Publishes state-change events via `EventPublisher` port (M2)
