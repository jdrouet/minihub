# minihub Architecture

## Overview

**minihub** is a tiny, Rust-only home automation server built using **hexagonal architecture** (also known as ports and adapters architecture). This architectural pattern keeps the core business logic independent of external frameworks, databases, and protocols.

### Why Hexagonal Architecture?

1. **Independence**: Core domain logic doesn't depend on any specific framework, database, or protocol
2. **Testability**: Business logic can be tested in isolation with mock implementations
3. **Flexibility**: Swap out implementations (e.g., SQLite to PostgreSQL, HTTP to gRPC) without touching core logic
4. **Clarity**: Clear boundaries between what is domain logic and what is infrastructure

### What It Means for minihub

- The **core** domain model (entities, devices, areas, automations) knows nothing about HTTP, SQLite, or MQTT
- The **application** layer defines contracts (ports) but doesn't implement infrastructure
- **Native adapters** implement those contracts using concrete technologies (axum, sqlx, rumqttc, btleplug)
- The **WASM dashboard adapter** is a Leptos CSR app that depends on domain types only and communicates with the server via HTTP/SSE
- The **binary** wires all native adapters together at runtime and serves the pre-built WASM dashboard as static files

---

## Layer Descriptions

### 1. `crates/domain` — Pure Domain Model

**Responsibilities:**
- Common types used across all layers (entity IDs, timestamps)
- Error types and result wrappers
- Utility functions with zero external dependencies
- Domain entities: `Entity`, `Device`, `Area`, `Service`, `Event`, `Automation`
- Domain rules and invariants (e.g., validation logic)
- Domain events that represent state changes
- Pure business logic with zero infrastructure concerns

**Dependencies:** None (pure Rust + std only)

**Forbidden:**
- No database types (no `sqlx`, no ORM)
- No HTTP types (no `axum`, no `hyper`)
- No serialization framework assumptions (no `serde` derive in public API)
- No I/O operations

**Examples:**
- `EntityId`, `DeviceId`, `AreaId`
- `Timestamp`, `Duration`
- Base error types
- `Device::new()` validates device properties
- `Automation::evaluate()` determines if triggers are met
- `Area::add_device()` maintains parent-child relationships
- `EntityHistory` records time-series state/attribute snapshots for sensor data

---

### 2. `crates/app` — Use Cases & Port Definitions

**Responsibilities:**
- Application use cases (business workflows)
- **Port trait definitions** (contracts for infrastructure)
- Application-level error handling
- Orchestration of domain entities

**Dependencies:** `minihub-domain` only

**Port Traits (examples):**
```rust
// Driven ports (called by app layer)
pub trait DeviceRepository {
    async fn find_by_id(&self, id: DeviceId) -> Result<Option<Device>>;
    async fn save(&self, device: Device) -> Result<()>;
    async fn list_all(&self) -> Result<Vec<Device>>;
}

pub trait EventBus {
    async fn publish(&self, event: Event) -> Result<()>;
}

// Driving ports (call into app layer)
pub trait DeviceService {
    async fn toggle_device(&self, id: DeviceId) -> Result<Device>;
    async fn get_device(&self, id: DeviceId) -> Result<Device>;
}
```

**Use Cases (examples):**
- `ToggleDeviceUseCase`: loads device, applies domain logic, persists changes
- `ListDevicesByAreaUseCase`: queries repository, aggregates data
- `TriggerAutomationUseCase`: evaluates automation rules, executes actions

---

### 3. `crates/adapters/*` — Infrastructure Implementations

#### `adapter_http_axum`
**Responsibilities:**
- REST API endpoints (JSON responses)
- Serves Leptos WASM dashboard as static files
- SSE endpoint for real-time entity state push
- HTTP request/response handling
- Implements **driving ports** (receives external requests)

**Dependencies:** `minihub-app`, `minihub-domain`, `axum`, `serde`, `tower-http` (static files)

---

#### `adapter_dashboard_leptos`
**Responsibilities:**
- Client-side dashboard UI compiled to WASM via Leptos (CSR mode)
- Pages: home overview, devices, entities (with state control), areas, events, automations
- Time-series charts for sensor history using `plotters`
- Real-time updates via SSE subscription
- Consumes `/api/*` JSON endpoints from `adapter_http_axum`

**Dependencies:** `minihub-domain` (shared types for API responses), `leptos`, `gloo-net`, `plotters`

