use crate::data::ElectricData;
use egui::Ui;
use egui_plot::{Line, Plot, PlotPoints, GridMark};

pub fn render_hourly_profile(ui: &mut Ui, data: &ElectricData) {
    ui.add_space(crate::ui::styles::CHART_SPACING);
    let profile = data.hourly_profile();
    let export_profile = data.hourly_export_profile();

    // Convert to plot points for usage
    let points: PlotPoints = profile
        .iter()
        .enumerate()
        .map(|(hour, kwh)| [hour as f64, *kwh])
        .collect();

    let line = Line::new(points)
        .color(egui::Color32::from_rgb(31, 119, 180)) // Standard blue
        .width(2.0)
        .name("Average Usage (kWh)");

    // Convert to plot points for export
    let export_points: PlotPoints = export_profile
        .iter()
        .enumerate()
        .map(|(hour, kwh)| [hour as f64, *kwh])
        .collect();

    let export_line = Line::new(export_points)
        .color(egui::Color32::from_rgb(220, 180, 0)) // Solar yellow
        .width(2.0)
        .name("Average Export (kWh)");

    Plot::new("hourly_profile_plot")
        .view_aspect(2.0)
        .include_x(0.0)
        .include_x(23.0)
        .set_margin_fraction(egui::vec2(0.02, 0.1))
        .allow_drag(false)
        .allow_zoom(false)
        .allow_scroll(false)
        .allow_double_click_reset(false)
        .legend(egui_plot::Legend::default())
        .show_grid([false, true]) // Hide default vertical grid lines (solid)
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
                // Use major step_size to ensure labels appear
                marks.push(GridMark {
                    value: i as f64,
                    step_size: 24.0,
                });
            }
            marks
        })
        .show(ui, |plot_ui| {
            // Manually draw dashed vertical grid lines
            for i in 0..=23 {
                plot_ui.vline(
                    egui_plot::VLine::new(i as f64)
                        .style(egui_plot::LineStyle::Dashed{length: 5.0})
                        .color(egui::Color32::from_gray(200)), // Subtle grid color
                );
            }
            plot_ui.line(line);
            plot_ui.line(export_line);
        });
}
