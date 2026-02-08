# minihub

A tiny, Rust-only, Home-Assistant-inspired home automation server with first-class dashboards that require **no JavaScript**.

## What it is

minihub is a minimal home automation hub that manages **entities** (lights, sensors, switches, …), organises them into **devices** and **areas**, exposes **services** to control them, logs **events**, and supports simple **automations** — all accessible through a server-rendered HTML dashboard and a JSON API.

## Non-goals

- **Not** a Home Assistant replacement. minihub targets hobbyists who want a small, understandable system.
- **Not** a plugin marketplace. Integrations are compiled-in Rust crates, not dynamically loaded.
- **No JavaScript.** The dashboard is fully server-rendered HTML. No JS bundles, no WASM-requiring-JS, no npm.
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
│ http_axum  │ storage    │ mqtt (future)         │  driven + driving
│            │ sqlite_sqlx│                       │  adapters
├────────────┴────────────┴───────────────────────┤
│                   app                           │  use-cases + port
│           (services + port traits)              │  trait definitions
├─────────────────────────────────────────────────┤
│                  domain                         │  pure domain model
│     (IDs, errors, time, entities, devices,      │  (no IO)
│      areas, events, services, automations)      │
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

## No-JS dashboard approach

The web dashboard is rendered entirely server-side. The techniques used:

1. **Full HTML pages** — every route returns a complete HTML document. No client-side routing.
2. **HTML forms** — interactive controls (toggle switches, sliders, buttons) are `<form>` elements that POST to the server. The server processes the action and redirects back (PRG pattern).
3. **Auto-refresh** — pages that display live state include `<meta http-equiv="refresh" content="5">` to reload automatically.
4. **CSS-only styling** — progressive visual enhancement using pure CSS (no JS-based component libraries).

This approach works in any browser, including text-mode browsers, curl, and accessibility tools.

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
| **M6** | MQTT integration (stretch) | MQTT adapter for real devices |

See [TASKS.md](TASKS.md) for detailed task breakdown and definitions of done.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
