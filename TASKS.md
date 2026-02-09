# minihub Implementation Plan

**Project**: minihub — A tiny Rust-only home automation server  
**Architecture**: Hexagonal (Ports & Adapters)  
**Last Updated**: 2026-02-09

---

## Table of Contents

- [Overview](#overview)
- [Quality Gates](#quality-gates)
- [Coverage Targets](#coverage-targets)
- [Milestones](#milestones)
  - [M0 — Scaffold & Plan ✅](#m0--scaffold--plan-)
  - [M1 — Domain Model & App Core ✅](#m1--domain-model--app-core-)
  - [M2 — HTTP API + SQLite Storage + Dashboard ✅](#m2--http-api--sqlite-storage--dashboard-)
  - [M3 — Events & Automations ✅](#m3--events--automations-)
  - [M4 — Virtual Integration ✅](#m4--virtual-integration-)
  - [M5 — Polish & Harden ✅](#m5--polish--harden-)
  - [M6 — MQTT Integration (Stretch) ✅](#m6--mqtt-integration-stretch-)
  - [M7 — Passive BLE Integration ✅](#m7--passive-ble-integration-)
- [Task Dependencies Graph](#task-dependencies-graph)
- [Glossary](#glossary)

---

## Overview

This document provides a detailed, actionable implementation plan for minihub organized into milestones. Each milestone represents a coherent unit of functionality that can be developed, tested, and validated independently.

### Crate Structure

```
minihub/
├── crates/
│   ├── domain/              # minihub-domain: Domain model (Entity, Device, Area, etc.) and common types (IDs, errors, time)
│   ├── app/                 # minihub-app: Use-cases and port trait definitions
│   └── adapters/
│       ├── adapter_http_axum/              # HTTP API + SSR dashboard
│       ├── adapter_storage_sqlite_sqlx/    # SQLite persistence
│       ├── adapter_mqtt/                   # MQTT client
│       └── adapter_ble/                    # Passive BLE scanner
├── crates/bin/
│   └── minihubd/            # Composition root & binary
└── Cargo.toml               # Workspace manifest
```

### Effort Estimates

- **S (Small)**: 1-4 hours
- **M (Medium)**: 4-12 hours
- **L (Large)**: 12-24 hours

---

## Quality Gates

**All milestones must satisfy these quality gates before being considered complete:**

1. **Formatting**: `cargo fmt -- --check` passes
2. **Linting**: `cargo clippy --all-targets --all-features -- -D warnings` passes
3. **Tests**: `cargo test --all` passes
4. **Coverage**: Measured with `cargo llvm-cov`, target varies per milestone

---

## Coverage Targets

| Milestone | Target Coverage | Rationale |
|-----------|----------------|-----------|
| M0 | 0% | Scaffold only, no implementation |
| M1 | 40% | Domain and application logic with unit tests |
| M2 | 60% | Add integration tests for HTTP and storage |
| M3 | 70% | Event system and automation engine tests |
| M4 | 70% | Virtual integration tests |
| M5 | 80% | Comprehensive testing, production-ready |
| M6 | 80% | Maintain coverage with MQTT tests |
| M7 | 80% | Maintain coverage with BLE tests |

---

## Milestones

### M0 — Scaffold & Plan ✅

**Goal**: Establish workspace structure, documentation, and development tooling.

**Status**: ✅ Done

**Prerequisites**: None

**Deliverables**: Working Cargo workspace with empty crates, core documentation, CI stub, development commands.

#### Tasks

| Task ID | Description | Effort | Dependencies | DoD | Key Files |
|---------|-------------|--------|--------------|-----|-----------|
| M0-T1 | ✅ Create workspace layout | S | None | All `Cargo.toml` files exist at correct locations. `cargo check` passes on all empty crates. Workspace compiles without errors. | `Cargo.toml`, `crates/*/Cargo.toml`, `crates/adapters/*/Cargo.toml`, `crates/bin/minihubd/Cargo.toml` |
| M0-T2 | ✅ Write documentation | M | None | `README.md`, `ARCHITECTURE.md`, `DECISIONS.md`, `TASKS.md`, `CONTRIBUTING.md` exist and contain comprehensive information about the project structure, goals, and development process. | `README.md`, `ARCHITECTURE.md`, `DECISIONS.md`, `TASKS.md`, `CONTRIBUTING.md` |
| M0-T3 | ✅ Create CI workflow stub | S | None | `.github/workflows/ci.yml` exists with jobs for fmt, clippy, test, and coverage placeholders. Workflow triggers on push and PR. | `.github/workflows/ci.yml` |
| M0-T4 | ✅ Create Justfile | S | None | `Justfile` exists with recipes for common development tasks: `check`, `test`, `fmt`, `clippy`, `run`, `coverage`, `clean`. `just --list` displays all available commands. | `Justfile` |

#### Detailed Task Breakdown

**M0-T1: Create workspace layout**

Create the following directory structure and Cargo.toml files:

```
minihub/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── domain/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── app/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   └── adapters/
│       ├── adapter_http_axum/
│       │   ├── Cargo.toml
│       │   └── src/lib.rs
│       ├── adapter_storage_sqlite_sqlx/
│       │   ├── Cargo.toml
│       │   └── src/lib.rs
│       └── adapter_mqtt/
│           ├── Cargo.toml
│           └── src/lib.rs
└── crates/bin/
    └── minihubd/
        ├── Cargo.toml
        └── src/main.rs
```

Each `lib.rs` should contain a basic docstring and empty module. `main.rs` should contain `fn main() {}`.

**M0-T2: Write documentation**

Create comprehensive documentation:

- `README.md`: Project overview, quick start, feature list, architecture summary
- `ARCHITECTURE.md`: Detailed hexagonal architecture explanation, crate responsibilities, data flow diagrams
- `DECISIONS.md`: Architecture Decision Records (ADRs) template and initial decisions
- `TASKS.md`: This file
- `CONTRIBUTING.md`: Development setup, coding standards, PR process, testing guidelines

**M0-T3: Create CI workflow stub**

Create `.github/workflows/ci.yml` with:
- Checkout and Rust toolchain setup
- Job for `cargo fmt -- --check`
- Job for `cargo clippy --all-targets --all-features -- -D warnings`
- Job for `cargo test --all`
- Job for `cargo llvm-cov` (placeholder, configure properly in M1+)

**M0-T4: Create Justfile**

Create `Justfile` with recipes:
```just
# List all commands
default:
    @just --list

# Check compilation
check:
    cargo check --all-targets --all-features

# Run tests
test:
    cargo test --all

# Format code
fmt:
    cargo fmt --all

# Run linter
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run the server
run:
    cargo run --bin minihubd

# Generate coverage report
coverage:
    cargo llvm-cov --all --html

# Clean build artifacts
clean:
    cargo clean
```

---

### M1 — Domain Model & App Core ✅

**Goal**: Implement core domain types, shared utilities, and application service layer with comprehensive unit tests. Achieve 40% code coverage.

**Status**: ✅ Done

**Prerequisites**: M0 complete

**Deliverables**: Fully tested domain model, application services with port trait definitions, mock implementations for testing.

#### Tasks

| Task ID | Description | Effort | Dependencies | DoD | Key Files |
|---------|-------------|--------|--------------|-----|-----------|
| M1-T1 | ✅ Implement shared types | S | None | `EntityId`, `DeviceId`, `AreaId`, `AutomationId` types implemented using newtype pattern in `minihub-domain`. `MiniHubError` with variants for validation, not-found, storage errors. `Timestamp` utility for UTC time handling. Unit tests for all types. | `crates/domain/src/lib.rs`, `crates/domain/src/id.rs`, `crates/domain/src/error.rs`, `crates/domain/src/time.rs` |
| M1-T2 | ✅ Implement Entity, EntityState, AttributeValue | M | M1-T1 | `Entity` struct with id, device_id, entity_id (string), friendly_name, state, attributes, last_changed, last_updated. `EntityState` enum with On/Off/Unknown/Unavailable. `AttributeValue` enum supporting String/Int/Float/Bool/Json. Builder pattern for Entity. Validation logic (non-empty names, valid state transitions). Unit tests covering validation, state changes, attribute manipulation. | `crates/domain/src/entity.rs`, `crates/domain/src/entity_state.rs`, `crates/domain/src/attribute_value.rs` |
| M1-T3 | ✅ Implement Device, Area | S | M1-T1 | `Device` struct with id, name, manufacturer, model, area_id. `Area` struct with id, name, parent_id. Builder patterns. Validation (non-empty names). Unit tests. | `crates/domain/src/device.rs`, `crates/domain/src/area.rs` |
| M1-T4 | ✅ Define storage port traits | M | M1-T2, M1-T3 | `EntityRepository` trait with methods: `create`, `get_by_id`, `get_all`, `update`, `delete`, `find_by_device_id`. `DeviceRepository` trait with CRUD methods. `AreaRepository` trait with CRUD methods. All methods return `Result<T, MiniHubError>`. Async trait using `#[async_trait]`. Documentation on each trait method. | `crates/app/src/ports/entity_repository.rs`, `crates/app/src/ports/device_repository.rs`, `crates/app/src/ports/area_repository.rs`, `crates/app/src/ports/mod.rs` |
| M1-T5 | ✅ Implement EntityService | M | M1-T4 | `EntityService` struct that wraps `Arc<dyn EntityRepository>`. Methods: `create_entity`, `get_entity`, `list_entities`, `update_entity_state`, `update_entity_attributes`, `delete_entity`. Business logic validation before repository calls. Unit tests using mock repository (e.g., HashMap-based in-memory mock). Achieve >= 40% coverage on app crate. | `crates/app/src/services/entity_service.rs`, `crates/app/src/services/entity_service_test.rs` or inline tests |
| M1-T6 | ✅ Implement DeviceService, AreaService | S | M1-T4 | Similar to EntityService. `DeviceService` with CRUD operations. `AreaService` with CRUD operations and optional parent-child relationship queries. Unit tests with mock repositories. | `crates/app/src/services/device_service.rs`, `crates/app/src/services/area_service.rs` |

#### Detailed Task Breakdown

**M1-T1: Implement shared types**

Create strong type IDs using newtype pattern:

```rust
// crates/domain/src/id.rs
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(uuid::Uuid);

impl EntityId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: uuid::Uuid) -> Self {
        Self(uuid)
    }
    
    pub fn as_uuid(&self) -> uuid::Uuid {
        self.0
    }
}

impl Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Similar for DeviceId, AreaId, AutomationId, EventId
```

Create error types with typed source errors (no `String` variants):

```rust
// crates/domain/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("entity_id cannot be empty")]
    EmptyEntityId,
    #[error("friendly_name cannot be empty")]
    EmptyFriendlyName,
    #[error("name cannot be empty")]
    EmptyName,
    #[error("at least one action required")]
    NoActions,
    #[error("port must be non-zero")]
    InvalidPort,
}

#[derive(Debug, Error)]
#[error("entity {id} not found")]
pub struct NotFoundError {
    pub id: String,
}

#[derive(Debug, Error)]
pub enum MiniHubError {
    #[error("Validation error")]
    Validation(#[from] ValidationError),

    #[error("Not found")]
    NotFound(#[from] NotFoundError),

    #[error("Storage error")]
    Storage(#[from] StorageError),

    #[error("Internal error")]
    Internal(#[from] InternalError),
}

pub type Result<T> = std::result::Result<T, MiniHubError>;
```

Each adapter crate defines its own concrete error types (e.g., `StorageError` wrapping `sqlx::Error`,
`InternalError` wrapping serde/IO errors) and converts via `#[from]`.

Create time utilities:

```rust
// crates/domain/src/time.rs
use chrono::{DateTime, Utc};

pub type Timestamp = DateTime<Utc>;

pub fn now() -> Timestamp {
    Utc::now()
}
```

Add dependencies to `crates/domain/Cargo.toml`:
```toml
[dependencies]
uuid = { version = "1.6", features = ["v4", "serde"] }
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
chrono = { version = "0.4", features = ["serde"] }
```

**M1-T2: Implement Entity, EntityState, AttributeValue**

```rust
// crates/domain/src/entity_state.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntityState {
    On,
    Off,
    Unknown,
    Unavailable,
}

impl EntityState {
    pub fn is_available(&self) -> bool {
        !matches!(self, EntityState::Unavailable)
    }
}

// crates/domain/src/attribute_value.rs
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Json(JsonValue),
}

// crates/domain/src/entity.rs
use crate::{id::{EntityId, DeviceId}, time::Timestamp, error::{Result, ValidationError}};
use std::collections::HashMap;
use crate::{EntityState, AttributeValue};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub device_id: DeviceId,
    pub entity_id: String,  // e.g., "light.living_room"
    pub friendly_name: String,
    pub state: EntityState,
    pub attributes: HashMap<String, AttributeValue>,
    pub last_changed: Timestamp,
    pub last_updated: Timestamp,
}

impl Entity {
    pub fn builder() -> EntityBuilder {
        EntityBuilder::default()
    }
    
    pub fn update_state(&mut self, new_state: EntityState, timestamp: Timestamp) {
        if self.state != new_state {
            self.state = new_state;
            self.last_changed = timestamp;
        }
        self.last_updated = timestamp;
    }
    
    pub fn set_attribute(&mut self, key: String, value: AttributeValue) {
        self.attributes.insert(key, value);
    }
    
    pub fn get_attribute(&self, key: &str) -> Option<&AttributeValue> {
        self.attributes.get(key)
    }
    
    pub fn validate(&self) -> Result<()> {
        if self.entity_id.is_empty() {
            return Err(ValidationError::EmptyEntityId)?;
        }
        if self.friendly_name.is_empty() {
            return Err(ValidationError::EmptyFriendlyName)?;
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct EntityBuilder {
    // fields...
}

impl EntityBuilder {
    // builder methods...
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::now;
    
    #[test]
    fn test_entity_state_change() {
        // test state transitions
    }
    
    #[test]
    fn test_entity_validation() {
        // test validation rules
    }
    
    #[test]
    fn test_attribute_manipulation() {
        // test setting and getting attributes
    }
}
```

Add dependencies to `crates/domain/Cargo.toml`:
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**M1-T3: Implement Device, Area**

```rust
// crates/domain/src/device.rs
use crate::{id::{DeviceId, AreaId}, error::{Result, ValidationError}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: DeviceId,
    pub name: String,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub area_id: Option<AreaId>,
}

impl Device {
    pub fn builder() -> DeviceBuilder {
        DeviceBuilder::default()
    }
    
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(ValidationError::EmptyName)?;
        }
        Ok(())
    }
}

// crates/domain/src/area.rs
use crate::{id::AreaId, error::{Result, ValidationError}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Area {
    pub id: AreaId,
    pub name: String,
    pub parent_id: Option<AreaId>,
}

impl Area {
    pub fn builder() -> AreaBuilder {
        AreaBuilder::default()
    }
    
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(ValidationError::EmptyName)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // tests for Device and Area
}
```

**M1-T4: Define storage port traits**

```rust
// crates/app/src/ports/entity_repository.rs
use async_trait::async_trait;
use minihub_domain::Entity;
use minihub_domain::{id::{EntityId, DeviceId}, error::Result};

#[async_trait]
pub trait EntityRepository: Send + Sync {
    /// Create a new entity in storage
    async fn create(&self, entity: Entity) -> Result<Entity>;
    
    /// Get an entity by ID
    async fn get_by_id(&self, id: EntityId) -> Result<Option<Entity>>;
    
    /// Get all entities
    async fn get_all(&self) -> Result<Vec<Entity>>;
    
    /// Find entities by device ID
    async fn find_by_device_id(&self, device_id: DeviceId) -> Result<Vec<Entity>>;
    
    /// Update an existing entity
    async fn update(&self, entity: Entity) -> Result<Entity>;
    
    /// Delete an entity by ID
    async fn delete(&self, id: EntityId) -> Result<()>;
}

// Similar traits for DeviceRepository, AreaRepository
```

Add dependencies to `crates/app/Cargo.toml`:
```toml
[dependencies]
minihub-domain = { path = "../domain" }
async-trait = "0.1"
```

**M1-T5: Implement EntityService**

```rust
// crates/app/src/services/entity_service.rs
use std::sync::Arc;
use minihub_domain::Entity;
use minihub_domain::{id::EntityId, error::{Result, NotFoundError}, time::now};
use crate::ports::EntityRepository;

pub struct EntityService {
    repo: Arc<dyn EntityRepository>,
}

impl EntityService {
    pub fn new(repo: Arc<dyn EntityRepository>) -> Self {
        Self { repo }
    }
    
    pub async fn create_entity(&self, mut entity: Entity) -> Result<Entity> {
        entity.validate()?;
        entity.last_updated = now();
        entity.last_changed = now();
        self.repo.create(entity).await
    }
    
    pub async fn get_entity(&self, id: EntityId) -> Result<Entity> {
        self.repo.get_by_id(id)
            .await?
            .ok_or_else(|| NotFoundError { id: id.to_string() })?
    }
    
    pub async fn list_entities(&self) -> Result<Vec<Entity>> {
        self.repo.get_all().await
    }
    
    pub async fn update_entity_state(&self, id: EntityId, new_state: EntityState) -> Result<Entity> {
        let mut entity = self.get_entity(id).await?;
        entity.update_state(new_state, now());
        self.repo.update(entity).await
    }
    
    pub async fn delete_entity(&self, id: EntityId) -> Result<()> {
        self.repo.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;
    
    // Mock repository implementation
    struct MockEntityRepository {
        entities: Arc<Mutex<HashMap<EntityId, Entity>>>,
    }
    
    #[async_trait]
    impl EntityRepository for MockEntityRepository {
        async fn create(&self, entity: Entity) -> Result<Entity> {
            let mut entities = self.entities.lock().unwrap();
            entities.insert(entity.id, entity.clone());
            Ok(entity)
        }
        
        // ... other methods
    }
    
    #[tokio::test]
    async fn test_create_entity() {
        // test entity creation
    }
    
    #[tokio::test]
    async fn test_get_entity_not_found() {
        // test not found error
    }
    
    #[tokio::test]
    async fn test_update_entity_state() {
        // test state update
    }
}
```

**M1-T6: Implement DeviceService, AreaService**

Similar structure to EntityService, with CRUD operations for Device and Area domain objects.

---

### M2 — HTTP API + SQLite Storage + Dashboard ✅

**Goal**: Create a working server with REST API, SQLite persistence, and server-side rendered dashboard. Achieve 60% code coverage.

**Status**: ✅ Done

**Prerequisites**: M1 complete

**Deliverables**: Running server accessible via HTTP with JSON API and HTML dashboard, data persisted to SQLite, integration tests passing.

#### Tasks

| Task ID | Description | Effort | Dependencies | DoD | Key Files |
|---------|-------------|--------|--------------|-----|-----------|
| M2-T1 | Implement SQLite connection pool + migrations | M | M1-T4 | `Database` struct wraps `SqlitePool`. Async `initialize` function creates pool, runs migrations. Migration files for initial schema (entities, devices, areas tables). Integration test verifies pool creation and migration execution. | `crates/adapters/adapter_storage_sqlite_sqlx/src/db.rs`, `crates/adapters/adapter_storage_sqlite_sqlx/migrations/001_initial.sql` |
| M2-T2 | Implement SqliteEntityRepository | M | M2-T1, M1-T4 | `SqliteEntityRepository` struct implements `EntityRepository` trait. All CRUD methods use sqlx for async queries. Typed `StorageError` wrapping `sqlx::Error` with `#[from]` conversion to `MiniHubError`. Integration tests against real SQLite database (use in-memory or temp file). | `crates/adapters/adapter_storage_sqlite_sqlx/src/entity_repository.rs`, `crates/adapters/adapter_storage_sqlite_sqlx/src/entity_repository_test.rs` |
| M2-T3 | Implement SqliteDeviceRepository, SqliteAreaRepository | S | M2-T1 | Similar to M2-T2. Implement both repositories with full CRUD operations. Integration tests for both. | `crates/adapters/adapter_storage_sqlite_sqlx/src/device_repository.rs`, `crates/adapters/adapter_storage_sqlite_sqlx/src/area_repository.rs` |
| M2-T4 | Implement axum router skeleton + app state | M | M1-T5 | `AppState` struct holds `Arc` references to all services. Create axum `Router` with health check endpoint `/`. Server starts on configurable port (default 8080). Graceful startup logging. | `crates/adapters/adapter_http_axum/src/state.rs`, `crates/adapters/adapter_http_axum/src/router.rs`, `crates/adapters/adapter_http_axum/src/lib.rs` |
| M2-T5 | Implement JSON REST API handlers | L | M2-T4, M2-T2 | REST endpoints for entities, devices, areas: `GET /api/entities`, `POST /api/entities`, `GET /api/entities/:id`, `PUT /api/entities/:id/state`, `DELETE /api/entities/:id`. Similar for `/api/devices` and `/api/areas`. Proper HTTP status codes (200, 201, 404, 400, 500). JSON request/response bodies. Error handling middleware. Integration tests using HTTP client (reqwest or hyper). | `crates/adapters/adapter_http_axum/src/handlers/api/entities.rs`, `crates/adapters/adapter_http_axum/src/handlers/api/devices.rs`, `crates/adapters/adapter_http_axum/src/handlers/api/areas.rs`, `crates/adapters/adapter_http_axum/src/handlers/api/mod.rs` |
| M2-T6 | Choose HTML templating approach | S | None | Evaluate maud (compile-time), askama (Jinja-like), manual string building. Write ADR documenting decision rationale. Commit ADR and add chosen dependency to `adapter_http_axum/Cargo.toml`. | `DECISIONS.md` (new ADR entry), `crates/adapters/adapter_http_axum/Cargo.toml` |
| M2-T7 | Implement SSR dashboard pages | L | M2-T5, M2-T6 | Pages: `/` (home with overview), `/entities` (list all), `/entities/:id` (detail + control form), `/devices` (list), `/areas` (list). Forms use POST + Redirect (POST-REDIRECT-GET pattern). Meta refresh tag for auto-reload. Minimal inline CSS for basic layout. No JavaScript required. | `crates/adapters/adapter_http_axum/src/handlers/web/home.rs`, `crates/adapters/adapter_http_axum/src/handlers/web/entities.rs`, `crates/adapters/adapter_http_axum/src/handlers/web/devices.rs`, `crates/adapters/adapter_http_axum/src/handlers/web/areas.rs`, `crates/adapters/adapter_http_axum/src/templates/` (if using askama) |
| M2-T8 | Wire everything in minihubd | M | M2-T5, M2-T7 | `main.rs` composition root: initialize SQLite pool, create repository instances, create service instances, build axum router with state, start server. Proper error handling for startup failures. Logging at startup. | `crates/bin/minihubd/src/main.rs` |
| M2-T9 | End-to-end smoke tests | M | M2-T8 | Integration tests that start the full server, make HTTP requests to both API and web endpoints, verify responses. Test full create-read-update-delete cycles. Verify data persistence across server restarts. Achieve >= 60% coverage. | `crates/bin/minihubd/tests/integration_test.rs` |

#### Detailed Task Breakdown

**M2-T1: Implement SQLite connection pool + migrations**

```rust
// crates/adapters/adapter_storage_sqlite_sqlx/src/db.rs
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use minihub_domain::error::Result;
use std::str::FromStr;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn initialize(database_url: &str) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true);
        
        let pool = SqlitePool::connect_with(options).await?;
        
        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;
        
        Ok(Self { pool })
    }
    
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
```

Create migration file:

```sql
-- crates/adapters/adapter_storage_sqlite_sqlx/migrations/001_initial.sql
CREATE TABLE IF NOT EXISTS devices (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    manufacturer TEXT,
    model TEXT,
    area_id TEXT,
    FOREIGN KEY (area_id) REFERENCES areas(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS areas (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    parent_id TEXT,
    FOREIGN KEY (parent_id) REFERENCES areas(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS entities (
    id TEXT PRIMARY KEY NOT NULL,
    device_id TEXT NOT NULL,
    entity_id TEXT NOT NULL UNIQUE,
    friendly_name TEXT NOT NULL,
    state TEXT NOT NULL,
    attributes TEXT NOT NULL, -- JSON blob
    last_changed TEXT NOT NULL, -- ISO 8601 timestamp
    last_updated TEXT NOT NULL, -- ISO 8601 timestamp
    FOREIGN KEY (device_id) REFERENCES devices(id) ON DELETE CASCADE
);

CREATE INDEX idx_entities_device_id ON entities(device_id);
CREATE INDEX idx_entities_entity_id ON entities(entity_id);
```

Add dependencies to `crates/adapters/adapter_storage_sqlite_sqlx/Cargo.toml`:
```toml
[dependencies]
minihub-domain = { path = "../../domain" }
minihub-app = { path = "../../app" }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite", "migrate"] }
async-trait = "0.1"
serde_json = "1.0"
```

**M2-T2: Implement SqliteEntityRepository**

```rust
// crates/adapters/adapter_storage_sqlite_sqlx/src/entity_repository.rs
use async_trait::async_trait;
use sqlx::SqlitePool;
use minihub_app::ports::EntityRepository;
use minihub_domain::Entity;
use minihub_domain::{id::{EntityId, DeviceId}, error::Result};

pub struct SqliteEntityRepository {
    pool: SqlitePool,
}

impl SqliteEntityRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EntityRepository for SqliteEntityRepository {
    async fn create(&self, entity: Entity) -> Result<Entity> {
        let attributes_json = serde_json::to_string(&entity.attributes)?;
        
        sqlx::query(
            r#"
            INSERT INTO entities (id, device_id, entity_id, friendly_name, state, attributes, last_changed, last_updated)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(entity.id.to_string())
        .bind(entity.device_id.to_string())
        .bind(&entity.entity_id)
        .bind(&entity.friendly_name)
        .bind(serde_json::to_string(&entity.state)?)
        .bind(attributes_json)
        .bind(entity.last_changed.to_rfc3339())
        .bind(entity.last_updated.to_rfc3339())
        .execute(&self.pool)
        .await?;
        
        Ok(entity)
    }
    
    async fn get_by_id(&self, id: EntityId) -> Result<Option<Entity>> {
        let row = sqlx::query_as::<_, EntityRow>(
            "SELECT * FROM entities WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row.map(|r| r.into_entity()).transpose()?)
    }
    
    // ... implement other methods
}

struct EntityRow {
    id: String,
    device_id: String,
    entity_id: String,
    friendly_name: String,
    state: String,
    attributes: String,
    last_changed: String,
    last_updated: String,
}

impl EntityRow {
    fn into_entity(self) -> Result<Entity> {
        // Parse row into Entity
        Ok(Entity {
            id: EntityId::from_str(&self.id)?,
            device_id: DeviceId::from_str(&self.device_id)?,
            entity_id: self.entity_id,
            friendly_name: self.friendly_name,
            state: serde_json::from_str(&self.state)?,
            attributes: serde_json::from_str(&self.attributes)?,
            last_changed: chrono::DateTime::parse_from_rfc3339(&self.last_changed)?.into(),
            last_updated: chrono::DateTime::parse_from_rfc3339(&self.last_updated)?.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_and_get_entity() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        
        let repo = SqliteEntityRepository::new(pool);
        
        // Create test entity
        // Assert creation and retrieval
    }
}
```

**M2-T3: Implement SqliteDeviceRepository, SqliteAreaRepository**

Similar pattern to SqliteEntityRepository.

**M2-T4: Implement axum router skeleton + app state**

```rust
// crates/adapters/adapter_http_axum/src/state.rs
use std::sync::Arc;
use minihub_app::services::{EntityService, DeviceService, AreaService};

#[derive(Clone)]
pub struct AppState {
    pub entity_service: Arc<EntityService>,
    pub device_service: Arc<DeviceService>,
    pub area_service: Arc<AreaService>,
}

// crates/adapters/adapter_http_axum/src/router.rs
use axum::{Router, routing::get};
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(health_check))
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}
```

Add dependencies to `crates/adapters/adapter_http_axum/Cargo.toml`:
```toml
[dependencies]
minihub-domain = { path = "../../domain" }
minihub-app = { path = "../../app" }
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**M2-T5: Implement JSON REST API handlers**

```rust
// crates/adapters/adapter_http_axum/src/handlers/api/entities.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use minihub_domain::{Entity, EntityState};
use minihub_domain::id::EntityId;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct CreateEntityRequest {
    pub device_id: String,
    pub entity_id: String,
    pub friendly_name: String,
}

#[derive(Serialize)]
pub struct EntityResponse {
    // serializable Entity representation
}

pub async fn list_entities(
    State(state): State<AppState>,
) -> Result<Json<Vec<EntityResponse>>, StatusCode> {
    let entities = state.entity_service.list_entities()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(entities.into_iter().map(EntityResponse::from).collect()))
}

pub async fn get_entity(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<EntityResponse>, StatusCode> {
    let entity_id = EntityId::from_str(&id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let entity = state.entity_service.get_entity(entity_id)
        .await
        .map_err(|err| match err {
            MiniHubError::NotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;
    
    Ok(Json(EntityResponse::from(entity)))
}

pub async fn create_entity(
    State(state): State<AppState>,
    Json(req): Json<CreateEntityRequest>,
) -> Result<(StatusCode, Json<EntityResponse>), StatusCode> {
    // Create entity
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn update_entity_state(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateStateRequest>,
) -> Result<Json<EntityResponse>, StatusCode> {
    // Update state
    Ok(Json(response))
}

pub async fn delete_entity(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    // Delete
    Ok(StatusCode::NO_CONTENT)
}

// Wire up in router
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/entities", get(list_entities).post(create_entity))
        .route("/entities/:id", get(get_entity).delete(delete_entity))
        .route("/entities/:id/state", axum::routing::put(update_entity_state))
}
```

Create similar handlers for devices and areas.

**M2-T6: Choose HTML templating approach**

Evaluate options:
1. **maud**: Compile-time HTML templating with Rust syntax
2. **askama**: Jinja2-like templates, compile-time checked
3. **Manual string building**: Simple but error-prone

Document decision in `DECISIONS.md`:

```markdown
## ADR-003: HTML Templating for Dashboard

**Status**: Accepted

**Context**: We need server-side rendered HTML for the no-JS dashboard.

**Decision**: Use maud for compile-time HTML generation.

**Rationale**:
- Type-safe and compile-time checked
- No separate template files to manage
- Excellent performance (zero-cost abstraction)
- Natural Rust syntax for Rust developers
- Simpler than askama for small projects

**Consequences**:
- HTML structure in Rust code (mixing concerns)
- Steeper learning curve for designers
- Excellent IDE support and refactoring
```

**M2-T7: Implement SSR dashboard pages**

```rust
// crates/adapters/adapter_http_axum/src/handlers/web/home.rs
use axum::{extract::State, response::Html};
use maud::{html, Markup, DOCTYPE};
use crate::state::AppState;

pub async fn home(State(state): State<AppState>) -> Html<String> {
    let entities = state.entity_service.list_entities().await.unwrap_or_default();
    let devices = state.device_service.list_devices().await.unwrap_or_default();
    let areas = state.area_service.list_areas().await.unwrap_or_default();
    
    let markup = html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta http-equiv="refresh" content="10"; // Auto-refresh every 10s
                title { "minihub Dashboard" }
                style {
                    "body { font-family: sans-serif; margin: 2rem; }"
                    "nav { margin-bottom: 2rem; }"
                    "nav a { margin-right: 1rem; }"
                    ".card { border: 1px solid #ccc; padding: 1rem; margin-bottom: 1rem; }"
                }
            }
            body {
                h1 { "minihub Dashboard" }
                nav {
                    a href="/" { "Home" }
                    a href="/entities" { "Entities" }
                    a href="/devices" { "Devices" }
                    a href="/areas" { "Areas" }
                }
                
                div class="card" {
                    h2 { "Overview" }
                    p { "Entities: " (entities.len()) }
                    p { "Devices: " (devices.len()) }
                    p { "Areas: " (areas.len()) }
                }
            }
        }
    };
    
    Html(markup.into_string())
}

// crates/adapters/adapter_http_axum/src/handlers/web/entities.rs
pub async fn list_entities(State(state): State<AppState>) -> Html<String> {
    // Similar structure, list all entities with links to detail page
}

pub async fn entity_detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let entity_id = EntityId::from_str(&id)?;
    let entity = state.entity_service.get_entity(entity_id).await?;
    
    let markup = html! {
        // ... entity detail view with control form
        form method="POST" action={"/entities/" (id) "/state"} {
            input type="hidden" name="state" value="on";
            button type="submit" { "Turn On" }
        }
        form method="POST" action={"/entities/" (id) "/state"} {
            input type="hidden" name="state" value="off";
            button type="submit" { "Turn Off" }
        }
    };
    
    Ok(Html(markup.into_string()))
}

pub async fn update_entity_state_web(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Form(form): Form<UpdateStateForm>,
) -> Result<Redirect, StatusCode> {
    // Update state
    state.entity_service.update_entity_state(entity_id, new_state).await?;
    
    // POST-REDIRECT-GET pattern
    Ok(Redirect::to(&format!("/entities/{}", id)))
}
```

Add maud to dependencies:
```toml
[dependencies]
maud = { version = "0.25", features = ["axum"] }
```

**M2-T8: Wire everything in minihubd**

```rust
// crates/bin/minihubd/src/main.rs
use std::sync::Arc;
use adapter_http_axum::{create_router, AppState};
use adapter_storage_sqlite_sqlx::{Database, SqliteEntityRepository, SqliteDeviceRepository, SqliteAreaRepository};
use minihub_app::services::{EntityService, DeviceService, AreaService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Initialize database
    let db = Database::initialize("sqlite:minihub.db").await?;
    let pool = db.pool().clone();
    
    // Create repositories
    let entity_repo = Arc::new(SqliteEntityRepository::new(pool.clone())) as Arc<dyn EntityRepository>;
    let device_repo = Arc::new(SqliteDeviceRepository::new(pool.clone())) as Arc<dyn DeviceRepository>;
    let area_repo = Arc::new(SqliteAreaRepository::new(pool.clone())) as Arc<dyn AreaRepository>;
    
    // Create services
    let entity_service = Arc::new(EntityService::new(entity_repo));
    let device_service = Arc::new(DeviceService::new(device_repo));
    let area_service = Arc::new(AreaService::new(area_repo));
    
    // Create app state
    let state = AppState {
        entity_service,
        device_service,
        area_service,
    };
    
    // Create router
    let app = create_router(state);
    
    // Start server
    let addr = "0.0.0.0:8080";
    tracing::info!("Starting server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

Add dependencies to `crates/bin/minihubd/Cargo.toml`:
```toml
[dependencies]
adapter-http-axum = { path = "../../adapters/adapter_http_axum" }
adapter-storage-sqlite-sqlx = { path = "../../adapters/adapter_storage_sqlite_sqlx" }
minihub-app = { path = "../../app" }
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

**M2-T9: End-to-end smoke tests**

```rust
// crates/bin/minihubd/tests/integration_test.rs
use reqwest;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_full_crud_cycle() {
    // Start server in background
    let server_handle = tokio::spawn(async {
        // Start minihubd
    });
    
    // Wait for server to start
    sleep(Duration::from_secs(1)).await;
    
    let client = reqwest::Client::new();
    
    // Test health check
    let resp = client.get("http://localhost:8080/").send().await.unwrap();
    assert_eq!(resp.status(), 200);
    
    // Test API endpoints
    // Create device
    // Create entity
    // Get entity
    // Update entity state
    // List entities
    // Delete entity
    
    // Test web pages
    let resp = client.get("http://localhost:8080/entities").send().await.unwrap();
    assert_eq!(resp.status(), 200);
    assert!(resp.text().await.unwrap().contains("Entities"));
    
    // Cleanup
    server_handle.abort();
}
```

---

### M3 — Events & Automations ✅

**Goal**: Implement event logging, in-process event bus, and automation engine. Achieve 70% code coverage.

**Status**: ✅ Done

**Prerequisites**: M2 complete

**Deliverables**: Event system with persistence, automation engine that responds to events, dashboard pages for events and automations.

#### Tasks

| Task ID | Description | Effort | Dependencies | DoD | Key Files |
|---------|-------------|--------|--------------|-----|-----------|
| M3-T1 | Implement Event domain type | S | None | `Event` struct with id, event_type, entity_id, timestamp, data (JSON). Unit tests for validation and serialization. | `crates/domain/src/event.rs` |
| M3-T2 | Define EventStore port + EventPublisher port | S | M3-T1 | `EventStore` trait with `store`, `get_by_id`, `get_recent`, `find_by_entity`. `EventPublisher` trait with `publish` and `subscribe`. Both async. | `crates/app/src/ports/event_store.rs`, `crates/app/src/ports/event_publisher.rs` |
| M3-T3 | Implement in-process event bus | M | M3-T2 | `InProcessEventBus` using tokio `broadcast` channel. Implements `EventPublisher`. Thread-safe, async publish/subscribe. Unit tests for pub/sub, multiple subscribers, dropped messages handling. | `crates/app/src/event_bus.rs` |
| M3-T4 | Implement SqliteEventStore | M | M3-T2 | `SqliteEventStore` implements `EventStore`. Migration for events table. Queries for recent events, filtering by entity. Integration tests. | `crates/adapters/adapter_storage_sqlite_sqlx/src/event_store.rs`, `crates/adapters/adapter_storage_sqlite_sqlx/migrations/002_events.sql` |
| M3-T5 | Add event publishing to EntityService | S | M3-T3 | Inject `EventPublisher` into `EntityService`. Publish `StateChanged` event on state updates. Unit tests verify events are published. | `crates/app/src/services/entity_service.rs` (modified) |
| M3-T6 | Implement Automation domain types | M | M3-T1 | `Automation` struct with id, name, enabled, trigger, conditions, actions. `Trigger` enum (StateChanged, TimePattern, Manual). `Condition` enum (StateIs, TimeRange, Custom). `Action` enum (CallService, Delay, Custom). Validation logic. Unit tests. | `crates/domain/src/automation.rs`, `crates/domain/src/automation/trigger.rs`, `crates/domain/src/automation/condition.rs`, `crates/domain/src/automation/action.rs` |
| M3-T7 | Implement AutomationEngine | L | M3-T6, M3-T5 | `AutomationEngine` subscribes to event bus. On event, evaluates all enabled automations. For matching triggers, checks conditions, executes actions. Action execution calls EntityService. Full unit test coverage with mock services and event publishers. Handle errors gracefully. Logging for debugging. | `crates/app/src/automation_engine.rs` |
| M3-T8 | Add event log + automation pages to dashboard | M | M3-T4, M3-T7 | `/events` page lists recent events (paginated). `/automations` page lists all automations with status. `/automations/:id` shows detail. Forms to enable/disable automations. Basic SSR rendering. | `crates/adapters/adapter_http_axum/src/handlers/web/events.rs`, `crates/adapters/adapter_http_axum/src/handlers/web/automations.rs` |
| M3-T9 | Add event + automation API endpoints | S | M3-T7, M3-T4 | REST endpoints: `GET /api/events`, `GET /api/events/:id`, `GET /api/automations`, `POST /api/automations`, `GET /api/automations/:id`, `PUT /api/automations/:id`, `DELETE /api/automations/:id`. Integration tests. Achieve >= 70% coverage. | `crates/adapters/adapter_http_axum/src/handlers/api/events.rs`, `crates/adapters/adapter_http_axum/src/handlers/api/automations.rs` |

#### Detailed Task Breakdown

**M3-T1: Implement Event domain type**

```rust
// crates/domain/src/event.rs
use crate::{id::{EventId, EntityId}, time::Timestamp};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub event_type: EventType,
    pub entity_id: Option<EntityId>,
    pub timestamp: Timestamp,
    pub data: JsonValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    StateChanged,
    AttributeChanged,
    EntityAdded,
    EntityRemoved,
    AutomationTriggered,
    ServiceCalled,
    Custom(String),
}

impl Event {
    pub fn new(event_type: EventType, entity_id: Option<EntityId>, data: JsonValue) -> Self {
        Self {
            id: EventId::new(),
            event_type,
            entity_id,
            timestamp: crate::time::now(),
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_creation() {
        let event = Event::new(
            EventType::StateChanged,
            Some(EntityId::new()),
            serde_json::json!({"from": "off", "to": "on"}),
        );
        
        assert_eq!(event.event_type, EventType::StateChanged);
        assert!(event.entity_id.is_some());
    }
    
    #[test]
    fn test_event_serialization() {
        // Test JSON serialization round-trip
    }
}
```

**M3-T2: Define EventStore port + EventPublisher port**

```rust
// crates/app/src/ports/event_store.rs
use async_trait::async_trait;
use minihub_domain::Event;
use minihub_domain::{id::{EventId, EntityId}, error::Result};

#[async_trait]
pub trait EventStore: Send + Sync {
    /// Store a new event
    async fn store(&self, event: Event) -> Result<Event>;
    
    /// Get an event by ID
    async fn get_by_id(&self, id: EventId) -> Result<Option<Event>>;
    
    /// Get recent events (up to limit)
    async fn get_recent(&self, limit: usize) -> Result<Vec<Event>>;
    
    /// Find events by entity ID
    async fn find_by_entity(&self, entity_id: EntityId, limit: usize) -> Result<Vec<Event>>;
}

// crates/app/src/ports/event_publisher.rs
use async_trait::async_trait;
use minihub_domain::Event;
use minihub_domain::error::Result;
use tokio::sync::broadcast::Receiver;

#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish an event to all subscribers
    async fn publish(&self, event: Event) -> Result<()>;
    
    /// Subscribe to events
    fn subscribe(&self) -> Receiver<Event>;
}
```

**M3-T3: Implement in-process event bus**

```rust
// crates/app/src/event_bus.rs
use async_trait::async_trait;
use minihub_domain::Event;
use minihub_domain::error::Result;
use tokio::sync::broadcast::{self, Sender, Receiver};
use crate::ports::EventPublisher;

pub struct InProcessEventBus {
    sender: Sender<Event>,
}

impl InProcessEventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }
}

#[async_trait]
impl EventPublisher for InProcessEventBus {
    async fn publish(&self, event: Event) -> Result<()> {
        self.sender.send(event)
            .map(|_| ())
            .map_err(|err| BroadcastError(err.to_string()))?;
        Ok(())
    }
    
    fn subscribe(&self) -> Receiver<Event> {
        self.sender.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use minihub_domain::{Event, EventType};
    
    #[tokio::test]
    async fn test_publish_subscribe() {
        let bus = InProcessEventBus::new(100);
        let mut subscriber = bus.subscribe();
        
        let event = Event::new(EventType::StateChanged, None, serde_json::json!({}));
        bus.publish(event.clone()).await.unwrap();
        
        let received = subscriber.recv().await.unwrap();
        assert_eq!(received.id, event.id);
    }
    
    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = InProcessEventBus::new(100);
        let mut sub1 = bus.subscribe();
        let mut sub2 = bus.subscribe();
        
        let event = Event::new(EventType::StateChanged, None, serde_json::json!({}));
        bus.publish(event.clone()).await.unwrap();
        
        let recv1 = sub1.recv().await.unwrap();
        let recv2 = sub2.recv().await.unwrap();
        
        assert_eq!(recv1.id, recv2.id);
    }
}
```

**M3-T4: Implement SqliteEventStore**

```sql
-- crates/adapters/adapter_storage_sqlite_sqlx/migrations/002_events.sql
CREATE TABLE IF NOT EXISTS events (
    id TEXT PRIMARY KEY NOT NULL,
    event_type TEXT NOT NULL,
    entity_id TEXT,
    timestamp TEXT NOT NULL,
    data TEXT NOT NULL,
    FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE CASCADE
);

CREATE INDEX idx_events_timestamp ON events(timestamp DESC);
CREATE INDEX idx_events_entity_id ON events(entity_id, timestamp DESC);
CREATE INDEX idx_events_type ON events(event_type, timestamp DESC);
```

```rust
// crates/adapters/adapter_storage_sqlite_sqlx/src/event_store.rs
use async_trait::async_trait;
use sqlx::SqlitePool;
use minihub_app::ports::EventStore;
use minihub_domain::Event;
use minihub_domain::{id::{EventId, EntityId}, error::Result};

pub struct SqliteEventStore {
    pool: SqlitePool,
}

impl SqliteEventStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventStore for SqliteEventStore {
    async fn store(&self, event: Event) -> Result<Event> {
        sqlx::query(
            r#"
            INSERT INTO events (id, event_type, entity_id, timestamp, data)
            VALUES (?, ?, ?, ?, ?)
            "#
        )
        .bind(event.id.to_string())
        .bind(serde_json::to_string(&event.event_type)?)
        .bind(event.entity_id.map(|id| id.to_string()))
        .bind(event.timestamp.to_rfc3339())
        .bind(serde_json::to_string(&event.data)?)
        .execute(&self.pool)
        .await?;
        
        Ok(event)
    }
    
    async fn get_recent(&self, limit: usize) -> Result<Vec<Event>> {
        let rows = sqlx::query_as::<_, EventRow>(
            "SELECT * FROM events ORDER BY timestamp DESC LIMIT ?"
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;
        
        rows.into_iter()
            .map(|r| r.into_event())
            .collect::<Result<Vec<_>>>()
    }
    
    // ... other methods
}

struct EventRow {
    id: String,
    event_type: String,
    entity_id: Option<String>,
    timestamp: String,
    data: String,
}

impl EventRow {
    fn into_event(self) -> Result<Event> {
        // Parse row into Event
        Ok(Event { /* ... */ })
    }
}

#[cfg(test)]
mod tests {
    // Integration tests against real SQLite
}
```

**M3-T5: Add event publishing to EntityService**

```rust
// Modify crates/app/src/services/entity_service.rs
use crate::ports::EventPublisher;

pub struct EntityService {
    repo: Arc<dyn EntityRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl EntityService {
    pub fn new(repo: Arc<dyn EntityRepository>, event_publisher: Arc<dyn EventPublisher>) -> Self {
        Self { repo, event_publisher }
    }
    
    pub async fn update_entity_state(&self, id: EntityId, new_state: EntityState) -> Result<Entity> {
        let mut entity = self.get_entity(id).await?;
        let old_state = entity.state.clone();
        
        entity.update_state(new_state.clone(), now());
        let updated = self.repo.update(entity).await?;
        
        // Publish event
        let event = Event::new(
            EventType::StateChanged,
            Some(id),
            serde_json::json!({
                "old_state": old_state,
                "new_state": new_state,
            }),
        );
        self.event_publisher.publish(event).await?;
        
        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_state_change_publishes_event() {
        // Setup mock repo and event bus
        // Update state
        // Verify event was published
    }
}
```

**M3-T6: Implement Automation domain types**

```rust
// crates/domain/src/automation.rs
use crate::{id::{AutomationId, EntityId}, error::Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Automation {
    pub id: AutomationId,
    pub name: String,
    pub enabled: bool,
    pub trigger: Trigger,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
}

impl Automation {
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(ValidationError::EmptyName)?;
        }
        if self.actions.is_empty() {
            return Err(ValidationError::NoActions)?;
        }
        Ok(())
    }
}

// crates/domain/src/automation/trigger.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Trigger {
    StateChanged {
        entity_id: EntityId,
        from: Option<String>,
        to: Option<String>,
    },
    TimePattern {
        cron: String,
    },
    Manual,
}

impl Trigger {
    pub fn matches_event(&self, event: &Event) -> bool {
        match self {
            Trigger::StateChanged { entity_id, from, to } => {
                if event.event_type != EventType::StateChanged {
                    return false;
                }
                if event.entity_id != Some(*entity_id) {
                    return false;
                }
                // Check from/to states if specified
                true
            },
            _ => false,
        }
    }
}

// crates/domain/src/automation/condition.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Condition {
    StateIs {
        entity_id: EntityId,
        state: String,
    },
    TimeRange {
        after: String,  // HH:MM format
        before: String,
    },
}

impl Condition {
    pub async fn evaluate(&self, context: &ConditionContext) -> Result<bool> {
        match self {
            Condition::StateIs { entity_id, state } => {
                let entity = context.get_entity(*entity_id).await?;
                Ok(entity.state.to_string() == *state)
            },
            Condition::TimeRange { after, before } => {
                // Parse time and check if now is in range
                Ok(true)  // Simplified
            },
        }
    }
}

// crates/domain/src/automation/action.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    CallService {
        entity_id: EntityId,
        service: String,  // e.g., "turn_on", "turn_off"
        data: serde_json::Value,
    },
    Delay {
        seconds: u64,
    },
}

#[cfg(test)]
mod tests {
    // Tests for trigger matching, condition evaluation, action execution
}
```

**M3-T7: Implement AutomationEngine**

```rust
// crates/app/src/automation_engine.rs
use std::sync::Arc;
use tokio::sync::broadcast::Receiver;
use minihub_domain::{Event, Automation};
use minihub_domain::error::Result;
use crate::ports::{EventPublisher, AutomationRepository};
use crate::services::EntityService;

pub struct AutomationEngine {
    automations: Arc<dyn AutomationRepository>,
    entity_service: Arc<EntityService>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl AutomationEngine {
    pub fn new(
        automations: Arc<dyn AutomationRepository>,
        entity_service: Arc<EntityService>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            automations,
            entity_service,
            event_publisher,
        }
    }
    
    pub async fn start(&self) -> Result<()> {
        let mut receiver = self.event_publisher.subscribe();
        
        loop {
            match receiver.recv().await {
                Ok(event) => {
                    if let Err(e) = self.process_event(event).await {
                        tracing::error!("Error processing event: {}", e);
                    }
                },
                Err(e) => {
                    tracing::error!("Error receiving event: {}", e);
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    async fn process_event(&self, event: Event) -> Result<()> {
        let automations = self.automations.get_enabled().await?;
        
        for automation in automations {
            if automation.trigger.matches_event(&event) {
                tracing::info!("Automation {} triggered by event {}", automation.name, event.id);
                
                // Check conditions
                let all_conditions_met = self.check_conditions(&automation).await?;
                
                if all_conditions_met {
                    self.execute_actions(&automation).await?;
                }
            }
        }
        
        Ok(())
    }
    
    async fn check_conditions(&self, automation: &Automation) -> Result<bool> {
        for condition in &automation.conditions {
            let context = ConditionContext {
                entity_service: self.entity_service.clone(),
            };
            
            if !condition.evaluate(&context).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    
    async fn execute_actions(&self, automation: &Automation) -> Result<()> {
        for action in &automation.actions {
            match action {
                Action::CallService { entity_id, service, data } => {
                    match service.as_str() {
                        "turn_on" => {
                            self.entity_service.update_entity_state(*entity_id, EntityState::On).await?;
                        },
                        "turn_off" => {
                            self.entity_service.update_entity_state(*entity_id, EntityState::Off).await?;
                        },
                        _ => {
                            tracing::warn!("Unknown service: {}", service);
                        }
                    }
                },
                Action::Delay { seconds } => {
                    tokio::time::sleep(tokio::time::Duration::from_secs(*seconds)).await;
                },
            }
        }
        
        // Publish automation triggered event
        let event = Event::new(
            EventType::AutomationTriggered,
            None,
            serde_json::json!({ "automation_id": automation.id }),
        );
        self.event_publisher.publish(event).await?;
        
        Ok(())
    }
}

pub struct ConditionContext {
    entity_service: Arc<EntityService>,
}

impl ConditionContext {
    async fn get_entity(&self, id: EntityId) -> Result<Entity> {
        self.entity_service.get_entity(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_trigger_matching() {
        // Create mock automation with state change trigger
        // Create matching event
        // Verify trigger matches
    }
    
    #[tokio::test]
    async fn test_condition_evaluation() {
        // Create automation with conditions
        // Setup mock entity service
        // Verify conditions are evaluated correctly
    }
    
    #[tokio::test]
    async fn test_action_execution() {
        // Create automation with actions
        // Execute actions
        // Verify entity service was called
    }
    
    #[tokio::test]
    async fn test_full_automation_flow() {
        // End-to-end test of event -> trigger -> condition -> action
    }
}
```

**M3-T8: Add event log + automation pages to dashboard**

```rust
// crates/adapters/adapter_http_axum/src/handlers/web/events.rs
pub async fn list_events(State(state): State<AppState>) -> Html<String> {
    let events = state.event_store.get_recent(100).await.unwrap_or_default();
    
    let markup = html! {
        // ... page header, nav
        
        h2 { "Recent Events" }
        table {
            thead {
                tr {
                    th { "Time" }
                    th { "Type" }
                    th { "Entity" }
                    th { "Data" }
                }
            }
            tbody {
                @for event in events {
                    tr {
                        td { (event.timestamp.format("%Y-%m-%d %H:%M:%S")) }
                        td { (format!("{:?}", event.event_type)) }
                        td {
                            @if let Some(entity_id) = event.entity_id {
                                a href={"/entities/" (entity_id)} { (entity_id) }
                            } @else {
                                "-"
                            }
                        }
                        td { (event.data.to_string()) }
                    }
                }
            }
        }
    };
    
    Html(markup.into_string())
}

// crates/adapters/adapter_http_axum/src/handlers/web/automations.rs
pub async fn list_automations(State(state): State<AppState>) -> Html<String> {
    let automations = state.automation_service.list_automations().await.unwrap_or_default();
    
    let markup = html! {
        // ... page with automation list
        
        @for automation in automations {
            div class="card" {
                h3 { (automation.name) }
                p { "Status: " @if automation.enabled { "Enabled" } @else { "Disabled" } }
                form method="POST" action={"/automations/" (automation.id) "/toggle"} {
                    button { @if automation.enabled { "Disable" } @else { "Enable" } }
                }
                a href={"/automations/" (automation.id)} { "Details" }
            }
        }
    };
    
    Html(markup.into_string())
}
```

**M3-T9: Add event + automation API endpoints**

Similar to M2-T5, create REST API endpoints for events and automations following the same patterns.

---

### M4 — Virtual Integration ✅

**Goal**: Build a demo integration with simulated devices to prove the integration model works. Achieve 70% coverage.

**Status**: ✅ Done

**Prerequisites**: M3 complete

**Deliverables**: Integration trait definition, virtual integration implementation with fake devices, full integration tests.

#### Tasks

| Task ID | Description | Effort | Dependencies | DoD | Key Files |
|---------|-------------|--------|--------------|-----|-----------|
| M4-T1 | ✅ Design integration trait/lifecycle | M | None | `Integration` trait defining `initialize`, `start`, `stop`, `discover_devices`, `handle_service_call`. Documentation on integration lifecycle, registration process. ADR documenting design. | `crates/app/src/integration.rs`, `DECISIONS.md` |
| M4-T2 | ✅ Implement virtual integration | M | M4-T1 | `VirtualIntegration` creates fake devices (light, sensor, switch). Devices respond to service calls (turn_on/off). Sensor generates random values periodically. Integration registers devices with EntityService. Virtual entities appear in dashboard and API. | `crates/adapters/adapter_virtual/src/lib.rs`, `crates/adapters/adapter_virtual/src/devices/light.rs`, `crates/adapters/adapter_virtual/src/devices/sensor.rs`, `crates/adapters/adapter_virtual/src/devices/switch.rs` |
| M4-T3 | ✅ Integration tests for virtual integration | S | M4-T2 | Full lifecycle test: start integration, verify devices created, call services, verify state changes, verify events published. Coverage >= 70%. | `crates/adapters/adapter_virtual/tests/integration_test.rs` |

#### Detailed Task Breakdown

**M4-T1: Design integration trait/lifecycle**

```rust
// crates/app/src/integration.rs
use async_trait::async_trait;
use minihub_domain::{Device, Entity};
use minihub_domain::{error::Result, id::EntityId};
use serde_json::Value as JsonValue;

#[async_trait]
pub trait Integration: Send + Sync {
    /// Get the unique name of this integration
    fn name(&self) -> &str;
    
    /// Initialize the integration (load config, setup connections)
    async fn initialize(&mut self) -> Result<()>;
    
    /// Start the integration (begin discovery, start polling)
    async fn start(&mut self, context: IntegrationContext) -> Result<()>;
    
    /// Stop the integration (cleanup resources)
    async fn stop(&mut self) -> Result<()>;
    
    /// Discover devices provided by this integration
    async fn discover_devices(&self) -> Result<Vec<(Device, Vec<Entity>)>>;
    
    /// Handle a service call for an entity managed by this integration
    async fn handle_service_call(
        &self,
        entity_id: EntityId,
        service: &str,
        data: JsonValue,
    ) -> Result<()>;
}

/// Context provided to integrations for interacting with the system
#[derive(Clone)]
pub struct IntegrationContext {
    pub entity_service: Arc<EntityService>,
    pub event_publisher: Arc<dyn EventPublisher>,
}

impl IntegrationContext {
    pub fn new(
        entity_service: Arc<EntityService>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            entity_service,
            event_publisher,
        }
    }
}
```

Document in ADR:

```markdown
## ADR-004: Integration Model

**Status**: Accepted

**Context**: We need a way for integrations to register devices and handle service calls.

**Decision**: Use trait-based integration model with lifecycle methods and context injection.

**Rationale**:
- Clear separation between integration code and core system
- Integration context provides controlled access to services
- Lifecycle methods (init, start, stop) enable proper resource management
- Service call handling keeps device-specific logic in integration

**Consequences**:
- Each integration must implement the trait
- System manages integration lifecycle
- Integrations are responsible for their own device discovery
```

**M4-T2: Implement virtual integration**

```rust
// crates/adapters/adapter_virtual/src/lib.rs
use async_trait::async_trait;
use minihub_app::integration::{Integration, IntegrationContext};
use minihub_domain::{Device, Entity, EntityState};
use minihub_domain::{error::Result, id::{DeviceId, EntityId}};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

mod devices;
use devices::{VirtualLight, VirtualSensor, VirtualSwitch};

pub struct VirtualIntegration {
    context: Option<IntegrationContext>,
    devices: Arc<RwLock<HashMap<EntityId, Box<dyn VirtualDevice>>>>,
}

impl Default for VirtualIntegration {
    fn default() -> Self {
        Self {
            context: None,
            devices: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Integration for VirtualIntegration {
    fn name(&self) -> &str {
        "virtual"
    }
    
    async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing virtual integration");
        Ok(())
    }
    
    async fn start(&mut self, context: IntegrationContext) -> Result<()> {
        tracing::info!("Starting virtual integration");
        self.context = Some(context.clone());
        
        // Discover and register devices
        let discovered = self.discover_devices().await?;
        for (device, entities) in discovered {
            // Register with entity service
            context.entity_service.register_device(device).await?;
            
            for entity in entities {
                let entity_id = entity.id;
                context.entity_service.register_entity(entity).await?;
                
                // Store virtual device for service call handling
                // Simplified: would need to map entity to virtual device instance
            }
        }
        
        // Start sensor polling
        self.start_sensor_polling(context).await?;
        
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        tracing::info!("Stopping virtual integration");
        Ok(())
    }
    
    async fn discover_devices(&self) -> Result<Vec<(Device, Vec<Entity>)>> {
        let mut result = Vec::new();
        
        // Virtual light
        let light_device = Device::builder()
            .id(DeviceId::new())
            .name("Virtual Light".to_string())
            .manufacturer(Some("Virtual Inc.".to_string()))
            .model(Some("VLight-1".to_string()))
            .build()?;
        
        let light_entity = Entity::builder()
            .id(EntityId::new())
            .device_id(light_device.id)
            .entity_id("light.virtual_light".to_string())
            .friendly_name("Virtual Light".to_string())
            .state(EntityState::Off)
            .build()?;
        
        result.push((light_device, vec![light_entity]));
        
        // Virtual sensor
        let sensor_device = Device::builder()
            .id(DeviceId::new())
            .name("Virtual Sensor".to_string())
            .manufacturer(Some("Virtual Inc.".to_string()))
            .model(Some("VSensor-1".to_string()))
            .build()?;
        
        let sensor_entity = Entity::builder()
            .id(EntityId::new())
            .device_id(sensor_device.id)
            .entity_id("sensor.virtual_temperature".to_string())
            .friendly_name("Virtual Temperature".to_string())
            .state(EntityState::Unknown)
            .build()?;
        
        result.push((sensor_device, vec![sensor_entity]));
        
        // Virtual switch
        let switch_device = Device::builder()
            .id(DeviceId::new())
            .name("Virtual Switch".to_string())
            .manufacturer(Some("Virtual Inc.".to_string()))
            .model(Some("VSwitch-1".to_string()))
            .build()?;
        
        let switch_entity = Entity::builder()
            .id(EntityId::new())
            .device_id(switch_device.id)
            .entity_id("switch.virtual_switch".to_string())
            .friendly_name("Virtual Switch".to_string())
            .state(EntityState::Off)
            .build()?;
        
        result.push((switch_device, vec![switch_entity]));
        
        Ok(result)
    }
    
    async fn handle_service_call(
        &self,
        entity_id: EntityId,
        service: &str,
        _data: serde_json::Value,
    ) -> Result<()> {
        let devices = self.devices.read().await;
        
        if let Some(device) = devices.get(&entity_id) {
            device.handle_service(service).await?;
        }
        
        Ok(())
    }
}

impl VirtualIntegration {
    async fn start_sensor_polling(&self, context: IntegrationContext) -> Result<()> {
        // Spawn background task to update sensor values periodically
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                
                // Update sensor value
                // This would look up the sensor entity and update its attributes
                tracing::debug!("Updating virtual sensor values");
            }
        });
        
        Ok(())
    }
}

// crates/adapters/adapter_virtual/src/devices/mod.rs
#[async_trait]
trait VirtualDevice: Send + Sync {
    async fn handle_service(&self, service: &str) -> Result<()>;
}

// Implement VirtualLight, VirtualSensor, VirtualSwitch
```

**M4-T3: Integration tests for virtual integration**

```rust
// crates/adapters/adapter_virtual/tests/integration_test.rs
use adapter_virtual::VirtualIntegration;
use minihub_app::integration::Integration;

#[tokio::test]
async fn test_virtual_integration_lifecycle() {
    // Setup
    let mut integration = VirtualIntegration::default();
    
    // Initialize
    integration.initialize().await.unwrap();
    
    // Discover devices
    let devices = integration.discover_devices().await.unwrap();
    assert_eq!(devices.len(), 3); // light, sensor, switch
    
    // Verify device types
    assert!(devices.iter().any(|(d, _)| d.name == "Virtual Light"));
    assert!(devices.iter().any(|(d, _)| d.name == "Virtual Sensor"));
    assert!(devices.iter().any(|(d, _)| d.name == "Virtual Switch"));
    
    // Start integration (would need mock context)
    // Test service calls
    // Test sensor updates
    
    // Stop
    integration.stop().await.unwrap();
}

#[tokio::test]
async fn test_virtual_light_service_calls() {
    // Test turning light on/off
}

#[tokio::test]
async fn test_virtual_sensor_polling() {
    // Test that sensor values update periodically
}
```

---

### M5 — Polish & Harden ✅

**Goal**: Production-readiness: structured logging, configuration, graceful shutdown, error handling audit, CSS styling. Achieve 80% coverage.

**Status**: Done (branch: M5-polish-harden)

**Prerequisites**: M4 complete

**Deliverables**: Production-ready server with proper logging, configuration file support, clean error handling, professional-looking dashboard.

#### Tasks

| Task ID | Description | Effort | Dependencies | DoD | Key Files |
|---------|-------------|--------|--------------|-----|-----------|
| M5-T1 | Structured logging with tracing | M | None | Replace all `println!` with `tracing` macros. Add spans for HTTP requests, database queries, service calls. JSON logging option for production. Log levels configurable. | All crates (add tracing), `crates/adapters/adapter_http_axum/src/middleware/logging.rs` |
| M5-T2 | Configuration loading | M | None | `Config` struct with server port, database path, log level, integration settings. Load from TOML file (default `minihub.toml`). Environment variable overrides (e.g., `MINIHUB_PORT`). Config validation. Example config file. | `crates/bin/minihubd/src/config.rs`, `minihub.toml.example` |
| M5-T3 | Graceful shutdown | S | None | Handle SIGTERM/SIGINT. Drain in-flight HTTP requests. Close database connections. Stop integrations cleanly. Log shutdown process. | `crates/bin/minihubd/src/main.rs` (modify), `crates/bin/minihubd/src/shutdown.rs` |
| M5-T4 | Error handling audit | M | None | Audit all error handling. Remove `unwrap()` and `expect()` from non-test code. Ensure all errors are properly logged. HTTP handlers return appropriate status codes. Add error response types. | All crates (audit and fix) |
| M5-T5 | CSS styling for dashboard | M | None | Clean, responsive CSS for dashboard. Mobile-friendly layout. Light theme with good contrast. Professional typography. Forms styled. Tables formatted. Navigation bar. Status indicators (on/off states). | `crates/adapters/adapter_http_axum/src/static/style.css` or inline in maud templates |
| M5-T6 | Final coverage push | S | M5-T1 through M5-T5 | Add missing tests to reach 80% coverage. Focus on uncovered branches and error paths. Run `cargo llvm-cov` and verify >= 80%. All quality gates pass. | Various test files across crates |

#### Detailed Task Breakdown

**M5-T1: Structured logging with tracing**

```rust
// Add to all Cargo.toml files
[dependencies]
tracing = "0.1"

// In minihubd/Cargo.toml
[dependencies]
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

// crates/bin/minihubd/src/main.rs
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn setup_logging(config: &Config) {
    let format_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_level(true);
    
    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            format!("minihub={},adapter={}", config.log_level, config.log_level).into()
        });
    
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(format_layer)
        .init();
}

// crates/adapters/adapter_http_axum/src/middleware/logging.rs
use axum::{extract::Request, middleware::Next, response::Response};
use tracing::Instrument;

pub async fn trace_request(req: Request, next: Next) -> Response {
    let span = tracing::info_span!(
        "http_request",
        method = %req.method(),
        uri = %req.uri(),
        version = ?req.version(),
    );
    
    async move {
        tracing::info!("started processing request");
        let response = next.run(req).await;
        tracing::info!(status = %response.status(), "finished processing request");
        response
    }
    .instrument(span)
    .await
}

// Usage in services
impl EntityService {
    #[tracing::instrument(skip(self), fields(entity_id = %id))]
    pub async fn get_entity(&self, id: EntityId) -> Result<Entity> {
        tracing::debug!("fetching entity from repository");
        let entity = self.repo.get_by_id(id).await?;
        tracing::info!("successfully fetched entity");
        Ok(entity)
    }
}
```

**M5-T2: Configuration loading**

```rust
// crates/bin/minihubd/src/config.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use minihub_domain::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub integrations: IntegrationsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_database_path")]
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    
    #[serde(default)]
    pub json: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationsConfig {
    #[serde(default)]
    pub virtual: VirtualIntegrationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualIntegrationConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_host() -> String { "0.0.0.0".to_string() }
fn default_port() -> u16 { 8080 }
fn default_database_path() -> PathBuf { PathBuf::from("minihub.db") }
fn default_log_level() -> String { "info".to_string() }
fn default_true() -> bool { true }

impl Config {
    pub fn load() -> Result<Self> {
        // Try to load from file
        let config = if let Ok(content) = std::fs::read_to_string("minihub.toml") {
            toml::from_str(&content)?
        } else {
            Config::default()
        };
        
        // Apply environment variable overrides
        let config = config.apply_env_overrides()?;
        
        // Validate
        config.validate()?;
        
        Ok(config)
    }
    
    fn apply_env_overrides(mut self) -> Result<Self> {
        if let Ok(port) = std::env::var("MINIHUB_PORT") {
            self.server.port = port.parse()?;
        }
        
        if let Ok(db_path) = std::env::var("MINIHUB_DATABASE_PATH") {
            self.database.path = PathBuf::from(db_path);
        }
        
        if let Ok(log_level) = std::env::var("MINIHUB_LOG_LEVEL") {
            self.logging.level = log_level;
        }
        
        Ok(self)
    }
    
    fn validate(&self) -> Result<()> {
        if self.server.port == 0 {
            return Err(ValidationError::InvalidPort)?;
        }
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: default_host(),
                port: default_port(),
            },
            database: DatabaseConfig {
                path: default_database_path(),
            },
            logging: LoggingConfig {
                level: default_log_level(),
                json: false,
            },
            integrations: IntegrationsConfig {
                virtual: VirtualIntegrationConfig {
                    enabled: default_true(),
                },
            },
        }
    }
}
```

Create example config:

```toml
# minihub.toml.example
[server]
host = "0.0.0.0"
port = 8080

[database]
path = "minihub.db"

[logging]
level = "info"
json = false

[integrations.virtual]
enabled = true
```

**M5-T3: Graceful shutdown**

```rust
// crates/bin/minihubd/src/shutdown.rs
use tokio::signal;
use tracing::info;

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("received Ctrl+C, initiating graceful shutdown");
        },
        _ = terminate => {
            info!("received SIGTERM, initiating graceful shutdown");
        },
    }
}

// In main.rs
#[tokio::main]
async fn main() -> Result<()> {
    // ... setup
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    
    // Cleanup
    info!("shutting down integrations");
    for integration in integrations {
        if let Err(e) = integration.stop().await {
            tracing::error!("error stopping integration: {}", e);
        }
    }
    
    info!("closing database connections");
    db.close().await?;
    
    info!("shutdown complete");
    
    Ok(())
}
```

**M5-T4: Error handling audit**

Systematically go through all crates:

1. Search for `unwrap()` and `expect()` outside of tests
2. Replace with proper error handling
3. Ensure errors are logged at appropriate levels
4. Add error response types for HTTP handlers

```rust
// crates/adapters/adapter_http_axum/src/error.rs
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use minihub_domain::error::MiniHubError;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for MiniHubError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            MiniHubError::NotFound(err) => (StatusCode::NOT_FOUND, err.to_string()),
            MiniHubError::Validation(err) => (StatusCode::BAD_REQUEST, err.to_string()),
            MiniHubError::Storage(err) => {
                tracing::error!("storage error: {err:?}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            },
            MiniHubError::Internal(err) => {
                tracing::error!("internal error: {err:?}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            },
        };
        
        (status, Json(ErrorResponse { error: message })).into_response()
    }
}

// Usage in handlers
pub async fn get_entity(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<EntityResponse>, MiniHubError> {
    let entity_id = EntityId::from_str(&id)?;
    
    let entity = state.entity_service.get_entity(entity_id).await?;
    
    Ok(Json(EntityResponse::from(entity)))
}
```

**M5-T5: CSS styling for dashboard**

```rust
// Add CSS to all maud templates
fn base_layout(content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "minihub Dashboard" }
                style {
                    r#"
                    * {
                        box-sizing: border-box;
                        margin: 0;
                        padding: 0;
                    }
                    
                    body {
                        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
                        line-height: 1.6;
                        color: #333;
                        background: #f5f5f5;
                    }
                    
                    .container {
                        max-width: 1200px;
                        margin: 0 auto;
                        padding: 1rem;
                    }
                    
                    nav {
                        background: #2c3e50;
                        padding: 1rem;
                        margin-bottom: 2rem;
                    }
                    
                    nav a {
                        color: white;
                        text-decoration: none;
                        margin-right: 1.5rem;
                        font-weight: 500;
                    }
                    
                    nav a:hover {
                        text-decoration: underline;
                    }
                    
                    h1 {
                        color: white;
                        margin-bottom: 1rem;
                    }
                    
                    h2 {
                        margin-bottom: 1rem;
                        color: #2c3e50;
                    }
                    
                    .card {
                        background: white;
                        border-radius: 8px;
                        padding: 1.5rem;
                        margin-bottom: 1rem;
                        box-shadow: 0 2px 4px rgba(0,0,0,0.1);
                    }
                    
                    table {
                        width: 100%;
                        border-collapse: collapse;
                    }
                    
                    th, td {
                        text-align: left;
                        padding: 0.75rem;
                        border-bottom: 1px solid #ddd;
                    }
                    
                    th {
                        background: #f8f9fa;
                        font-weight: 600;
                    }
                    
                    button, .button {
                        background: #3498db;
                        color: white;
                        border: none;
                        padding: 0.5rem 1rem;
                        border-radius: 4px;
                        cursor: pointer;
                        text-decoration: none;
                        display: inline-block;
                    }
                    
                    button:hover, .button:hover {
                        background: #2980b9;
                    }
                    
                    button.danger {
                        background: #e74c3c;
                    }
                    
                    button.danger:hover {
                        background: #c0392b;
                    }
                    
                    .state-on {
                        color: #27ae60;
                        font-weight: 600;
                    }
                    
                    .state-off {
                        color: #95a5a6;
                    }
                    
                    .state-unavailable {
                        color: #e74c3c;
                    }
                    
                    form {
                        display: inline-block;
                        margin-right: 0.5rem;
                    }
                    
                    @media (max-width: 768px) {
                        nav a {
                            display: block;
                            margin-bottom: 0.5rem;
                        }
                        
                        table {
                            font-size: 0.9rem;
                        }
                        
                        th, td {
                            padding: 0.5rem;
                        }
                    }
                    "#
                }
            }
            body {
                nav {
                    .container {
                        h1 { "minihub" }
                        div {
                            a href="/" { "Home" }
                            a href="/entities" { "Entities" }
                            a href="/devices" { "Devices" }
                            a href="/areas" { "Areas" }
                            a href="/events" { "Events" }
                            a href="/automations" { "Automations" }
                        }
                    }
                }
                .container {
                    (content)
                }
            }
        }
    }
}
```

**M5-T6: Final coverage push**

```bash
# Run coverage report
cargo llvm-cov --all --html

# Open coverage report
open target/llvm-cov/html/index.html

# Identify uncovered code
# Add tests for uncovered branches, error paths, edge cases
# Focus on business logic and error handling
```

---

### M6 — MQTT Integration (Stretch) ✅

**Goal**: Add real device connectivity via MQTT protocol. Maintain 80% coverage.

**Status**: Done (branch: M6-mqtt-integration)

**Prerequisites**: M5 complete, M4-T1 (Integration trait)

**Deliverables**: MQTT adapter that connects to broker, subscribes to topics, translates messages to entities and events.

#### Tasks

| Task ID | Description | Effort | Dependencies | DoD | Key Files |
|---------|-------------|--------|--------------|-----|-----------|
| M6-T1 | ✅ Choose MQTT client crate | S | None | Evaluate rumqttc vs paho-mqtt. Consider async support, reconnection, TLS. Write ADR with decision rationale. Add dependency. | `DECISIONS.md` |
| M6-T2 | ✅ Implement MQTT adapter | L | M6-T1, M4-T1 | `MqttIntegration` implements `Integration` trait. Connects to broker (configurable host/port). Subscribes to discovery topic. Translates MQTT messages to entity state updates. Publishes service calls as MQTT commands. Handles reconnection. Proper error handling. | `crates/adapters/adapter_mqtt/src/lib.rs`, `crates/adapters/adapter_mqtt/src/config.rs`, `crates/adapters/adapter_mqtt/src/error.rs` |
| M6-T3 | ✅ MQTT integration tests | M | M6-T2 | 28 unit tests covering config, error types, discovery parsing, service calls, edge cases. Coverage >= 80% overall (87.55%). | `crates/adapters/adapter_mqtt/src/lib.rs` (inline tests) |

#### Detailed Task Breakdown

**M6-T1: Choose MQTT client crate**

Evaluate options:

1. **rumqttc**: Pure Rust, async-first, good performance
2. **paho-mqtt**: Mature, C bindings, feature-complete

Write ADR:

```markdown
## ADR-005: MQTT Client Library

**Status**: Accepted

**Context**: We need an MQTT client library for device connectivity.

**Decision**: Use rumqttc for MQTT client.

**Rationale**:
- Pure Rust implementation (no C dependencies)
- Async/await native support
- Good performance and active development
- Simpler build process than paho-mqtt
- Sufficient features for home automation use case

**Consequences**:
- Pure Rust stack
- Easier cross-compilation
- Good integration with tokio ecosystem
```

Add dependency:

```toml
[dependencies]
rumqttc = "0.24"
```

**M6-T2: Implement MQTT adapter**

```rust
// crates/adapters/adapter_mqtt/src/lib.rs
use async_trait::async_trait;
use minihub_app::integration::{Integration, IntegrationContext};
use minihub_domain::{Device, Entity, EntityState};
use minihub_domain::error::Result;
use rumqttc::{AsyncClient, MqttOptions, QoS, Event, Packet};
use serde::{Deserialize, Serialize};

mod client;
mod discovery;

pub struct MqttIntegration {
    config: MqttConfig,
    client: Option<AsyncClient>,
    context: Option<IntegrationContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    pub broker_host: String,
    pub broker_port: u16,
    pub client_id: String,
    pub discovery_prefix: String,
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            broker_host: "localhost".to_string(),
            broker_port: 1883,
            client_id: "minihub".to_string(),
            discovery_prefix: "homeassistant".to_string(),
        }
    }
}

impl MqttIntegration {
    pub fn new(config: MqttConfig) -> Self {
        Self {
            config,
            client: None,
            context: None,
        }
    }
}

#[async_trait]
impl Integration for MqttIntegration {
    fn name(&self) -> &str {
        "mqtt"
    }
    
    async fn initialize(&mut self) -> Result<()> {
        tracing::info!("initializing MQTT integration");
        
        let mut mqttoptions = MqttOptions::new(
            &self.config.client_id,
            &self.config.broker_host,
            self.config.broker_port,
        );
        mqttoptions.set_keep_alive(std::time::Duration::from_secs(30));
        
        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        self.client = Some(client);
        
        // Start event loop in background
        tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(event) => {
                        tracing::debug!("MQTT event: {:?}", event);
                    },
                    Err(e) => {
                        tracing::error!("MQTT error: {}", e);
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    }
                }
            }
        });
        
        Ok(())
    }
    
    async fn start(&mut self, context: IntegrationContext) -> Result<()> {
        tracing::info!("starting MQTT integration");
        self.context = Some(context.clone());
        
        let client = self.client.as_ref()
            .ok_or(MqttError::ClientNotInitialized)?;
        
        // Subscribe to discovery topic
        let discovery_topic = format!("{}/+/+/config", self.config.discovery_prefix);
        client.subscribe(&discovery_topic, QoS::AtLeastOnce)
            .await?;
        
        tracing::info!("subscribed to discovery topic: {}", discovery_topic);
        
        // Subscribe to state topics (would be dynamic based on discovered devices)
        
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        tracing::info!("stopping MQTT integration");
        
        if let Some(client) = &self.client {
            client.disconnect().await?;
        }
        
        Ok(())
    }
    
    async fn discover_devices(&self) -> Result<Vec<(Device, Vec<Entity>)>> {
        // Discovery would be event-driven via MQTT messages
        // Return empty initially, devices added as they announce themselves
        Ok(Vec::new())
    }
    
    async fn handle_service_call(
        &self,
        entity_id: EntityId,
        service: &str,
        data: serde_json::Value,
    ) -> Result<()> {
        let client = self.client.as_ref()
            .ok_or(MqttError::ClientNotInitialized)?;
        
        // Publish command to appropriate topic
        // Format depends on device protocol (e.g., Home Assistant MQTT, Tasmota, etc.)
        let command_topic = format!("minihub/{}/set", entity_id);
        let payload = serde_json::json!({
            "service": service,
            "data": data,
        }).to_string();
        
        client.publish(command_topic, QoS::AtLeastOnce, false, payload)
            .await?;
        
        Ok(())
    }
}

// crates/adapters/adapter_mqtt/src/discovery.rs
// Handle Home Assistant MQTT discovery protocol
#[derive(Debug, Deserialize)]
pub struct DiscoveryMessage {
    pub name: String,
    pub device_class: Option<String>,
    pub state_topic: String,
    pub command_topic: Option<String>,
    pub unique_id: String,
    pub device: DeviceInfo,
}

#[derive(Debug, Deserialize)]
pub struct DeviceInfo {
    pub name: String,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub identifiers: Vec<String>,
}

impl DiscoveryMessage {
    pub fn into_device_and_entity(self) -> Result<(Device, Entity)> {
        // Convert MQTT discovery message into minihub Device and Entity
        let device = Device::builder()
            .id(DeviceId::new())
            .name(self.device.name)
            .manufacturer(self.device.manufacturer)
            .model(self.device.model)
            .build()?;
        
        let entity = Entity::builder()
            .id(EntityId::new())
            .device_id(device.id)
            .entity_id(self.unique_id)
            .friendly_name(self.name)
            .state(EntityState::Unknown)
            .build()?;
        
        Ok((device, entity))
    }
}
```

**M6-T3: MQTT integration tests**

```yaml
# crates/adapters/adapter_mqtt/tests/docker-compose.yml
version: '3.8'
services:
  mosquitto:
    image: eclipse-mosquitto:2
    ports:
      - "1883:1883"
    volumes:
      - ./mosquitto.conf:/mosquitto/config/mosquitto.conf
```

```
# mosquitto.conf
listener 1883
allow_anonymous true
```

```rust
// crates/adapters/adapter_mqtt/tests/integration_test.rs
use adapter_mqtt::{MqttIntegration, MqttConfig};
use minihub_app::integration::Integration;
use rumqttc::{AsyncClient, MqttOptions, QoS};

#[tokio::test]
#[ignore] // Requires Docker
async fn test_mqtt_connection() {
    // Ensure mosquitto is running
    // docker-compose up -d
    
    let config = MqttConfig {
        broker_host: "localhost".to_string(),
        broker_port: 1883,
        ..Default::default()
    };
    
    let mut integration = MqttIntegration::new(config);
    integration.initialize().await.unwrap();
    
    // Test connection by publishing and receiving message
    // ...
    
    integration.stop().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_mqtt_discovery() {
    // Publish discovery message
    // Verify device and entity are created
}

#[tokio::test]
#[ignore]
async fn test_mqtt_state_updates() {
    // Publish state update messages
    // Verify entity state changes
}

#[tokio::test]
#[ignore]
async fn test_mqtt_service_calls() {
    // Call service
    // Verify MQTT command is published
}
```

---

### M7 — Passive BLE Integration ✅

**Goal**: Add passive BLE sensor support for Xiaomi LYWSD03MMC thermometers running ATC/PVVX custom firmware. Parse BLE advertisements to expose temperature, humidity, and battery data as entities. Maintain 80% coverage.

**Status**: ✅ Done

**Prerequisites**: M5 complete (Integration trait)

**Deliverables**: `adapter_ble` crate with btleplug-based passive scanner, PVVX and ATC1441 payload parsers, config/wiring in minihubd.

#### Tasks

| Task ID | Description | Effort | Dependencies | DoD | Key Files |
|---------|-------------|--------|--------------|-----|-----------|
| M7-T1 | ✅ ADR-010 + crate scaffold | S | None | ADR-010 documents btleplug choice. `adapter_ble` crate in workspace. `cargo check --all` passes. | `docs/DECISIONS.md`, `Cargo.toml`, `crates/adapters/adapter_ble/Cargo.toml`, `crates/adapters/adapter_ble/src/lib.rs` |
| M7-T2 | ✅ BLE config + error types | S | M7-T1 | `BleConfig` with scan/update/filter fields and defaults. `BleError` with typed variants. Unit tests for defaults and display. | `crates/adapters/adapter_ble/src/config.rs`, `crates/adapters/adapter_ble/src/error.rs` |
| M7-T3 | ✅ Advertisement payload parser | M | M7-T2 | `parse_pvvx` (19 bytes LE) and `parse_atc1441` (13 bytes BE) for UUID `0x181A`. `SensorReading` struct. `format_mac`/`mac_slug` helpers. Comprehensive unit tests. | `crates/adapters/adapter_ble/src/parser.rs` |
| M7-T4 | ✅ Implement `BleIntegration` | L | M7-T3 | `BleIntegration` implements `Integration` trait. `setup()` scans via btleplug, parses advertisements, builds devices/entities. `handle_service_call()` returns entity unchanged (read-only sensor). `teardown()` aborts scan task. MAC filtering. Unit tests. | `crates/adapters/adapter_ble/src/lib.rs` |
| M7-T5 | ✅ Wire into minihubd config + main | S | M7-T4 | `BleIntegrationConfig` in config.rs. TOML section `[integrations.ble]`. Env overrides `MINIHUB_BLE_ENABLED`, `MINIHUB_BLE_SCAN_DURATION_SECS`. Wiring block in main.rs. Config tests updated. | `crates/bin/minihubd/Cargo.toml`, `crates/bin/minihubd/src/config.rs`, `crates/bin/minihubd/src/main.rs` |
| M7-T6 | ✅ Tests + coverage | M | M7-T5 | All quality gates pass. `cargo test --all` passes (296 tests). `cargo llvm-cov` >= 80% (86.75%). | All crate test modules |

---

## Task Dependencies Graph

```
M0 (Scaffold)
 └─> M1 (Domain & App)
      └─> M2 (HTTP + Storage + Dashboard)
           └─> M3 (Events & Automations)
                └─> M4 (Virtual Integration)
                     └─> M5 (Polish)
                          ├─> M6 (MQTT) [Stretch]
                          └─> M7 (BLE)
```

Within each milestone, tasks have dependencies noted in the DoD column.

---

## Glossary

- **DoD**: Definition of Done
- **ADR**: Architecture Decision Record
- **CRUD**: Create, Read, Update, Delete
- **PRG**: POST-REDIRECT-GET pattern
- **SSR**: Server-Side Rendering
- **MQTT**: Message Queuing Telemetry Transport protocol
- **TLS**: Transport Layer Security

---

## Notes

- All quality gates must pass before a milestone is considered complete
- Coverage targets are cumulative (each milestone should maintain or improve coverage)
- Tasks marked with effort "L" (Large) may need to be split into subtasks during execution
- Integration tests require external resources (database, MQTT broker) and may be slower
- Use `#[ignore]` attribute for tests requiring Docker or external services
- Document all architectural decisions in `DECISIONS.md` using ADR format

---

**End of Implementation Plan**
