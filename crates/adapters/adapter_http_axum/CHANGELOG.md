# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/jdrouet/minihub/releases/tag/minihub-adapter-http-axum-v0.1.0) - 2026-02-23

### Added

- *(M8-T12)* Add GET /api/events/stream SSE endpoint
- *(M8-T11)* Add GET /api/entities/{id}/history endpoint
- add static file serving for WASM dashboard assets
- device deduplication via integration + unique_id fields
- *(ble)* continuous background BLE scanning via mpsc channel
- *(app)* add find_by_entity_id to EntityRepository port + SQLite impl
- *(dashboard)* responsive CSS with professional styling
- *(logging)* add structured logging with tracing
- *(http)* add event + automation REST API endpoints
- *(http,storage)* add event log + automation dashboard pages
- *(app)* add event publishing to EntityService
- *(http)* implement SSR dashboard pages with askama templates
- *(http)* implement JSON REST API handlers for entities, devices, areas
- *(http)* add axum router skeleton and generic AppState
- add adapter crate placeholders

### Fixed

- resolve clippy warnings in SSE test (clone_on_copy, type_complexity)

### Other

- *(M8-T11)* Add EHR generic param to AppState and all handlers
- add comprehensive tests for static file serving and dashboard_dir config
- remove SSR dashboard module, templates, and askama dependency
- split log entry over two table lines
- *(http)* propagate errors in all dashboard handlers
- *(http)* propagate errors in events dashboard handler
- ADR-007 — choose askama for HTML templating
- scaffold Cargo workspace with crate boundaries
