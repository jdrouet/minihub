# minihub

A tiny, Rust-only, Home-Assistant-inspired home automation server with a reactive WASM dashboard — **no hand-written JavaScript**.

## What it is

minihub is a minimal home automation hub that manages **entities** (lights, sensors, switches, …), organises them into **devices** and **areas**, exposes **services** to control them, logs **events**, records **sensor history**, and supports simple **automations** — all accessible through a reactive Leptos WASM dashboard and a JSON API.

## Non-goals

- **Not** a plugin marketplace. Integrations are compiled-in Rust crates, not dynamically loaded.
- **No hand-written JavaScript.** The dashboard is a Leptos CSR app compiled to WASM. The only JS is a tiny auto-generated bootstrap script. No npm, no `node_modules`, no hand-written `<script>` tags.
- **No cloud.** minihub runs locally. There is no cloud account, no telemetry, no phone-home.
- **No YAML soup.** Configuration uses a single, well-typed format (TOML or code-level config).

## Architecture overview

minihub follows a **hexagonal architecture** (ports & adapters) enforced by Cargo workspace crate boundaries:

```
┌─────────────────────────────────────────────────┐
│                   minihubd                      │  composition root
│                (binary crate)                   │  (wires everything)
├────────────┬────────────┬───────────────────────┤
│ adapter    │ adapter    │ adapter               │
│ http_axum  │ storage    │ mqtt / ble / virtual  │  driven + driving
│ (API+SSE+  │ sqlite_sqlx│                       │  adapters
│  static)   │            │                       │
├────────────┴────────────┴───────────────────────┤
│                   app                           │  use-cases + port
│           (services + port traits)              │  trait definitions
├─────────────────────────────────────────────────┤
│                  domain                         │  pure domain model
│     (IDs, errors, time, entities, devices,      │  (no IO)
│      areas, events, services, automations)      │
├ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┤
│          adapter_dashboard_leptos               │  WASM (built via
│        (Leptos CSR → wasm32, depends            │  trunk, served as
│         on domain only, talks HTTP/SSE)         │  static assets)
└─────────────────────────────────────────────────┘
```

**Dependency rules:**
- `domain` has no internal dependencies.
- `app` depends on `domain`.
- Adapters depend on `app` + `domain`.
- `minihubd` depends on everything.
- No adapter types ever appear in `domain` or `app`.

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for full details.

## Concept model

| Concept        | Description |
|----------------|-------------|
| **Entity**     | A single observable/controllable data point (e.g., `light.kitchen`, `sensor.outdoor_temp`). Has a state and attributes. |
| **Device**     | A physical or virtual thing that exposes one or more entities. |
| **Area**       | A logical grouping (room, floor, zone) for organising devices and entities. |
| **Service**    | A callable command (`light.turn_on`, `switch.toggle`). Targets one or more entities. |
| **Event**      | An immutable record of something that happened (state change, service call, automation trigger). |
| **Automation** | A trigger → condition → action rule that reacts to events or state changes. |
| **Integration**| A Rust crate that connects an external protocol or virtual device into minihub. |
| **Entity History** | Time-series record of entity state/attribute changes, powering dashboard charts. |

## Dashboard approach

The web dashboard is a **Leptos CSR app** compiled to WASM — no hand-written JavaScript.

1. **Leptos components** — reactive UI built in Rust, compiled to `wasm32-unknown-unknown` via `trunk`.
2. **Client-side routing** — Leptos router handles page navigation without full reloads.
3. **API-driven** — all data fetched from `/api/*` JSON endpoints via `gloo-net`.
4. **Real-time updates** — SSE subscription (`/api/events/stream`) pushes entity state changes to the UI. When a sensor updates, only that widget re-renders (fine-grained reactivity, no virtual DOM).
5. **Sensor history charts** — `plotters` (pure Rust, compiles to WASM) renders time-series data to `<canvas>`.
6. **Static asset serving** — `minihubd` serves the pre-built WASM bundle at `/`, API at `/api/*`.

The only JavaScript is a ~10-line auto-generated bootstrap script that loads the `.wasm` binary.

## Development workflow

### Prerequisites

- Rust stable (>= 1.75)
- [just](https://github.com/casey/just) command runner (recommended)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) for coverage

### Commands

```bash
# Format code
just fmt

# Run clippy (deny all warnings)
just clippy

# Run tests
just test

# Run coverage
just cov

# Generate HTML coverage report
just cov-html

# Run all checks (fmt + clippy + test)
just check
```

Or without `just`:

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo llvm-cov
cargo llvm-cov --html
```

## Roadmap

| Milestone | Goal | Key deliverables |
|-----------|------|-----------------|
| **M0** | Scaffold & plan | Workspace structure, docs, CI stub |
| **M1** | Domain & app core | Entity/Device/Area types, port traits, entity service, unit tests |
| **M2** | HTTP + storage | axum API, SSR dashboard, SQLite repos, integration tests |
| **M3** | Events & automations | Event log, automation engine, automation dashboard |
| **M4** | Virtual integration | Built-in demo integration with simulated devices |
| **M5** | Polish & harden | Error handling, logging, config, graceful shutdown, docs |
| **M6** | MQTT integration | MQTT adapter for real devices |
| **M7** | Passive BLE integration | BLE scanning for Xiaomi LYWSD03MMC sensors |
| **M8** | Leptos dashboard + entity history | Replace SSR with Leptos WASM CSR, add sensor history charts, SSE real-time updates |

See [TASKS.md](TASKS.md) for detailed task breakdown and definitions of done.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
