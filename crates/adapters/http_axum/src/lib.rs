//! # minihub-adapter-http-axum
//!
//! HTTP adapter built on [axum](https://docs.rs/axum).
//!
//! ## Responsibilities
//! - Serve a **REST-ish JSON API** for programmatic access
//!   (`/api/entities`, `/api/devices`, `/api/areas`, …)
//! - Serve **static assets** (the Leptos WASM dashboard) at `/`
//! - Map HTTP requests into application service calls (driving adapter)
//! - Map application results into HTTP responses (JSON)
//!
//! ## Dependency rule
//! Depends on `minihub-app` (for port traits and services) and `minihub-domain`
//! (for domain types used in request/response mapping). Never leaks axum types
//! into the domain.

pub mod api;
mod error;
pub mod router;
pub mod state;
