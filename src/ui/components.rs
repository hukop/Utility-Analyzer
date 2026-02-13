use crate::ui::styles;
use egui::{vec2, Align2, Color32, FontId, Sense, Stroke, Ui};

pub struct HeaderConfig<'a> {
    pub label: &'a str,
    pub width: f32,
    pub height: f32,
    pub font_size: f32,
    pub is_collapsed: bool,
    pub summary: Option<String>,
    pub show_icon: bool,
}

/// Renders a collapsible header with standardized styling and returns true when clicked.
pub fn render_collapsible_header(
    ui: &mut Ui,
    id_salt: impl std::hash::Hash,
    config: HeaderConfig<'_>,
) -> bool {
    let (rect, _) = ui.allocate_exact_size(vec2(config.width, config.height), Sense::click());
    let response = ui.interact(rect, ui.id().with(id_salt), Sense::click());
    let clicked = response.clicked();

    let bg_color = if response.hovered() {
        if ui.visuals().dark_mode {
            Color32::from_gray(72)
        } else {
            Color32::from_gray(198)
        }
    } else if ui.visuals().dark_mode {
        Color32::from_gray(54)
    } else {
        Color32::from_gray(216)
    };

    let rounding = egui::CornerRadius::same(4);
    ui.painter().rect_filled(rect, rounding, bg_color);
    ui.painter().rect_stroke(
        rect,
        rounding,
        Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
        egui::StrokeKind::Middle,
    );

    if config.show_icon {
        let icon = if config.is_collapsed { ">" } else { "v" };
        ui.painter().text(
            rect.left_center() + vec2(styles::MONTH_TOGGLE_OFFSET, 0.0),
            Align2::LEFT_CENTER,
            icon,
            FontId::monospace(config.font_size),
            ui.visuals().text_color(),
        );
    }

    if !config.label.is_empty() {
        ui.painter().text(
            rect.left_center() + vec2(styles::MONTH_LABEL_OFFSET, 0.0),
            Align2::LEFT_CENTER,
            config.label,
            FontId::proportional(config.font_size),
            ui.visuals().text_color(),
        );
    }

    if let Some(text) = config.summary {
        let summary_size = if config.font_size > styles::MONTH_HEADER_FONT_SIZE {
            config.font_size - 2.0
        } else {
            styles::MONTH_SUMMARY_FONT_SIZE
        };

        ui.painter().text(
            rect.right_center() + vec2(-styles::MONTH_LABEL_OFFSET, 0.0),
            Align2::RIGHT_CENTER,
            text,
            FontId::proportional(summary_size),
            ui.visuals().text_color().gamma_multiply(0.85),
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
        let outer_margin: i8 = 8;
        let inner_padding: i8 = 10;

        egui::Frame::NONE
            .fill(ui.visuals().panel_fill)
            .corner_radius(ui.visuals().widgets.noninteractive.corner_radius)
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .outer_margin(egui::Margin {
                left: 0,
                right: outer_margin,
                top: 0,
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
