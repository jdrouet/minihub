//! Sensor history chart component using `plotters` with the HTML5 canvas backend.

use chrono::{DateTime, Duration, Utc};
use leptos::prelude::*;
use leptos::task::spawn_local;
use minihub_domain::entity::AttributeValue;
use minihub_domain::entity_history::EntityHistory;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use wasm_bindgen::JsCast;

use crate::api::fetch_entity_history;

/// Available time ranges for the history chart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeRange {
    OneHour,
    SixHours,
    TwentyFourHours,
    SevenDays,
}

impl TimeRange {
    /// Return the duration for this range.
    fn duration(self) -> Duration {
        match self {
            Self::OneHour => Duration::hours(1),
            Self::SixHours => Duration::hours(6),
            Self::TwentyFourHours => Duration::hours(24),
            Self::SevenDays => Duration::days(7),
        }
    }

    /// Label shown on the selector button.
    fn label(self) -> &'static str {
        match self {
            Self::OneHour => "1h",
            Self::SixHours => "6h",
            Self::TwentyFourHours => "24h",
            Self::SevenDays => "7d",
        }
    }
}

const TIME_RANGES: [TimeRange; 4] = [
    TimeRange::OneHour,
    TimeRange::SixHours,
    TimeRange::TwentyFourHours,
    TimeRange::SevenDays,
];

/// Extract numeric attribute values from history records for the given key.
fn extract_series(history: &[EntityHistory], attr_key: &str) -> Vec<(DateTime<Utc>, f64)> {
    history
        .iter()
        .filter_map(|record| {
            let value = record.get_attribute(attr_key)?;
            let num = match value {
                AttributeValue::Float(v) => *v,
                AttributeValue::Int(v) => *v as f64,
                _ => return None,
            };
            Some((record.recorded_at, num))
        })
        .collect()
}

/// Discover numeric attribute keys present in the history data.
fn discover_numeric_attributes(history: &[EntityHistory]) -> Vec<String> {
    let mut keys = Vec::new();
    for record in history {
        for (key, value) in &record.attributes {
            if matches!(value, AttributeValue::Float(_) | AttributeValue::Int(_))
                && !keys.contains(key)
            {
                keys.push(key.clone());
            }
        }
    }
    keys.sort();
    keys
}

const CHART_WIDTH: u32 = 720;
const CHART_HEIGHT: u32 = 360;

/// A named series of timestamped data points for plotting.
type NamedSeries = (String, Vec<(DateTime<Utc>, f64)>);

const SERIES_COLORS: [RGBColor; 4] = [
    RGBColor(66, 133, 244),
    RGBColor(234, 67, 53),
    RGBColor(52, 168, 83),
    RGBColor(251, 188, 4),
];

/// Draw history series onto the given canvas element.
fn draw_chart(canvas_id: &str, history: &[EntityHistory], range: TimeRange) -> Result<(), String> {
    let attr_keys = discover_numeric_attributes(history);
    if attr_keys.is_empty() {
        return Err("No numeric attributes to chart".into());
    }

    let backend = CanvasBackend::new(canvas_id)
        .ok_or_else(|| format!("canvas element {canvas_id:?} not found"))?;
    let root = backend.into_drawing_area();
    root.fill(&WHITE).map_err(|err| format!("{err}"))?;

    let now = Utc::now();
    let start = now - range.duration();

    let mut all_values: Vec<f64> = Vec::new();
    let series_data: Vec<NamedSeries> = attr_keys
        .iter()
        .map(|key| {
            let points = extract_series(history, key);
            all_values.extend(points.iter().map(|(_, v)| *v));
            (key.clone(), points)
        })
        .collect();

    if all_values.is_empty() {
        root.draw_text(
            "No data points in range",
            &TextStyle::from(("sans-serif", 16).into_font()).color(&BLACK),
            (CHART_WIDTH as i32 / 4, CHART_HEIGHT as i32 / 2),
        )
        .map_err(|err| format!("{err}"))?;
        root.present().map_err(|err| format!("{err}"))?;
        return Ok(());
    }

    let y_min = all_values.iter().copied().fold(f64::INFINITY, f64::min);
    let y_max = all_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let y_margin = (y_max - y_min).abs() * 0.1 + 0.1;

    let mut chart = ChartBuilder::on(&root)
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(start..now, (y_min - y_margin)..(y_max + y_margin))
        .map_err(|err| format!("{err}"))?;

    chart
        .configure_mesh()
        .x_labels(6)
        .x_label_formatter(&|dt: &DateTime<Utc>| dt.format("%H:%M").to_string())
        .y_labels(6)
        .draw()
        .map_err(|err| format!("{err}"))?;

    for (idx, (key, points)) in series_data.iter().enumerate() {
        let color = SERIES_COLORS[idx % SERIES_COLORS.len()];
        chart
            .draw_series(LineSeries::new(
                points.iter().map(|(t, v)| (*t, *v)),
                color.stroke_width(2),
            ))
            .map_err(|err| format!("{err}"))?
            .label(key.as_str())
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2))
            });
    }

    if series_data.len() > 1 {
        chart
            .configure_series_labels()
            .border_style(BLACK)
            .background_style(WHITE.mix(0.8))
            .draw()
            .map_err(|err| format!("{err}"))?;
    }

    root.present().map_err(|err| format!("{err}"))?;
    Ok(())
}

