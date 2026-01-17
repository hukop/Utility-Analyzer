use egui::Ui;
use crate::data::ElectricData;
use crate::charts::HeatmapState;
use chrono::Datelike;

/// Renders the export sparklines chart showing mini line charts of solar export
/// for each day from 6:00 to 18:00, with daily sums and collapsible months.
pub fn render_export_sparklines(ui: &mut Ui, data: &ElectricData, state: &mut HeatmapState) {
    let (dates, export_data, daily_sums) = data.daily_daytime_export_data();

    if dates.is_empty() {
        ui.label("No export data available");
        return;
    }

    // Find max value for scaling sparklines
    let max_val = export_data.iter()
        .flat_map(|day| day.iter())
        .cloned()
        .fold(0.0_f64, f64::max);

    ui.heading("Export Sparklines (6:00–18:00)");
    ui.label("Mini line charts showing solar export by hour (6–18) for each day. Click month headers to collapse/expand.");
    ui.add_space(crate::ui::styles::CHART_SPACING);

    let sparkline_width = 200.0;
    let row_height = 28.0;
    let date_label_width = 120.0;
    let sum_label_width = 80.0;

    egui::ScrollArea::both().show(ui, |ui| {
        let mut last_month = String::new();
        let mut last_year = String::new();

        for (day_idx, date_str) in dates.iter().enumerate() {
            let year = &date_str[0..4];
            let month = &date_str[0..7];

            // Year header
            if year != last_year {
                let header_rect = ui.allocate_exact_size(
                    egui::vec2(date_label_width + sparkline_width + sum_label_width, crate::ui::styles::YEAR_HEADER_HEIGHT),
                    egui::Sense::click(),
                ).0;

                let is_collapsed = state.collapsed_years.contains(year);
                let response = ui.interact(header_rect, ui.id().with(format!("export_year_{}", year)), egui::Sense::click());

                if response.clicked() {
                    if is_collapsed {
                        state.collapsed_years.remove(year);
                    } else {
                        state.collapsed_years.insert(year.to_string());
                    }
                }

                let bg_color = if response.hovered() {
                    if ui.visuals().dark_mode { egui::Color32::from_gray(80) } else { egui::Color32::from_gray(190) }
                } else if ui.visuals().dark_mode { egui::Color32::from_gray(60) } else { egui::Color32::from_gray(210) };

                ui.painter().rect_filled(header_rect, 0.0, bg_color);
                ui.painter().line_segment(
                    [header_rect.left_bottom(), header_rect.right_bottom()],
                    egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color)
                );

                let icon = if is_collapsed { "⏵" } else { "⏷" };
                ui.painter().text(
                    header_rect.left_center() + egui::vec2(crate::ui::styles::MONTH_TOGGLE_OFFSET, 0.0),
                    egui::Align2::LEFT_CENTER,
                    icon,
                    egui::FontId::monospace(crate::ui::styles::YEAR_HEADER_FONT_SIZE),
                    ui.visuals().text_color()
                );

                ui.painter().text(
                    header_rect.left_center() + egui::vec2(crate::ui::styles::MONTH_LABEL_OFFSET, 0.0),
                    egui::Align2::LEFT_CENTER,
                    year,
                    egui::FontId::proportional(crate::ui::styles::YEAR_HEADER_FONT_SIZE),
                    ui.visuals().text_color()
                );

                // Yearly sum on the right
                let yearly_sum = data.yearly_export_sums.get(year).cloned().unwrap_or(0.0);
                ui.painter().text(
                    header_rect.right_center() + egui::vec2(-crate::ui::styles::MONTH_LABEL_OFFSET, 0.0),
                    egui::Align2::RIGHT_CENTER,
                    format!("Year Total: {:.1} kWh", yearly_sum),
                    egui::FontId::proportional(crate::ui::styles::YEAR_HEADER_FONT_SIZE - 2.0),
                    ui.visuals().text_color()
                );

                last_year = year.to_string();
                last_month = String::new(); // Reset month to show first month of year
            }

            if state.collapsed_years.contains(year) {
                continue;
            }

            // Month header
            if month != last_month {
                let header_rect = ui.allocate_exact_size(
                    egui::vec2(date_label_width + sparkline_width + sum_label_width, crate::ui::styles::MONTH_HEADER_HEIGHT),
                    egui::Sense::click(),
                ).0;

                let is_collapsed = state.collapsed_months.contains(month);
                let response = ui.interact(header_rect, ui.id().with(format!("export_{}", month)), egui::Sense::click());

                if response.clicked() {
                    if is_collapsed {
                        state.collapsed_months.remove(month);
                    } else {
                        state.collapsed_months.insert(month.to_string());
                    }
                }

                let bg_color = if response.hovered() {
                    if ui.visuals().dark_mode { egui::Color32::from_gray(60) } else { egui::Color32::from_gray(210) }
                } else if ui.visuals().dark_mode { egui::Color32::from_gray(45) } else { egui::Color32::from_gray(225) };

                ui.painter().rect_filled(header_rect, 0.0, bg_color);
                ui.painter().line_segment(
                    [header_rect.left_bottom(), header_rect.right_bottom()],
                    egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color)
                );

                let icon = if is_collapsed { "⏵" } else { "⏷" };
                ui.painter().text(
                    header_rect.left_center() + egui::vec2(crate::ui::styles::MONTH_TOGGLE_OFFSET, 0.0),
                    egui::Align2::LEFT_CENTER,
                    icon,
                    egui::FontId::monospace(crate::ui::styles::MONTH_HEADER_FONT_SIZE),
                    ui.visuals().text_color()
                );

                ui.painter().text(
                    header_rect.left_center() + egui::vec2(crate::ui::styles::MONTH_LABEL_OFFSET, 0.0),
                    egui::Align2::LEFT_CENTER,
                    month,
                    egui::FontId::proportional(crate::ui::styles::MONTH_HEADER_FONT_SIZE),
                    ui.visuals().text_color()
                );

                // Monthly sum on the right
                let monthly_sum = data.monthly_export_sums.get(month).cloned().unwrap_or(0.0);
                ui.painter().text(
                    header_rect.right_center() + egui::vec2(-crate::ui::styles::MONTH_LABEL_OFFSET, 0.0),
                    egui::Align2::RIGHT_CENTER,
                    format!("Total: {:.1} kWh", monthly_sum),
                    egui::FontId::proportional(crate::ui::styles::MONTH_SUMMARY_FONT_SIZE),
                    ui.visuals().text_color()
                );

                last_month = month.to_string();
            }

            // Skip if collapsed
            if state.collapsed_months.contains(month) {
                continue;
            }

            let day_export = &export_data[day_idx];
            let day_sum = daily_sums[day_idx];

            // Parse date for weekday info
            let date_parsed = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok();
            let is_weekend = if let Some(d) = date_parsed {
                d.weekday() == chrono::Weekday::Sat || d.weekday() == chrono::Weekday::Sun
            } else {
                false
            };

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);

                // Date label
                let date_label = if let Some(d) = date_parsed {
                    format!("{}", d.format("%Y-%m-%d %a"))
                } else {
                    date_str.clone()
                };

                let (label_rect, _) = ui.allocate_exact_size(
                    egui::vec2(date_label_width, row_height),
                    egui::Sense::hover(),
                );

                if is_weekend {
                    ui.painter().rect_filled(label_rect, 0.0, crate::ui::styles::weekend_bg());
                }

                let text_color = if is_weekend {
                    crate::ui::styles::weekend_text()
                } else {
                    ui.visuals().text_color()
                };

                ui.painter().text(
                    label_rect.left_center() + egui::vec2(5.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    date_label,
                    egui::FontId::proportional(crate::ui::styles::BODY_FONT_SIZE),
                    text_color
                );

                // Sparkline
                let (spark_rect, spark_response) = ui.allocate_exact_size(
                    egui::vec2(sparkline_width, row_height),
                    egui::Sense::hover(),
                );

                // Background
                let spark_bg = if ui.visuals().dark_mode {
                    egui::Color32::from_gray(35)
                } else {
                    egui::Color32::from_gray(245)
                };
                ui.painter().rect_filled(spark_rect, 2.0, spark_bg);

                // Draw sparkline
                if max_val > 0.0 {
                    let padding = 4.0;
                    let draw_rect = spark_rect.shrink(padding);
                    let step_x = draw_rect.width() / (day_export.len() - 1).max(1) as f32;

                    let points: Vec<egui::Pos2> = day_export.iter().enumerate().map(|(i, &val)| {
                        let x = draw_rect.left() + i as f32 * step_x;
                        let normalized = (val / max_val) as f32;
                        let y = draw_rect.bottom() - normalized * draw_rect.height();
                        egui::pos2(x, y)
                    }).collect();

                    // Fill area under curve
                    if points.len() >= 2 {
                        let fill_color = egui::Color32::from_rgba_unmultiplied(220, 180, 0, 40);

                        for i in 0..points.len() - 1 {
                            let p1 = points[i];
                            let p2 = points[i + 1];

                            // Create a quad (convex) for this segment
                            let quad = vec![
                                p1,
                                p2,
                                egui::pos2(p2.x, draw_rect.bottom()),
                                egui::pos2(p1.x, draw_rect.bottom()),
                            ];

                            ui.painter().add(egui::Shape::convex_polygon(
                                quad,
                                fill_color,
                                egui::Stroke::NONE,
                            ));
                        }
                    }

                    // Draw line
                    let line_color = egui::Color32::from_rgb(220, 180, 0);
                    for i in 0..points.len().saturating_sub(1) {
                        ui.painter().line_segment(
                            [points[i], points[i + 1]],
                            egui::Stroke::new(1.5, line_color)
                        );
                    }
                }

                // Tooltip showing hourly breakdown
                if spark_response.hovered() {
                    spark_response.on_hover_ui(|ui| {
                        ui.label(format!("{}", date_str));
                        ui.separator();
                        for (i, &val) in day_export.iter().enumerate() {
                            let hour = i + 6;
                            ui.label(format!("{:02}:00 → {:.2} kWh", hour, val));
                        }
                        ui.separator();
                        ui.label(format!("Total: {:.2} kWh", day_sum));
                    });
                }

                // Daily sum label
                let (sum_rect, _) = ui.allocate_exact_size(
                    egui::vec2(sum_label_width, row_height),
                    egui::Sense::hover(),
                );

                let sum_color = egui::Color32::from_rgb(220, 180, 0);

                ui.painter().text(
                    sum_rect.right_center() + egui::vec2(-5.0, 0.0),
                    egui::Align2::RIGHT_CENTER,
                    format!("{:.1} kWh", day_sum),
                    egui::FontId::proportional(crate::ui::styles::BODY_FONT_SIZE),
                    sum_color
                );
            });
        }
    });
}
