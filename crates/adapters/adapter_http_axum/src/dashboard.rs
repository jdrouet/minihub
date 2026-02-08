//! Server-side rendered HTML dashboard (no JavaScript).
//!
//! TODO(M2): Implement dashboard pages:
//!   - `GET  /`                   — overview / home page
//!   - `GET  /entities`           — entity list with state badges
//!   - `GET  /entities/:id`       — entity detail + control form
//!   - `POST /entities/:id/action`— handle form submission (PRG)
//!   - `GET  /devices`            — device list
//!   - `GET  /areas`              — area list with assigned entities
//!   - `GET  /automations`        — automation list
//!   - `GET  /events`             — event log viewer
//!
//! All pages include `<meta http-equiv="refresh" content="N">` for auto-reload.
//! Forms use POST + redirect (PRG pattern) to avoid double-submission.
