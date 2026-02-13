use crate::charts::heatmap_base::{render_heatmap_component, HeatmapConfig};
use crate::charts::HeatmapState;
use crate::data::ElectricData;
use egui::Ui;
use crate::ui::HeatmapMetric;

fn render_heatmap_toggle_buttons(
    ui: &mut Ui,
    current_metric: HeatmapMetric,
    mut on_toggle: impl FnMut(HeatmapMetric),
    right_margin: f32,
    top_offset: f32,
) {
    let button_size = egui::vec2(60.0, 28.0);
    let overlap = 10.0;
    let toggle_width = button_size.x * 2.0 - overlap;
    let left_space = (ui.available_width() - right_margin - toggle_width).max(0.0);
    if left_space > 0.0 {
        ui.add_space(left_space);
    }

    let (toggle_rect_alloc, _) =
        ui.allocate_exact_size(egui::vec2(toggle_width, button_size.y), egui::Sense::hover());
    let toggle_rect = toggle_rect_alloc.translate(egui::vec2(0.0, top_offset));

    let energy_rect = egui::Rect::from_min_size(toggle_rect.min, button_size);
    let cost_rect = egui::Rect::from_min_size(
        egui::pos2(toggle_rect.min.x + button_size.x - overlap, toggle_rect.min.y),
        button_size,
    );

    let draw_energy = |ui: &mut egui::Ui, selected: bool| {
        ui.put(
            energy_rect,
            egui::Button::new("Energy")
                .selected(selected)
                .corner_radius(if selected {
                    egui::CornerRadius::same(16)
                } else {
                    egui::CornerRadius {
                        nw: 16,
                        ne: 0,
                        sw: 16,
                        se: 0,
                    }
                })
                .min_size(button_size),
        )
    };

    let draw_cost = |ui: &mut egui::Ui, selected: bool| {
        ui.put(
            cost_rect,
            egui::Button::new("Cost")
                .selected(selected)
                .corner_radius(if selected {
                    egui::CornerRadius::same(16)
                } else {
                    egui::CornerRadius {
                        nw: 0,
                        ne: 16,
                        sw: 0,
                        se: 16,
                    }
                })
                .min_size(button_size),
        )
    };

    let is_energy_selected = current_metric == HeatmapMetric::Energy;
    let (energy_response, cost_response) = if is_energy_selected {
        let cost_response = draw_cost(ui, false);
        let energy_response = draw_energy(ui, true);
        (energy_response, cost_response)
    } else {
        let energy_response = draw_energy(ui, false);
        let cost_response = draw_cost(ui, true);
        (energy_response, cost_response)
    };

    if energy_response.clicked() {
        on_toggle(HeatmapMetric::Energy);
    }
    if cost_response.clicked() {
        on_toggle(HeatmapMetric::Cost);
    }
}

pub fn render_daily_heatmap_with_toggle(
    ui: &mut Ui,
    data: &ElectricData,
    state: &mut HeatmapState,
    metric: &mut HeatmapMetric,
) {
    let title = match *metric {
        HeatmapMetric::Energy => "Daily kWh Heatmap: Day (rows) vs Hour (columns)",
        HeatmapMetric::Cost => "Daily Cost ($) Heatmap: Day (rows) vs Hour (columns)",
    };
    let selection_text = match *metric {
        HeatmapMetric::Energy => "Click and drag to select a range to view total kWh",
        HeatmapMetric::Cost => "Click and drag to select a range to view total Cost",
    };
    let right_margin = 100.0;
    let toggle_top_offset = -10.0;

    ui.heading(title);
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(selection_text);
        render_heatmap_toggle_buttons(
            ui,
            *metric,
            |new_metric| *metric = new_metric,
            right_margin,
            toggle_top_offset,
        );
    });
    ui.add_space(4.0);

    match *metric {
        HeatmapMetric::Energy => render_daily_heatmap(ui, data, state),
        HeatmapMetric::Cost => crate::charts::render_cost_heatmap(ui, data, state),
    }
}

pub fn render_daily_heatmap(ui: &mut Ui, data: &ElectricData, state: &mut HeatmapState) {
    render_daily_heatmap_with_selection_label(ui, data, state, "");
}

fn render_daily_heatmap_with_selection_label(
    ui: &mut Ui,
    data: &ElectricData,
    state: &mut HeatmapState,
    selection_label: &'static str,
) {
    let (dates, heatmap_data) = data.daily_hour_heatmap_cached();

    let config = HeatmapConfig {
        id: "daily_energy_heatmap",
        title: "Daily kWh Heatmap: Day (rows) vs Hour (columns)",
        show_title: false,
        unit: "kWh",
        selection_label,
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
