use crate::charts::heatmap_base::{render_heatmap_component, HeatmapConfig};
use crate::charts::HeatmapState;
use crate::data::{DateRangePreset, ElectricData};
use egui::Ui;

pub fn render_cost_heatmap(
    ui: &mut Ui,
    data: &ElectricData,
    state: &mut HeatmapState,
    range_preset: DateRangePreset,
    modern: bool,
) {
    render_cost_heatmap_with_selection_label(ui, data, state, "", range_preset, modern);
}

pub(crate) fn render_cost_heatmap_with_selection_label(
    ui: &mut Ui,
    data: &ElectricData,
    state: &mut HeatmapState,
    selection_label: &'static str,
    range_preset: DateRangePreset,
    modern: bool,
) {
    let (dates, heatmap_data, daily_sums, date_meta) =
        data.daily_hour_cost_heatmap_filtered(range_preset);

    let config = HeatmapConfig {
        id: "daily_cost_heatmap",
        title: "Daily Cost ($) Heatmap: Day (rows) vs Hour (columns)",
        show_title: false,
        unit: "$",
        selection_label,
        show_legend: false,
        show_weekend_emphasis: true,
        x_label_interval: 1,
        y_label_width: 100.0,
        cell_height: 25.0,
        monthly_sums: &data.monthly_cost_sums,
        yearly_sums: &data.yearly_cost_sums,
        daily_sum_width: 80.0,
        max_value_override: Some(2.0),
        daily_sums: Some(daily_sums),
        date_meta: Some(date_meta),
        modern,
    };

    render_heatmap_component(ui, dates, heatmap_data, state, config);
}
