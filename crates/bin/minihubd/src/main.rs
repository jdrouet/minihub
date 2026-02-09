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
    SqliteEntityRepository, SqliteEventStore,
};
use minihub_adapter_virtual::VirtualIntegration;
use minihub_app::event_bus::InProcessEventBus;
use minihub_app::ports::Integration;
use minihub_app::ports::integration::DiscoveredDevice;
use minihub_app::services::area_service::AreaService;
use minihub_app::services::automation_service::AutomationService;
use minihub_app::services::device_service::DeviceService;
use minihub_app::services::entity_service::EntityService;
use minihub_domain::error::MiniHubError;
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

    // Services (Arc-wrapped early so they can be shared with background tasks)
    let entity_service = Arc::new(EntityService::new(entity_repo, event_bus));
    let device_service = Arc::new(DeviceService::new(device_repo));
    let area_service = Arc::new(AreaService::new(area_repo));
    let automation_service = Arc::new(AutomationService::new(automation_repo));
    let event_store = Arc::new(event_store);

    // Integrations
    setup_integrations(&config, &device_service, &entity_service).await?;

    // HTTP
    let state = AppState::from_arcs(
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

/// Set up all enabled integrations, persisting discovered devices/entities.
async fn setup_integrations<DR, ER, EP>(
    config: &Config,
    device_service: &Arc<DeviceService<DR>>,
    entity_service: &Arc<EntityService<ER, EP>>,
) -> Result<(), MiniHubError>
where
    DR: minihub_app::ports::DeviceRepository + Send + Sync + 'static,
    ER: minihub_app::ports::EntityRepository + Send + Sync + 'static,
    EP: minihub_app::ports::EventPublisher + Send + Sync + 'static,
{
    // Virtual integration — discover and register simulated devices
    if config.integrations.virtual_enabled {
        let mut virtual_integration = VirtualIntegration::default();
        let discovered = virtual_integration.setup().await?;
        for dd in discovered {
            let _ = device_service.upsert_device(dd.device).await;
            for entity in dd.entities {
                let _ = entity_service.upsert_entity(entity).await;
            }
        }
        tracing::info!(
            integration = virtual_integration.name(),
            "virtual integration ready"
        );
    }

    // MQTT integration — connect to broker and discover devices
    if config.integrations.mqtt.enabled {
        let mqtt_config = MqttConfig {
            broker_host: config.integrations.mqtt.broker_host.clone(),
            broker_port: config.integrations.mqtt.broker_port,
            client_id: config.integrations.mqtt.client_id.clone(),
            base_topic: config.integrations.mqtt.base_topic.clone(),
            keep_alive_secs: config.integrations.mqtt.keep_alive_secs,
            discovery_timeout_secs: config.integrations.mqtt.discovery_timeout_secs,
        };
        let mut mqtt_integration = MqttIntegration::new(mqtt_config);
        let discovered = mqtt_integration.setup().await?;
        for dd in discovered {
            let _ = device_service.upsert_device(dd.device).await;
            for entity in dd.entities {
                let _ = entity_service.upsert_entity(entity).await;
            }
        }
        tracing::info!(
            integration = mqtt_integration.name(),
            broker = %config.integrations.mqtt.broker_host,
            port = config.integrations.mqtt.broker_port,
            "MQTT integration ready"
        );
    }

    // BLE integration — passively scan for BLE sensors
    if config.integrations.ble.enabled {
        let ble_config = BleConfig {
            scan_duration_secs: config.integrations.ble.scan_duration_secs,
            update_interval_secs: config.integrations.ble.update_interval_secs,
            device_filter: config.integrations.ble.device_filter.clone(),
        };

        let (ble_tx, mut ble_rx) = tokio::sync::mpsc::channel::<DiscoveredDevice>(64);
        let mut ble_integration = BleIntegration::new(ble_config, Some(ble_tx));

        let discovered = ble_integration.setup().await?;
        for dd in discovered {
            let _ = device_service.upsert_device(dd.device).await;
            for entity in dd.entities {
                let _ = entity_service.upsert_entity(entity).await;
            }
        }

        // Spawn receiver task for background BLE discoveries
        let ds = Arc::clone(device_service);
        let es = Arc::clone(entity_service);
        tokio::spawn(async move {
            while let Some(dd) = ble_rx.recv().await {
                let _ = ds.upsert_device(dd.device).await;
                for entity in dd.entities {
                    let _ = es.upsert_entity(entity).await;
                }
            }
            tracing::debug!("BLE discovery channel closed");
        });

        tracing::info!(
            integration = ble_integration.name(),
            "BLE integration ready"
        );
    }

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
