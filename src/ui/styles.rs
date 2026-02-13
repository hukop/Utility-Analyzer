use egui::{Context, Visuals};

pub fn apply_custom_style(ctx: &Context, dark_mode_pref: Option<bool>) {
    ctx.style_mut(|style| {
        // Choose visuals based on preference or system settings
        let mut visuals = if let Some(dark) = dark_mode_pref {
            if dark {
                Visuals::dark()
            } else {
                Visuals::light()
            }
        } else {
            Visuals::light() // Default to light if no preference
        };

        // Apply themed colors
        if visuals.dark_mode {
            // Windows 11-inspired dark theme
            visuals.window_fill = egui::Color32::TRANSPARENT; // Required for rounded viewport corners
            visuals.panel_fill = egui::Color32::from_rgb(28, 28, 28);
            visuals.extreme_bg_color = egui::Color32::from_rgb(24, 24, 24);
            visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(45, 45, 45);
            visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(50, 50, 50);
            visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(60, 60, 60);
            visuals.widgets.active.bg_fill = theme_accent();
        } else {
            // Windows 11-inspired light theme
            visuals.window_fill = egui::Color32::TRANSPARENT; // Required for rounded viewport corners
            visuals.panel_fill = panel_bg();
            visuals.extreme_bg_color = egui::Color32::WHITE;
            visuals.widgets.noninteractive.bg_fill = widget_bg();
            visuals.widgets.inactive.bg_fill = widget_inactive();
            visuals.widgets.hovered.bg_fill = widget_hovered();
            visuals.widgets.active.bg_fill = theme_accent();
        };

        // Core accent color
        visuals.selection.bg_fill = theme_accent();
        visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
        visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);

        // Rounded corners (modern feel)
        visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(4);
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(4);
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(4);
        visuals.widgets.active.corner_radius = egui::CornerRadius::same(4);

        style.visuals = visuals;
    });
}

pub const CHART_SPACING: f32 = 20.0;
pub const WINDOW_ROUNDING: f32 = 12.0;

// Sidebar & Layout Tokens
pub const SIDEBAR_SECTION_SIZE: f32 = 15.0;
pub const MONTH_LABEL_OFFSET: f32 = 10.0;
pub const MONTH_TOGGLE_OFFSET: f32 = 75.0;
pub const MONTH_HEADER_HEIGHT: f32 = 25.0;
pub const YEAR_HEADER_HEIGHT: f32 = 30.0;
pub const MONTH_HEADER_FONT_SIZE: f32 = 14.0;
pub const YEAR_HEADER_FONT_SIZE: f32 = 16.0;
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

/// Returns the actual intended window background color for themes.
pub fn actual_window_background(ctx: &egui::Context) -> egui::Color32 {
    if ctx.style().visuals.dark_mode {
        egui::Color32::from_rgb(32, 32, 32)
    } else {
        window_bg()
    }
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

pub fn weekend_bg(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgba_unmultiplied(0, 120, 212, 30) // Slightly more prominent in dark mode
    } else {
        egui::Color32::from_rgba_unmultiplied(0, 120, 212, 15)
    }
}

pub fn weekend_text(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgb(100, 180, 255)
    } else {
        egui::Color32::from_rgb(0, 100, 200)
    }
}
