//! Plant card component for displaying plant health status at a glance.

use leptos::prelude::*;
use leptos_router::components::A;
use minihub_domain::entity::{AttributeValue, Entity};

use super::sensor_card::{format_relative_time, int_attr};

/// Returns `true` when the entity is a plant (entity_id starts with `plant.`).
#[must_use]
pub fn is_plant(entity: &Entity) -> bool {
    entity.entity_id.starts_with("plant.")
}

/// Extract a string attribute value.
fn str_attr<'a>(entity: &'a Entity, key: &str) -> Option<&'a str> {
    match entity.get_attribute(key)? {
        AttributeValue::String(s) => Some(s.as_str()),
        _ => None,
    }
}

/// Human-readable label for the overall health status.
#[must_use]
pub fn health_label(health: &str) -> &'static str {
    match health {
        "healthy" => "Healthy",
        "attention" => "Attention",
        "critical" => "Critical",
        "unknown" => "Unknown",
        _ => "Unknown",
    }
}

/// CSS class for the health badge.
#[must_use]
pub fn health_class(health: &str) -> &'static str {
    match health {
        "healthy" => "plant-status-healthy",
        "attention" => "plant-status-attention",
        "critical" => "plant-status-critical",
        _ => "plant-status-unknown",
    }
}

/// CSS class for a per-metric status indicator.
#[must_use]
pub fn metric_status_class(status: &str) -> &'static str {
    match status {
        "low" => "metric-low",
        "ok" => "metric-ok",
        "high" => "metric-high",
        _ => "metric-unknown",
    }
}

/// A card displaying plant health status and key metrics.
#[component]
pub fn PlantCard(entity: Entity) -> impl IntoView {
    let href = format!("/entities/{}", entity.id);
    let name = entity.friendly_name.clone();
    let health = str_attr(&entity, "health").unwrap_or("unknown").to_owned();
    let badge_label = health_label(&health);
    let badge_class = format!("plant-status-badge {}", health_class(&health));

    let moisture_status = str_attr(&entity, "moisture_status")
        .unwrap_or("unknown")
        .to_owned();
    let temp_status = str_attr(&entity, "temperature_status")
        .unwrap_or("unknown")
        .to_owned();
    let conductivity_status = str_attr(&entity, "conductivity_status")
        .unwrap_or("unknown")
        .to_owned();

    let moisture = int_attr(&entity, "moisture");
    let temperature = entity.get_attribute("temperature").and_then(|v| match v {
        AttributeValue::Float(f) => Some(*f),
        _ => None,
    });
    let light = int_attr(&entity, "light");
    let conductivity = int_attr(&entity, "conductivity");
    let battery = int_attr(&entity, "battery_level");
    let last_updated = format_relative_time(entity.last_updated);

    view! {
        <A href=href attr:class="plant-card-link">
            <div class="plant-card">
                <div class="plant-card-header">
                    <span class="plant-card-name">{name}</span>
                    <span class=badge_class>{badge_label}</span>
                </div>
                <div class="plant-card-metrics">
                    {moisture.map(|m| {
                        let cls = metric_status_class(&moisture_status);
                        view! {
                            <div class="plant-metric">
                                <span class=format!("plant-metric-dot {cls}")></span>
                                <span class="plant-metric-label">"Moisture"</span>
                                <span class="plant-metric-value">{m.to_string()} "%"</span>
                            </div>
                        }
                    })}
                    {temperature.map(|t| {
                        let cls = metric_status_class(&temp_status);
                        view! {
                            <div class="plant-metric">
                                <span class=format!("plant-metric-dot {cls}")></span>
                                <span class="plant-metric-label">"Temp"</span>
                                <span class="plant-metric-value">{format!("{t:.1}")} "°C"</span>
                            </div>
                        }
                    })}
                    {light.map(|l| view! {
                        <div class="plant-metric">
                            <span class="plant-metric-dot metric-ok"></span>
                            <span class="plant-metric-label">"Light"</span>
                            <span class="plant-metric-value">{l.to_string()} " lux"</span>
                        </div>
                    })}
                    {conductivity.map(|c| {
                        let cls = metric_status_class(&conductivity_status);
                        view! {
                            <div class="plant-metric">
                                <span class=format!("plant-metric-dot {cls}")></span>
                                <span class="plant-metric-label">"Conductivity"</span>
                                <span class="plant-metric-value">{c.to_string()} " µS/cm"</span>
                            </div>
                        }
                    })}
                </div>
                <div class="plant-card-footer">
                    <span class="plant-card-updated">{last_updated}</span>
                    {battery.map(|b| view! {
                        <span class="plant-card-battery">{format!("{b}%")}</span>
                    })}
                </div>
            </div>
        </A>
    }
}

