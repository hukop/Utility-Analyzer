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
    pub modern: bool,
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

    let bg_color = if config.modern {
        if response.hovered() {
            if ui.visuals().dark_mode {
                Color32::from_white_alpha(20)
            } else {
                Color32::from_black_alpha(15)
            }
        } else if ui.visuals().dark_mode {
            Color32::from_white_alpha(10)
        } else {
            Color32::from_black_alpha(10)
        }
    } else {
        if response.hovered() {
            if ui.visuals().dark_mode {
                Color32::from_gray(72)
            } else {
                Color32::from_gray(198)
            }
        } else if ui.visuals().dark_mode {
            Color32::from_gray(54)
        } else {
            Color32::from_gray(216)
        }
    };

    let rounding = if config.modern {
        egui::CornerRadius::same(6)
    } else {
        egui::CornerRadius::same(4)
    };

    ui.painter().rect_filled(rect, rounding, bg_color);

    if !config.modern {
        ui.painter().rect_stroke(
            rect,
            rounding,
            Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
            egui::StrokeKind::Middle,
        );
    }

    if config.show_icon {
        let center = rect.left_center() + vec2(styles::MONTH_TOGGLE_OFFSET, 0.0);
        let size = config.font_size * 0.4;
        let color = ui.visuals().text_color();

        let points = if config.is_collapsed {
            // Right-pointing triangle
            vec![
                center + vec2(-size * 0.4, -size * 0.7),
                center + vec2(-size * 0.4, size * 0.7),
                center + vec2(size * 0.7, 0.0),
            ]
        } else {
            // Down-pointing triangle
            vec![
                center + vec2(-size * 0.7, -size * 0.4),
                center + vec2(size * 0.7, -size * 0.4),
                center + vec2(0.0, size * 0.5),
            ]
        };

        ui.painter().add(egui::Shape::convex_polygon(
            points,
            color,
            Stroke::NONE,
        ));
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

    pub fn show<R>(self, ui: &mut Ui, modern_ui: bool, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
        let outer_margin: i8 = 8;
        let inner_padding: i8 = 10;

        let frame = if modern_ui {
             egui::Frame::NONE
                .fill(ui.visuals().panel_fill)
                .corner_radius(egui::CornerRadius::same(12))
                .stroke(egui::Stroke {
                    color: ui.visuals().widgets.noninteractive.bg_stroke.color.gamma_multiply(0.5),
                    ..ui.visuals().widgets.noninteractive.bg_stroke
                })
                .outer_margin(egui::Margin {
                    left: 0,
                    right: outer_margin,
                    top: 0,
                    bottom: outer_margin,
                })
                .inner_margin(inner_padding)
        } else {
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
        };

        frame.show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add_contents(ui)
        })
        .inner
    }
}

/// Computes position-aware rounding for a segmented control item.
fn segmented_item_rounding(
    index: usize,
    count: usize,
    rounding: egui::CornerRadius,
) -> egui::CornerRadius {
    if count == 1 {
        rounding
    } else if index == 0 {
        egui::CornerRadius { nw: rounding.nw, sw: rounding.sw, ne: 0, se: 0 }
    } else if index == count - 1 {
        egui::CornerRadius { ne: rounding.ne, se: rounding.se, nw: 0, sw: 0 }
    } else {
        egui::CornerRadius::ZERO
    }
}

/// Renders a pill-style segmented control.
pub fn render_segmented_control<T: PartialEq + Copy>(
    ui: &mut Ui,
    current_value: &mut T,
    options: &[(T, &str)],
) {
    let height = 28.0;
    let item_width = 70.0;
    let font_size = 13.0;
    let rounding = egui::CornerRadius::same((height / 2.0) as u8);

    let frame_fill = if ui.visuals().dark_mode {
        Color32::from_gray(40)
    } else {
        Color32::from_gray(230)
    };

    egui::Frame::NONE
        .fill(frame_fill)
        .corner_radius(rounding)
        .show(ui, |ui| {
            // Use allocate_ui_with_layout to force LTR ordering (so rounding
            // indices match visual position) while constraining width to
            // exactly the content size.
            let total_width = item_width * options.len() as f32;
            ui.allocate_ui_with_layout(
                egui::vec2(total_width, height),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                    let count = options.len();

                    for (i, (value, label)) in options.iter().enumerate() {
                        let is_selected = *current_value == *value;

                        let text_color = if is_selected {
                            Color32::WHITE
                        } else {
                            ui.visuals().text_color().gamma_multiply(0.7)
                        };

                        let (rect, response) = ui.allocate_at_least(
                            egui::vec2(item_width, height),
                            Sense::click(),
                        );

                        if response.clicked() {
                            *current_value = *value;
                        }

                        let item_rounding = segmented_item_rounding(i, count, rounding);

                        if is_selected {
                            ui.painter().rect_filled(rect, item_rounding, styles::theme_accent());
                        } else if response.hovered() {
                            let hover_fill = if ui.visuals().dark_mode {
                                Color32::from_white_alpha(15)
                            } else {
                                Color32::from_black_alpha(15)
                            };
                            ui.painter().rect_filled(rect, item_rounding, hover_fill);
                        }

                        ui.painter().text(
                            rect.center(),
                            Align2::CENTER_CENTER,
                            *label,
                            FontId::proportional(font_size),
                            text_color,
                        );
                    }
                },
            );
        });
}

/// Renders a sidebar navigation item with a modern look and an optional active indicator.
pub fn render_sidebar_item(
    ui: &mut Ui,
    selected: bool,
    icon: &str,
    color: Color32,
    label: &str,
    modern: bool,
) -> bool {
    let height = if modern { 34.0 } else { 24.0 };
    let (rect, response) = ui.allocate_at_least(egui::vec2(ui.available_width(), height), Sense::click());

    if modern {
        let bg_fill = if selected {
            if ui.visuals().dark_mode {
                Color32::from_white_alpha(15)
            } else {
                Color32::from_black_alpha(10)
            }
        } else if response.hovered() {
            if ui.visuals().dark_mode {
                Color32::from_white_alpha(10)
            } else {
                Color32::from_black_alpha(5)
            }
        } else {
            Color32::TRANSPARENT
        };

        if bg_fill != Color32::TRANSPARENT {
            ui.painter().rect_filled(rect, egui::CornerRadius::same(6), bg_fill);
        }

        if selected {
            let indicator_rect = egui::Rect::from_min_max(
                rect.left_top() + vec2(2.0, 8.0),
                rect.left_bottom() + vec2(5.0, -8.0),
            );
            ui.painter().rect_filled(indicator_rect, egui::CornerRadius::same(2), styles::theme_accent());
        }

        ui.painter().text(
            rect.left_center() + vec2(12.0, 0.0),
            Align2::LEFT_CENTER,
            icon,
            FontId::proportional(18.0),
            color,
        );

        let text_color = if selected {
            ui.visuals().text_color()
        } else {
            ui.visuals().text_color().gamma_multiply(0.8)
        };

        ui.painter().text(
            rect.left_center() + vec2(40.0, 0.0),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(14.0),
            text_color,
        );
    } else {
        // Classic style
        ui.painter().text(
            rect.left_center(),
            Align2::LEFT_CENTER,
            icon,
            FontId::proportional(18.0),
            color,
        );

        let clicked = ui.put(
            egui::Rect::from_min_max(rect.left_top() + vec2(25.0, 0.0), rect.right_bottom()),
            egui::Button::new(label).selected(selected)
        ).clicked();

        return clicked || response.clicked();
    }

    response.clicked()
}
