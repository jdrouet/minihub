# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/jdrouet/minihub/releases/tag/minihub-adapter-mqtt-v0.1.0) - 2026-02-23

### Added

- device deduplication via integration + unique_id fields
- *(mqtt)* implement MQTT adapter with Integration trait (M6-T2)
- add adapter crate placeholders

### Other

- *(mqtt)* rework for IntegrationContext + background discovery
- *(mqtt)* add additional unit tests for edge cases (M6-T3)
- add ADR-009 for rumqttc MQTT client choice (M6-T1)
- scaffold Cargo workspace with crate boundaries
