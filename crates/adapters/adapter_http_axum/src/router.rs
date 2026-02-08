//! Axum router assembly.
//!
//! TODO(M2): Build the top-level `axum::Router` that merges:
//!   - `/api/*` routes from the `api` module
//!   - `/*` routes from the `dashboard` module
//!   - Static asset serving (CSS) if needed
