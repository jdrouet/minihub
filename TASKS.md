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
  - [M8 — Leptos Dashboard + Entity History](#m8--leptos-dashboard--entity-history)
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
│       ├── adapter_http_axum/              # HTTP JSON API + SSE + static file serving
│       ├── adapter_dashboard_leptos/       # Leptos CSR dashboard (WASM, built via trunk)
│       ├── adapter_storage_sqlite_sqlx/    # SQLite persistence
│       ├── adapter_mqtt/                   # MQTT client
│       └── adapter_ble/                    # Passive BLE scanner
├── crates/bin/
│   └── minihubd/            # Composition root & binary
└── Cargo.toml               # Workspace manifest
```

**Note:** `adapter_dashboard_leptos` is compiled separately to `wasm32-unknown-unknown` via `trunk` and is NOT a workspace member. It depends on `minihub-domain` only for shared API response types.

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
| M8 | 80% | Maintain coverage; Leptos WASM crate tested separately via browser/wasm-pack |

---

## Milestones

*Other milestones are in `TASKS_COMPLETED.md`*

### M8 — Leptos Dashboard + Entity History

**Goal**: Replace the SSR askama dashboard with a Leptos CSR (client-side rendered) WASM app. Add entity history for time-series sensor data. Add SSE for real-time updates. Maintain 80% coverage.

**Status**: Not Started

**Prerequisites**: M5 complete (working API, storage, integrations)

**Deliverables**: Leptos WASM dashboard served as static assets, entity history with time-range queries, SSE real-time push, sensor history charts.

**Key ADRs**: [ADR-011](docs/DECISIONS.md#adr-011-leptos-wasm-dashboard) (Leptos), [ADR-012](docs/DECISIONS.md#adr-012-entity-history-for-time-series-sensor-data) (Entity History)

#### Tasks

| Task ID | Description | Effort | Dependencies | DoD | Key Files |
|---------|-------------|--------|--------------|-----|-----------|
| M8-T1 | Strip SSR dashboard from adapter_http_axum | M | None | Remove `dashboard/` module (all handlers + templates). Remove askama dependency. Remove all `.html` template files. API routes (`/api/*`) and health check unchanged. Static file serving added at `/` for future WASM assets. `cargo check --all` passes. All API tests still pass. | `crates/adapters/adapter_http_axum/src/dashboard/` (delete), `crates/adapters/adapter_http_axum/src/router.rs`, `crates/adapters/adapter_http_axum/Cargo.toml` |
| M8-T2 | Scaffold adapter_dashboard_leptos crate | M | M8-T1 | New crate outside workspace (WASM target). `Cargo.toml` with leptos, leptos_router, gloo-net, minihub-domain deps. `index.html` for trunk. `trunk build` produces working WASM bundle. Basic "Hello minihub" page renders in browser. | `crates/adapters/adapter_dashboard_leptos/Cargo.toml`, `crates/adapters/adapter_dashboard_leptos/src/main.rs`, `crates/adapters/adapter_dashboard_leptos/index.html`, `crates/adapters/adapter_dashboard_leptos/Trunk.toml` |
| M8-T3 | Leptos app shell + routing | S | M8-T2 | Leptos router with routes: `/` (home), `/devices`, `/entities`, `/entities/{id}`, `/areas`, `/events`, `/automations`, `/automations/{id}`. Nav component with links. 404 fallback page. Client-side navigation works without full reloads. | `crates/adapters/adapter_dashboard_leptos/src/app.rs`, `crates/adapters/adapter_dashboard_leptos/src/components/nav.rs` |
| M8-T4 | Home page + API client | M | M8-T3 | API client module wrapping `gloo-net` calls to `/api/*`. Home page fetches entity count, device count, area count and displays stat cards. Loading and error states handled. | `crates/adapters/adapter_dashboard_leptos/src/api.rs`, `crates/adapters/adapter_dashboard_leptos/src/pages/home.rs`, `crates/adapters/adapter_dashboard_leptos/src/components/stat_card.rs` |
| M8-T5 | Device + entity list pages | M | M8-T4 | Device list page fetches and displays all devices in a table. Entity list page fetches and displays all entities with state badges (on/off/unavailable). Links to detail pages. | `crates/adapters/adapter_dashboard_leptos/src/pages/devices.rs`, `crates/adapters/adapter_dashboard_leptos/src/pages/entities.rs` |
| M8-T6 | Entity detail page + state control | M | M8-T5 | Entity detail page with state, attributes, timestamps. Turn on/off buttons call `PUT /api/entities/{id}/state`. UI reactively updates on successful response. | `crates/adapters/adapter_dashboard_leptos/src/pages/entity_detail.rs` |
| M8-T7 | Area, event, automation pages | M | M8-T5 | Area list page. Event log page (recent events, paginated). Automation list page with enable/disable toggle. Automation detail page with trigger/conditions/actions display. | `crates/adapters/adapter_dashboard_leptos/src/pages/areas.rs`, `crates/adapters/adapter_dashboard_leptos/src/pages/events.rs`, `crates/adapters/adapter_dashboard_leptos/src/pages/automations.rs` |
| M8-T8 | Wire WASM assets into minihubd | S | M8-T7 | `minihubd` serves `dist/` directory (trunk output) as static files at `/`. Fallback to `index.html` for client-side routing. API at `/api/*` takes precedence. `just build-dashboard` recipe added to Justfile. Update README with build instructions. | `crates/bin/minihubd/src/main.rs`, `crates/adapters/adapter_http_axum/src/router.rs`, `Justfile` |
| M8-T9 ✅ | EntityHistory domain + port + storage | M | None | `EntityHistory` struct in domain (id, entity_id, state, attributes, recorded_at). `EntityHistoryRepository` port trait with `record`, `find_by_entity_in_range`, `purge_before`. SQLite implementation with migration. Index on `(entity_id, recorded_at)`. Unit + integration tests. | `crates/domain/src/entity_history.rs`, `crates/app/src/ports/entity_history_repo.rs`, `crates/adapters/adapter_storage_sqlite_sqlx/src/entity_history_repo.rs`, `crates/adapters/adapter_storage_sqlite_sqlx/migrations/` |
| M8-T10 ✅ | Wire history recording to event bus | S | M8-T9 | Event worker (in minihubd main) listens for `StateChanged` / `AttributeChanged` events and appends `EntityHistory` records. Configurable retention (default: 30 days). Background purge task runs periodically. | `crates/bin/minihubd/src/main.rs` (modify event worker) |
| M8-T11 ✅ | History API endpoint | S | M8-T10 | `GET /api/entities/{id}/history?from=&to=&limit=` returns JSON array of history records. Defaults: last 24 hours, limit 1000. | `crates/adapters/adapter_http_axum/src/api/entity_history.rs` |
| M8-T12 ✅ | SSE endpoint for real-time updates | M | None | `GET /api/events/stream` returns SSE stream. Subscribes to event bus broadcast channel. Sends JSON-encoded events as SSE `data:` frames. Handles client disconnect gracefully. | `crates/adapters/adapter_http_axum/src/api/sse.rs` |
| M8-T13 ✅ | Leptos SSE subscription + live updates | M | M8-T12, M8-T6 | Leptos components subscribe to SSE via `EventSource` (web-sys). Entity state updates propagate reactively to widgets. Event log page shows new events without refresh. | `crates/adapters/adapter_dashboard_leptos/src/sse.rs`, modify page components |
| M8-T14 ✅ | Sensor history chart | M | M8-T11, M8-T6 | Chart component using `plotters` (WASM canvas backend). Entity detail page shows state history chart for sensor entities. Time range selector (1h, 6h, 24h, 7d). Temperature/humidity line charts. | `crates/adapters/adapter_dashboard_leptos/src/components/chart.rs` |
| M8-T15 | Styling + polish | M | M8-T7 | Responsive CSS (mobile-friendly). Dark nav bar, card layout, table styles, badge styles for states. Loading spinners. Error toast messages. Dark/light theme toggle (CSS-only, stored in localStorage via web-sys). | `crates/adapters/adapter_dashboard_leptos/src/styles.rs` or CSS file |
| M8-T16 | Update docs + coverage | S | M8-T15 | Update ARCHITECTURE.md, CONTRIBUTING.md, README.md (already done in planning). Verify all quality gates pass. `cargo test --all` passes. `cargo llvm-cov` >= 80%. Leptos crate tested via `wasm-pack test` or manual browser verification. | Docs, test files |

#### Phased Approach

**Phase A (M8-T1 → M8-T8):** Strip SSR, scaffold Leptos, rebuild all pages, wire into minihubd. At this point the dashboard is functionally equivalent to the old SSR version but reactive.

**Phase B (M8-T9 → M8-T11):** Entity history domain + storage + API. Backend work, no UI changes yet.

**Phase C (M8-T12 → M8-T14):** Real-time updates + sensor charts. The dashboard becomes significantly better than the SSR version.

**Phase D (M8-T15 → M8-T16):** Polish and docs.

---

## Task Dependencies Graph

```
M0 (Scaffold)
 └─> M1 (Domain & App)
      └─> M2 (HTTP + Storage + Dashboard)
           └─> M3 (Events & Automations)
                └─> M4 (Virtual Integration)
                     └─> M5 (Polish)
                          ├─> M6 (MQTT)
                          ├─> M7 (BLE)
                          └─> M8 (Leptos Dashboard + Entity History)
```

Within each milestone, tasks have dependencies noted in the DoD column.

---

## Glossary

- **DoD**: Definition of Done
- **ADR**: Architecture Decision Record
- **CRUD**: Create, Read, Update, Delete
- **PRG**: POST-REDIRECT-GET pattern
- **SSR**: Server-Side Rendering
- **CSR**: Client-Side Rendering
- **WASM**: WebAssembly
- **SSE**: Server-Sent Events
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
