use crate::data::GasData;
use chrono::DateTime;
use egui::{Ui, ScrollArea};
use egui_plot::{Line, Plot, PlotPoints};

pub fn render_gas_daily(ui: &mut Ui, data: &GasData) {
    ui.add_space(crate::ui::styles::CHART_SPACING);
    let daily = data.daily_totals();

    if daily.is_empty() {
        ui.label("No data available");
        return;
    }

    // Convert to plot points
    let points: PlotPoints = daily
        .iter()
        .map(|(dt, cost)| [dt.timestamp() as f64, *cost])
        .collect();

    let line = Line::new("Daily Cost ($)", points)
        .color(crate::ui::styles::primary_chart_color())
        .width(2.0);

    // Calculate 7-day rolling average
    let smoothed = crate::charts::calculate_rolling_average(&daily, 7);
    let smooth_points: PlotPoints = smoothed
        .iter()
        .map(|(dt, cost)| [dt.timestamp() as f64, *cost])
        .collect();

    let smooth_line = Line::new("7-day average", smooth_points)
        .color(crate::ui::styles::average_chart_color())
        .width(2.0)
        .style(egui_plot::LineStyle::Dashed { length: 10.0 });

    let first_timestamp = daily.first().map(|(dt, _)| dt.timestamp() as f64).unwrap_or(0.0);
    let last_timestamp = daily.last().map(|(dt, _)| dt.timestamp() as f64).unwrap_or(1.0);

    ScrollArea::both()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            Plot::new("gas_daily_plot")
                .view_aspect(2.5)
                .legend(egui_plot::Legend::default())
                .allow_zoom(true)
                .allow_drag(false)
                .include_x(first_timestamp)
                .include_x(last_timestamp)
                .x_axis_formatter(|x, _range| {
                    let timestamp = x.value as i64;
                    if let Some(dt) = DateTime::from_timestamp(timestamp, 0) {
                        dt.format("%Y-%m-%d").to_string()
                    } else {
                        "".to_string()
                    }
                })
                .label_formatter(|name, value| {
                    let timestamp = value.x as i64;
                    let date_str = if let Some(dt) = DateTime::from_timestamp(timestamp, 0) {
                        dt.format("%Y-%m-%d").to_string()
                    } else {
                        "".to_string()
                    };
                    format!("{}: {:.2}\n{}", name, value.y, date_str)
                })
                .show(ui, |plot_ui| {
                    plot_ui.line(line);
                    plot_ui.line(smooth_line);
                });
        });
}
