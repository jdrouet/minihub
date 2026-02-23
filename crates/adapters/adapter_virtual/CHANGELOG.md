# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/jdrouet/minihub/releases/tag/minihub-adapter-virtual-v0.1.0) - 2026-02-23

### Added

- device deduplication via integration + unique_id fields
- *(virtual)* add adapter_virtual crate with simulated devices

### Fixed

- *(virtual)* remove unwrap from non-test code in virtual adapter

### Other

- *(virtual)* adapt to IntegrationContext-based Integration trait