**Build:** Compiled to `wasm32-unknown-unknown` via `trunk`, output served as static assets by axum

---

#### `adapter_storage_sqlite_sqlx`
**Responsibilities:**
- SQLite database schema and migrations
- Implements **driven ports** like `DeviceRepository`, `AreaRepository`, etc.
- Data marshalling between domain entities and SQL rows

**Dependencies:** `minihub-app`, `minihub-domain`, `sqlx`, `sqlite`

---

#### `adapter_virtual`
**Responsibilities:**
- Built-in demo/testing integration with simulated devices
- Provides virtual Light, Sensor, and Switch devices
- Responds to service calls (turn on/off, toggle) with in-memory state
- Implements the `Integration` port trait for device discovery and service call handling

**Dependencies:** `minihub-app`, `minihub-domain`

---

#### `adapter_mqtt`
**Responsibilities:**
- MQTT client connection and reconnection (via `rumqttc`)
- Device discovery via config topics
- State updates via state topics
- Service call publishing to set topics
- Implements the `Integration` port trait

**Dependencies:** `minihub-app`, `minihub-domain`, `rumqttc`

---

#### `adapter_ble`
**Responsibilities:**
- Passive BLE scanning for sensor advertisements (via `btleplug`)
- Decodes PVVX custom and ATC1441 advertisement formats
- Exposes Xiaomi LYWSD03MMC sensors as minihub devices/entities (temperature, humidity, battery)
- Implements the `Integration` port trait

**Dependencies:** `minihub-app`, `minihub-domain`, `btleplug`

---

### 4. `crates/bin/minihubd` — Composition Root

**Responsibilities:**
- Application entry point (`fn main()`)
- Wiring up all adapters and use cases (dependency injection)
- Configuration loading
- Runtime initialization (HTTP server, database connection, etc.)

**Dependencies:** All crates (`minihub-app`, `minihub-domain`, all adapters)

**Example:**
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Load config
    let config = load_config()?;
    
    // Initialize adapters
    let db_pool = SqlitePool::connect(&config.database_url).await?;
    let device_repo = SqliteDeviceRepository::new(db_pool.clone());
    let event_bus = MqttEventBus::connect(&config.mqtt_url).await?;
    
    // Initialize use cases
    let device_service = DeviceServiceImpl::new(device_repo, event_bus);
    
    // Start HTTP server
    let app = HttpAdapter::new(device_service);
    axum::Server::bind(&config.listen_addr)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}
```

---

## Crate Dependency Graph

```
┌──────────────────────────────────────────────────────────────────┐
│                       crates/bin/minihubd                       │
│                       (Composition Root)                        │
└───────────────────────────┬──────────────────────────────────────┘
                            │ depends on all native adapters
     ┌──────────┬───────────┼───────────┬───────────┐
     │          │           │           │           │
     ▼          ▼           ▼           ▼           ▼
┌─────────┐┌─────────┐┌─────────┐┌─────────┐┌─────────┐
│ adapter  ││ adapter  ││ adapter  ││ adapter  ││ adapter  │
│http_axum ││storage   ││ virtual  ││  mqtt    ││  ble     │
│(API+SSE+ ││sqlite_sqlx│          ││(rumqttc) ││(btleplug)│
│ static)  ││          ││          ││          ││          │
└────┬─────┘└────┬─────┘└────┬─────┘└────┬─────┘└────┬─────┘
     │           │           │           │           │
     └───────────┴───────────┴─────┬─────┴───────────┘
                                   │ depend on
                                   ▼
                            ┌─────────────┐
                            │ crates/app  │
                            │  (Use Cases │
                            │  & Ports)   │
                            └──────┬──────┘
                                   │ depends on
                                   ▼
                            ┌─────────────┐
                            │crates/domain│
                            │  (Domain &  │
                            │ Foundation) │
                            └──────┬──────┘
                                   │ shared types
                                   ▼
                     ┌──────────────────────────┐
                     │  adapter_dashboard_leptos │
                     │   (WASM, built via trunk) │
                     │   depends on domain only  │
                     └──────────────────────────┘
