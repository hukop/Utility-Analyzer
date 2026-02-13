use crate::charts::heatmap_base::{render_heatmap_component, HeatmapConfig};
use crate::charts::HeatmapState;
use crate::data::ElectricData;
use egui::Ui;
use std::collections::HashMap;
use std::sync::OnceLock;

const WEEKDAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

static WEEKDAY_LABELS: OnceLock<Vec<String>> = OnceLock::new();
static EMPTY_SUMS: OnceLock<HashMap<String, f64>> = OnceLock::new();

fn weekday_labels() -> &'static [String] {
    WEEKDAY_LABELS.get_or_init(|| WEEKDAYS.iter().map(|s| (*s).to_string()).collect())
}

fn empty_sums() -> &'static HashMap<String, f64> {
    EMPTY_SUMS.get_or_init(HashMap::new)
}

pub fn render_weekday_heatmap(ui: &mut Ui, data: &ElectricData, state: &mut HeatmapState) {
    let dates = weekday_labels();
    let heatmap_data = data.weekday_hour_heatmap_cached();

    let config = HeatmapConfig {
        id: "weekday_heatmap",
        title: "Average kWh by Weekday and Hour",
        show_title: true,
        unit: "kWh",
        selection_label: "Click and drag to select a range to view total kWh",
        show_legend: true,
        show_weekend_emphasis: false,
        x_label_interval: 2,
        y_label_width: 80.0,
        cell_height: 30.0,
        monthly_sums: empty_sums(),
        yearly_sums: empty_sums(),
        daily_sum_width: 0.0,
        max_value_override: Some(data.weekday_hour_heatmap_max_cached()),
        daily_sums: None,
        date_meta: None,
    };

    render_heatmap_component(ui, dates, heatmap_data, state, config);
}