/// Sensor history chart component.
///
/// Fetches entity history from the API and renders a line chart using `plotters`
/// onto an HTML5 canvas. Provides a time range selector (1h, 6h, 24h, 7d).
#[component]
pub fn HistoryChart(entity_id: ReadSignal<String>) -> impl IntoView {
    let (range, set_range) = signal(TimeRange::TwentyFourHours);
    let (chart_error, set_chart_error) = signal(None::<String>);
    let (loading, set_loading) = signal(false);
    let (has_data, set_has_data) = signal(false);

    let canvas_id = "history-chart-canvas";

    Effect::new(move |_| {
        let eid = entity_id.get();
        let selected_range = range.get();
        if eid.is_empty() {
            return;
        }

        set_loading.set(true);
        set_chart_error.set(None);

        let now = Utc::now();
        let from = (now - selected_range.duration()).to_rfc3339();

        spawn_local(async move {
            match fetch_entity_history(&eid, Some(&from), None).await {
                Ok(history) => {
                    let has_numeric = !discover_numeric_attributes(&history).is_empty();
                    set_has_data.set(has_numeric);

                    if has_numeric {
                        request_animation_frame(move || {
                            if let Err(err) = draw_chart(canvas_id, &history, selected_range) {
                                set_chart_error.set(Some(err));
                            }
                        });
                    }
                    set_loading.set(false);
                }
                Err(err) => {
                    set_chart_error.set(Some(err.message));
                    set_loading.set(false);
                }
            }
        });
    });

    view! {
        <div class="history-chart">
            <h3>"State History"</h3>

            <div class="time-range-selector">
                {TIME_RANGES
                    .into_iter()
                    .map(|tr| {
                        let is_active = move || range.get() == tr;
                        view! {
                            <button
                                class=move || {
                                    if is_active() { "btn btn-primary btn-sm" } else { "btn btn-secondary btn-sm" }
                                }
                                on:click=move |_| set_range.set(tr)
                            >
                                {tr.label()}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>

            {move || {
                if loading.get() {
                    view! { <p>"Loading history..."</p> }.into_any()
                } else if let Some(err) = chart_error.get() {
                    view! { <p class="error">"Chart error: " {err}</p> }.into_any()
                } else if !has_data.get() {
                    view! { <p><em>"No numeric sensor data available for this entity."</em></p> }
                        .into_any()
                } else {
                    view! {
                        <canvas
                            id=canvas_id
                            width=CHART_WIDTH.to_string()
                            height=CHART_HEIGHT.to_string()
                            style="max-width: 100%; border: 1px solid #ddd; border-radius: 4px;"
                        />
                    }
                        .into_any()
                }
            }}
        </div>
    }
}

/// Schedule a closure to run on the next animation frame.
///
/// This gives the browser a chance to render the canvas element into the DOM
/// before we attempt to draw on it.
fn request_animation_frame(f: impl FnOnce() + 'static) {
    let closure = wasm_bindgen::closure::Closure::once_into_js(f);
    web_sys::window()
        .expect("window should exist")
        .request_animation_frame(closure.as_ref().unchecked_ref())
        .expect("should register requestAnimationFrame");
}
