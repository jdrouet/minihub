//! Plant health computation from sensor attributes and thresholds.

use minihub_domain::entity::{AttributeValue, Entity};

use crate::PlantConfig;

/// Overall health status of a plant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// All metrics are within acceptable thresholds.
    Healthy,
    /// At least one metric is above its high threshold.
    Attention,
    /// At least one metric is below its low threshold.
    Critical,
    /// No sensor data available.
    Unknown,
}

impl HealthStatus {
    /// String label used as the entity attribute value.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Attention => "attention",
            Self::Critical => "critical",
            Self::Unknown => "unknown",
        }
    }
}

/// Per-metric threshold status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThresholdStatus {
    /// Value is below the low threshold.
    Low,
    /// Value is within the acceptable range.
    Ok,
    /// Value is above the high threshold.
    High,
    /// No value available.
    Unknown,
}

impl ThresholdStatus {
    /// String label used as the entity attribute value.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Ok => "ok",
            Self::High => "high",
            Self::Unknown => "unknown",
        }
    }
}

/// Extract a numeric value from an attribute as `f64`.
fn attr_as_f64(attr: Option<&AttributeValue>) -> Option<f64> {
    match attr? {
        AttributeValue::Float(v) => Some(*v),
        #[allow(clippy::cast_precision_loss)]
        AttributeValue::Int(v) => Some(*v as f64),
        _ => None,
    }
}

/// Compute the threshold status for a single numeric attribute.
#[must_use]
pub fn compute_threshold_status(
    attr: Option<&AttributeValue>,
    low: f64,
    high: f64,
) -> ThresholdStatus {
    let Some(value) = attr_as_f64(attr) else {
        return ThresholdStatus::Unknown;
    };

    if value < low {
        ThresholdStatus::Low
    } else if value > high {
        ThresholdStatus::High
    } else {
        ThresholdStatus::Ok
    }
}

/// Compute the overall health status from sensor attributes and plant config thresholds.
///
/// Priority: `Critical` > `Attention` > `Healthy`. If any metric is below its
/// low threshold, the plant is `Critical`. If any is above its high threshold,
/// it needs `Attention`. If all are within range (or unavailable), it is `Healthy`.
#[must_use]
pub fn compute_health(sensor: &Entity, config: &PlantConfig) -> HealthStatus {
    let statuses = [
        compute_threshold_status(
            sensor.get_attribute("moisture"),
            f64::from(config.moisture_low),
            f64::from(config.moisture_high),
        ),
        compute_threshold_status(
            sensor.get_attribute("temperature"),
            config.temperature_low,
            config.temperature_high,
        ),
        compute_threshold_status(
            sensor.get_attribute("conductivity"),
            f64::from(config.conductivity_low),
            f64::from(config.conductivity_high),
        ),
    ];

    let has_low = statuses.contains(&ThresholdStatus::Low);
    let has_high = statuses.contains(&ThresholdStatus::High);

    if has_low {
        HealthStatus::Critical
    } else if has_high {
        HealthStatus::Attention
    } else {
        HealthStatus::Healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use minihub_domain::entity::EntityState;

    fn config() -> PlantConfig {
        PlantConfig {
            name: "Test Plant".to_owned(),
            source_entity_id: "sensor.miflora_test".to_owned(),
            moisture_low: 15,
            moisture_high: 60,
            temperature_low: 10.0,
            temperature_high: 35.0,
            conductivity_low: 350,
            conductivity_high: 2000,
        }
    }

    fn sensor_with(moisture: i64, temperature: f64, conductivity: i64) -> Entity {
        Entity::builder()
            .entity_id("sensor.miflora_test")
            .friendly_name("Test Sensor")
            .state(EntityState::On)
            .attribute("moisture", AttributeValue::Int(moisture))
            .attribute("temperature", AttributeValue::Float(temperature))
            .attribute("conductivity", AttributeValue::Int(conductivity))
            .build()
            .unwrap()
    }

    #[test]
    fn should_return_healthy_when_all_within_thresholds() {
        let sensor = sensor_with(40, 22.0, 800);
        assert_eq!(compute_health(&sensor, &config()), HealthStatus::Healthy);
    }

    #[test]
    fn should_return_critical_when_moisture_below_low() {
        let sensor = sensor_with(10, 22.0, 800);
        assert_eq!(compute_health(&sensor, &config()), HealthStatus::Critical);
    }

    #[test]
    fn should_return_critical_when_temperature_below_low() {
        let sensor = sensor_with(40, 5.0, 800);
        assert_eq!(compute_health(&sensor, &config()), HealthStatus::Critical);
    }

    #[test]
    fn should_return_attention_when_moisture_above_high() {
        let sensor = sensor_with(70, 22.0, 800);
        assert_eq!(compute_health(&sensor, &config()), HealthStatus::Attention);
    }

    #[test]
    fn should_return_attention_when_temperature_above_high() {
        let sensor = sensor_with(40, 40.0, 800);
        assert_eq!(compute_health(&sensor, &config()), HealthStatus::Attention);
    }

    #[test]
    fn should_return_critical_when_both_low_and_high_present() {
        let sensor = sensor_with(10, 40.0, 800);
        assert_eq!(compute_health(&sensor, &config()), HealthStatus::Critical);
    }

    #[test]
    fn should_return_healthy_when_at_exact_boundaries() {
        let sensor = sensor_with(15, 10.0, 350);
        assert_eq!(compute_health(&sensor, &config()), HealthStatus::Healthy);
    }

    #[test]
    fn should_return_healthy_when_at_high_boundaries() {
        let sensor = sensor_with(60, 35.0, 2000);
        assert_eq!(compute_health(&sensor, &config()), HealthStatus::Healthy);
    }

    #[test]
    fn should_return_low_threshold_status_when_below() {
        let status = compute_threshold_status(Some(&AttributeValue::Int(10)), 15.0, 60.0);
        assert_eq!(status, ThresholdStatus::Low);
    }

    #[test]
    fn should_return_ok_threshold_status_when_within() {
        let status = compute_threshold_status(Some(&AttributeValue::Int(30)), 15.0, 60.0);
        assert_eq!(status, ThresholdStatus::Ok);
    }

    #[test]
    fn should_return_high_threshold_status_when_above() {
        let status = compute_threshold_status(Some(&AttributeValue::Int(70)), 15.0, 60.0);
        assert_eq!(status, ThresholdStatus::High);
    }

    #[test]
    fn should_return_unknown_threshold_status_when_attr_missing() {
        let status = compute_threshold_status(None, 15.0, 60.0);
        assert_eq!(status, ThresholdStatus::Unknown);
    }

    #[test]
    fn should_handle_float_attribute_in_threshold_check() {
        let status = compute_threshold_status(Some(&AttributeValue::Float(5.0)), 10.0, 35.0);
        assert_eq!(status, ThresholdStatus::Low);
    }

    #[test]
    fn should_return_correct_health_labels() {
        assert_eq!(HealthStatus::Healthy.label(), "healthy");
        assert_eq!(HealthStatus::Attention.label(), "attention");
        assert_eq!(HealthStatus::Critical.label(), "critical");
        assert_eq!(HealthStatus::Unknown.label(), "unknown");
    }

    #[test]
    fn should_return_correct_threshold_labels() {
        assert_eq!(ThresholdStatus::Low.label(), "low");
        assert_eq!(ThresholdStatus::Ok.label(), "ok");
        assert_eq!(ThresholdStatus::High.label(), "high");
        assert_eq!(ThresholdStatus::Unknown.label(), "unknown");
    }
}
