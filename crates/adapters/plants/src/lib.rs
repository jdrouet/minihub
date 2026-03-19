//! # minihub-adapter-plants
//!
//! Plant health integration that creates virtual plant entities from Mi Flora
//! sensor data. Each plant entity tracks the health of a physical plant by
//! computing threshold-based status from its linked sensor's attributes.
//!
//! ## How it works
//!
//! 1. On `setup()`, creates/updates a `plant.*` entity for each configured plant.
//! 2. On `start_background()`, subscribes to the event bus and recomputes plant
//!    health whenever the linked Mi Flora sensor emits `AttributeChanged` or
//!    `StateChanged`.
//!
//! ## Dependency rule
//!
//! Depends on `minihub-app` (port traits) and `minihub-domain` only.

mod health;

use std::collections::HashMap;

use minihub_app::ports::integration::{Integration, IntegrationContext};
use minihub_domain::entity::{AttributeValue, Entity, EntityState};
use minihub_domain::error::{MiniHubError, NotFoundError};
use minihub_domain::event::EventType;
use minihub_domain::id::EntityId;
use tokio::task::JoinHandle;

pub use health::{HealthStatus, ThresholdStatus, compute_health, compute_threshold_status};

/// Configuration for a single plant.
#[derive(Debug, Clone)]
pub struct PlantConfig {
    /// User-facing plant name (e.g. "Monstera").
    pub name: String,
    /// Domain-level entity id of the linked Mi Flora sensor.
    pub source_entity_id: String,
    /// Moisture percentage below which the plant needs water.
    pub moisture_low: u8,
    /// Moisture percentage above which the plant is overwatered.
    pub moisture_high: u8,
    /// Minimum acceptable temperature in degrees Celsius.
    pub temperature_low: f64,
    /// Maximum acceptable temperature in degrees Celsius.
    pub temperature_high: f64,
    /// Minimum acceptable conductivity in µS/cm.
    pub conductivity_low: u16,
    /// Maximum acceptable conductivity in µS/cm.
    pub conductivity_high: u16,
}

impl PlantConfig {
    /// Derive the plant entity id from the plant name (e.g. "Monstera" → "plant.monstera").
    #[must_use]
    pub fn plant_entity_id(&self) -> String {
        let slug: String = self
            .name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();
        format!("plant.{slug}")
    }

    /// Build a plant entity from the current sensor data.
    fn build_entity(&self, sensor: Option<&Entity>) -> Result<Entity, MiniHubError> {
        let mut builder = Entity::builder()
            .entity_id(self.plant_entity_id())
            .friendly_name(&self.name)
            .attribute(
                "source_entity_id",
                AttributeValue::String(self.source_entity_id.clone()),
            )
            .attribute(
                "moisture_low",
                AttributeValue::Int(i64::from(self.moisture_low)),
            )
            .attribute(
                "moisture_high",
                AttributeValue::Int(i64::from(self.moisture_high)),
            )
            .attribute(
                "temperature_low",
                AttributeValue::Float(self.temperature_low),
            )
            .attribute(
                "temperature_high",
                AttributeValue::Float(self.temperature_high),
            )
            .attribute(
                "conductivity_low",
                AttributeValue::Int(i64::from(self.conductivity_low)),
            )
            .attribute(
                "conductivity_high",
                AttributeValue::Int(i64::from(self.conductivity_high)),
            );

        if let Some(sensor) = sensor {
            let health = compute_health(sensor, self);
            builder = builder.state(EntityState::On);
            builder =
                builder.attribute("health", AttributeValue::String(health.label().to_owned()));
            builder = builder.attribute(
                "moisture_status",
                AttributeValue::String(
                    compute_threshold_status(
                        sensor.get_attribute("moisture"),
                        f64::from(self.moisture_low),
                        f64::from(self.moisture_high),
                    )
                    .label()
                    .to_owned(),
                ),
            );
            builder = builder.attribute(
                "temperature_status",
                AttributeValue::String(
                    compute_threshold_status(
                        sensor.get_attribute("temperature"),
                        self.temperature_low,
                        self.temperature_high,
                    )
                    .label()
                    .to_owned(),
                ),
            );
            builder = builder.attribute(
                "conductivity_status",
                AttributeValue::String(
                    compute_threshold_status(
                        sensor.get_attribute("conductivity"),
                        f64::from(self.conductivity_low),
                        f64::from(self.conductivity_high),
                    )
                    .label()
                    .to_owned(),
                ),
            );

            for key in &[
                "temperature",
                "moisture",
                "light",
                "conductivity",
                "battery_level",
            ] {
                if let Some(value) = sensor.get_attribute(key) {
                    builder = builder.attribute(*key, value.clone());
                }
            }
        } else {
            builder = builder
                .state(EntityState::Unknown)
                .attribute("health", AttributeValue::String("unknown".to_owned()));
        }

        builder.build()
    }
}

