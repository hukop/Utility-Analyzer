use crate::data::ElectricData;
use egui::Ui;
use egui_plot::{Line, Plot, PlotPoints};

pub fn render_hourly_profile(ui: &mut Ui, data: &ElectricData) {
    let profile = data.hourly_profile();
    
    // Convert to plot points
    let points: PlotPoints = profile
        .iter()
        .enumerate()
        .map(|(hour, kwh)| [hour as f64, *kwh])
        .collect();
    
    let line = Line::new(points)
        .color(egui::Color32::from_rgb(44, 160, 44))
        .width(2.0)
        .name("Average kWh");
    
    Plot::new("hourly_profile_plot")
        .view_aspect(2.5)
        .legend(egui_plot::Legend::default())
        .show(ui, |plot_ui| {
            plot_ui.line(line);
        });
}
