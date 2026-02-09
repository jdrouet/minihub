//! End-to-end smoke tests for the full minihubd stack.
//!
//! Each test spins up the complete application (in-memory `SQLite`, real repos,
//! real services, real axum router) and exercises the HTTP layer via
//! `tower::ServiceExt::oneshot` — no TCP port is bound.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use minihub_adapter_http_axum::router;
use minihub_adapter_http_axum::state::AppState;
use minihub_adapter_storage_sqlite_sqlx::{
    Config, SqliteAreaRepository, SqliteAutomationRepository, SqliteDeviceRepository,
    SqliteEntityRepository, SqliteEventStore,
};
use minihub_adapter_virtual::VirtualIntegration;
use minihub_app::event_bus::InProcessEventBus;
use minihub_app::ports::{EventStore, Integration};
use minihub_app::services::area_service::AreaService;
use minihub_app::services::automation_service::AutomationService;
use minihub_app::services::device_service::DeviceService;
use minihub_app::services::entity_service::EntityService;
use std::sync::Arc;
use tower::ServiceExt;

/// Build a fully-wired router backed by an in-memory `SQLite` database,
/// including an event-bus → event-store subscriber (mirroring `main.rs`).
async fn app() -> axum::Router {
    let db = Config {
        database_url: "sqlite::memory:".to_string(),
    }
    .build()
    .await
    .expect("in-memory database should initialise");

    let pool = db.pool().clone();

    let entity_repo = SqliteEntityRepository::new(pool.clone());
    let device_repo = SqliteDeviceRepository::new(pool.clone());
    let area_repo = SqliteAreaRepository::new(pool.clone());
    let event_store = SqliteEventStore::new(pool.clone());
    let automation_repo = SqliteAutomationRepository::new(pool);

    let event_bus = InProcessEventBus::new(256);
    let mut event_rx = event_bus.subscribe();

    let entity_service = Arc::new(EntityService::new(entity_repo, event_bus));
    let device_service = Arc::new(DeviceService::new(device_repo));
    let area_service = Arc::new(AreaService::new(area_repo));
    let event_store = Arc::new(event_store);
    let automation_service = Arc::new(AutomationService::new(automation_repo));

    // Wire event-bus → event-store subscriber (same as main.rs)
    let es = Arc::clone(&event_store);
    tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            let _ = es.store(event).await;
        }
    });

    let state = AppState::from_arcs(
        entity_service,
        device_service,
        area_service,
        event_store,
        automation_service,
    );

    router::build(state)
}

// ---------------------------------------------------------------------------
// Health check
// ---------------------------------------------------------------------------