/// Plant health integration.
pub struct PlantIntegration {
    configs: Vec<PlantConfig>,
    /// Maps source sensor `entity_id` to plant config index.
    source_map: HashMap<String, usize>,
    subscriber_handle: Option<JoinHandle<()>>,
}

impl PlantIntegration {
    /// Create a new plant integration with the given plant configurations.
    #[must_use]
    pub fn new(configs: Vec<PlantConfig>) -> Self {
        let source_map = configs
            .iter()
            .enumerate()
            .map(|(idx, cfg)| (cfg.source_entity_id.clone(), idx))
            .collect();

        Self {
            configs,
            source_map,
            subscriber_handle: None,
        }
    }
}

impl Integration for PlantIntegration {
    fn name(&self) -> &'static str {
        "plants"
    }

    async fn setup(&mut self, ctx: &impl IntegrationContext) -> Result<(), MiniHubError> {
        for config in &self.configs {
            let plant_entity = config.build_entity(None)?;
            ctx.upsert_entity(plant_entity).await?;

            tracing::info!(
                plant = %config.name,
                source = %config.source_entity_id,
                "plant entity created (sensor data will be populated by background task)"
            );
        }

        Ok(())
    }

    async fn start_background(
        &mut self,
        ctx: impl IntegrationContext + Clone + 'static,
    ) -> Result<(), MiniHubError> {
        let configs = self.configs.clone();
        let source_map = self.source_map.clone();

        self.subscriber_handle = Some(tokio::spawn(run_plant_subscriber(ctx, configs, source_map)));

        Ok(())
    }

    async fn handle_service_call(
        &self,
        entity_id: EntityId,
        _service: &str,
        _data: serde_json::Value,
    ) -> Result<Entity, MiniHubError> {
        Err(NotFoundError {
            entity: "PlantService",
            id: entity_id.to_string(),
        }
        .into())
    }

    async fn teardown(&mut self) -> Result<(), MiniHubError> {
        if let Some(handle) = self.subscriber_handle.take() {
            handle.abort();
            tracing::debug!("plant subscriber task aborted");
        }
        Ok(())
    }
}

