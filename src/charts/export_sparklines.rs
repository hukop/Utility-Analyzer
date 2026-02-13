use crate::charts::HeatmapState;
use crate::data::ElectricData;
use egui::Ui;

/// Renders the export sparklines chart showing mini line charts of solar export
/// for each day from 6:00 to 18:00, with daily sums and collapsible months.
pub fn render_export_sparklines(ui: &mut Ui, data: &ElectricData, state: &mut HeatmapState) {
    let (dates, export_data, daily_sums, max_val, date_meta) =
        data.daily_daytime_export_data_cached();

    if dates.is_empty() {
        ui.label("No export data available");
        return;
    }

    ui.heading("Export Sparklines (6:00-18:00)");
    ui.label("Mini line charts showing solar export by hour (6-18) for each day. Click month headers to collapse/expand.");
    ui.add_space(crate::ui::styles::CHART_SPACING);

    let date_label_width = 120.0;
    let sum_label_width = 80.0;
    let padding = 20.0;
    let available_width = ui.available_width();
    let sparkline_width =
        (available_width - date_label_width - sum_label_width - padding).max(100.0);

    let row_height = 28.0;

    egui::ScrollArea::both().show(ui, |ui| {
        let mut last_month: Option<&str> = None;
        let mut last_year: Option<&str> = None;

        for day_idx in 0..dates.len() {
            let day_meta = &date_meta[day_idx];
            let year = day_meta.year_key.as_str();
            let month = day_meta.month_key.as_str();

            if last_year != Some(year) {
                let is_collapsed = state.collapsed_years.contains(year);
                let sum = data.yearly_export_sums.get(year).copied().unwrap_or(0.0);

                let mut clicked = false;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);

                    if crate::ui::components::render_collapsible_header(
                        ui,
                        ("export_year_left", year),
                        crate::ui::components::HeaderConfig {
                            label: year,
                            width: date_label_width,
                            height: crate::ui::styles::YEAR_HEADER_HEIGHT,
                            font_size: crate::ui::styles::YEAR_HEADER_FONT_SIZE,
                            is_collapsed,
                            summary: None,
                            show_icon: true,
                        },
                    ) {
                        clicked = true;
                    }

                    let right_width = sparkline_width + 4.0 + sum_label_width;
                    if crate::ui::components::render_collapsible_header(
                        ui,
                        ("export_year_right", year),
                        crate::ui::components::HeaderConfig {
                            label: "",
                            width: right_width,
                            height: crate::ui::styles::YEAR_HEADER_HEIGHT,
                            font_size: crate::ui::styles::YEAR_HEADER_FONT_SIZE,
                            is_collapsed,
                            summary: Some(format!("Year Total: {:.1} kWh", sum)),
                            show_icon: false,
                        },
                    ) {
                        clicked = true;
                    }
                });

                if clicked {
                    if is_collapsed {
                        state.collapsed_years.remove(year);
                    } else {
                        state.collapsed_years.insert(year.to_owned());
                    }
                }

                last_year = Some(year);
                last_month = None;
            }

            if state.collapsed_years.contains(year) {
                continue;
            }

            if last_month != Some(month) {
                let is_collapsed = state.collapsed_months.contains(month);
                let sum = data.monthly_export_sums.get(month).copied().unwrap_or(0.0);

                let mut clicked = false;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);

                    if crate::ui::components::render_collapsible_header(
                        ui,
                        ("export_month_left", month),
                        crate::ui::components::HeaderConfig {
                            label: month,
                            width: date_label_width,
                            height: crate::ui::styles::MONTH_HEADER_HEIGHT,
                            font_size: crate::ui::styles::MONTH_HEADER_FONT_SIZE,
                            is_collapsed,
                            summary: None,
                            show_icon: true,
                        },
                    ) {
                        clicked = true;
                    }

                    let right_width = sparkline_width + 4.0 + sum_label_width;
                    if crate::ui::components::render_collapsible_header(
                        ui,
                        ("export_month_right", month),
                        crate::ui::components::HeaderConfig {
                            label: "",
                            width: right_width,
                            height: crate::ui::styles::MONTH_HEADER_HEIGHT,
                            font_size: crate::ui::styles::MONTH_HEADER_FONT_SIZE,
                            is_collapsed,
                            summary: Some(format!("Total: {:.2} kWh", sum)),
                            show_icon: false,
                        },
                    ) {
                        clicked = true;
                    }
                });

                if clicked {
                    if is_collapsed {
                        state.collapsed_months.remove(month);
                    } else {
                        state.collapsed_months.insert(month.to_owned());
                    }
                }

                last_month = Some(month);
            }

            if state.collapsed_months.contains(month) {
                continue;
            }

            let day_export = &export_data[day_idx];
            let day_sum = daily_sums[day_idx];

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);

                let (label_rect, _) = ui.allocate_exact_size(
                    egui::vec2(date_label_width, row_height),
                    egui::Sense::hover(),
                );

                if day_meta.is_weekend {
                    ui.painter().rect_filled(
                        label_rect,
                        0.0,
                        crate::ui::styles::weekend_bg(ui.visuals().dark_mode),
                    );
                }

                let text_color = if day_meta.is_weekend {
                    crate::ui::styles::weekend_text(ui.visuals().dark_mode)
                } else {
                    ui.visuals().text_color()
                };

                ui.painter().text(
                    label_rect.left_center() + egui::vec2(5.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    day_meta.display_label.as_str(),
                    egui::FontId::proportional(crate::ui::styles::BODY_FONT_SIZE),
                    text_color,
                );

                let (spark_rect, spark_response) = ui.allocate_exact_size(
                    egui::vec2(sparkline_width, row_height),
                    egui::Sense::hover(),
                );

                let spark_bg = if ui.visuals().dark_mode {
                    egui::Color32::from_gray(35)
                } else {
                    egui::Color32::from_gray(245)
                };
                ui.painter().rect_filled(spark_rect, 2.0, spark_bg);

                if max_val > 0.0 {
                    let padding = 4.0;
                    let draw_rect = spark_rect.shrink(padding);
                    let painter = ui.painter();
                    let step_x =
                        draw_rect.width() / (day_export.len().saturating_sub(1).max(1) as f32);
                    let line_color = egui::Color32::from_rgb(220, 180, 0);

                    let mut prev: Option<egui::Pos2> = None;
                    for (i, &val) in day_export.iter().enumerate() {
                        let x = draw_rect.left() + i as f32 * step_x;
                        let normalized = (val / max_val) as f32;
                        let y = draw_rect.bottom() - normalized * draw_rect.height();
                        let current = egui::pos2(x, y);

                        if let Some(previous) = prev {
                            painter.line_segment(
                                [previous, current],
                                egui::Stroke::new(2.0, line_color),
                            );
                        }

                        prev = Some(current);
                    }
                }

                if spark_response.hovered() {
                    spark_response.on_hover_ui(|ui| {
                        ui.label(day_meta.date_key.as_str());
                        ui.label(egui::RichText::new(day_meta.display_label.as_str()).strong());
                        ui.separator();
                        for (i, &val) in day_export.iter().enumerate() {
                            let hour = i + 6;
                            ui.label(
                                egui::RichText::new(format!("{:2}:00 -> {:6.2} kWh", hour, val))
                                    .monospace(),
                            );
                        }
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!("Total: {:.2} kWh", day_sum)).strong(),
                        );
                    });
                }

                let (sum_rect, _) = ui.allocate_exact_size(
                    egui::vec2(sum_label_width, row_height),
                    egui::Sense::hover(),
                );

                ui.painter().text(
                    sum_rect.right_center() + egui::vec2(-5.0, 0.0),
                    egui::Align2::RIGHT_CENTER,
                    format!("{:.1} kWh", day_sum),
                    egui::FontId::proportional(crate::ui::styles::BODY_FONT_SIZE),
                    egui::Color32::from_rgb(220, 180, 0),
                );
            });
        }
    });
}
