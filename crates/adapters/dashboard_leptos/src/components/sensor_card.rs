//! Sensor card component for displaying BLE sensor entity data at a glance.

use chrono::{DateTime, Utc};
use leptos::prelude::*;
use leptos_router::components::A;
use minihub_domain::entity::{AttributeValue, Entity};

/// Recognised sensor variant, determined by inspecting the entity's `entity_id`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorKind {
    /// Xiaomi LYWSD03MMC temperature/humidity sensor.
    TempHumidity,
    /// Xiaomi Mi Flora plant sensor.
    MiFlora,
}

impl SensorKind {
    /// Try to classify an entity as a known BLE sensor kind.
    #[must_use]
    pub fn detect(entity: &Entity) -> Option<Self> {
        if entity.entity_id.starts_with("sensor.ble_") {
            Some(Self::TempHumidity)
        } else if entity.entity_id.starts_with("sensor.miflora_") {
            Some(Self::MiFlora)
        } else {
            None
        }
    }

    /// Human-readable label for the sensor kind.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::TempHumidity => "Temp / Humidity",
            Self::MiFlora => "Mi Flora",
        }
    }
}

/// Extract a float attribute value, returning `None` when the key is absent.
fn float_attr(entity: &Entity, key: &str) -> Option<f64> {
    match entity.get_attribute(key)? {
        AttributeValue::Float(v) => Some(*v),
        AttributeValue::Int(v) => Some(*v as f64),
        _ => None,
    }
}

/// Extract an integer attribute value.
pub(crate) fn int_attr(entity: &Entity, key: &str) -> Option<i64> {
    match entity.get_attribute(key)? {
        AttributeValue::Int(v) => Some(*v),
        AttributeValue::Float(v) => Some(*v as i64),
        _ => None,
    }
}

/// Format a float to one decimal place.
fn fmt_f1(v: f64) -> String {
    format!("{v:.1}")
}

/// A card that shows key metrics for a BLE sensor entity.
///
/// Renders differently depending on the [`SensorKind`]:
/// - **`TempHumidity`**: temperature, humidity, battery level.
/// - **`MiFlora`**: temperature, moisture, light, conductivity, battery level.
///
/// The card links to the entity detail page.
#[component]
pub fn SensorCard(
    /// The sensor entity to display.
    entity: Entity,
    /// The detected sensor kind.
    kind: SensorKind,
) -> impl IntoView {
    let href = format!("/entities/{}", entity.id);
    let name = entity.friendly_name.clone();
    let kind_label = kind.label();

    let body = match kind {
        SensorKind::TempHumidity => temp_humidity_body(&entity).into_any(),
        SensorKind::MiFlora => miflora_body(&entity).into_any(),
    };

    let battery = int_attr(&entity, "battery_level");
    let last_updated = format_relative_time(entity.last_updated);

    view! {
        <A href=href attr:class="sensor-card-link">
            <div class="sensor-card">
                <div class="sensor-card-header">
                    <span class="sensor-card-name">{name}</span>
                    <span class="sensor-card-kind">{kind_label}</span>
                </div>
                <div class="sensor-card-body">{body}</div>
                <div class="sensor-card-footer">
                    <span class="sensor-card-updated">{last_updated}</span>
                    {battery.map(|level| {
                        let css = battery_class(level);
                        let icon = battery_icon(level);
                        view! {
                            <span class=format!("sensor-battery {css}")>
                                <span class="sensor-battery-icon">{icon}</span>
                                {format!("{level}%")}
                            </span>
                        }
                    })}
                </div>
            </div>
        </A>
    }
}