/// Background task: subscribe to event bus and recompute plant health on sensor changes.
async fn run_plant_subscriber(
    ctx: impl IntegrationContext + 'static,
    configs: Vec<PlantConfig>,
    source_map: HashMap<String, usize>,
) {
    let mut rx = ctx.subscribe();

    loop {
        let event = match rx.recv().await {
            Ok(event) => event,
            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                tracing::warn!(skipped, "plant subscriber lagged, some events were missed");
                continue;
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                tracing::info!("plant subscriber channel closed, stopping");
                break;
            }
        };

        if !matches!(
            event.event_type,
            EventType::AttributeChanged | EventType::StateChanged
        ) {
            continue;
        }

        let Some(entity_id) = event.entity_id else {
            continue;
        };

        let Some(sensor) = ctx.find_entity_by_id(entity_id).await.ok().flatten() else {
            continue;
        };

        let Some(&config_idx) = source_map.get(&sensor.entity_id) else {
            continue;
        };

        let config = &configs[config_idx];
        let plant_entity = match config.build_entity(Some(&sensor)) {
            Ok(entity) => entity,
            Err(err) => {
                tracing::warn!(%err, plant = %config.name, "failed to build plant entity");
                continue;
            }
        };

        if let Err(err) = ctx.upsert_entity(plant_entity).await {
            tracing::warn!(%err, plant = %config.name, "failed to upsert plant entity");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> PlantConfig {
        PlantConfig {
            name: "Monstera".to_owned(),
            source_entity_id: "sensor.miflora_c47c8d6a1234".to_owned(),
            moisture_low: 15,
            moisture_high: 60,
            temperature_low: 10.0,
            temperature_high: 35.0,
            conductivity_low: 350,
            conductivity_high: 2000,
        }
    }

    #[test]
    fn should_derive_plant_entity_id_from_name() {
        let config = sample_config();
        assert_eq!(config.plant_entity_id(), "plant.monstera");
    }

    #[test]
    fn should_derive_slug_with_spaces_and_special_chars() {
        let mut config = sample_config();
        config.name = "My Ficus (Living Room)".to_owned();
        assert_eq!(config.plant_entity_id(), "plant.my_ficus__living_room_");
    }

    #[test]
    fn should_build_unknown_entity_when_sensor_is_none() {
        let config = sample_config();
        let entity = config.build_entity(None).unwrap();

        assert_eq!(entity.entity_id, "plant.monstera");
        assert_eq!(entity.friendly_name, "Monstera");
        assert_eq!(entity.state, EntityState::Unknown);
        assert_eq!(
            entity.get_attribute("health"),
            Some(&AttributeValue::String("unknown".to_owned()))
        );
        assert_eq!(
            entity.get_attribute("source_entity_id"),
            Some(&AttributeValue::String(
                "sensor.miflora_c47c8d6a1234".to_owned()
            ))
        );
    }

    #[test]
    fn should_build_healthy_entity_when_sensor_within_thresholds() {
        let config = sample_config();
        let sensor = Entity::builder()
            .entity_id("sensor.miflora_c47c8d6a1234")
            .friendly_name("Mi Flora")
            .state(EntityState::On)
            .attribute("temperature", AttributeValue::Float(22.0))
            .attribute("moisture", AttributeValue::Int(45))
            .attribute("light", AttributeValue::Int(5000))
            .attribute("conductivity", AttributeValue::Int(800))
            .attribute("battery_level", AttributeValue::Int(90))
            .build()
            .unwrap();

        let entity = config.build_entity(Some(&sensor)).unwrap();

        assert_eq!(entity.state, EntityState::On);
        assert_eq!(
            entity.get_attribute("health"),
            Some(&AttributeValue::String("healthy".to_owned()))
        );
        assert_eq!(
            entity.get_attribute("moisture_status"),
            Some(&AttributeValue::String("ok".to_owned()))
        );
        assert_eq!(
            entity.get_attribute("temperature"),
            Some(&AttributeValue::Float(22.0))
        );
        assert_eq!(
            entity.get_attribute("battery_level"),
            Some(&AttributeValue::Int(90))
        );
    }

    #[test]
    fn should_build_critical_entity_when_moisture_below_threshold() {
        let config = sample_config();
        let sensor = Entity::builder()
            .entity_id("sensor.miflora_c47c8d6a1234")
            .friendly_name("Mi Flora")
            .state(EntityState::On)
            .attribute("temperature", AttributeValue::Float(22.0))
            .attribute("moisture", AttributeValue::Int(10))
            .attribute("conductivity", AttributeValue::Int(800))
            .build()
            .unwrap();

        let entity = config.build_entity(Some(&sensor)).unwrap();

        assert_eq!(
            entity.get_attribute("health"),
            Some(&AttributeValue::String("critical".to_owned()))
        );
        assert_eq!(
            entity.get_attribute("moisture_status"),
            Some(&AttributeValue::String("low".to_owned()))
        );
    }

    #[test]
    fn should_create_source_map_from_configs() {
        let configs = vec![sample_config()];
        let integration = PlantIntegration::new(configs);
        assert_eq!(
            integration.source_map.get("sensor.miflora_c47c8d6a1234"),
            Some(&0)
        );
    }

    #[test]
    fn should_return_plants_as_integration_name() {
        let integration = PlantIntegration::new(Vec::new());
        assert_eq!(integration.name(), "plants");
    }
}
