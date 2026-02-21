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
  - [M8 — Leptos Dashboard + Entity History ✅](#m8--leptos-dashboard--entity-history)
  - [M9 — Mi Flora BLE Integration](#m9--mi-flora-ble-integration)
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

## Milestones

*Other milestones are in `TASKS_COMPLETED.md`*

### M9 — Mi Flora BLE Integration

**Goal**: Add support for Xiaomi Mi Flora (HHCCJCY01) plant sensors to the existing BLE adapter. Mi Flora devices require active GATT connections (connect → write command → read data → disconnect), unlike the passive advertisement parsing used for PVVX/ATC1441 sensors. Maintain 80% coverage.

**Status**: Not started

**Prerequisites**: M7 complete (working BLE adapter with passive scanning)

**Deliverables**: Mi Flora GATT readout integrated into the BLE scan loop, new parser module for Mi Flora payloads, configuration for enabling/filtering Mi Flora devices.

**Why extend `adapter_ble` instead of a new crate**: Both use `btleplug` and share the single host BLE adapter. A separate crate would cause adapter contention. The existing scanner already discovers Mi Flora peripherals during scans — they just need to be connected to and read.

#### Mi Flora BLE Protocol Reference

The device advertises as `"Flower care"` with service UUID `0xFE95`.

| Characteristic | UUID | Mode | Purpose |
|----------------|------|------|---------|
| CMD | `00001a00-0000-1000-8000-00805f9b34fb` | Write | Activate sensor mode (write `[0xa0, 0x1f]`) |
| DATA | `00001a01-0000-1000-8000-00805f9b34fb` | Read | 16-byte sensor payload |
| FIRMWARE | `00001a02-0000-1000-8000-00805f9b34fb` | Read | 7-byte battery + firmware version |

**DATA payload (16 bytes, little-endian)**:

| Bytes | Type | Field | Example |
|-------|------|-------|---------|
| 0–1 | i16 LE (×0.1 °C) | Temperature | `0x00C9` → 20.1 °C |
| 2 | — | Padding | — |
| 3–6 | u32 LE (lux) | Light | `0x000141D2` → 82 386 lux |
| 7 | u8 (%) | Moisture | `0x38` → 56 % |
| 8–9 | u16 LE (µS/cm) | Conductivity | `0x0619` → 1 561 µS/cm |
| 10–15 | — | Reserved | — |

**FIRMWARE payload (7 bytes)**:

| Bytes | Type | Field | Example |
|-------|------|-------|---------|
| 0 | u8 (%) | Battery level | `0x63` → 99 % |
| 1 | — | Separator | — |
| 2–6 | ASCII | Firmware version | `"3.1.8"` |

#### Domain Model

Each Mi Flora device produces:

- **Device**: name=`"Mi Flora {MAC}"`, manufacturer=`"Xiaomi"`, model=`"HHCCJCY01"`, integration=`"ble"`, unique_id=`"{MAC}"`
- **Entity**: entity_id=`"sensor.miflora_{mac_slug}"`, state=`On`, attributes: `temperature` (Float, °C), `light` (Int, lux), `moisture` (Int, %), `conductivity` (Int, µS/cm), `battery_level` (Int, %), `firmware` (String)

#### Architecture

The existing scanner loop runs: passive scan → sleep → repeat. Mi Flora adds a GATT phase after each passive scan completes:

1. **Passive phase** (unchanged): collect advertisement-based readings (PVVX/ATC1441)
2. **GATT phase** (new): after `stop_scan()`, iterate discovered peripherals, identify Mi Flora devices by local name `"Flower care"`, connect via GATT, write CMD, read DATA + FIRMWARE, disconnect, persist via `IntegrationContext`

Each Mi Flora device connection is wrapped in a per-device timeout so one unresponsive device cannot block the entire scan cycle. A disconnect guard ensures the peripheral is always disconnected even on read errors.

#### Tasks

| Task ID | Description | Effort | Dependencies | DoD | Key Files |
|---------|-------------|--------|--------------|-----|-----------|
| M9-T1 | Mi Flora payload parsers ✅ | M | None | New `miflora.rs` module in `adapter_ble` with: `MifloraReading` struct (temperature, light, moisture, conductivity, battery_level, firmware_version fields). Pure parser functions `parse_sensor_data(&[u8]) -> Result<MifloraSensorData>` for the 16-byte DATA payload and `parse_firmware(&[u8]) -> Result<MifloraFirmware>` for the 7-byte FIRMWARE payload. GATT characteristic UUID constants (`CMD_CHAR`, `DATA_CHAR`, `FIRMWARE_CHAR`). `build_discovered(reading: &MifloraReading) -> Result<DiscoveredDevice>` to map a reading into domain Device + Entity. Register `mod miflora` in `lib.rs`. Unit tests: positive temperature, negative temperature, zero values, max lux, moisture boundary, conductivity, firmware version parsing, wrong payload lengths, device building with correct entity_id/attributes. ~12 tests. All existing tests still pass. `just check` passes. | `crates/adapters/adapter_ble/src/miflora.rs` (new), `crates/adapters/adapter_ble/src/lib.rs` |
| M9-T2 | GATT connection helper + error variants ✅ | M | M9-T1 | New `gatt.rs` module in `adapter_ble` with async GATT wrapper functions using `btleplug`: `read_miflora(peripheral) -> Result<MifloraReading>` that connects, discovers services, finds characteristics by UUID, writes `[0xa0, 0x1f]` to CMD, reads DATA + FIRMWARE, disconnects, returns parsed `MifloraReading`. Helper `find_characteristic(peripheral, uuid) -> Result<Characteristic>`. Disconnect guard pattern: always disconnect in a drop/finally path even on read errors. New error variants in `error.rs`: `BleError::GattConnect` (connection failed), `BleError::GattTimeout` (per-device timeout expired), `BleError::CharacteristicNotFound { uuid }` (GATT char missing). `PayloadParseError::MifloraWrongLength { expected, actual }` variant. Register `mod gatt` in `lib.rs`. Unit tests for error display strings and `MiniHubError` conversion. ~5 tests. `just check` passes. | `crates/adapters/adapter_ble/src/gatt.rs` (new), `crates/adapters/adapter_ble/src/error.rs`, `crates/adapters/adapter_ble/src/lib.rs` |
| M9-T3 | Mi Flora configuration | S | None | Extend `BleConfig` in `adapter_ble/src/config.rs` with: `miflora_enabled: bool` (default `false`), `miflora_filter: Vec<String>` (MAC allowlist, default empty = accept all), `miflora_connect_timeout_secs: u16` (default `10`). Extend `BleIntegrationConfig` in `crates/bin/minihubd/src/config.rs` with matching fields. Add env var override `MINIHUB_BLE_MIFLORA_ENABLED` in `apply_env_overrides()`. Add `ServiceUuid::MIFLORA` (`0xFE95`) to `parser.rs` and include it in `ServiceUuid::all()` so the scan filter picks up Mi Flora advertisements. Update `minihub.toml.example` with commented Mi Flora config block. Unit tests: config defaults (miflora disabled), TOML deserialization with miflora fields, partial TOML with defaults, env override for miflora_enabled. ~6 tests. `just check` passes. | `crates/adapters/adapter_ble/src/config.rs`, `crates/adapters/adapter_ble/src/parser.rs`, `crates/bin/minihubd/src/config.rs`, `minihub.toml.example` |
| M9-T4 | Integrate Mi Flora GATT readout into scanner | M | M9-T1, M9-T2, M9-T3 | Extend `BleScanner` struct with `miflora_enabled: bool`, `miflora_filter: Vec<String>`, `miflora_connect_timeout: Duration` fields. Modify `BleScanner::start()` to accept the new config and pass it through. In `iterate()`, after `central.stop_scan().await`, if `miflora_enabled`, call new method `read_miflora_devices(&central)`. This method: lists peripherals, checks each peripheral's local name for `"Flower care"`, applies `miflora_filter` (MAC allowlist), calls `gatt::read_miflora()` with a `tokio::time::timeout` wrapper, persists via `self.context.persist_discovered()`, logs warnings on individual device failures but continues to next device. Wire the new config fields from `BleIntegration::start_background()` through to `BleScanner::start()`. Unit tests: `passes_miflora_filter` logic (empty filter accepts all, non-empty filter matches case-insensitively). ~3 tests. `just check` passes. | `crates/adapters/adapter_ble/src/scanner.rs`, `crates/adapters/adapter_ble/src/lib.rs` |
| M9-T5 | Update docs + verify quality gates | S | M9-T4 | Update `adapter_ble` crate-level doc comment in `lib.rs` to add Mi Flora to the supported formats table (active GATT, UUID `0xFE95`). Update `Cargo.toml` description to mention Mi Flora. Verify: `just check` passes (fmt + clippy + tests). `just cov` >= 80%. All existing BLE tests still pass. | `crates/adapters/adapter_ble/src/lib.rs`, `crates/adapters/adapter_ble/Cargo.toml` |

#### Key Design Decisions

- **Disabled by default** (`miflora_enabled: false`) — zero behavior change for existing users
- **Sequential GATT after passive scan** — avoids interleaving passive events with active connections; the passive scan must be stopped before connecting to peripherals
- **Per-device timeout** (`miflora_connect_timeout_secs`) — one unresponsive Mi Flora cannot block the entire scan cycle
- **Separate MAC filter** (`miflora_filter`) — independent from `device_filter` since they target different device types
- **Disconnect guard** — always disconnect even on read errors, preventing leaked BLE connections

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
                          │    └─> M9 (Mi Flora BLE)
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
