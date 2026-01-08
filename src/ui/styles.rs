use egui::{Context, Style, Visuals};

pub fn apply_custom_style(ctx: &Context) {
    let mut style = Style::default();
    let mut visuals = Visuals::light();
    
    // Windows 11-inspired colors
    visuals.window_fill = window_bg();
    visuals.panel_fill = panel_bg();
    visuals.extreme_bg_color = egui::Color32::WHITE;
    
    // Accent color
    visuals.selection.bg_fill = theme_accent();
    visuals.selection.stroke.color = theme_accent();
    
    // Widget colors
    visuals.widgets.noninteractive.bg_fill = widget_bg();
    visuals.widgets.inactive.bg_fill = widget_inactive();
    visuals.widgets.hovered.bg_fill = widget_hovered();
    visuals.widgets.active.bg_fill = theme_accent();
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

// Sidebar & Layout Tokens
pub const SIDEBAR_SECTION_SIZE: f32 = 15.0;
pub const MONTH_LABEL_OFFSET: f32 = 10.0;
pub const MONTH_TOGGLE_OFFSET: f32 = 75.0;
pub const MONTH_HEADER_HEIGHT: f32 = 25.0;
pub const MONTH_HEADER_FONT_SIZE: f32 = 14.0;
pub const MONTH_SUMMARY_FONT_SIZE: f32 = 12.0;
pub const BODY_FONT_SIZE: f32 = 12.0;
pub const AXIS_FONT_SIZE: f32 = 12.0;

// Chart Component Tokens
pub fn primary_chart_color() -> egui::Color32 {
    egui::Color32::from_rgb(31, 119, 180)
}

pub fn average_chart_color() -> egui::Color32 {
    egui::Color32::from_rgb(255, 127, 14)
}

// Core Theme Colors
pub fn window_bg() -> egui::Color32 {
    egui::Color32::from_rgb(243, 243, 243)
}

pub fn panel_bg() -> egui::Color32 {
    egui::Color32::from_rgb(249, 249, 249)
}

pub fn theme_accent() -> egui::Color32 {
    egui::Color32::from_rgb(0, 120, 212)
}

pub fn widget_bg() -> egui::Color32 {
    egui::Color32::from_rgb(240, 240, 240)
}

pub fn widget_inactive() -> egui::Color32 {
    egui::Color32::from_rgb(251, 251, 251)
}

pub fn widget_hovered() -> egui::Color32 {
    egui::Color32::from_rgb(229, 229, 229)
}

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
