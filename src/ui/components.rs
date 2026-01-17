use egui::{Ui, Sense, Color32, Stroke, Align2, FontId, vec2};
use crate::ui::styles;

pub struct HeaderConfig<'a> {
    pub label: &'a str,
    pub width: f32,
    pub height: f32,
    pub font_size: f32,
    pub is_collapsed: bool,
    pub summary: Option<String>,
    pub show_icon: bool,
}

/// Renders a collapsible header with standardized styling (background, border, icon, label, summary).
/// Returns `true` if the header was clicked (toggle requested).
pub fn render_collapsible_header(
    ui: &mut Ui,
    id_salt: impl std::hash::Hash,
    config: HeaderConfig<'_>,
) -> bool {
    // Only allocate clicking space if width/height > 0, otherwise it might be auto-sized layout which needs different handling.
    // However, our usage is always passing explicit dimensions.
    let (rect, _) = ui.allocate_exact_size(vec2(config.width, config.height), Sense::click());

    // Interact with specific ID to allow persistent state if needed, though we use `clicked()` return.
    let response = ui.interact(rect, ui.id().with(id_salt), Sense::click());
    let clicked = response.clicked();

    // Background color
    let bg_color = if response.hovered() {
        if ui.visuals().dark_mode { Color32::from_gray(80) } else { Color32::from_gray(190) }
    } else if ui.visuals().dark_mode {
        Color32::from_gray(60)
    } else {
        Color32::from_gray(210)
    };

    // Draw background
    ui.painter().rect_filled(rect, 0.0, bg_color);

    // Draw bottom border
    ui.painter().line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color)
    );

    // Draw Icon (⏵/⏷)
    if config.show_icon {
        let icon = if config.is_collapsed { "⏵" } else { "⏷" };
        ui.painter().text(
            rect.left_center() + vec2(styles::MONTH_TOGGLE_OFFSET, 0.0),
            Align2::LEFT_CENTER,
            icon,
            FontId::monospace(config.font_size),
            ui.visuals().text_color()
        );
    }

    // Draw Label (only if provided)
    if !config.label.is_empty() {
        ui.painter().text(
            rect.left_center() + vec2(styles::MONTH_LABEL_OFFSET, 0.0),
            Align2::LEFT_CENTER,
            config.label,
            FontId::proportional(config.font_size),
            ui.visuals().text_color()
        );
    }

    // Draw Summary (right-aligned)
    if let Some(text) = config.summary {
        // Slightly smaller font for summary, mimicking previous logic
        let summary_size = if config.font_size > styles::MONTH_HEADER_FONT_SIZE {
             config.font_size - 2.0 // Year header case
        } else {
             styles::MONTH_SUMMARY_FONT_SIZE // Month header case
        };

        ui.painter().text(
            rect.right_center() + vec2(-styles::MONTH_LABEL_OFFSET, 0.0),
            Align2::RIGHT_CENTER,
            text,
            FontId::proportional(summary_size),
            ui.visuals().text_color()
        );
    }

    clicked
}

/// Renders a custom title bar with drag support and window controls.
pub fn render_title_bar(
    ui: &mut Ui,
    title: &str,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

        // Title & Drag Area
        let title_bar_height = 32.0;
        let button_size = vec2(46.0, title_bar_height);
        let (rect, response) = ui.allocate_exact_size(
            vec2(ui.available_width() - 3.0 * button_size.x, title_bar_height), // Reserve space for 3 buttons
            Sense::click_and_drag(),
        );

        if response.dragged_by(egui::PointerButton::Primary) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }

        ui.painter().text(
            rect.left_center() + vec2(10.0, 0.0),
            Align2::LEFT_CENTER,
            title,
            FontId::proportional(12.0),
            ui.visuals().text_color(),
        );

        // Window Controls
        // Minimize
        if custom_window_button(ui, button_size, WindowAction::Minimize).clicked() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }

        // Maximize/Restore
        let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        let action = if is_maximized { WindowAction::Restore } else { WindowAction::Maximize };
        if custom_window_button(ui, button_size, action).clicked() {
             ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
        }

        // Close
        if custom_window_button(ui, button_size, WindowAction::Close).clicked() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    });
}

enum WindowAction {
    Minimize,
    Maximize,
    Restore,
    Close,
}

fn custom_window_button(ui: &mut Ui, size: egui::Vec2, action: WindowAction) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact_selectable(&response, false);

        let bg_color = match action {
            WindowAction::Close if response.hovered() => Color32::from_rgb(232, 17, 35), // Native windows close red
            _ if response.hovered() => ui.visuals().widgets.hovered.bg_fill,
            _ => Color32::TRANSPARENT,
        };

        ui.painter().rect_filled(rect, 0.0, bg_color);

        // Icon color
        let stroke_color = if let WindowAction::Close = action {
            if response.hovered() { Color32::WHITE } else { visuals.text_color() }
        } else {
             visuals.text_color()
        };
        let stroke = Stroke::new(1.0, stroke_color);

        let center = rect.center();
        let half_w = 5.0; // Half-width of the icon shape

        match action {
            WindowAction::Minimize => {
                 ui.painter().line_segment(
                    [
                        center + vec2(-half_w, half_w),
                        center + vec2(half_w, half_w)
                    ],
                    stroke
                );
            }
            WindowAction::Maximize => {
                 ui.painter().rect_stroke(
                    egui::Rect::from_center_size(center, vec2(10.0, 10.0)),
                    0.0,
                    stroke,
                );
            }
            WindowAction::Restore => {
                // Background overlapping rect
                let offset = 2.0;
                 ui.painter().rect_stroke(
                    egui::Rect::from_center_size(center + vec2(offset, -offset), vec2(9.0, 9.0)),
                    0.0,
                    stroke,
                );
                // Foreground rect
                 ui.painter().rect_filled(
                    egui::Rect::from_center_size(center + vec2(-offset, offset), vec2(9.0, 9.0)),
                    0.0,
                    bg_color, // clear what's behind
                );
                 ui.painter().rect_stroke(
                    egui::Rect::from_center_size(center + vec2(-offset, offset), vec2(9.0, 9.0)),
                    0.0,
                    stroke,
                );
            }
            WindowAction::Close => {
                ui.painter().line_segment([center + vec2(-half_w, -half_w), center + vec2(half_w, half_w)], stroke);
                ui.painter().line_segment([center + vec2(half_w, -half_w), center + vec2(-half_w, half_w)], stroke);
            }
        }
    }
    response
}
