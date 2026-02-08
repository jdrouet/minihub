//! # minihubd — minihub daemon
//!
//! Composition root that wires all adapters together and starts the server.
//!
//! ## Responsibilities
//! - Parse configuration (CLI args, env vars, config file)
//! - Initialize the `SQLite` connection pool and run migrations
//! - Construct repository implementations (adapters)
//! - Construct application services, injecting repositories via port traits
//! - Build the axum router, injecting application services
//! - Bind to a TCP port and serve
//! - Handle graceful shutdown (SIGTERM/SIGINT)
//!
//! ## Dependency rule
//! This is the **only** crate that depends on all other crates.
//! It is the wiring layer — no domain logic belongs here.

fn main() {
    // TODO(M2): Replace with tokio::main and actual wiring.
    println!("minihubd: not yet implemented — see TASKS.md for the roadmap.");
}
