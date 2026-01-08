use crate::data::ElectricData;
use chrono::{DateTime, Utc};
use egui::Ui;
use egui_plot::{Line, Plot, PlotPoints};

pub fn render_daily_kwh(ui: &mut Ui, data: &ElectricData) {
    ui.add_space(crate::ui::styles::CHART_SPACING);
    let daily = data.daily_totals();
    
    if daily.is_empty() {
        ui.label("No data available");
        return;
    }
    
    // Convert to plot points
    let points: PlotPoints = daily
        .iter()
        .map(|(dt, kwh)| [dt.timestamp() as f64, *kwh])
        .collect();
    
    let line = Line::new(points)
        .color(crate::ui::styles::primary_chart_color())
        .width(2.0)
        .name("Daily kWh");
    
    // Calculate 7-day rolling average
    let smoothed = calculate_rolling_average(&daily, 7);
    let smooth_points: PlotPoints = smoothed
        .iter()
        .map(|(dt, kwh)| [dt.timestamp() as f64, *kwh])
        .collect();
    
    let smooth_line = Line::new(smooth_points)
        .color(crate::ui::styles::average_chart_color())
        .width(2.0)
        .style(egui_plot::LineStyle::Dashed { length: 10.0 })
        .name("7-day average");
    
    Plot::new("daily_kwh_plot")
        .view_aspect(2.5)
        .legend(egui_plot::Legend::default())
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
}

fn calculate_rolling_average(
    data: &[(DateTime<Utc>, f64)],
    window: usize,
) -> Vec<(DateTime<Utc>, f64)> {
    if data.len() < window {
        return data.to_vec();
    }
    
    let mut result = Vec::new();
    let half_window = window / 2;
    
    for i in 0..data.len() {
        let start = i.saturating_sub(half_window);
        let end = (i + half_window + 1).min(data.len());
        
        let sum: f64 = data[start..end].iter().map(|(_, v)| v).sum();
        let count = (end - start) as f64;
        let avg = sum / count;
        
        result.push((data[i].0, avg));
    }
    
    result
}
