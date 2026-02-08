# Architecture Decision Records

This document contains Architecture Decision Records (ADRs) for the minihub project.

minihub is a tiny Rust-only home automation server.

## Table of Contents

- [ADR-001: Use axum as the HTTP framework](#adr-001-use-axum-as-the-http-framework)
- [ADR-002: Use sqlx with SQLite for persistence](#adr-002-use-sqlx-with-sqlite-for-persistence)
- [ADR-003: No-JavaScript dashboard](#adr-003-no-javascript-dashboard)
- [ADR-004: Hexagonal architecture with Cargo workspace](#adr-004-hexagonal-architecture-with-cargo-workspace)
- [ADR-005: Dual MIT/Apache-2.0 license](#adr-005-dual-mitapache-20-license)
- [ADR-006: cargo-llvm-cov for code coverage](#adr-006-cargo-llvm-cov-for-code-coverage)
- [ADR-007: Use askama for HTML templating](#adr-007-use-askama-for-html-templating)

---

## ADR-001: Use axum as the HTTP framework

**Status:** Accepted

**Date:** 2026-02-08

### Context

The minihub project needs an async HTTP framework to handle REST API endpoints and serve the server-side rendered dashboard. The framework should integrate well with the async ecosystem, provide good performance, and have strong typing support.

### Decision

Use axum as the HTTP framework. axum is tower-based and maintained by the tokio team, providing excellent integration with the async ecosystem.

### Alternatives Considered

- **actix-web**: Mature and performant, but uses actor model which adds complexity
- **warp**: Filter-based approach, but less intuitive API and smaller ecosystem
- **poem**: Similar to axum but less mature and smaller community
- **rocket**: More ergonomic API with less boilerplate, but historically had less mature async support

### Consequences

**Positive:**
- Access to the entire tower middleware ecosystem
- Excellent async support through tight tokio integration
- Strong typing with extractors and handlers
- Active maintenance and strong community support

**Negative:**
- Slightly more boilerplate compared to rocket
- Steeper learning curve for developers unfamiliar with tower

---

## ADR-002: Use sqlx with SQLite for persistence

**Status:** Accepted

**Date:** 2026-02-08

### Context

The minihub project needs lightweight embedded persistence for storing device states, automation rules, and historical data. The solution should not require an external database server and should be easy to deploy as a single binary with a local database file.

### Decision

Use sqlx with SQLite for persistence. sqlx provides compile-time query checking and async database access without requiring a full ORM layer.

### Alternatives Considered

- **diesel**: Popular ORM with excellent type safety, but primarily synchronous and heavier weight
- **rusqlite**: Synchronous SQLite bindings, would require blocking thread pool for async usage
- **sled**: Pure Rust embedded database, but less mature and different query model
- **redb**: Pure Rust embedded database, but very new and minimal ecosystem

### Consequences

**Positive:**
- Compile-time safety with query checking via `sqlx::query!` macros
- Async support that integrates naturally with tokio
- Good migration support with sqlx-cli
- Standard SQL rather than custom query DSL

**Negative:**
- Requires sqlx CLI for offline mode and compile-time verification
- SQLite has concurrency limitations (single writer)
- Need to manage prepared statements and connection pooling

---

## ADR-003: No-JavaScript dashboard

**Status:** Accepted

**Date:** 2026-02-08

### Context

The minihub dashboard needs to be simple, accessible, and work everywhere without requiring a complex frontend build pipeline. The Rust-only constraint means avoiding JavaScript-heavy solutions that would require maintaining a separate frontend codebase.

### Decision

Use server-side rendered HTML with standard HTML forms following the POST-Redirect-GET (PRG) pattern. Use meta refresh tags for periodic live updates of device states.

### Alternatives Considered

- **HTMX**: Adds progressive enhancement but still requires JavaScript runtime
- **Leptos/Yew WASM**: Rust-based frontend frameworks, but add significant build complexity
- **Traditional SPA (React/Vue)**: Would violate the Rust-only constraint and require separate build pipeline

### Consequences

**Positive:**
- Works in any browser, including text-mode browsers
- No frontend build pipeline or tooling required
- Simple to understand and maintain
- Excellent accessibility by default
- Fast initial page loads

**Negative:**
- No real-time updates without page reload or meta refresh
- Limited interactivity compared to JavaScript-based solutions
- Full page reloads on user actions
- Cannot use modern UI patterns like optimistic updates

---

## ADR-004: Hexagonal architecture with Cargo workspace

**Status:** Accepted

**Date:** 2026-02-08

### Context

The project needs strict separation of concerns to ensure testability, maintainability, and the ability to swap out infrastructure adapters (e.g., different device protocols, storage backends) without affecting the core domain logic.

### Decision

Implement hexagonal (ports and adapters) architecture enforced through separate Cargo crates within a workspace. Core domain logic is isolated from infrastructure concerns through trait-based ports.

### Alternatives Considered

- **Monolithic crate structure**: Simpler initially but harder to enforce boundaries
- **Module-based separation**: Runtime boundaries only, no compile-time enforcement
- **Microservices**: Too heavy for a lightweight embedded project

### Consequences

**Positive:**
- Compile-time enforcement of dependency rules (core cannot depend on adapters)
- Easy to test core domain logic in isolation with mock adapters
- Clear architectural boundaries visible in the project structure
- Easier to add new adapters without modifying core logic

**Negative:**
- More crates to manage in the workspace
- Some boilerplate for trait definitions and adapter implementations
- Slightly longer compile times due to inter-crate dependencies

---

## ADR-005: Dual MIT/Apache-2.0 license

**Status:** Accepted

**Date:** 2026-02-08

### Context

The project should be open source with a license that is standard in the Rust ecosystem and provides maximum compatibility with downstream users and contributors.

### Decision

Use dual licensing under MIT OR Apache-2.0, following the standard Rust ecosystem convention.

### Alternatives Considered

- **MIT only**: Simpler but lacks explicit patent grant
- **Apache-2.0 only**: Provides patent grant but more verbose and some consider less permissive
- **GPL**: Too restrictive for a library/framework-style project

### Consequences

**Positive:**
- Maximum compatibility with the Rust ecosystem
- Downstream users can choose the license that works best for them
- Follows established community conventions
- Apache-2.0 provides explicit patent grant
- MIT provides simple, permissive terms

**Negative:**
- Slightly more complex license management (two license files)
- Contributors must agree to dual license terms

---

## ADR-006: cargo-llvm-cov for code coverage

**Status:** Accepted

**Date:** 2026-02-08

### Context

The project needs accurate code coverage measurement for both local development and CI workflows. The solution should work well with Cargo workspaces and produce reports that can be used in CI pipelines.

### Decision

Use cargo-llvm-cov as the code coverage tool. It leverages LLVM's source-based coverage instrumentation for accurate measurements.

### Alternatives Considered

- **tarpaulin**: Popular Rust coverage tool but slower and less accurate on some platforms (especially macOS)
- **grcov**: Requires more manual setup and configuration

### Consequences

**Positive:**
- Accurate coverage measurements using LLVM instrumentation
- Good HTML report generation for local development
- Excellent support for Cargo workspaces
- CI-friendly output formats (JSON, LCOV)
- Works consistently across platforms

**Negative:**
- Requires installing llvm-tools-preview rustup component
- Slightly more complex setup than some alternatives
- LLVM-based tooling can have larger binary sizes during testing

---

## ADR-007: Use askama for HTML templating

**Status:** Accepted

**Date:** 2026-02-08

### Context

The no-JavaScript SSR dashboard needs an HTML templating approach. The templates must produce complete HTML pages server-side, integrate well with Rust's type system, and be easy to read and maintain.

### Decision

Use askama for compile-time-checked Jinja2-style HTML templates. Templates live as `.html` files alongside the Rust code, with type-safe variable binding verified at compile time.

### Alternatives Considered

- **maud**: Rust macro DSL for HTML. Keeps everything in Rust code but mixes markup with logic, and compatibility with the latest axum version can lag behind.
- **Manual string building**: No dependencies but error-prone, no compile-time guarantees, and poor ergonomics for anything beyond trivial markup.

### Consequences

**Positive:**
- Compile-time checked — template variables and types are verified during build
- Clean separation of HTML templates from Rust logic
- Familiar Jinja2/Django-style syntax accessible to non-Rust contributors
- Template inheritance for shared layout
- Straightforward axum integration via manual `IntoResponse` rendering to `Html`

**Negative:**
- Separate template files to manage alongside Rust code
- Jinja2 syntax has a learning curve for those unfamiliar with it
- Template errors surface as compile errors, which can be cryptic
