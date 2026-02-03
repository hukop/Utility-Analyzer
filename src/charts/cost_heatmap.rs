use crate::data::ElectricData;
use egui::Ui;
use crate::charts::HeatmapState;
use crate::charts::heatmap_base::{render_heatmap_component, HeatmapConfig};

pub fn render_cost_heatmap(ui: &mut Ui, data: &ElectricData, state: &mut HeatmapState) {
    let (dates, heatmap_data) = data.daily_hour_cost_heatmap();

    let config = HeatmapConfig {
        title: "Daily Cost ($) Heatmap: Day (rows) vs Hour (columns)".to_string(),
        unit: "$".to_string(),
        selection_label: "Click and drag to select a range to view total Cost".to_string(),
        show_weekend_emphasis: true,
        x_label_interval: 1,
        y_label_width: 100.0,
        cell_height: 25.0,
        monthly_sums: &data.monthly_cost_sums,
        yearly_sums: &data.yearly_cost_sums,
        daily_sum_width: 80.0,
        max_value_override: Some(2.0),
    };

    render_heatmap_component(ui, &dates, &heatmap_data, state, config);
}
