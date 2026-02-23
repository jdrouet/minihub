# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/jdrouet/minihub/releases/tag/minihubd-v0.1.0) - 2026-02-23

### Added

- *(M9-T3)* extend binary config with Mi Flora fields
- *(M8-T12)* Add GET /api/events/stream SSE endpoint
- *(M8-T10)* Wire entity history recording to event bus
- add static file serving for WASM dashboard assets
- device deduplication via integration + unique_id fields
- *(ble)* continuous background BLE scanning via mpsc channel
- *(minihubd)* wire BLE integration into config and main.rs (M7-T5)
- *(minihubd)* wire MQTT integration into config and main.rs
- *(shutdown)* add graceful shutdown on Ctrl+C and SIGTERM
- *(config)* add TOML configuration with env var overrides
- *(logging)* add structured logging with tracing
- *(minihubd)* wire VirtualIntegration into composition root
- *(http,storage)* add event log + automation dashboard pages
- *(app)* add event publishing to EntityService
- *(bin)* wire minihubd composition root
- add minihubd composition root placeholder

### Fixed

- *(minihubd)* survive broadcast channel lag in event persistence worker
- wire event bus subscriber to persist events to store

### Other

- *(M8-T11)* Add EHR generic param to AppState and all handlers
- add comprehensive tests for static file serving and dashboard_dir config
- remove SSR dashboard integration tests
- *(minihubd)* wire ServiceContext, simplify integration setup
- *(e2e)* wire event-bus subscriber in integration tests
- add coverage tests for automation engine and config
- *(e2e)* add virtual integration E2E tests
- *(e2e)* add automation CRUD and event listing E2E tests
- *(e2e)* add end-to-end smoke tests for minihubd
- scaffold Cargo workspace with crate boundaries
