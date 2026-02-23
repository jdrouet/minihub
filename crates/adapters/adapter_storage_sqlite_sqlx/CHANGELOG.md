# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/jdrouet/minihub/releases/tag/minihub-adapter-storage-sqlite-sqlx-v0.1.0) - 2026-02-23

### Added

- *(M8-T9)* Add SQLite migration for entity_history table
- device deduplication via integration + unique_id fields
- *(app)* add find_by_entity_id to EntityRepository port + SQLite impl
- *(http,storage)* add event log + automation dashboard pages
- *(storage)* implement SqliteEventStore with events migration
- *(storage)* implement SqliteDeviceRepository and SqliteAreaRepository
- *(storage)* implement SqliteEntityRepository
- *(storage)* add SQLite connection pool, migrations, and StorageError
- add adapter crate placeholders

### Other

- *(storage)* migrate UUID columns from TEXT to BLOB (16-byte)
- *(sqlite)* use async fn directly and as_uuid() in repo impls
- address PR review — use async fn for find_by_entity_id
- scaffold Cargo workspace with crate boundaries
