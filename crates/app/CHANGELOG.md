# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/jdrouet/minihub/releases/tag/minihub-app-v0.1.0) - 2026-02-23

### Added

- *(M8-T11)* Add InvalidTimestamp validation error and re-export EntityHistoryRepository
- *(M8-T9)* Add EntityHistoryRepository port trait
- *(app)* add ServiceContext and Arc<T> EventPublisher blanket impl
- *(app)* add IntegrationContext port and rework Integration trait
- device deduplication via integration + unique_id fields
- *(ble)* continuous background BLE scanning via mpsc channel
- *(app)* add upsert_entity to EntityService for background discovery
- *(app)* add find_by_entity_id to EntityRepository port + SQLite impl
- *(logging)* add structured logging with tracing
- *(virtual)* add adapter_virtual crate with simulated devices
- *(app)* add Integration trait and DiscoveredDevice port
- *(app)* implement AutomationEngine, AutomationService, and AutomationRepository port
- *(app)* add event publishing to EntityService
- *(app)* implement in-process event bus with tokio broadcast
- *(app)* define EventStore and EventPublisher port traits
- *(app)* implement DeviceService and AreaService (M1-T6)
- *(app)* implement EntityService with in-memory mock tests (M1-T5)
- *(app)* define storage port traits (M1-T4)
- add app crate with port traits and service placeholders

### Other

- address PR review — use async fn for find_by_entity_id
- add coverage tests for automation engine and config
- scaffold Cargo workspace with crate boundaries