#[tokio::test]
async fn should_return_ok_when_health_check_called() {
    let resp = app()
        .await
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// Dashboard (SSR) pages
// ---------------------------------------------------------------------------

#[tokio::test]
async fn should_render_home_page() {
    let resp = app()
        .await
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = String::from_utf8(
        resp.into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(body.contains("Dashboard"));
}

#[tokio::test]
async fn should_render_entities_page() {
    let resp = app()
        .await
        .oneshot(
            Request::builder()
                .uri("/entities")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = String::from_utf8(
        resp.into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(body.contains("Entities"));
}

#[tokio::test]
async fn should_render_devices_page() {
    let resp = app()
        .await
        .oneshot(
            Request::builder()
                .uri("/devices")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = String::from_utf8(
        resp.into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(body.contains("Devices"));
}

#[tokio::test]
async fn should_render_areas_page() {
    let resp = app()
        .await
        .oneshot(
            Request::builder()
                .uri("/areas")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = String::from_utf8(
        resp.into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(body.contains("Areas"));
}

#[tokio::test]
async fn should_render_events_page() {
    let resp = app()
        .await
        .oneshot(
            Request::builder()
                .uri("/events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = String::from_utf8(
        resp.into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(body.contains("Event Log"));
}

#[tokio::test]
async fn should_render_automations_page() {
    let resp = app()
        .await
        .oneshot(
            Request::builder()
                .uri("/automations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = String::from_utf8(
        resp.into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(body.contains("Automations"));
}

// ---------------------------------------------------------------------------
// API: full CRUD cycle for devices → entities
// ---------------------------------------------------------------------------

#[tokio::test]
async fn should_complete_device_crud_cycle() {
    let app = app().await;

    // Create device
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/devices")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name":"Test Light","manufacturer":"Acme","model":"X100","integration":"test","unique_id":"test_light_1"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: serde_json::Value =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let device_id = body["id"].as_str().unwrap().to_string();

    // List devices
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/devices")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Vec<serde_json::Value> =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body.len(), 1);
    assert_eq!(body[0]["name"], "Test Light");

    // Get device
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/devices/{device_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    // Delete device
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/devices/{device_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Verify gone
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/devices")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body: Vec<serde_json::Value> =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body.len(), 0);
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn should_complete_entity_crud_cycle() {
    let app = app().await;

    // First create a device (entities need a device_id FK)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/devices")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name":"Sensor Hub","integration":"test","unique_id":"sensor_hub_1"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: serde_json::Value =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let device_id = body["id"].as_str().unwrap().to_string();

    // Create entity
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/entities")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"device_id":"{device_id}","entity_id":"sensor.temp","friendly_name":"Temperature"}}"#,
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: serde_json::Value =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let entity_id = body["id"].as_str().unwrap().to_string();
    assert_eq!(body["state"], "unknown");

    // Update state
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/entities/{entity_id}/state"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"state":"on"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body["state"], "on");

    // Get entity
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/entities/{entity_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body["state"], "on");

    // List entities
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/entities")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body: Vec<serde_json::Value> =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body.len(), 1);

    // Delete entity
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/entities/{entity_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Verify gone
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/entities")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body: Vec<serde_json::Value> =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body.len(), 0);
}

#[tokio::test]
async fn should_complete_area_crud_cycle() {
    let app = app().await;

    // Create area
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/areas")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Living Room"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: serde_json::Value =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let area_id = body["id"].as_str().unwrap().to_string();
    assert_eq!(body["name"], "Living Room");

    // Get area
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/areas/{area_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    // Delete area
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/areas/{area_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ---------------------------------------------------------------------------
// Dashboard shows data after API creates it
// ---------------------------------------------------------------------------

#[tokio::test]
async fn should_show_entity_on_dashboard_after_api_creation() {
    let app = app().await;

    // Create device via API
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/devices")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name":"Hub","integration":"test","unique_id":"hub_1"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let dev: serde_json::Value =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let device_id = dev["id"].as_str().unwrap();

    // Create entity via API
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/entities")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"device_id":"{device_id}","entity_id":"light.desk","friendly_name":"Desk Lamp"}}"#,
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    // Dashboard entities page should list it
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/entities")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let html = String::from_utf8(
        resp.into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(html.contains("Desk Lamp"));
    assert!(html.contains("light.desk"));

    // Home page should show counts
    let resp = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let html = String::from_utf8(
        resp.into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(html.contains("Dashboard"));
}

// ---------------------------------------------------------------------------
// API: automation CRUD cycle
// ---------------------------------------------------------------------------

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn should_complete_automation_crud_cycle() {
    let app = app().await;

    // Create automation
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/automations")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "name": "Night mode",
                        "trigger": {"type": "manual"},
                        "actions": [{"type": "delay", "seconds": 5}]
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: serde_json::Value =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let automation_id = body["id"].as_str().unwrap().to_string();
    assert_eq!(body["name"], "Night mode");
    assert_eq!(body["enabled"], true);

    // List automations
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/automations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Vec<serde_json::Value> =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body.len(), 1);

    // Get automation
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/automations/{automation_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body["name"], "Night mode");

    // Update automation
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/automations/{automation_id}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "name": "Night mode v2",
                        "enabled": false,
                        "trigger": {"type": "manual"},
                        "conditions": [],
                        "actions": [{"type": "delay", "seconds": 10}]
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body["name"], "Night mode v2");
    assert_eq!(body["enabled"], false);

    // Delete automation
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/automations/{automation_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Verify gone
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/automations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body: Vec<serde_json::Value> =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body.len(), 0);
}

// ---------------------------------------------------------------------------
// API: events are visible after entity state changes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn should_list_events_after_entity_state_change() {
    let app = app().await;

    // Create device
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/devices")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name":"Hub","integration":"test","unique_id":"hub_1"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let dev: serde_json::Value =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let device_id = dev["id"].as_str().unwrap();

    // Create entity
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/entities")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"device_id":"{device_id}","entity_id":"light.test","friendly_name":"Test"}}"#,
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    let ent: serde_json::Value =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let entity_id = ent["id"].as_str().unwrap();

    // Update state to generate a StateChanged event
    app.clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/entities/{entity_id}/state"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"state":"on"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    // Give the subscriber task time to persist the events
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Events should now be persisted: EntityCreated + StateChanged
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Vec<serde_json::Value> =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body.len(), 2);

    let types: Vec<&str> = body
        .iter()
        .map(|e| e["event_type"].as_str().unwrap())
        .collect();
    assert!(types.contains(&"entity_created"));
    assert!(types.contains(&"state_changed"));
}

// ---------------------------------------------------------------------------
// Virtual integration — full lifecycle through the stack
// ---------------------------------------------------------------------------

/// Build a fully-wired router that also runs the virtual integration setup,
/// mirroring what `minihubd` does on startup.
async fn app_with_virtual() -> axum::Router {
    let db = Config {
        database_url: "sqlite::memory:".to_string(),
    }
    .build()
    .await
    .expect("in-memory database should initialise");

    let pool = db.pool().clone();

    let entity_repo = SqliteEntityRepository::new(pool.clone());
    let device_repo = SqliteDeviceRepository::new(pool.clone());
    let area_repo = SqliteAreaRepository::new(pool.clone());
    let event_store = SqliteEventStore::new(pool.clone());
    let automation_repo = SqliteAutomationRepository::new(pool);

    let event_bus = InProcessEventBus::new(256);
    let mut event_rx = event_bus.subscribe();

    let entity_service = Arc::new(EntityService::new(entity_repo, event_bus));
    let device_service = Arc::new(DeviceService::new(device_repo));
    let area_service = Arc::new(AreaService::new(area_repo));
    let event_store = Arc::new(event_store);
    let automation_service = Arc::new(AutomationService::new(automation_repo));

    // Wire event-bus → event-store subscriber
    let es = Arc::clone(&event_store);
    tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            let _ = es.store(event).await;
        }
    });

    // Run virtual integration setup — same as minihubd main()
    let mut virtual_integration = VirtualIntegration::default();
    let discovered = virtual_integration.setup().await.unwrap();
    for dd in discovered {
        let _ = device_service.upsert_device(dd.device).await;
        for entity in dd.entities {
            let _ = entity_service.upsert_entity(entity).await;
        }
    }

    let state = AppState::from_arcs(
        entity_service,
        device_service,
        area_service,
        event_store,
        automation_service,
    );

    router::build(state)
}

#[tokio::test]
async fn should_list_virtual_entities_via_api() {
    let app = app_with_virtual().await;

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/entities")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Vec<serde_json::Value> =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();

    assert_eq!(body.len(), 3);

    let entity_ids: Vec<&str> = body
        .iter()
        .map(|e| e["entity_id"].as_str().unwrap())
        .collect();
    assert!(entity_ids.contains(&"light.virtual_light"));
    assert!(entity_ids.contains(&"sensor.virtual_temperature"));
    assert!(entity_ids.contains(&"switch.virtual_switch"));
}

#[tokio::test]
async fn should_list_virtual_devices_via_api() {
    let app = app_with_virtual().await;

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/devices")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Vec<serde_json::Value> =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();

    assert_eq!(body.len(), 3);

    let names: Vec<&str> = body.iter().map(|d| d["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"Virtual Light"));
    assert!(names.contains(&"Virtual Sensor"));
    assert!(names.contains(&"Virtual Switch"));
}

#[tokio::test]
async fn should_show_virtual_entities_on_dashboard() {
    let app = app_with_virtual().await;

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/entities")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let html = String::from_utf8(
        resp.into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();

    assert!(html.contains("Virtual Light"));
    assert!(html.contains("Virtual Temperature"));
    assert!(html.contains("Virtual Switch"));
}

#[tokio::test]
async fn should_show_virtual_devices_on_dashboard() {
    let app = app_with_virtual().await;

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/devices")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let html = String::from_utf8(
        resp.into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();

    assert!(html.contains("Virtual Light"));
    assert!(html.contains("Virtual Sensor"));
    assert!(html.contains("Virtual Switch"));
}

#[tokio::test]
async fn should_update_virtual_entity_state_via_api() {
    let app = app_with_virtual().await;

    // Get the light entity id
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/entities")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let entities: Vec<serde_json::Value> =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let light = entities
        .iter()
        .find(|e| e["entity_id"] == "light.virtual_light")
        .unwrap();
    let light_id = light["id"].as_str().unwrap();
    assert_eq!(light["state"], "off");

    // Turn on the light via state update API
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/entities/{light_id}/state"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"state":"on"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body["state"], "on");

    // Verify it persisted
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/entities/{light_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body["state"], "on");
}

#[tokio::test]
async fn should_get_virtual_entity_with_sensor_attributes() {
    let app = app_with_virtual().await;

    // Get all entities, find the sensor
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/entities")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let entities: Vec<serde_json::Value> =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let sensor = entities
        .iter()
        .find(|e| e["entity_id"] == "sensor.virtual_temperature")
        .unwrap();
    let sensor_id = sensor["id"].as_str().unwrap();

    // Get sensor detail
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/entities/{sensor_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value =
        serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body["entity_id"], "sensor.virtual_temperature");
    assert_eq!(body["friendly_name"], "Virtual Temperature");
    assert_eq!(body["attributes"]["temperature"], 21.5);
    assert_eq!(body["attributes"]["unit"], "\u{b0}C");
}
