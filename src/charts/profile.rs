use crate::data::ElectricData;
use egui::Ui;
use egui_plot::{GridMark, Line, Plot, PlotPoints};

pub fn render_hourly_profile(ui: &mut Ui, data: &ElectricData) {
    ui.add_space(crate::ui::styles::CHART_SPACING);
    let profile = data.hourly_profile_cached();
    let export_profile = data.hourly_export_profile_cached();

    // Convert Points (All points)
    let points: PlotPoints = profile
        .iter()
        .enumerate()
        .map(|(hour, kwh)| [hour as f64, *kwh])
        .collect();

    let line = Line::new("Average Usage (kWh)", points)
        .color(egui::Color32::from_rgb(31, 119, 180))
        .width(2.0);

    let export_points: PlotPoints = export_profile
        .iter()
        .enumerate()
        .map(|(hour, kwh)| [hour as f64, *kwh])
        .collect();

    let export_line = Line::new("Average Export (kWh)", export_points)
        .color(egui::Color32::from_rgb(220, 180, 0))
        .width(2.0);

    Plot::new("hourly_profile_plot")
        .view_aspect(2.0)
        .set_margin_fraction(egui::vec2(0.02, 0.1))
        .allow_drag(false)
        .allow_zoom(false)
        .allow_scroll(false)
        .allow_double_click_reset(false)
        .legend(egui_plot::Legend::default())
        .show_grid([false, true])
        .include_x(0.0)
        .include_x(23.0)
        .x_axis_formatter(|x, _range| {
            let hr = x.value.round() as i32;
            if (x.value - hr as f64).abs() < 0.1 && (0..=23).contains(&hr) {
                format!("{}", hr)
            } else {
                "".to_string()
            }
        })
        .x_grid_spacer(|_input| {
            let mut marks: Vec<GridMark> = vec![];
            for i in 0..=23 {
                marks.push(GridMark {
                    value: i as f64,
                    step_size: 24.0,
                });
            }
            marks
        })
        .show(ui, |plot_ui| {
            // Re-drawing dashed grid lines (manually for all hours)
            let max_val = profile
                .iter()
                .chain(export_profile.iter())
                .copied()
                .fold(0.0, f64::max)
                .max(0.1);
            let y_limit = max_val * 1.2;

            for i in 0..=23 {
                let x = i as f64;
                plot_ui.line(
                    egui_plot::Line::new("", vec![[x, 0.0], [x, y_limit]])
                        .style(egui_plot::LineStyle::Dashed { length: 5.0 })
                        .color(egui::Color32::from_gray(100))
                        .width(1.0),
                );
            }

            plot_ui.line(line);
            plot_ui.line(export_line);
        });
}
