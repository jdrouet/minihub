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

mod config;

use minihub_adapter_http_axum::state::AppState;
use minihub_adapter_storage_sqlite_sqlx::{
    Config as DbConfig, SqliteAreaRepository, SqliteAutomationRepository, SqliteDeviceRepository,
    SqliteEntityRepository, SqliteEventStore,
};
use minihub_adapter_virtual::VirtualIntegration;
use minihub_app::event_bus::InProcessEventBus;
use minihub_app::ports::Integration;
use minihub_app::services::area_service::AreaService;
use minihub_app::services::automation_service::AutomationService;
use minihub_app::services::device_service::DeviceService;
use minihub_app::services::entity_service::EntityService;
use tracing_subscriber::EnvFilter;

use crate::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuration
    let config = Config::load()?;

    // Logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&config.logging.filter))
        .init();

    tracing::info!("configuration loaded");

    // Database
    let db_config = DbConfig {
        database_url: config.database_url().to_string(),
    };
    let db = db_config.build().await?;
    let pool = db.pool().clone();
    tracing::info!("database ready");

    // Repositories
    let entity_repo = SqliteEntityRepository::new(pool.clone());
    let device_repo = SqliteDeviceRepository::new(pool.clone());
    let area_repo = SqliteAreaRepository::new(pool.clone());
    let event_store = SqliteEventStore::new(pool.clone());
    let automation_repo = SqliteAutomationRepository::new(pool);

    // Event bus
    let event_bus = InProcessEventBus::new(256);

    // Services
    let entity_service = EntityService::new(entity_repo, event_bus);
    let device_service = DeviceService::new(device_repo);
    let area_service = AreaService::new(area_repo);
    let automation_service = AutomationService::new(automation_repo);

    // Virtual integration — discover and register simulated devices
    if config.integrations.virtual_enabled {
        let mut virtual_integration = VirtualIntegration::default();
        let discovered = virtual_integration.setup().await?;
        for dd in discovered {
            let _ = device_service.create_device(dd.device).await;
            for entity in dd.entities {
                let _ = entity_service.create_entity(entity).await;
            }
        }
        tracing::info!(
            integration = virtual_integration.name(),
            "virtual integration ready"
        );
    }

    // HTTP
    let state = AppState::new(
        entity_service,
        device_service,
        area_service,
        event_store,
        automation_service,
    );
    let app = minihub_adapter_http_axum::router::build(state);

    let bind_addr = config.bind_addr();
    tracing::info!(addr = %bind_addr, "minihubd listening");

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("shutdown complete");
    Ok(())
}

/// Wait for a shutdown signal (Ctrl-C or SIGTERM).
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => tracing::info!("received Ctrl+C, shutting down"),
        () = terminate => tracing::info!("received SIGTERM, shutting down"),
    }
}