/// Build the body view for a Temp/Humidity sensor.
fn temp_humidity_body(entity: &Entity) -> impl IntoView {
    let temperature = float_attr(entity, "temperature");
    let humidity = float_attr(entity, "humidity");

    view! {
        <div class="sensor-metrics">
            {temperature.map(|t| view! {
                <div class="sensor-metric">
                    <span class="sensor-metric-label">"Temp"</span>
                    <span class="sensor-metric-value">{fmt_f1(t)}</span>
                    <span class="sensor-metric-unit">"°C"</span>
                </div>
            })}
            {humidity.map(|h| view! {
                <div class="sensor-metric">
                    <span class="sensor-metric-label">"Humidity"</span>
                    <span class="sensor-metric-value">{fmt_f1(h)}</span>
                    <span class="sensor-metric-unit">"%"</span>
                </div>
            })}
        </div>
    }
}

/// Build the body view for a Mi Flora plant sensor.
fn miflora_body(entity: &Entity) -> impl IntoView {
    let temperature = float_attr(entity, "temperature");
    let moisture = int_attr(entity, "moisture");
    let light = int_attr(entity, "light");
    let conductivity = int_attr(entity, "conductivity");

    view! {
        <div class="sensor-metrics sensor-metrics-grid">
            {temperature.map(|t| view! {
                <div class="sensor-metric">
                    <span class="sensor-metric-label">"Temp"</span>
                    <span class="sensor-metric-value">{fmt_f1(t)}</span>
                    <span class="sensor-metric-unit">"°C"</span>
                </div>
            })}
            {moisture.map(|m| view! {
                <div class="sensor-metric">
                    <span class="sensor-metric-label">"Moisture"</span>
                    <span class="sensor-metric-value">{m.to_string()}</span>
                    <span class="sensor-metric-unit">"%"</span>
                </div>
            })}
            {light.map(|l| view! {
                <div class="sensor-metric">
                    <span class="sensor-metric-label">"Light"</span>
                    <span class="sensor-metric-value">{l.to_string()}</span>
                    <span class="sensor-metric-unit">"lux"</span>
                </div>
            })}
            {conductivity.map(|c| view! {
                <div class="sensor-metric">
                    <span class="sensor-metric-label">"Conductivity"</span>
                    <span class="sensor-metric-value">{c.to_string()}</span>
                    <span class="sensor-metric-unit">"µS/cm"</span>
                </div>
            })}
        </div>
    }
}

/// Format a timestamp as a short relative time string (e.g. "2d ago", "10min ago").
pub(crate) fn format_relative_time(timestamp: DateTime<Utc>) -> String {
    let delta = Utc::now() - timestamp;
    let secs = delta.num_seconds();

    if secs < 0 {
        return "just now".to_owned();
    }

    let mins = delta.num_minutes();
    let hours = delta.num_hours();
    let days = delta.num_days();

    if secs < 60 {
        format!("{secs}s ago")
    } else if mins < 60 {
        format!("{mins}min ago")
    } else if hours < 24 {
        format!("{hours}h ago")
    } else {
        format!("{days}d ago")
    }
}

/// Return a CSS class for the battery level indicator.
fn battery_class(level: i64) -> &'static str {
    if level > 60 {
        "sensor-battery-good"
    } else if level > 20 {
        "sensor-battery-medium"
    } else {
        "sensor-battery-low"
    }
}

/// Return a Unicode battery icon matching the charge level.
///
/// Uses the standard battery emoji (U+1F50B) for good/medium levels
/// and the low-battery emoji (U+1FAAB) for low levels.
fn battery_icon(level: i64) -> &'static str {
    if level > 20 { "\u{1F50B}" } else { "\u{1FAAB}" }
}

/// A classified sensor entity with its kind, ready for rendering.
#[derive(Debug, Clone)]
pub struct SensorEntry {
    /// The entity to display.
    pub entity: Entity,
    /// The detected sensor kind.
    pub kind: SensorKind,
}

/// Filter entities to only those matching a known sensor kind.
#[must_use]
pub fn filter_sensor_entities(entities: Vec<Entity>) -> Vec<SensorEntry> {
    entities
        .into_iter()
        .filter_map(|entity| {
            let kind = SensorKind::detect(&entity)?;
            Some(SensorEntry { entity, kind })
        })
        .collect()
}

