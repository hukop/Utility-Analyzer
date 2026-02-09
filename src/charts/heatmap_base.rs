use egui::Ui;
use crate::charts::HeatmapState;

pub struct HeatmapConfig<'a> {
    pub title: String,
    pub unit: String,
    pub selection_label: String,
    pub show_weekend_emphasis: bool,
    pub x_label_interval: usize,
    pub y_label_width: f32,
    pub cell_height: f32,
    pub monthly_sums: &'a std::collections::HashMap<String, f64>,
    pub yearly_sums: &'a std::collections::HashMap<String, f64>,
    pub daily_sum_width: f32,
    pub max_value_override: Option<f64>,
}

pub fn render_heatmap_component(
    ui: &mut Ui,
    dates: &[String],
    heatmap_data: &[Vec<f64>],
    state: &mut HeatmapState,
    config: HeatmapConfig<'_>,
) {
    if dates.is_empty() {
        ui.label("No data available");
        return;
    }

    // Find min/max for color scaling
    let data_max = heatmap_data.iter()
        .flat_map(|day| day.iter())
        .copied()
        .fold(f64::MIN, f64::max);

    let max_val = config.max_value_override.unwrap_or(data_max);

    ui.heading(&config.title);

    // Calculate dynamic cell width to fill available space
    let available_width = ui.available_width();
    let reserved_width = config.y_label_width + config.daily_sum_width + 20.0; // Labels + Sum column + padding
    let calculated_cell_width = (available_width - reserved_width) / 24.0;
    let cell_width = calculated_cell_width.max(15.0); // Don't go below 15px (was 35px)
    let cell_height = config.cell_height;

    // Calculate selection sum if active
    let mut selection_sum = 0.0;
    let mut selection_count = 0;
    let mut show_selection_info = false;
    let mut selection_rect = Option::<egui::Rect>::None;

    let mut selected_indices = None;
    if let (Some((start_day, start_hour)), Some((end_day, end_hour))) = (state.selection_start, state.selection_end) {
        let (min_day, max_day) = (start_day.min(end_day), start_day.max(end_day));
        let (min_hour, max_hour) = (start_hour.min(end_hour), start_hour.max(end_hour));
        selected_indices = Some(((min_day, max_day), (min_hour, max_hour)));

        for d in min_day..=max_day {
            if d < heatmap_data.len() {
                for h in min_hour..=max_hour {
                    if h < heatmap_data[d].len() {
                        selection_sum += heatmap_data[d][h];
                        selection_count += 1;
                    }
                }
            }
        }
        show_selection_info = true;
    }

    ui.label(&config.selection_label);
    ui.add_space(crate::ui::styles::CHART_SPACING);

    // Sticky Header
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
        ui.add_space(config.y_label_width);
        ui.add_space(ui.style().spacing.item_spacing.x);

        egui::ScrollArea::horizontal()
            .id_salt("header_scroll_ignore")
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .horizontal_scroll_offset(state.scroll_offset)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                    for hour in 0..24 {
                        let label = if hour % config.x_label_interval == 0 {
                            format!("{}", hour)
                        } else {
                            String::new()
                        };

                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(cell_width, 20.0),
                            egui::Sense::hover(),
                        );

                        if !label.is_empty() {
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                label,
                                egui::FontId::proportional(crate::ui::styles::AXIS_FONT_SIZE),
                                ui.visuals().text_color(),
                            );
                        }
                    }
                });
            });
    });

    let scroll_output = egui::ScrollArea::both().id_salt(format!("{}_main", config.title)).show(ui, |ui| {
        let content_start_pos = ui.cursor().left_top();

        ui.horizontal(|ui| {
            // Y-axis labels
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                // ui.add_space(20.0); // Header offset REMOVED

                let mut last_month = String::new();
                let mut last_year = String::new();
                for label in dates.iter() {
                    let year = if config.show_weekend_emphasis { &label[0..4] } else { "" };
                    let month = if config.show_weekend_emphasis { &label[0..7] } else { "" };

                    // Year Header
                    if config.show_weekend_emphasis && year != last_year {
                        let is_collapsed = state.collapsed_years.contains(year);
                        if crate::ui::components::render_collapsible_header(
                            ui,
                            format!("{}_year_left", year),
                            crate::ui::components::HeaderConfig {
                                label: year,
                                width: config.y_label_width,
                                height: crate::ui::styles::YEAR_HEADER_HEIGHT,
                                font_size: crate::ui::styles::YEAR_HEADER_FONT_SIZE,
                                is_collapsed,
                                summary: None,
                                show_icon: true,
                            },
                        ) {
                            if is_collapsed {
                                state.collapsed_years.remove(year);
                            } else {
                                state.collapsed_years.insert(year.to_string());
                            }
                        }

                        last_year = year.to_string();
                        // Reset last_month so first month of new year gets header
                        last_month = String::new();
                    }

                    if config.show_weekend_emphasis && state.collapsed_years.contains(year) {
                        continue;
                    }

                    if config.show_weekend_emphasis && month != last_month {
                        let is_collapsed = state.collapsed_months.contains(month);
                        if crate::ui::components::render_collapsible_header(
                            ui,
                            format!("{}_left", month),
                            crate::ui::components::HeaderConfig {
                                label: month,
                                width: config.y_label_width,
                                height: crate::ui::styles::MONTH_HEADER_HEIGHT,
                                font_size: crate::ui::styles::MONTH_HEADER_FONT_SIZE,
                                is_collapsed,
                                summary: None,
                                show_icon: true,
                            },
                        ) {
                            if is_collapsed {
                                state.collapsed_months.remove(month);
                            } else {
                                state.collapsed_months.insert(month.to_string());
                            }
                        }

                        last_month = month.to_string();
                    }

                    if config.show_weekend_emphasis && state.collapsed_months.contains(month) {
                        continue;
                    }

                    let (rect, _response) = ui.allocate_exact_size(
                        egui::vec2(config.y_label_width, cell_height),
                        egui::Sense::hover(),
                    );

                    let is_weekend = config.show_weekend_emphasis && crate::ui::UiUtils::is_weekend(label);

                    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                        if is_weekend {
                            ui.painter().rect_filled(rect, 0, crate::ui::UiUtils::weekend_bg(ui.visuals().dark_mode));
                        }

                        let text = if is_weekend {
                             crate::ui::UiUtils::styled_date_label(label, ui.visuals().dark_mode)
                        } else {
                             egui::RichText::new(label.clone()).size(crate::ui::styles::BODY_FONT_SIZE).color(ui.visuals().text_color())
                        };

                        if config.show_weekend_emphasis {
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                ui.add_space(5.0);
                                ui.label(text);
                            });
                        } else {
                            ui.centered_and_justified(|ui| {
                                ui.label(text);
                            });
                        }
                    });
                }
            });

            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                // X-axis labels REMOVED

                // Heatmap cells
                let mut last_month = String::new();
                let mut last_year = String::new();
                for (day_idx, day_data) in heatmap_data.iter().enumerate() {
                    let label = &dates[day_idx];
                    let year = if config.show_weekend_emphasis { &label[0..4] } else { "" };
                    let month = if config.show_weekend_emphasis { &label[0..7] } else { "" };

                    // Year Header
                    if config.show_weekend_emphasis && year != last_year {
                        let is_collapsed = state.collapsed_years.contains(year);

                        let sum = config.yearly_sums.get(year).cloned().unwrap_or(0.0);
                        let val_text = if config.unit == "$" {
                            format!("Year Total: ${:.2}", sum)
                        } else {
                            format!("Year Total: {:.1} {}", sum, config.unit)
                        };

                        if crate::ui::components::render_collapsible_header(
                            ui,
                            format!("{}_year_right", year),
                            crate::ui::components::HeaderConfig {
                                label: "", // Don't repeat label on right
                                width: 24.0 * cell_width,
                                height: crate::ui::styles::YEAR_HEADER_HEIGHT,
                                font_size: crate::ui::styles::YEAR_HEADER_FONT_SIZE,
                                is_collapsed,
                                summary: Some(val_text),
                                show_icon: false,
                            },
                        ) {
                            if is_collapsed {
                                state.collapsed_years.remove(year);
                            } else {
                                state.collapsed_years.insert(year.to_string());
                            }
                        }

                        last_year = year.to_string();
                        // Reset last_month so first month of new year gets header
                        last_month = String::new();
                    }

                    if config.show_weekend_emphasis && state.collapsed_years.contains(year) {
                        continue;
                    }

                    if config.show_weekend_emphasis && month != last_month {
                        let is_collapsed = state.collapsed_months.contains(month);

                        let sum = config.monthly_sums.get(month).cloned().unwrap_or(0.0);
                        let val_text = if config.unit == "$" {
                            format!("Total: ${:.2}", sum)
                        } else {
                            format!("Total: {:.2} {}", sum, config.unit)
                        };

                        if crate::ui::components::render_collapsible_header(
                            ui,
                            format!("{}_right", month),
                            crate::ui::components::HeaderConfig {
                                label: "", // Don't repeat label on right
                                width: 24.0 * cell_width,
                                height: crate::ui::styles::MONTH_HEADER_HEIGHT,
                                font_size: crate::ui::styles::MONTH_HEADER_FONT_SIZE,
                                is_collapsed,
                                summary: Some(val_text),
                                show_icon: false,
                            },
                        ) {
                            if is_collapsed {
                                state.collapsed_months.remove(month);
                            } else {
                                state.collapsed_months.insert(month.to_string());
                            }
                        }

                        last_month = month.to_string();
                    }

                    if config.show_weekend_emphasis && state.collapsed_months.contains(month) {
                        continue;
                    }

                    let date_parsed = if config.show_weekend_emphasis {
                        chrono::NaiveDate::parse_from_str(&dates[day_idx], "%Y-%m-%d").ok()
                    } else {
                        None
                    };

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                        for (hour, &val) in day_data.iter().enumerate() {
                            let color = crate::charts::colormap::get_heatmap_color(val, 0.0, max_val, state.palette);

                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(cell_width, cell_height),
                                egui::Sense::drag(),
                            );

                            if response.drag_started() {
                                state.selection_start = Some((day_idx, hour));
                                state.selection_end = Some((day_idx, hour));
                                state.is_dragging = true;
                            } else if response.drag_stopped() {
                                state.is_dragging = false;
                            }

                            if state.is_dragging {
                                if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
                                    if rect.contains(pointer_pos) {
                                         state.selection_end = Some((day_idx, hour));
                                    }
                                }
                            }

                            if let Some(((min_d, max_d), (min_h, max_h))) = selected_indices {
                                if (min_d..=max_d).contains(&day_idx) && (min_h..=max_h).contains(&hour) {
                                    selection_rect = Some(selection_rect.map_or(rect, |r| r.union(rect)));
                                }
                            }

                            ui.painter().rect_filled(rect, 0.0, color);

                            if response.hovered() {
                                response.on_hover_ui(|ui| {
                                    let date_label = if let Some(d) = date_parsed {
                                        d.format("%Y-%m-%d %a").to_string()
                                    } else {
                                        dates[day_idx].clone()
                                    };
                                    let value_formatted = if config.unit == "$" {
                                        format!("${:.2}", val)
                                    } else {
                                        format!("{:.2} {}", val, config.unit)
                                    };
                                    ui.label(egui::RichText::new(date_label).strong());
                                    ui.label(egui::RichText::new(format!("{:2}:00 -> {}", hour, value_formatted)).monospace());
                                });
                            }
                        }

                        // Daily sum column
                        if config.daily_sum_width > 0.0 {
                            let daily_sum: f64 = day_data.iter().sum();
                            let (sum_rect, _) = ui.allocate_exact_size(
                                egui::vec2(config.daily_sum_width, cell_height),
                                egui::Sense::hover(),
                            );

                            let sum_text = if config.unit == "$" {
                                format!("${:.2}", daily_sum)
                            } else {
                                format!("{:.1} {}", daily_sum, config.unit)
                            };

                            let sum_color = egui::Color32::from_rgb(100, 200, 100);

                            ui.painter().text(
                                sum_rect.right_center() + egui::vec2(-5.0, 0.0),
                                egui::Align2::RIGHT_CENTER,
                                sum_text,
                                egui::FontId::proportional(crate::ui::styles::BODY_FONT_SIZE),
                                sum_color
                            );
                        }
                    });
                }

                if let Some(rect) = selection_rect {
                    ui.painter().rect_stroke(rect, egui::CornerRadius::ZERO, egui::Stroke::new(2.0, egui::Color32::WHITE), egui::StrokeKind::Middle);
                }

                // Final pass: Draw weekend boundaries OVER everything
                let stroke = egui::Stroke::new(1.0, egui::Color32::from_white_alpha(120));
                let total_width = config.y_label_width + ui.style().spacing.item_spacing.x + (24.0 * cell_width) + 7.0;
                let mut current_y = 0.0; // Starting offset (was 20.0)

                let mut last_month = String::new();
                let mut last_year = String::new();
                for label in dates.iter() {
                    let year = if config.show_weekend_emphasis { &label[0..4] } else { "" };
                    let month = if config.show_weekend_emphasis { &label[0..7] } else { "" };

                    if config.show_weekend_emphasis && year != last_year {
                        current_y += crate::ui::styles::YEAR_HEADER_HEIGHT;
                        last_year = year.to_string();
                        last_month = String::new();
                    }

                    if config.show_weekend_emphasis && state.collapsed_years.contains(year) {
                        continue;
                    }

                    if config.show_weekend_emphasis && month != last_month {
                        current_y += crate::ui::styles::MONTH_HEADER_HEIGHT;
                        last_month = month.to_string();
                    }

                    if config.show_weekend_emphasis && state.collapsed_months.contains(month) {
                        continue;
                    }

                    let is_weekend = crate::ui::UiUtils::is_weekend(label);
                    let (is_sat, is_sun) = if is_weekend {
                        (label.contains("Sat") || label == "Saturday", label.contains("Sun") || label == "Sunday")
                    } else {
                        (false, false)
                    };

                    let x_start = content_start_pos.x;

                    if is_sat {
                        let y = content_start_pos.y + current_y;
                        ui.painter().line_segment([egui::pos2(x_start, y), egui::pos2(x_start + total_width, y)], stroke);
                    }

                    if is_sun {
                        let y = content_start_pos.y + current_y + cell_height;
                        ui.painter().line_segment([egui::pos2(x_start, y), egui::pos2(x_start + total_width, y)], stroke);
                    }

                    current_y += cell_height;
                }
            });
        });
    });

    state.scroll_offset = scroll_output.state.offset.x;

    if show_selection_info {
        if let Some(rect) = selection_rect {
            let pos = rect.right_top() + egui::vec2(10.0, 0.0);
            egui::Area::new(format!("{}_selection", config.title).into())
                .fixed_pos(pos)
                .order(egui::Order::Foreground)
                .show(ui.ctx(), |ui| {
                    egui::Frame::default()
                        .fill(egui::Color32::from_black_alpha(240))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::WHITE))
                        .corner_radius(4)
                        .inner_margin(6)
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(2.0, 2.0);
                            ui.label(egui::RichText::new("SELECTION").size(10.0).color(egui::Color32::LIGHT_GRAY));
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("{}", selection_count)).color(egui::Color32::WHITE).strong().size(18.0));
                                ui.label(egui::RichText::new("cells").size(12.0).color(egui::Color32::GRAY));
                            });
                            ui.horizontal(|ui| {
                                let (val_text, unit_text) = if config.unit == "$" {
                                    (format!("${:.2}", selection_sum), String::new())
                                } else {
                                    (format!("{:.2}", selection_sum), config.unit.clone())
                                };
                                ui.label(egui::RichText::new(val_text).color(egui::Color32::GREEN).strong().size(22.0));
                                if !unit_text.is_empty() {
                                    ui.label(egui::RichText::new(unit_text).size(14.0).color(egui::Color32::GREEN));
                                }
                            });
                        });
                });
        }
    }

    if ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary)) && !ui.ctx().is_using_pointer() {
        state.selection_start = None;
        state.selection_end = None;
        state.is_dragging = false;
    }
}
