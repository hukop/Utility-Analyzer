use crate::data::ElectricData;
use egui::Ui;
use crate::charts::HeatmapState;
use crate::charts::heatmap_base::{render_heatmap_component, HeatmapConfig};

const WEEKDAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

pub fn render_weekday_heatmap(ui: &mut Ui, data: &ElectricData, state: &mut HeatmapState) {
    let raw_data = data.weekday_hour_average();
    let dates: Vec<String> = WEEKDAYS.iter().map(|&s| s.to_string()).collect();
    
    let heatmap_data: Vec<Vec<f64>> = raw_data.iter()
        .map(|day_arr| day_arr.to_vec())
        .collect();
    
    let config = HeatmapConfig {
        title: "Average kWh by Weekday and Hour".to_string(),
        unit: "kWh".to_string(),
        selection_label: "Click and drag to select a range to view total kWh".to_string(),
        show_weekend_emphasis: false,
        x_label_interval: 2,
        y_label_width: 80.0,
        cell_height: 30.0,
    };

    render_heatmap_component(ui, &dates, &heatmap_data, state, config);
}