/// A grid of sensor cards populated from a signal of entities.
///
/// Only entities that match a known [`SensorKind`] are rendered.
/// Unrecognised entities are silently skipped. Uses keyed `<For>`
/// for efficient DOM updates on SSE-driven refreshes.
#[component]
pub fn SensorCardGrid(
    /// Reactive signal providing the entity list.
    entities: ReadSignal<Vec<Entity>>,
) -> impl IntoView {
    let sensor_entries = move || filter_sensor_entities(entities.get());

    view! {
        <Show
            when=move || !sensor_entries().is_empty()
            fallback=|| view! { <p class="hint">"No BLE sensors found."</p> }
        >
            <div class="sensor-card-grid">
                <For
                    each=sensor_entries
                    key=|entry| entry.entity.id
                    let(entry)
                >
                    <SensorCard entity=entry.entity kind=entry.kind/>
                </For>
            </div>
        </Show>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use minihub_domain::entity::{Entity, EntityState};

    fn ble_temp_entity() -> Entity {
        Entity::builder()
            .entity_id("sensor.ble_a4c1385b0edf")
            .friendly_name("BLE Temp/Humidity A4:C1:38:5B:0E:DF")
            .state(EntityState::On)
            .attribute("temperature", AttributeValue::Float(23.1))
            .attribute("humidity", AttributeValue::Float(45.0))
            .attribute("battery_level", AttributeValue::Int(87))
            .attribute("battery_voltage", AttributeValue::Float(3.05))
            .build()
            .unwrap()
    }

    fn miflora_entity() -> Entity {
        Entity::builder()
            .entity_id("sensor.miflora_c47c8d6a1234")
            .friendly_name("Mi Flora C4:7C:8D:6A:12:34")
            .state(EntityState::On)
            .attribute("temperature", AttributeValue::Float(20.1))
            .attribute("light", AttributeValue::Int(82_386))
            .attribute("moisture", AttributeValue::Int(56))
            .attribute("conductivity", AttributeValue::Int(1561))
            .attribute("battery_level", AttributeValue::Int(99))
            .attribute("firmware", AttributeValue::String("3.1.8".to_owned()))
            .build()
            .unwrap()
    }

    fn light_entity() -> Entity {
        Entity::builder()
            .entity_id("light.living_room")
            .friendly_name("Living Room Light")
            .state(EntityState::On)
            .build()
            .unwrap()
    }

    #[test]
    fn should_detect_temp_humidity_when_entity_id_starts_with_sensor_ble() {
        let entity = ble_temp_entity();
        assert_eq!(SensorKind::detect(&entity), Some(SensorKind::TempHumidity));
    }

    #[test]
    fn should_detect_miflora_when_entity_id_starts_with_sensor_miflora() {
        let entity = miflora_entity();
        assert_eq!(SensorKind::detect(&entity), Some(SensorKind::MiFlora));
    }

    #[test]
    fn should_return_none_when_entity_is_not_a_sensor() {
        let entity = light_entity();
        assert_eq!(SensorKind::detect(&entity), None);
    }

    #[test]
    fn should_extract_float_attribute_when_present() {
        let entity = ble_temp_entity();
        assert_eq!(float_attr(&entity, "temperature"), Some(23.1));
    }

    #[test]
    fn should_extract_float_from_int_attribute() {
        let entity = miflora_entity();
        assert_eq!(float_attr(&entity, "moisture"), Some(56.0));
    }

    #[test]
    fn should_return_none_when_float_attribute_missing() {
        let entity = light_entity();
        assert_eq!(float_attr(&entity, "temperature"), None);
    }

    #[test]
    fn should_extract_int_attribute_when_present() {
        let entity = miflora_entity();
        assert_eq!(int_attr(&entity, "battery_level"), Some(99));
    }

    #[test]
    fn should_extract_int_from_float_attribute() {
        let entity = ble_temp_entity();
        assert_eq!(int_attr(&entity, "temperature"), Some(23));
    }

    #[test]
    fn should_return_none_when_int_attribute_missing() {
        let entity = light_entity();
        assert_eq!(int_attr(&entity, "battery_level"), None);
    }

    #[test]
    fn should_return_none_when_attribute_is_string() {
        let entity = miflora_entity();
        assert_eq!(float_attr(&entity, "firmware"), None);
        assert_eq!(int_attr(&entity, "firmware"), None);
    }

    #[test]
    fn should_return_good_class_when_battery_above_60() {
        assert_eq!(battery_class(100), "sensor-battery-good");
        assert_eq!(battery_class(61), "sensor-battery-good");
    }

    #[test]
    fn should_return_medium_class_when_battery_between_21_and_60() {
        assert_eq!(battery_class(60), "sensor-battery-medium");
        assert_eq!(battery_class(21), "sensor-battery-medium");
    }

    #[test]
    fn should_return_low_class_when_battery_at_or_below_20() {
        assert_eq!(battery_class(20), "sensor-battery-low");
        assert_eq!(battery_class(0), "sensor-battery-low");
    }

    #[test]
    fn should_format_float_to_one_decimal() {
        assert_eq!(fmt_f1(23.1), "23.1");
        assert_eq!(fmt_f1(20.16), "20.2");
        assert_eq!(fmt_f1(-3.0), "-3.0");
    }

    #[test]
    fn should_return_correct_label_for_temp_humidity() {
        assert_eq!(SensorKind::TempHumidity.label(), "Temp / Humidity");
    }

    #[test]
    fn should_return_correct_label_for_miflora() {
        assert_eq!(SensorKind::MiFlora.label(), "Mi Flora");
    }

    #[test]
    fn should_filter_only_sensor_entities_when_mixed_list_provided() {
        let entities = vec![ble_temp_entity(), light_entity(), miflora_entity()];
        let entries = filter_sensor_entities(entities);

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].kind, SensorKind::TempHumidity);
        assert_eq!(entries[1].kind, SensorKind::MiFlora);
    }

    #[test]
    fn should_return_empty_vec_when_no_sensors_in_list() {
        let entities = vec![light_entity()];
        let entries = filter_sensor_entities(entities);

        assert!(entries.is_empty());
    }

    #[test]
    fn should_return_empty_vec_when_entity_list_is_empty() {
        let entries = filter_sensor_entities(Vec::new());
        assert!(entries.is_empty());
    }

    #[test]
    fn should_return_full_battery_icon_when_level_above_20() {
        assert_eq!(battery_icon(100), "\u{1F50B}");
        assert_eq!(battery_icon(21), "\u{1F50B}");
    }

    #[test]
    fn should_return_low_battery_icon_when_level_at_or_below_20() {
        assert_eq!(battery_icon(20), "\u{1FAAB}");
        assert_eq!(battery_icon(0), "\u{1FAAB}");
    }

    #[test]
    fn should_format_seconds_ago() {
        let ts = Utc::now() - chrono::Duration::seconds(30);
        assert_eq!(format_relative_time(ts), "30s ago");
    }

    #[test]
    fn should_format_minutes_ago() {
        let ts = Utc::now() - chrono::Duration::minutes(10);
        assert_eq!(format_relative_time(ts), "10min ago");
    }

    #[test]
    fn should_format_hours_ago() {
        let ts = Utc::now() - chrono::Duration::hours(3);
        assert_eq!(format_relative_time(ts), "3h ago");
    }

    #[test]
    fn should_format_days_ago() {
        let ts = Utc::now() - chrono::Duration::days(2);
        assert_eq!(format_relative_time(ts), "2d ago");
    }

    #[test]
    fn should_format_just_now_when_timestamp_is_in_the_future() {
        let ts = Utc::now() + chrono::Duration::minutes(5);
        assert_eq!(format_relative_time(ts), "just now");
    }

    #[test]
    fn should_preserve_entity_data_in_sensor_entry() {
        let entity = ble_temp_entity();
        let entity_id = entity.entity_id.clone();
        let entries = filter_sensor_entities(vec![entity]);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entity.entity_id, entity_id);
        assert_eq!(entries[0].kind, SensorKind::TempHumidity);
    }
}
