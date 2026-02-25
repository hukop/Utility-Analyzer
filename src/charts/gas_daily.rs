use crate::charts::render_zoomable_daily_chart;
use crate::data::GasData;
use egui::Ui;
use egui_plot::Line;

pub fn render_gas_daily(
    ui: &mut Ui,
    data: &GasData,
    state: &mut crate::charts::ChartZoomState,
    preset: crate::data::DateRangePreset,
) {
    ui.add_space(crate::ui::styles::CHART_SPACING);

    let (points, avg7_points, bounds) = data.daily_plot_points_filtered(preset);
    let Some((min_ts, max_ts)) = bounds else {
        ui.label("No data available");
        return;
    };

    let line = Line::new("Daily Cost ($)", points)
        .color(crate::ui::styles::primary_chart_color())
        .width(2.0);

    let smooth_line = Line::new("7-day average", avg7_points)
        .color(crate::ui::styles::average_chart_color())
        .width(2.0)
        .style(egui_plot::LineStyle::Dashed { length: 10.0 });

    render_zoomable_daily_chart(
        ui,
        state,
        "gas_daily",
        (min_ts, max_ts),
        Some(preset),
        [line, smooth_line],
    );
}