/// A grid of plant cards filtered from a signal of all entities.
#[component]
pub fn PlantCardGrid(entities: ReadSignal<Vec<Entity>>) -> impl IntoView {
    let plant_entries = move || {
        entities
            .get()
            .into_iter()
            .filter(is_plant)
            .collect::<Vec<_>>()
    };

    view! {
        <Show
            when=move || !plant_entries().is_empty()
            fallback=|| view! { <span></span> }
        >
            <h2>"Plants"</h2>
            <div class="plant-card-grid">
                <For
                    each=plant_entries
                    key=|entity| entity.id
                    let(entity)
                >
                    <PlantCard entity/>
                </For>
            </div>
        </Show>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use minihub_domain::entity::EntityState;

    fn plant_entity(health: &str, moisture_status: &str) -> Entity {
        Entity::builder()
            .entity_id("plant.monstera")
            .friendly_name("Monstera")
            .state(EntityState::On)
            .attribute("health", AttributeValue::String(health.to_owned()))
            .attribute(
                "moisture_status",
                AttributeValue::String(moisture_status.to_owned()),
            )
            .attribute("temperature", AttributeValue::Float(22.0))
            .attribute("moisture", AttributeValue::Int(45))
            .build()
            .unwrap()
    }

    fn non_plant_entity() -> Entity {
        Entity::builder()
            .entity_id("sensor.miflora_abc")
            .friendly_name("Mi Flora")
            .state(EntityState::On)
            .build()
            .unwrap()
    }

    #[test]
    fn should_detect_plant_entity_by_prefix() {
        let entity = plant_entity("healthy", "ok");
        assert!(is_plant(&entity));
    }

    #[test]
    fn should_not_detect_non_plant_entity() {
        let entity = non_plant_entity();
        assert!(!is_plant(&entity));
    }

    #[test]
    fn should_return_correct_health_labels() {
        assert_eq!(health_label("healthy"), "Healthy");
        assert_eq!(health_label("attention"), "Attention");
        assert_eq!(health_label("critical"), "Critical");
        assert_eq!(health_label("unknown"), "Unknown");
        assert_eq!(health_label("bogus"), "Unknown");
    }

    #[test]
    fn should_return_correct_health_classes() {
        assert_eq!(health_class("healthy"), "plant-status-healthy");
        assert_eq!(health_class("attention"), "plant-status-attention");
        assert_eq!(health_class("critical"), "plant-status-critical");
        assert_eq!(health_class("unknown"), "plant-status-unknown");
    }

    #[test]
    fn should_return_correct_metric_status_classes() {
        assert_eq!(metric_status_class("low"), "metric-low");
        assert_eq!(metric_status_class("ok"), "metric-ok");
        assert_eq!(metric_status_class("high"), "metric-high");
        assert_eq!(metric_status_class("unknown"), "metric-unknown");
    }

    #[test]
    fn should_extract_string_attribute() {
        let entity = plant_entity("healthy", "ok");
        assert_eq!(str_attr(&entity, "health"), Some("healthy"));
        assert_eq!(str_attr(&entity, "missing"), None);
    }
}
