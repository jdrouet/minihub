//! Sensor history chart component using `leptos-chartistry` with SVG rendering.

use chrono::{DateTime, Duration, Utc};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_chartistry::*;
use minihub_domain::entity::AttributeValue;
use minihub_domain::entity_history::EntityHistory;

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

/// A single data point for a chart series.
#[derive(Clone)]
struct ChartPoint {
    timestamp: DateTime<Utc>,
    value: f64,
}

/// Extract numeric attribute values from history records for the given key.
fn extract_series(history: &[EntityHistory], attr_key: &str) -> Vec<ChartPoint> {
    history
        .iter()
        .filter_map(|record| {
            let value = record.get_attribute(attr_key)?;
            let num = match value {
                AttributeValue::Float(v) => *v,
                AttributeValue::Int(v) => *v as f64,
                _ => return None,
            };
            Some(ChartPoint {
                timestamp: record.recorded_at,
                value: num,
            })
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

/// Build timestamp tick labels (extracted to avoid turbofish inside `view!` macro).
fn timestamp_ticks() -> TickLabels<DateTime<Utc>> {
    TickLabels::timestamps()
}

/// Render a single attribute chart.
#[component]
fn AttributeChart(
    name: String,
    data: Signal<Vec<ChartPoint>>,
) -> impl IntoView {
    let label = name.clone();
    let series = Series::new(|p: &ChartPoint| p.timestamp)
        .line(Line::new(|p: &ChartPoint| p.value).with_name(name));
    let inner = vec![
        AxisMarker::left_edge().into_inner(),
        AxisMarker::bottom_edge().into_inner(),
        XGridLine::default().into_inner(),
        YGridLine::default().into_inner(),
        XGuideLine::over_data().into_inner(),
        YGuideLine::over_mouse().into_inner(),
    ];
    view! {
        <div class="attribute-chart">
            <h4>{label}</h4>
            <Chart
                aspect_ratio=AspectRatio::from_env_width_apply_ratio(3.0)
                left=TickLabels::aligned_floats()
                bottom=timestamp_ticks()
                inner=inner
                tooltip=Tooltip::left_cursor()
                series=series
                data=data
            />
        </div>
    }
}

/// Sensor history chart component.
///
/// Fetches entity history from the API and renders interactive SVG line charts
/// using `leptos-chartistry`. One chart per numeric attribute, each full-width
/// and responsive. Provides a time range selector (1h, 6h, 24h, 7d).
#[component]
pub fn HistoryChart(entity_id: ReadSignal<String>) -> impl IntoView {
    let (range, set_range) = signal(TimeRange::TwentyFourHours);
    let (chart_error, set_chart_error) = signal(None::<String>);
    let (loading, set_loading) = signal(false);
    let (series_list, set_series_list) =
        signal(Vec::<(String, Vec<ChartPoint>)>::new());

    Effect::new(move |_| {
        let eid = entity_id.get();
        let selected_range = range.get();
        if eid.is_empty() {
            return;
        }

        set_loading.set(true);
        set_chart_error.set(None);
        set_series_list.set(Vec::new());

        let now = Utc::now();
        let from = (now - selected_range.duration()).to_rfc3339();

        spawn_local(async move {
            match fetch_entity_history(&eid, Some(&from), None).await {
                Ok(history) => {
                    let attr_keys = discover_numeric_attributes(&history);
                    let series: Vec<(String, Vec<ChartPoint>)> = attr_keys
                        .into_iter()
                        .map(|key| {
                            let points = extract_series(&history, &key);
                            (key, points)
                        })
                        .filter(|(_, points)| !points.is_empty())
                        .collect();
                    set_series_list.set(series);
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

            <Show when=move || loading.get()>
                <p>"Loading history..."</p>
            </Show>
            <Show when=move || chart_error.get().is_some()>
                <p class="error">"Chart error: " {move || chart_error.get().unwrap_or_default()}</p>
            </Show>
            <Show when=move || !loading.get() && chart_error.get().is_none() && series_list.get().is_empty()>
                <p><em>"No numeric sensor data available for this entity."</em></p>
            </Show>
            <For
                each=move || series_list.get()
                key=|(name, _)| name.clone()
                let((name, points))
            >
                <AttributeChart name=name data=Signal::derive(move || points.clone()) />
            </For>
        </div>
    }
}
