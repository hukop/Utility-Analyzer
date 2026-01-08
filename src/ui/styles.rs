use egui::{Context, Style, Visuals};

pub fn apply_custom_style(ctx: &Context) {
    let mut style = Style::default();
    let mut visuals = Visuals::light();
    
    // Windows 11-inspired colors
    visuals.window_fill = egui::Color32::from_rgb(243, 243, 243);
    visuals.panel_fill = egui::Color32::from_rgb(249, 249, 249);
    visuals.extreme_bg_color = egui::Color32::WHITE;
    
    // Accent color (blue)
    visuals.selection.bg_fill = egui::Color32::from_rgb(0, 120, 212);
    visuals.selection.stroke.color = egui::Color32::from_rgb(0, 120, 212);
    
    // Widget colors
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(240, 240, 240);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(251, 251, 251);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(229, 229, 229);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0, 120, 212);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    
    // Rounded corners
    visuals.widgets.noninteractive.rounding = egui::Rounding::same(4.0);
    visuals.widgets.inactive.rounding = egui::Rounding::same(4.0);
    visuals.widgets.hovered.rounding = egui::Rounding::same(4.0);
    visuals.widgets.active.rounding = egui::Rounding::same(4.0);
    
    style.visuals = visuals;
    ctx.set_style(style);
}

pub const CHART_SPACING: f32 = 20.0;

pub fn weekend_bg() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(0, 120, 212, 15)
}

pub fn weekend_text() -> egui::Color32 {
    egui::Color32::from_rgb(0, 100, 200)
}

pub fn status_green() -> egui::Color32 {
    egui::Color32::from_rgb(0, 150, 0)
}

pub fn status_red() -> egui::Color32 {
    egui::Color32::from_rgb(200, 0, 0)
}
