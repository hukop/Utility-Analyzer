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
                let is_collapsed = state.collapsed_years.contains(year);
                let sum = data.yearly_export_sums.get(year).cloned().unwrap_or(0.0);

                let mut clicked = false;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);

                    // Left Header (Date Column)
                    if crate::ui::components::render_collapsible_header(
                        ui,
                        format!("export_year_{}_left", year),
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

                    // Right Header (Sparkline + Sum Columns)
                    // Width = sparkline (200) + spacing (4) + sum (80) = 284
                    let right_width = sparkline_width + 4.0 + sum_label_width;
                    if crate::ui::components::render_collapsible_header(
                        ui,
                        format!("export_year_{}_right", year),
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
                        state.collapsed_years.insert(year.to_string());
                    }
                }

                last_year = year.to_string();
                last_month = String::new(); // Reset month to show first month of year
            }

            if state.collapsed_years.contains(year) {
                continue;
            }

            // Month header
            if month != last_month {
                let is_collapsed = state.collapsed_months.contains(month);
                let sum = data.monthly_export_sums.get(month).cloned().unwrap_or(0.0);

                let mut clicked = false;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);

                    // Left Header (Date Column)
                    if crate::ui::components::render_collapsible_header(
                        ui,
                        format!("export_{}_left", month),
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

                    // Right Header (Sparkline + Sum Columns)
                    let right_width = sparkline_width + 4.0 + sum_label_width;
                    if crate::ui::components::render_collapsible_header(
                        ui,
                        format!("export_{}_right", month),
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
                        state.collapsed_months.insert(month.to_string());
                    }
                }

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
                    ui.painter().rect_filled(label_rect, 0.0, crate::ui::styles::weekend_bg(ui.visuals().dark_mode));
                }

                let text_color = if is_weekend {
                    crate::ui::styles::weekend_text(ui.visuals().dark_mode)
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
                    let painter = ui.painter();

                    let step_x = draw_rect.width() / (day_export.len() - 1).max(1) as f32;

                    let points: Vec<egui::Pos2> = day_export.iter().enumerate().map(|(i, &val)| {
                        let x = draw_rect.left() + i as f32 * step_x;
                        let normalized = (val / max_val) as f32;
                        let y = draw_rect.bottom() - normalized * draw_rect.height();
                        egui::pos2(x, y)
                    }).collect();

                    // Draw line with thicker stroke to give it weight
                    let line_color = egui::Color32::from_rgb(220, 180, 0);
                    for i in 0..points.len().saturating_sub(1) {
                        painter.line_segment(
                            [points[i], points[i + 1]],
                            egui::Stroke::new(2.0, line_color)
                        );
                    }
                }

                // Tooltip showing hourly breakdown
                if spark_response.hovered() {
                    spark_response.on_hover_ui(|ui| {
                        ui.label(date_str.to_string());
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
