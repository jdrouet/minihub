# Architecture Decision Records

This document contains Architecture Decision Records (ADRs) for the minihub project.

minihub is a tiny Rust-only home automation server.

## Table of Contents

- [ADR-001: Use axum as the HTTP framework](#adr-001-use-axum-as-the-http-framework)
- [ADR-002: Use sqlx with SQLite for persistence](#adr-002-use-sqlx-with-sqlite-for-persistence)
- [ADR-003: No-JavaScript dashboard](#adr-003-no-javascript-dashboard)
- [ADR-004: Hexagonal architecture with Cargo workspace](#adr-004-hexagonal-architecture-with-cargo-workspace)
- [ADR-005: Dual MIT/Apache-2.0 license](#adr-005-dual-mitapache-20-license)
- [ADR-006: cargo-llvm-cov for code coverage](#adr-006-cargo-llvm-cov-for-code-coverage)
- [ADR-007: Use askama for HTML templating](#adr-007-use-askama-for-html-templating)
- [ADR-008: Trait-based integration model with lifecycle and context injection](#adr-008-trait-based-integration-model-with-lifecycle-and-context-injection)
- [ADR-009: Use rumqttc for MQTT client](#adr-009-use-rumqttc-for-mqtt-client)
- [ADR-010: Use btleplug for passive BLE scanning](#adr-010-use-btleplug-for-passive-ble-scanning)
- [ADR-011: Leptos WASM dashboard](#adr-011-leptos-wasm-dashboard)
- [ADR-012: Entity history for time-series sensor data](#adr-012-entity-history-for-time-series-sensor-data)

---

## ADR-001: Use axum as the HTTP framework

**Status:** Accepted

**Date:** 2026-02-08

### Context

The minihub project needs an async HTTP framework to handle REST API endpoints and serve the server-side rendered dashboard. The framework should integrate well with the async ecosystem, provide good performance, and have strong typing support.

### Decision

Use axum as the HTTP framework. axum is tower-based and maintained by the tokio team, providing excellent integration with the async ecosystem.

### Alternatives Considered

- **actix-web**: Mature and performant, but uses actor model which adds complexity
- **warp**: Filter-based approach, but less intuitive API and smaller ecosystem
- **poem**: Similar to axum but less mature and smaller community
- **rocket**: More ergonomic API with less boilerplate, but historically had less mature async support

### Consequences

**Positive:**
- Access to the entire tower middleware ecosystem
- Excellent async support through tight tokio integration
- Strong typing with extractors and handlers
- Active maintenance and strong community support

**Negative:**
- Slightly more boilerplate compared to rocket
- Steeper learning curve for developers unfamiliar with tower

---

## ADR-002: Use sqlx with SQLite for persistence

**Status:** Accepted

**Date:** 2026-02-08

### Context

The minihub project needs lightweight embedded persistence for storing device states, automation rules, and historical data. The solution should not require an external database server and should be easy to deploy as a single binary with a local database file.

### Decision

Use sqlx with SQLite for persistence. sqlx provides compile-time query checking and async database access without requiring a full ORM layer.

### Alternatives Considered

- **diesel**: Popular ORM with excellent type safety, but primarily synchronous and heavier weight
- **rusqlite**: Synchronous SQLite bindings, would require blocking thread pool for async usage
- **sled**: Pure Rust embedded database, but less mature and different query model
- **redb**: Pure Rust embedded database, but very new and minimal ecosystem

### Consequences

**Positive:**
- Compile-time safety with query checking via `sqlx::query!` macros
- Async support that integrates naturally with tokio
- Good migration support with sqlx-cli
- Standard SQL rather than custom query DSL

**Negative:**
- Requires sqlx CLI for offline mode and compile-time verification
- SQLite has concurrency limitations (single writer)
- Need to manage prepared statements and connection pooling

---

## ADR-003: No-JavaScript dashboard

**Status:** Superseded by [ADR-011](#adr-011-leptos-wasm-dashboard)

**Date:** 2026-02-08

### Context

The minihub dashboard needs to be simple, accessible, and work everywhere without requiring a complex frontend build pipeline. The Rust-only constraint means avoiding JavaScript-heavy solutions that would require maintaining a separate frontend codebase.

### Decision

Use server-side rendered HTML with standard HTML forms following the POST-Redirect-GET (PRG) pattern. Use meta refresh tags for periodic live updates of device states.

### Alternatives Considered

- **HTMX**: Adds progressive enhancement but still requires JavaScript runtime
- **Leptos/Yew WASM**: Rust-based frontend frameworks, but add significant build complexity
- **Traditional SPA (React/Vue)**: Would violate the Rust-only constraint and require separate build pipeline

### Consequences

**Positive:**
- Works in any browser, including text-mode browsers
- No frontend build pipeline or tooling required
- Simple to understand and maintain
- Excellent accessibility by default
- Fast initial page loads

**Negative:**
- No real-time updates without page reload or meta refresh
- Limited interactivity compared to JavaScript-based solutions
- Full page reloads on user actions
- Cannot use modern UI patterns like optimistic updates

### Supersession Note

This decision served the project well through M1–M7 but limits the dashboard UX for a real Home Assistant replacement. A home automation dashboard needs real-time device state updates, interactive charts for sensor history, and responsive controls — none of which are practical with pure SSR and meta-refresh. See [ADR-011](#adr-011-leptos-wasm-dashboard) for the replacement approach.

---

## ADR-004: Hexagonal architecture with Cargo workspace

**Status:** Accepted

**Date:** 2026-02-08

### Context

The project needs strict separation of concerns to ensure testability, maintainability, and the ability to swap out infrastructure adapters (e.g., different device protocols, storage backends) without affecting the core domain logic.

### Decision

Implement hexagonal (ports and adapters) architecture enforced through separate Cargo crates within a workspace. Core domain logic is isolated from infrastructure concerns through trait-based ports.

### Alternatives Considered

- **Monolithic crate structure**: Simpler initially but harder to enforce boundaries
- **Module-based separation**: Runtime boundaries only, no compile-time enforcement
- **Microservices**: Too heavy for a lightweight embedded project

### Consequences

**Positive:**
- Compile-time enforcement of dependency rules (core cannot depend on adapters)
- Easy to test core domain logic in isolation with mock adapters
- Clear architectural boundaries visible in the project structure
- Easier to add new adapters without modifying core logic

**Negative:**
- More crates to manage in the workspace
- Some boilerplate for trait definitions and adapter implementations
- Slightly longer compile times due to inter-crate dependencies

---

## ADR-005: Dual MIT/Apache-2.0 license

**Status:** Accepted

**Date:** 2026-02-08

### Context

The project should be open source with a license that is standard in the Rust ecosystem and provides maximum compatibility with downstream users and contributors.

### Decision

Use dual licensing under MIT OR Apache-2.0, following the standard Rust ecosystem convention.

### Alternatives Considered

- **MIT only**: Simpler but lacks explicit patent grant
- **Apache-2.0 only**: Provides patent grant but more verbose and some consider less permissive
- **GPL**: Too restrictive for a library/framework-style project

### Consequences

**Positive:**
- Maximum compatibility with the Rust ecosystem
- Downstream users can choose the license that works best for them
- Follows established community conventions
- Apache-2.0 provides explicit patent grant
- MIT provides simple, permissive terms

**Negative:**
- Slightly more complex license management (two license files)
- Contributors must agree to dual license terms

---

## ADR-006: cargo-llvm-cov for code coverage

**Status:** Accepted

**Date:** 2026-02-08

### Context

The project needs accurate code coverage measurement for both local development and CI workflows. The solution should work well with Cargo workspaces and produce reports that can be used in CI pipelines.

### Decision

Use cargo-llvm-cov as the code coverage tool. It leverages LLVM's source-based coverage instrumentation for accurate measurements.

### Alternatives Considered

- **tarpaulin**: Popular Rust coverage tool but slower and less accurate on some platforms (especially macOS)
- **grcov**: Requires more manual setup and configuration

### Consequences

**Positive:**
- Accurate coverage measurements using LLVM instrumentation
- Good HTML report generation for local development
- Excellent support for Cargo workspaces
- CI-friendly output formats (JSON, LCOV)
- Works consistently across platforms

**Negative:**
- Requires installing llvm-tools-preview rustup component
- Slightly more complex setup than some alternatives
- LLVM-based tooling can have larger binary sizes during testing

---

## ADR-007: Use askama for HTML templating

**Status:** Superseded by [ADR-011](#adr-011-leptos-wasm-dashboard)

**Date:** 2026-02-08

### Context

The no-JavaScript SSR dashboard needs an HTML templating approach. The templates must produce complete HTML pages server-side, integrate well with Rust's type system, and be easy to read and maintain.

### Decision

Use askama for compile-time-checked Jinja2-style HTML templates. Templates live as `.html` files alongside the Rust code, with type-safe variable binding verified at compile time.

### Alternatives Considered

- **maud**: Rust macro DSL for HTML. Keeps everything in Rust code but mixes markup with logic, and compatibility with the latest axum version can lag behind.
- **Manual string building**: No dependencies but error-prone, no compile-time guarantees, and poor ergonomics for anything beyond trivial markup.

### Consequences

**Positive:**
- Compile-time checked — template variables and types are verified during build
- Clean separation of HTML templates from Rust logic
- Familiar Jinja2/Django-style syntax accessible to non-Rust contributors
- Template inheritance for shared layout
- Straightforward axum integration via manual `IntoResponse` rendering to `Html`

**Negative:**
- Separate template files to manage alongside Rust code
- Jinja2 syntax has a learning curve for those unfamiliar with it
- Template errors surface as compile errors, which can be cryptic

### Supersession Note

With the move to a Leptos WASM dashboard (ADR-011), askama templates are no longer needed. The dashboard UI is now defined as Leptos components in Rust, compiled to WASM. The askama dependency and all `.html` template files are removed from `adapter_http_axum`.

---

## ADR-008: Trait-based integration model with lifecycle and context injection

**Status:** Accepted

**Date:** 2026-02-09

### Context

minihub needs a pluggable way for "integrations" (device protocol adapters such as virtual/demo devices, MQTT, Zigbee, etc.) to register devices and entities, respond to service calls, and optionally run background tasks. The design must fit within the hexagonal architecture: integrations are *driven* adapters that call *into* the application layer through port traits.

### Decision

Define an `Integration` trait in the `app` crate with explicit lifecycle methods (`setup`, `teardown`) and a callback-style `handle_service_call`. Integrations receive an `IntegrationContext` that exposes only the application services they need (entity/device creation, event publishing). The system manages the integration lifecycle; integrations do not hold direct references to repositories.

Key design choices:

- **No `dyn`**: The binary crate knows all concrete integration types at compile time, so integrations are stored in a generic `IntegrationManager<I>` rather than behind trait objects.
- **RPITIT**: The trait uses `impl Future` return types (RPITIT), consistent with all other port traits.
- **Context injection**: Integrations receive an `IntegrationContext` on `setup()` rather than constructor injection, because the context requires fully-wired services that are only available after the composition root assembles them.

### Alternatives Considered

- **Dynamic dispatch (`Box<dyn Integration>`)**: Simpler collection management but violates the project's no-`dyn` rule and adds indirection.
- **Channel-based message passing**: Integrations communicate via channels. More decoupled but adds complexity and makes error propagation harder.
- **Direct service injection in constructor**: Would create a circular dependency — integrations need services, but the binary constructs both.

### Consequences

**Positive:**
- Clean lifecycle: setup → run → teardown
- Integrations are testable in isolation with mock contexts
- No dynamic dispatch — monomorphized at the binary crate level
- Consistent with existing port trait patterns (RPITIT, no `async-trait`)

**Negative:**
- Adding a new integration type requires updating the binary crate's generic parameters
- The `IntegrationContext` must be kept minimal to avoid coupling integrations to internal details

---

## ADR-009: Use rumqttc for MQTT client

**Status:** Accepted

**Date:** 2026-02-09

### Context

minihub needs an MQTT client library to connect to brokers and bridge MQTT-based devices (e.g. Zigbee2MQTT, Tasmota, ESPHome) into the system. The client must support async/await with tokio, automatic reconnection, and fit within the project's pure-Rust, no-unsafe constraint.

### Decision

Use rumqttc (v0.25) as the MQTT client library. rumqttc is a pure-Rust, async-first MQTT client backed by tokio, with built-in automatic reconnection via its eventloop model.

### Alternatives Considered

- **paho-mqtt**: Mature and feature-complete, but wraps the Eclipse Paho C library via FFI. Introduces a C toolchain dependency and potential unsafe code, violating the project's `unsafe_code = "forbid"` lint.
- **mqttrs**: Low-level codec library — would require building the entire client layer (connection management, reconnection, keep-alive) from scratch.
- **ntex-mqtt**: Tied to the ntex runtime rather than tokio.

### Consequences

**Positive:**
- Pure Rust — no C dependencies, compatible with `unsafe_code = "forbid"`
- Native tokio integration — fits the existing async runtime
- Built-in automatic reconnection via the eventloop poll model
- TLS support via rustls (default feature)
- Active maintenance and widely used in the Rust IoT ecosystem

**Negative:**
- Eventloop model requires spawning a background task to drive `eventloop.poll()`
- MQTT v5 support is available but less battle-tested than v3.1.1
- Client and eventloop are separate objects that must be coordinated

---

## ADR-010: Use btleplug for passive BLE scanning

**Status:** Accepted

**Date:** 2026-02-09

### Context

minihub needs to passively scan for BLE advertisements from Xiaomi LYWSD03MMC temperature/humidity sensors running ATC/PVVX custom firmware. The firmware broadcasts sensor data (temperature, humidity, battery) as BLE service data advertisements — no connection is required. The BLE crate must support passive scanning with access to raw service data bytes, work with the tokio async runtime, and not require `unsafe` in adapter code.

### Decision

Use btleplug (v0.11) as the BLE scanning library. btleplug is a cross-platform Rust BLE library with async/tokio support that exposes advertisement service data via `PeripheralProperties::service_data`.

### Alternatives Considered

- **bluer**: Official BlueZ Rust bindings with excellent Linux/D-Bus integration and a dedicated `monitor()` API for passive advertisement scanning. However, it is **Linux-only**, which prevents development and testing on macOS.
- **bluest**: Cross-platform and wraps bluer on Linux, but has a smaller community and is less battle-tested than btleplug.

### Consequences

**Positive:**
- Cross-platform — enables development on macOS and deployment on Linux (Raspberry Pi)
- Tokio-native async scanning via `start_scan()` + `events()` stream
- `service_data: HashMap<Uuid, Vec<u8>>` provides direct access to raw advertisement payloads for parsing ATC/PVVX formats
- No `unsafe` required in adapter code (platform internals are encapsulated by the crate)
- Most widely used and maintained Rust BLE crate

**Negative:**
- Platform-native backends (CoreBluetooth on macOS, BlueZ/D-Bus on Linux) mean behaviour can differ slightly across platforms
- Higher-level abstraction may not expose every BlueZ-specific feature

---

## ADR-011: Leptos WASM dashboard

**Status:** Accepted

**Date:** 2026-02-21

**Supersedes:** [ADR-003](#adr-003-no-javascript-dashboard), [ADR-007](#adr-007-use-askama-for-html-templating)

### Context

minihub aims to replace Home Assistant for hobbyist use. The SSR dashboard (ADR-003) served early milestones well but has fundamental limitations for a home automation UI:

- No real-time device state updates (only meta-refresh polling)
- No interactive charts for sensor history (temperature over 24h, humidity trends)
- Full page reloads on every action (toggling a light reloads the entire page)
- No drag-and-drop automation builder possible
- No responsive, app-like feel for mobile use (checking sensors on phone)

The project's core principle remains **minimal hand-written JavaScript**. The dashboard should stay in the Rust ecosystem.

### Decision

Use Leptos in CSR (client-side rendering) mode. The dashboard is compiled to WASM and served as static assets by the existing axum server. The dashboard consumes the existing `/api/*` JSON endpoints. A small auto-generated JS bootstrap script (~10 lines) loads the `.wasm` binary — no hand-written JavaScript.

The `adapter_http_axum` crate becomes API-only (JSON endpoints + static file serving). A new `adapter_dashboard_leptos` crate contains all UI components, compiled to WASM via `trunk`.

### Alternatives Considered

- **Yew**: Older, larger community. React-like virtual DOM model. Larger WASM bundle sizes. Less ergonomic API. No built-in server function support.
- **Dioxus**: Similar to Leptos but less mature axum integration. Broader platform targets (desktop, mobile) that minihub doesn't need.
- **HTMX**: Adds progressive enhancement to SSR but still requires a JavaScript runtime and doesn't support charts or complex interactivity natively.
- **Keep SSR + add chart images**: Server-side chart rendering (e.g., plotters to PNG). Avoids WASM but produces static images, no interactivity, high server load on Raspberry Pi.

### Consequences

**Positive:**
- Fine-grained reactivity — when one sensor updates, only that widget re-renders (no vdom diffing)
- Real-time updates via SSE subscription from Leptos components
- Interactive charts for sensor history using `plotters` (pure Rust, compiles to WASM, renders to `<canvas>`)
- No hand-written JavaScript — entire dashboard is Rust compiled to WASM
- Smaller WASM bundles than Yew (important for Raspberry Pi on local network)
- Native axum integration via `leptos_axum` (same server, same binary)
- CSR mode means minimal server load — axum just serves static files + JSON API
- Same type system for API responses and UI rendering (shared domain types)

**Negative:**
- Adds `trunk` as a build tool for WASM compilation
- Requires `wasm32-unknown-unknown` target installed via rustup
- Initial page load requires downloading WASM bundle (mitigated by local network use)
- Leptos ecosystem is younger than Yew — fewer third-party component libraries
- Two compilation targets in one workspace (native for server, wasm32 for dashboard)

### Build & Deployment

- `trunk build` compiles `adapter_dashboard_leptos` to WASM + generates `index.html` with bootstrap JS
- Output goes to a `dist/` directory
- `minihubd` serves `dist/` at `/` as static files, API at `/api/*`
- Single binary deployment: embed WASM assets via `include_dir` or serve from filesystem

---

## ADR-012: Entity history for time-series sensor data

**Status:** Accepted

**Date:** 2026-02-21

### Context

A home automation dashboard needs to display sensor data over time — temperature graphs, humidity trends, switch on/off timelines, energy usage patterns. The current `events` table stores immutable event records but is not optimized for time-range queries on entity state/attribute values.

Integrations (MQTT, BLE, virtual) already push state changes through the event bus. The missing piece is persisting these changes in a format optimized for time-series queries that power dashboard charts.

### Decision

Add an `EntityHistory` domain type and `EntityHistoryRepository` port. Store entity state and attribute snapshots in a dedicated `entity_history` SQLite table, indexed on `(entity_id, recorded_at)` for efficient range queries. Wire history recording into the existing event bus worker — when a `StateChanged` or `AttributeChanged` event fires, a history record is appended.

Schema:

```sql
CREATE TABLE entity_history (
    id BLOB PRIMARY KEY NOT NULL,
    entity_id BLOB NOT NULL,
    state TEXT NOT NULL,
    attributes TEXT NOT NULL,  -- JSON
    recorded_at TEXT NOT NULL,  -- ISO 8601
    FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE CASCADE
);

CREATE INDEX idx_entity_history_range
    ON entity_history(entity_id, recorded_at DESC);
```

### Alternatives Considered

- **Reuse events table**: Already stores state changes, but the JSON `data` field is unstructured and not indexed for range queries. Adding indexes would bloat the general event table.
- **External time-series DB (InfluxDB, TimescaleDB)**: Better for high-volume metrics but adds an external dependency, violating the single-binary deployment model.
- **RRDtool / round-robin approach**: Fixed-size storage with automatic downsampling. Interesting but adds complexity and an external dependency.

### Consequences

**Positive:**
- Dedicated index on `(entity_id, recorded_at)` enables fast range queries for charts
- Separate from events table — history can have its own retention policy without affecting event log
- Simple schema — just snapshots of state + attributes at a point in time
- Powers dashboard charts: temperature over 24h, switch timeline, etc.
- Configurable retention (default: 30 days) with periodic purge

**Negative:**
- Additional storage overhead (one row per state change per entity)
- Requires a background purge task for retention enforcement
- SQLite single-writer constraint may bottleneck under very high-frequency sensor updates (mitigated: batch inserts, WAL mode)
