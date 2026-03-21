# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3](https://github.com/jdrouet/minihub/compare/minihub-adapter-ble-v0.1.2...minihub-adapter-ble-v0.1.3) - 2026-03-21

### Fixed

- *(ble)* correct MiBeacon MAC offset from 7 to 5
- address review feedback on plant integration

### Other

- *(ble)* update MiBeacon doc comments and add real-world test fixtures
- fix publishing

## [0.1.2](https://github.com/jdrouet/minihub/compare/minihub-adapter-ble-v0.1.1...minihub-adapter-ble-v0.1.2) - 2026-03-03

### Fixed

- clippy issues

### Other

- format code

## [0.1.1](https://github.com/jdrouet/minihub/releases/tag/minihub-adapter-ble-v0.1.1) - 2026-03-03

### Added

- *(ble)* subscribe to event bus and handle service calls
- *(ble)* add blink_miflora GATT command
- *(ble)* add parse_mibeacon_mac to extract MAC from 0xFE95 service data
- *(M9-T5)* update adapter_ble docs with Mi Flora support
- *(M9-T4)* integrate Mi Flora GATT readout into BLE scanner
- *(M9-T3)* add Mi Flora config fields and service UUID
- *(M9-T2)* add GATT connection helper and error variants
- *(M9-T1)* add Mi Flora payload parsers and domain mapping
- *(ble)* emit DeviceDetected events for all BLE devices
- device deduplication via integration + unique_id fields
- *(ble)* continuous background BLE scanning via mpsc channel
- *(ble)* implement BleIntegration with Integration trait (M7-T4)
- *(ble)* implement PVVX and ATC1441 advertisement parsers (M7-T3)
- *(ble)* add BleConfig and BleError with unit tests (M7-T2)

### Fixed

- *(ble)* extract Mi Flora MAC from MiBeacon service data in scanner

### Other

- release v0.1.1
- format code
- *(ble)* introduce BleDeviceHandler trait and device modules
- *(error)* use anyhow for Storage variant and transparent error display
- *(app)* rename get_entity to find_entity_by_id on IntegrationContext
- *(ble)* replace in-memory EntityMacMap with database-backed mac_address
- implement subscribe() in adapter test mocks
- release v0.1.0
- *(ble)* accept MAC parameter in gatt::read_miflora
- filter devices
- *(ble)* simplify BleScanner into self-contained owned struct
- *(ble)* rework for IntegrationContext + real-time BleScanner
- *(ble)* use typed errors with #[from] instead of String variants
- add ADR-010 for btleplug BLE client choice (M7-T1)

## [0.1.0](https://github.com/jdrouet/minihub/releases/tag/minihub-adapter-ble-v0.1.0) - 2026-02-23

### Added

- *(ble)* add parse_mibeacon_mac to extract MAC from 0xFE95 service data
- *(M9-T5)* update adapter_ble docs with Mi Flora support
- *(M9-T4)* integrate Mi Flora GATT readout into BLE scanner
- *(M9-T3)* add Mi Flora config fields and service UUID
- *(M9-T2)* add GATT connection helper and error variants
- *(M9-T1)* add Mi Flora payload parsers and domain mapping
- *(ble)* emit DeviceDetected events for all BLE devices
- device deduplication via integration + unique_id fields
- *(ble)* continuous background BLE scanning via mpsc channel
- *(ble)* implement BleIntegration with Integration trait (M7-T4)
- *(ble)* implement PVVX and ATC1441 advertisement parsers (M7-T3)
- *(ble)* add BleConfig and BleError with unit tests (M7-T2)

### Fixed

- *(ble)* extract Mi Flora MAC from MiBeacon service data in scanner

### Other

- *(ble)* accept MAC parameter in gatt::read_miflora
- filter devices
- *(ble)* simplify BleScanner into self-contained owned struct
- *(ble)* rework for IntegrationContext + real-time BleScanner
- *(ble)* use typed errors with #[from] instead of String variants
- add ADR-010 for btleplug BLE client choice (M7-T1)
