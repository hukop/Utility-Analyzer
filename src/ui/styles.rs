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
    
    // Rounded corners
    visuals.widgets.noninteractive.rounding = egui::Rounding::same(4.0);
    visuals.widgets.inactive.rounding = egui::Rounding::same(4.0);
    visuals.widgets.hovered.rounding = egui::Rounding::same(4.0);
    visuals.widgets.active.rounding = egui::Rounding::same(4.0);
    
    style.visuals = visuals;
    ctx.set_style(style);
}
