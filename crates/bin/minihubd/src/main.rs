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

use std::sync::Arc;

use minihub_adapter_ble::{BleConfig, BleIntegration};
use minihub_adapter_http_axum::state::AppState;
use minihub_adapter_mqtt::{MqttConfig, MqttIntegration};
use minihub_adapter_storage_sqlite_sqlx::{
    Config as DbConfig, SqliteAreaRepository, SqliteAutomationRepository, SqliteDeviceRepository,
    SqliteEntityHistoryRepository, SqliteEntityRepository, SqliteEventStore,
};
use minihub_adapter_virtual::VirtualIntegration;
use minihub_app::event_bus::InProcessEventBus;
use minihub_app::ports::storage::EntityHistoryRepository;
use minihub_app::ports::{EventStore, Integration};
use minihub_app::services::area_service::AreaService;
use minihub_app::services::automation_service::AutomationService;
use minihub_app::services::device_service::DeviceService;
use minihub_app::services::entity_service::EntityService;
use minihub_app::services::integration_context::ServiceContext;
use tracing_subscriber::EnvFilter;

use crate::config::Config;

#[tokio::main]
#[allow(clippy::too_many_lines)]
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
    let automation_repo = SqliteAutomationRepository::new(pool.clone());
    let history_repo = Arc::new(SqliteEntityHistoryRepository::new(pool));

    // Event bus (Arc-wrapped so it can be shared with ServiceContext)
    let event_bus = Arc::new(InProcessEventBus::new(256));
    let mut event_rx = event_bus.subscribe();

    // Services (Arc-wrapped early so they can be shared with background tasks)
    let entity_service = Arc::new(EntityService::new(entity_repo, Arc::clone(&event_bus)));
    let device_service = Arc::new(DeviceService::new(device_repo));
    let area_service = Arc::new(AreaService::new(area_repo));
    let automation_service = Arc::new(AutomationService::new(automation_repo));
    let event_store = Arc::new(event_store);

    // Event worker — persists events from the bus to the store and records entity history
    let es = Arc::clone(&event_store);
    let hr = Arc::clone(&history_repo);
    let entity_svc_for_history = Arc::clone(&entity_service);
    tokio::spawn(async move {
        loop {
            match event_rx.recv().await {
                Ok(event) => {
                    // Persist event to store
                    if let Err(err) = es.store(event.clone()).await {
                        tracing::warn!(%err, "failed to persist event");
                    }

                    // Record entity history for state/attribute changes
                    if matches!(
                        event.event_type,
                        minihub_domain::event::EventType::StateChanged
                            | minihub_domain::event::EventType::AttributeChanged
                    ) {
                        if let Some(entity_id) = event.entity_id {
                            match entity_svc_for_history.get_entity(entity_id).await {
                                Ok(entity) => {
                                    let history =
                                        minihub_domain::entity_history::EntityHistory::builder()
                                            .entity_id(entity.id)
                                            .state(entity.state.clone())
                                            .attributes(entity.attributes.clone())
                                            .recorded_at(event.timestamp)
                                            .build();

                                    if let Err(err) = hr.record(history).await {
                                        tracing::warn!(%err, entity_id = %entity_id, "failed to record entity history");
                                    }
                                }
                                Err(err) => {
                                    tracing::warn!(%err, entity_id = %entity_id, "failed to fetch entity for history recording");
                                }
                            }
                        }
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(
                        skipped = n,
                        "event store subscriber lagged, some events were dropped"
                    );
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
        tracing::debug!("event store subscriber stopped");
    });

    // Integration context — shared by all integrations
    let ctx = ServiceContext::new(
        Arc::clone(&device_service),
        Arc::clone(&entity_service),
        Arc::clone(&event_bus),
    );

    // Integrations
    if config.integrations.virtual_enabled {
        let mut integration = VirtualIntegration::default();
        integration.setup(&ctx).await?;
        tracing::info!(
            integration = integration.name(),
            "virtual integration ready"
        );
    }

    if config.integrations.mqtt.enabled {
        let mqtt_config = MqttConfig {
            broker_host: config.integrations.mqtt.broker_host.clone(),
            broker_port: config.integrations.mqtt.broker_port,
            client_id: config.integrations.mqtt.client_id.clone(),
            base_topic: config.integrations.mqtt.base_topic.clone(),
            keep_alive_secs: config.integrations.mqtt.keep_alive_secs,
        };
        let mut integration = MqttIntegration::new(mqtt_config);
        integration.setup(&ctx).await?;
        integration.start_background(ctx.clone()).await?;
        tracing::info!(
            integration = integration.name(),
            broker = %config.integrations.mqtt.broker_host,
            port = config.integrations.mqtt.broker_port,
            "MQTT integration ready"
        );
    }

    if config.integrations.ble.enabled {
        let ble_config = BleConfig {
            scan_duration_secs: config.integrations.ble.scan_duration_secs,
            update_interval_secs: config.integrations.ble.update_interval_secs,
            device_filter: config.integrations.ble.device_filter.clone(),
        };
        let mut integration = BleIntegration::new(ble_config);
        integration.setup(&ctx).await?;
        integration.start_background(ctx.clone()).await?;
        tracing::info!(integration = integration.name(), "BLE integration ready");
    }

    // Background purge task — removes old entity history records
    let hr_purge = Arc::clone(&history_repo);
    let retention_days = config.history.retention_days;
    let purge_interval_hours = config.history.purge_interval_hours;
    tokio::spawn(async move {
        let interval_duration =
            std::time::Duration::from_secs(u64::from(purge_interval_hours) * 3600);
        let mut interval = tokio::time::interval(interval_duration);
        interval.tick().await; // First tick completes immediately

        loop {
            interval.tick().await;

            let retention_secs = i64::from(retention_days) * 24 * 3600;
            let cutoff = minihub_domain::time::now()
                - std::time::Duration::from_secs(retention_secs.unsigned_abs());
            match hr_purge.purge_before(cutoff).await {
                Ok(count) => {
                    if count > 0 {
                        tracing::info!(count, retention_days, "purged old entity history records");
                    } else {
                        tracing::debug!(retention_days, "no old entity history records to purge");
                    }
                }
                Err(err) => {
                    tracing::warn!(%err, "failed to purge old entity history");
                }
            }
        }
    });
    tracing::info!(
        retention_days,
        purge_interval_hours,
        "entity history retention configured"
    );

    // HTTP
    let state = AppState::from_arcs(
        entity_service,
        device_service,
        area_service,
        event_store,
        automation_service,
        history_repo,
        event_bus,
    );
    let dashboard_dir = config.dashboard_dir();
    let app = minihub_adapter_http_axum::router::build(state, dashboard_dir.as_deref());

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
