# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/jdrouet/minihub/releases/tag/minihub-domain-v0.1.0) - 2026-02-23

### Added

- *(M8-T11)* Add InvalidTimestamp validation error and re-export EntityHistoryRepository
- *(M8-T9)* Add SQLite migration for entity_history table
- *(M8-T9)* Add EntityHistory domain model with builder pattern
- *(domain)* add DeviceDetected event type
- device deduplication via integration + unique_id fields
- *(domain)* implement Automation domain types
- *(domain)* implement Event and EventType domain types
- *(storage)* add SQLite connection pool, migrations, and StorageError
- *(domain)* implement Device and Area with builders (M1-T3)
- *(domain)* implement Entity, EntityState, AttributeValue (M1-T2)
- *(domain)* implement shared types — IDs, errors, time (M1-T1)
- add domain crate placeholder modules

### Other

- *(storage)* migrate UUID columns from TEXT to BLOB (16-byte)
- *(domain)* extract EventType::as_str const method
- prefer typed errors with #[from], Default over new()
- scaffold Cargo workspace with crate boundaries
