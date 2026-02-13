use crate::charts::heatmap_base::{render_heatmap_component, HeatmapConfig};
use crate::charts::HeatmapState;
use crate::data::ElectricData;
use egui::Ui;

pub fn render_daily_heatmap(ui: &mut Ui, data: &ElectricData, state: &mut HeatmapState) {
    let (dates, heatmap_data) = data.daily_hour_heatmap_cached();

    let config = HeatmapConfig {
        id: "daily_energy_heatmap",
        title: "Daily kWh Heatmap: Day (rows) vs Hour (columns)",
        unit: "kWh",
        selection_label: "Click and drag to select a range to view total kWh",
        show_weekend_emphasis: true,
        x_label_interval: 1,
        y_label_width: 100.0,
        cell_height: 25.0,
        monthly_sums: &data.monthly_kwh_sums,
        yearly_sums: &data.yearly_kwh_sums,
        daily_sum_width: 80.0,
        max_value_override: Some(6.0),
        daily_sums: Some(data.daily_hour_heatmap_row_sums_cached()),
        date_meta: Some(data.daily_hour_heatmap_meta_cached()),
    };

    render_heatmap_component(ui, dates, heatmap_data, state, config);
}
