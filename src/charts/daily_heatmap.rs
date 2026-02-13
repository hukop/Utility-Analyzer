use crate::charts::heatmap_base::{render_heatmap_component, HeatmapConfig};
use crate::charts::HeatmapState;
use crate::data::{DateRangePreset, ElectricData};
use crate::ui::HeatmapMetric;
use egui::Ui;

fn render_heatmap_toggle_buttons(
    ui: &mut Ui,
    current_metric: HeatmapMetric,
    mut on_toggle: impl FnMut(HeatmapMetric),
    y_offset: f32,
) {
    let button_size = egui::vec2(60.0, 28.0);
    let overlap = 10.0;
    let toggle_width = button_size.x * 2.0 - overlap;
    let (toggle_rect_alloc, _) = ui.allocate_exact_size(
        egui::vec2(toggle_width, button_size.y),
        egui::Sense::hover(),
    );
    let toggle_rect = toggle_rect_alloc.translate(egui::vec2(0.0, y_offset));

    let energy_rect = egui::Rect::from_min_size(toggle_rect.min, button_size);
    let cost_rect = egui::Rect::from_min_size(
        egui::pos2(
            toggle_rect.min.x + button_size.x - overlap,
            toggle_rect.min.y,
        ),
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

fn render_heat_legend(
    ui: &mut Ui,
    max_val: f64,
    unit: &str,
    palette: crate::charts::HeatmapPalette,
) {
    ui.label(
        egui::RichText::new(format!("0 to {:.1} {}", max_val, unit))
            .size(11.0)
            .color(ui.visuals().text_color().gamma_multiply(0.75)),
    );
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(1.0, 0.0);
        for i in 0..10 {
            let t0 = i as f64 / 10.0;
            let t1 = (i + 1) as f64 / 10.0;
            let v = (t0 + t1) * 0.5 * max_val;
            let color = crate::charts::colormap::get_heatmap_color(v, 0.0, max_val, palette);
            let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 10.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 1.0, color);
        }
    });
}

pub fn render_daily_heatmap_with_toggle(
    ui: &mut Ui,
    data: &ElectricData,
    state: &mut HeatmapState,
    metric: &mut HeatmapMetric,
    range_preset: DateRangePreset,
) {
    let title = match *metric {
        HeatmapMetric::Energy => "Daily kWh Heatmap: Day (rows) vs Hour (columns)",
        HeatmapMetric::Cost => "Daily Cost ($) Heatmap: Day (rows) vs Hour (columns)",
    };
    let selection_text = match *metric {
        HeatmapMetric::Energy => "Click and drag to select a range to view total kWh",
        HeatmapMetric::Cost => "Click and drag to select a range to view total Cost",
    };
    let (legend_max, legend_unit) = match *metric {
        HeatmapMetric::Energy => (6.0, "kWh"),
        HeatmapMetric::Cost => (2.0, "$"),
    };
    let toggle_y_offset = -8.0;

    ui.heading(title);
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(selection_text);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            render_heat_legend(ui, legend_max, legend_unit, state.palette);
            ui.add_space(12.0);
            render_heatmap_toggle_buttons(
                ui,
                *metric,
                |new_metric| *metric = new_metric,
                toggle_y_offset,
            );
        });
    });
    ui.add_space(6.0);

    match *metric {
        HeatmapMetric::Energy => render_daily_heatmap(ui, data, state, range_preset),
        HeatmapMetric::Cost => crate::charts::render_cost_heatmap(ui, data, state, range_preset),
    }
}

pub fn render_daily_heatmap(
    ui: &mut Ui,
    data: &ElectricData,
    state: &mut HeatmapState,
    range_preset: DateRangePreset,
) {
    render_daily_heatmap_with_selection_label(ui, data, state, "", range_preset);
}

fn render_daily_heatmap_with_selection_label(
    ui: &mut Ui,
    data: &ElectricData,
    state: &mut HeatmapState,
    selection_label: &'static str,
    range_preset: DateRangePreset,
) {
    let (dates, heatmap_data, daily_sums, date_meta) =
        data.daily_hour_heatmap_filtered(range_preset);

    let config = HeatmapConfig {
        id: "daily_energy_heatmap",
        title: "Daily kWh Heatmap: Day (rows) vs Hour (columns)",
        show_title: false,
        unit: "kWh",
        selection_label,
        show_legend: false,
        show_weekend_emphasis: true,
        x_label_interval: 1,
        y_label_width: 100.0,
        cell_height: 25.0,
        monthly_sums: &data.monthly_kwh_sums,
        yearly_sums: &data.yearly_kwh_sums,
        daily_sum_width: 80.0,
        max_value_override: Some(6.0),
        daily_sums: Some(daily_sums),
        date_meta: Some(date_meta),
    };

    render_heatmap_component(ui, dates, heatmap_data, state, config);
}
