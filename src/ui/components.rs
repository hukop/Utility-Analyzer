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
        let icon = if config.is_collapsed { " >" } else { " v" };
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

/// A styled container for grouping content (like a material card).
pub struct Card;

impl Card {
    pub fn new() -> Self {
        Self
    }

    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
        // Use smaller fixed margins instead of borrowing widget spacing
        let outer_margin = 8.0;
        let inner_padding = 10.0;

        egui::Frame::none()
            .fill(ui.visuals().panel_fill)
            .rounding(ui.visuals().widgets.noninteractive.rounding)
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .outer_margin(egui::Margin {
                left: 0.0,    // No extra left margin (CentralPanel handles it)
                right: outer_margin,
                top: 0.0,
                bottom: outer_margin,
            })
            .inner_margin(inner_padding)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                add_contents(ui)
            })
            .inner
    }
}
