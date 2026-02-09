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
    Config, SqliteAreaRepository, SqliteDeviceRepository, SqliteEntityRepository,
};
use minihub_app::event_bus::InProcessEventBus;
use minihub_app::services::area_service::AreaService;
use minihub_app::services::device_service::DeviceService;
use minihub_app::services::entity_service::EntityService;
use tower::ServiceExt;

/// Build a fully-wired router backed by an in-memory `SQLite` database.
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
    let area_repo = SqliteAreaRepository::new(pool);

    let event_bus = InProcessEventBus::new(256);

    let state = AppState::new(
        EntityService::new(entity_repo, event_bus),
        DeviceService::new(device_repo),
        AreaService::new(area_repo),
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
                    r#"{"name":"Test Light","manufacturer":"Acme","model":"X100"}"#,
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
                .body(Body::from(r#"{"name":"Sensor Hub"}"#))
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
                .body(Body::from(r#"{"name":"Hub"}"#))
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