```

**Note:** `adapter_dashboard_leptos` is compiled separately to `wasm32-unknown-unknown` via `trunk`. It depends only on `minihub-domain` for shared API response types. It does NOT depend on `app` or any other adapter — it communicates with the server exclusively via HTTP/SSE.

**Allowed Dependencies:**
- `domain` → (none)
- `app` → `domain`
- `adapter_*` (native) → `app`, `domain`
- `adapter_dashboard_leptos` (WASM) → `domain` only (communicates with server via HTTP)
- `minihubd` → all native crates (not the WASM dashboard — it is built separately via `trunk`)

---

## Dependency Rules

These rules are **enforced at the workspace level** and must never be violated:

1. **Domain has NO internal dependencies**
   - No database libraries
   - No HTTP frameworks
   - No serialization frameworks in public API
   - Pure Rust + std only
   
2. **App depends on domain only**
   - Defines port traits (interfaces)
   - No concrete adapter implementations
   - No framework-specific types

3. **Adapters depend on app + domain**
   - Implement port traits from `app`
   - May use any external framework or library
   - Must not depend on other adapters

4. **Binary depends on all**
   - Only place where adapters are instantiated
   - Wires everything together
   - Contains no business logic

5. **NO adapter types in domain or app**
   - Domain and app must remain framework-agnostic
   - Example: do NOT put `axum::Json` in a use case return type

6. **NO framework types in domain or app**
   - No `sqlx::Row`, no `axum::Router`, no `mqtt::Client`
   - Use domain types only

---

## Ports & Adapters Explanation

### What Are Ports?

**Ports** are **trait definitions** in the `crates/app` layer that define contracts for external interactions. They are interfaces without implementation.

**Two types of ports:**

1. **Driving Ports (Primary/API)**: How the outside world **calls into** your application
   - Example: `DeviceService`, `AutomationService`
   - **Implemented in:** `app` crate (use cases implement these traits)
   - **Called by:** Adapters (e.g., HTTP adapter calls `DeviceService::toggle_device()`)

2. **Driven Ports (Secondary/SPI)**: How your application **calls out to** infrastructure
   - Example: `DeviceRepository`, `EventBus`, `TimeProvider`
   - **Implemented in:** Adapter crates (e.g., `SqliteDeviceRepository`)
   - **Called by:** Use cases in the `app` layer

### What Are Adapters?

**Adapters** are **concrete implementations** of port traits, located in `crates/adapters/*`.

**Examples:**
- `adapter_http_axum` implements driving adapter (receives HTTP requests, calls use cases)
- `adapter_storage_sqlite_sqlx` implements `DeviceRepository` (driven port)
- `adapter_mqtt` implements `EventBus` (driven port)

### Direction of Dependencies

```
External World (HTTP, MQTT, CLI)
        │
        ▼
  ┌──────────────┐
  │   Adapters   │ ◄─── implements driving ports
  │  (Driving)   │      (receives requests)
  └──────┬───────┘
         │ calls
         ▼
  ┌──────────────┐
  │  Use Cases   │
  │  (App Layer) │
  └──────┬───────┘
         │ calls
         ▼
  ┌──────────────┐
  │   Adapters   │ ◄─── implements driven ports
  │   (Driven)   │      (provides data/services)
  └──────┬───────┘
         │
         ▼
External World (Database, MQTT broker, Filesystem)
```

---

## How to Add a New Integration

Let's say you want to add a **Zigbee adapter** to control Zigbee devices.

### Step 1: Create a New Adapter Crate

```bash
mkdir -p crates/adapters/adapter_zigbee
cd crates/adapters/adapter_zigbee
cargo init --lib
```

### Step 2: Add Dependencies

In `crates/adapters/adapter_zigbee/Cargo.toml`:

```toml
[package]
name = "minihub-adapter-zigbee"
version.workspace = true
edition.workspace = true

[dependencies]
minihub-app = { workspace = true }
minihub-domain = { workspace = true }
zigbee-rs = "0.5"  # example external library
tokio = { version = "1", features = ["full"] }
```

### Step 3: Implement Port Traits from `minihub-app`

In `crates/adapters/adapter_zigbee/src/lib.rs`:

```rust
use minihub_app::ports::{DeviceController, EventBus};
use minihub_domain::{Device, DeviceId, Event};

pub struct ZigbeeAdapter {
    client: zigbee_rs::Client,
}

impl ZigbeeAdapter {
    pub async fn connect(coordinator_path: &str) -> Result<Self> {
        let client = zigbee_rs::Client::connect(coordinator_path).await?;
        Ok(Self { client })
    }
}

#[async_trait]
impl DeviceController for ZigbeeAdapter {
    async fn send_command(&self, device_id: DeviceId, command: Command) -> Result<()> {
        // Translate domain command to Zigbee protocol
        self.client.send_zigbee_command(device_id, command).await?;
        Ok(())
    }
}

#[async_trait]
impl EventBus for ZigbeeAdapter {
    async fn publish(&self, event: Event) -> Result<()> {
        // Optional: publish to Zigbee network if needed
        Ok(())
    }
}
```

### Step 4: Wire It in `minihubd`

In `crates/bin/minihubd/src/main.rs`:

```rust
use minihub_adapter_zigbee::ZigbeeAdapter;

#[tokio::main]
async fn main() -> Result<()> {
    // ... existing setup ...
    
    // Initialize Zigbee adapter
    let zigbee = ZigbeeAdapter::connect("/dev/ttyUSB0").await?;
    
    // Pass it to use cases
    let device_service = DeviceServiceImpl::new(
        device_repo,
        Arc::new(zigbee),  // as EventBus or DeviceController
    );
    
    // ... rest of setup ...
}
```

### Step 5: Add to Workspace Members

In root `Cargo.toml`:

```toml
[workspace]
members = [
    # ... existing members ...
    "crates/adapters/adapter_zigbee",
    # NOTE: adapter_dashboard_leptos is NOT a workspace member — it is
    # compiled separately to wasm32-unknown-unknown via trunk.
]

[workspace.dependencies]
minihub-adapter-zigbee = { path = "crates/adapters/adapter_zigbee" }
```

### Step 6: Test Independently

Write integration tests in `crates/adapters/adapter_zigbee/tests/`:

```rust
#[tokio::test]
async fn test_zigbee_connection() {
    let adapter = ZigbeeAdapter::connect("mock://coordinator").await.unwrap();
    // Test adapter behavior
}
```

---

## Data Flow Examples

### Example: User Toggles a Light via Dashboard

**Request Flow:**

1. **User Action**: Clicks "Toggle" button in the Leptos dashboard (WASM running in browser)

2. **Dashboard** (`adapter_dashboard_leptos`, WASM):
   - Leptos component calls `PUT /api/entities/{id}/state` via `gloo-net` fetch
   - Sends JSON body with new state

3. **HTTP Adapter** (`adapter_http_axum`):
   - Receives PUT request at `/api/entities/{id}/state`
   - Extracts `EntityId` from path
   - Calls `EntityService::update_entity_state(entity_id, new_state)`

3. **Use Case** (`app/device_service.rs`):
   ```rust
   async fn toggle_device(&self, id: DeviceId) -> Result<Device> {
       // Load device from repository (driven port)
       let device = self.device_repo.find_by_id(id).await?
           .ok_or(DeviceError::NotFound)?;
       
       // Apply domain logic (domain layer)
       let updated_device = device.toggle()?;
       
       // Persist changes (driven port)
       self.device_repo.save(updated_device.clone()).await?;
       
       // Publish event (driven port)
       let event = Event::DeviceStateChanged { device_id: id };
       self.event_bus.publish(event).await?;
       
       Ok(updated_device)
   }
   ```

4. **Domain Model** (`domain/device.rs`):
   ```rust
   impl Device {
       pub fn toggle(&self) -> Result<Self> {
           let new_state = match self.state {
               DeviceState::On => DeviceState::Off,
               DeviceState::Off => DeviceState::On,
           };
           Ok(Self { state: new_state, ..self.clone() })
       }
   }
   ```

5. **Storage Adapter** (`adapter_storage_sqlite_sqlx`):
   - `SqliteDeviceRepository::save()` executes UPDATE query
   - Commits transaction

6. **Event Bus Adapter** (`adapter_mqtt`):
   - `MqttEventBus::publish()` sends MQTT message to topic `minihub/events/device_state_changed`

7. **HTTP Adapter** (returns):
   - Serializes updated `Device` to JSON
   - Returns HTTP 200 with device state

8. **SSE Push** (`adapter_http_axum`):
   - Event bus broadcast triggers SSE frame on `/api/events/stream`
   - Connected dashboard clients receive the `StateChanged` event in real time

9. **Entity History** (`adapter_storage_sqlite_sqlx`):
   - Event worker (in `minihubd`) listens for `StateChanged` events
   - Appends an `EntityHistory` record with a snapshot of the entity state/attributes
   - Background purge task periodically removes records older than the retention period

**Crate Involvement:**
- `adapter_dashboard_leptos` (WASM): User clicks toggle, sends API request, SSE subscription reactively updates UI
- `adapter_http_axum`: Receives HTTP request, returns JSON response, pushes SSE event to connected clients
- `app`: Orchestrates the workflow via use case
- `domain`: Provides `EntityId`, `EntityHistory`, error types, validates and applies domain logic
- `adapter_storage_sqlite_sqlx`: Persists state change to database, records entity history
- `adapter_mqtt`: Publishes event to MQTT topic (if entity is MQTT-managed)

---

## Testing Strategy

### 1. Unit Tests (Domain & App Layers)

**Where:** `crates/domain/tests/`, `crates/app/tests/`

**What:** Test domain logic and use cases in isolation

**How:** Use **mock implementations** of port traits

```rust
// Mock repository for testing
struct MockDeviceRepository {
    devices: Arc<Mutex<HashMap<DeviceId, Device>>>,
}

#[async_trait]
impl DeviceRepository for MockDeviceRepository {
    async fn find_by_id(&self, id: DeviceId) -> Result<Option<Device>> {
        Ok(self.devices.lock().unwrap().get(&id).cloned())
    }
    
    async fn save(&self, device: Device) -> Result<()> {
        self.devices.lock().unwrap().insert(device.id, device);
        Ok(())
    }
}

#[tokio::test]
async fn test_toggle_device_use_case() {
    let repo = MockDeviceRepository::new();
    let event_bus = MockEventBus::new();
    let service = DeviceServiceImpl::new(repo, event_bus);
    
    // Given: a device in "off" state
    let device = Device::new("Light", DeviceState::Off);
    repo.save(device.clone()).await.unwrap();
    
    // When: toggling the device
    let result = service.toggle_device(device.id).await.unwrap();
    
    // Then: device state is "on"
    assert_eq!(result.state, DeviceState::On);
}
```

**Benefits:**
- Fast (no I/O)
- Isolated (no external dependencies)
- Validates business logic correctness

---

### 2. Integration Tests (Adapter Layers)

**Where:** `crates/adapters/*/tests/`

**What:** Test adapters against real or embedded infrastructure

**How:** Use real database (SQLite in-memory), mock MQTT broker, etc.

```rust
#[tokio::test]
async fn test_sqlite_repository_save_and_find() {
    // Given: in-memory SQLite database
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
    let repo = SqliteDeviceRepository::new(pool);
    
    // When: saving a device
    let device = Device::new("Light", DeviceState::Off);
    repo.save(device.clone()).await.unwrap();
    
    // Then: device can be retrieved
    let found = repo.find_by_id(device.id).await.unwrap().unwrap();
    assert_eq!(found.name, "Light");
}
```

**Benefits:**
- Validates adapter implementation correctness
- Catches SQL syntax errors, schema mismatches, etc.
- Still fast (in-memory databases)

---

### 3. End-to-End Tests (HTTP API)

**Where:** `crates/bin/minihubd/tests/` or separate `tests/` directory at workspace root

**What:** Test the entire application via HTTP API

**How:** Start the server in a test process, make real HTTP requests

```rust
#[tokio::test]
async fn test_toggle_device_via_http() {
    // Given: application running on localhost
    let app = spawn_test_app().await;
    let client = reqwest::Client::new();
    
    // When: POST to /api/devices/{id}/toggle
    let response = client
        .post(&format!("{}/api/devices/{}/toggle", app.url, device_id))
        .send()
        .await
        .unwrap();
    
    // Then: response is 200 with updated device
    assert_eq!(response.status(), 200);
    let device: Device = response.json().await.unwrap();
    assert_eq!(device.state, DeviceState::On);
}
```

**Benefits:**
- Validates the entire system integration
- Tests wiring in `minihubd`
- Catches HTTP routing issues, serialization bugs, etc.

---

## Summary

**minihub** uses hexagonal architecture to maintain a clean separation between:
- **Domain logic** (pure, framework-agnostic)
- **Application use cases** (orchestration with defined contracts)
- **Infrastructure adapters** (concrete implementations using external libraries)

This architecture enables:
- Easy testing at all levels
- Swapping implementations without changing business logic
- Adding new integrations (HTTP, MQTT, Zigbee, etc.) without modifying domain code
- Clear mental model of what depends on what

**Key Principle:** Dependencies flow inward. Domain is the center, adapters are the outside. Outside depends on inside, never the reverse.
