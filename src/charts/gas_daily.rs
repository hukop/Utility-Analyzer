use crate::data::GasData;
use egui::Ui;
use egui_plot::{Line, PlotPoints};
use crate::charts::render_zoomable_daily_chart;

pub fn render_gas_daily(ui: &mut Ui, data: &GasData, state: &mut crate::charts::ChartZoomState) {
    ui.add_space(crate::ui::styles::CHART_SPACING);
    let daily = data.daily_totals();

    if daily.is_empty() {
        ui.label("No data available");
        return;
    }

    let min_ts = daily.first().unwrap().0.timestamp() as f64;
    let max_ts = daily.last().unwrap().0.timestamp() as f64;

    let points: PlotPoints = daily.iter()
        .map(|(dt, cost)| [dt.timestamp() as f64, *cost])
        .collect();

    let line = Line::new("Daily Cost ($)", points)
        .color(crate::ui::styles::primary_chart_color())
        .width(2.0);

    let smoothed = crate::charts::calculate_rolling_average(&daily, 7);
    let smooth_points: PlotPoints = smoothed.iter()
        .map(|(dt, cost)| [dt.timestamp() as f64, *cost])
        .collect();

    let smooth_line = Line::new("7-day average", smooth_points)
        .color(crate::ui::styles::average_chart_color())
        .width(2.0)
        .style(egui_plot::LineStyle::Dashed { length: 10.0 });

    render_zoomable_daily_chart(
        ui,
        state,
        "gas_daily",
        (min_ts, max_ts),
        vec![line, smooth_line]
    );
}
