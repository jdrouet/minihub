//! # minihub-adapter-http-axum
//!
//! HTTP adapter built on [axum](https://docs.rs/axum).
//!
//! ## Responsibilities
//! - Serve a **REST-ish JSON API** for programmatic access
//!   (`/api/entities`, `/api/devices`, `/api/services/call`, …)
//! - Serve a **server-side-rendered HTML dashboard** that works with
//!   **zero JavaScript** — pure HTML forms + `<meta http-equiv="refresh">`
//!   for live updates
//! - Map HTTP requests into application service calls (driving adapter)
//! - Map application results into HTTP responses (JSON or HTML)
//!
//! ## No-JS dashboard approach
//! - Every page is rendered server-side as complete HTML.
//! - Interactive controls (toggle, slider) are `<form>` elements that POST
//!   back to the server and redirect (PRG pattern).
//! - Live-updating pages use `<meta http-equiv="refresh" content="5">` to
//!   auto-reload at a configurable interval.
//! - CSS-only progressive enhancement for visual polish.
//!
//! ## Dependency rule
//! Depends on `minihub-app` (for port traits and services) and `minihub-domain`
//! (for domain types used in request/response mapping). Never leaks axum types
//! into the domain.

// TODO(M2): Implement `api` module — JSON REST handlers.
// TODO(M2): Implement `dashboard` module — SSR HTML handlers.
// TODO(M2): Implement `router` module — axum Router assembly.
// TODO(M2): Implement `templates` module — HTML template rendering (e.g., using `maud` or manual string building).
// TODO(M2): Implement `state` module — shared application state for axum.

pub mod api;
pub mod dashboard;
pub mod router;
