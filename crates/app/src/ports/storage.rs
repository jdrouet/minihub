//! Storage port — repository traits for persistence.
//!
//! TODO(M1): Define trait `EntityRepository` with methods:
//!   - `async fn save(&self, entity: &Entity) -> Result<()>`
//!   - `async fn get(&self, id: &EntityId) -> Result<Option<Entity>>`
//!   - `async fn list(&self) -> Result<Vec<Entity>>`
//!   - `async fn delete(&self, id: &EntityId) -> Result<()>`
//!
//! TODO(M1): Define trait `DeviceRepository` (similar CRUD pattern).
//! TODO(M1): Define trait `AreaRepository` (similar CRUD pattern).
//! TODO(M2): Define trait `EventStore` for append-only event storage.
//! TODO(M3): Define trait `AutomationRepository`.
