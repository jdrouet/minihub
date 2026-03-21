# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2](https://github.com/jdrouet/minihub/compare/minihub-adapter-virtual-v0.1.1...minihub-adapter-virtual-v0.1.2) - 2026-03-21

### Fixed

- address review feedback on plant integration

### Other

- fix publishing

## [0.1.1](https://github.com/jdrouet/minihub/releases/tag/minihub-adapter-virtual-v0.1.1) - 2026-03-03

### Added

- device deduplication via integration + unique_id fields
- *(virtual)* add adapter_virtual crate with simulated devices

### Fixed

- *(virtual)* remove unwrap from non-test code in virtual adapter

### Other

- release v0.1.1
- *(app)* rename get_entity to find_entity_by_id on IntegrationContext
- *(ble)* replace in-memory EntityMacMap with database-backed mac_address
- implement subscribe() in adapter test mocks
- release v0.1.0
- *(virtual)* adapt to IntegrationContext-based Integration trait

## [0.1.0](https://github.com/jdrouet/minihub/releases/tag/minihub-adapter-virtual-v0.1.0) - 2026-02-23

### Added

- device deduplication via integration + unique_id fields
- *(virtual)* add adapter_virtual crate with simulated devices

### Fixed

- *(virtual)* remove unwrap from non-test code in virtual adapter

### Other

- *(virtual)* adapt to IntegrationContext-based Integration trait
