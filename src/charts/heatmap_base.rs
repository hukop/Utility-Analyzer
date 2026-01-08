use egui::Ui;
use crate::charts::colormap::get_viridis_color;
use crate::charts::HeatmapState;
use chrono::Datelike;

pub struct HeatmapConfig {
    pub title: String,
    pub unit: String,
    pub selection_label: String,
    pub show_weekend_emphasis: bool,
    pub x_label_interval: usize,
    pub y_label_width: f32,
    pub cell_height: f32,
}

pub fn render_heatmap_component(
    ui: &mut Ui,
    dates: &[String],
    heatmap_data: &[Vec<f64>],
    state: &mut HeatmapState,
    config: HeatmapConfig,
) {
    if dates.is_empty() {
        ui.label("No data available");
        return;
    }
    
    // Find min/max for color scaling
    let mut max_val = f64::MIN;
    for day_data in heatmap_data {
        for &val in day_data {
            max_val = max_val.max(val);
        }
    }
    
    ui.heading(&config.title);
    
    let cell_width = 35.0;
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
    ui.add_space(20.0);
    
    egui::ScrollArea::both().show(ui, |ui| {
        let content_start_pos = ui.cursor().left_top();
        
        // Month summaries
        let mut month_sums = std::collections::HashMap::<String, f64>::new();
        if config.show_weekend_emphasis {
            for (idx, label) in dates.iter().enumerate() {
                let month = &label[0..7]; // YYYY-MM
                *month_sums.entry(month.to_string()).or_insert(0.0) += heatmap_data[idx].iter().sum::<f64>();
            }
        }

        ui.horizontal(|ui| {
            // Y-axis labels
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                ui.add_space(20.0); // Header offset
                
                let mut last_month = String::new();
                for (_idx, label) in dates.iter().enumerate() {
                    let month = if config.show_weekend_emphasis { &label[0..7] } else { "" };
                    
                    if config.show_weekend_emphasis && month != last_month {
                        let header_rect = ui.allocate_exact_size(
                            egui::vec2(config.y_label_width, 25.0),
                            egui::Sense::click(),
                        ).0;
                        
                        let is_collapsed = state.collapsed_months.contains(month);
                        let response = ui.interact(header_rect, ui.id().with(format!("{}_left", month)), egui::Sense::click());
                        
                        if response.clicked() {
                            if is_collapsed {
                                state.collapsed_months.remove(month);
                            } else {
                                state.collapsed_months.insert(month.to_string());
                            }
                        }

                        let bg_color = if response.hovered() {
                            if ui.visuals().dark_mode { egui::Color32::from_gray(60) } else { egui::Color32::from_gray(210) }
                        } else {
                            if ui.visuals().dark_mode { egui::Color32::from_gray(45) } else { egui::Color32::from_gray(225) }
                        };
                        
                        ui.painter().rect_filled(header_rect, 0.0, bg_color);
                        ui.painter().line_segment(
                            [header_rect.left_bottom(), header_rect.right_bottom()], 
                            egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color)
                        );
                        
                        ui.painter().text(
                            header_rect.left_center() + egui::vec2(10.0, 0.0),
                            egui::Align2::LEFT_CENTER,
                            month,
                            egui::FontId::proportional(14.0),
                            ui.visuals().text_color()
                        );
                        
                        let icon = if is_collapsed { "⏵" } else { "⏷" };
                        ui.painter().text(
                            header_rect.left_center() + egui::vec2(75.0, 0.0),
                            egui::Align2::LEFT_CENTER,
                            icon,
                            egui::FontId::monospace(14.0),
                            ui.visuals().text_color()
                        );
                        
                        last_month = month.to_string();
                    }

                    if config.show_weekend_emphasis && state.collapsed_months.contains(month) {
                        continue;
                    }

                    let (rect, _response) = ui.allocate_exact_size(
                        egui::vec2(config.y_label_width, cell_height),
                        egui::Sense::hover(),
                    );
                    
                    let date_parsed = chrono::NaiveDate::parse_from_str(label, "%Y-%m-%d").ok();
                    let is_weekend = if config.show_weekend_emphasis {
                        if let Some(d) = date_parsed {
                            d.weekday() == chrono::Weekday::Sat || d.weekday() == chrono::Weekday::Sun
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    
                    let label_text = if let Some(d) = date_parsed {
                        format!("{}", d.format("%Y-%m-%d %a"))
                    } else {
                        label.clone()
                    };
                    
                    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |ui| {
                        if is_weekend {
                            ui.painter().rect_filled(rect, 0.0, egui::Color32::from_rgb(230, 242, 255));
                        }
                        
                        let mut text = egui::RichText::new(label_text);
                        if is_weekend {
                            text = text.size(13.0)
                                .strong()
                                .color(egui::Color32::from_rgb(0, 120, 212));
                        } else {
                            text = text.size(12.0)
                                .color(ui.visuals().text_color());
                        }
                        
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
                // X-axis labels
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
                                egui::FontId::proportional(12.0),
                                ui.visuals().text_color(),
                            );
                        }
                    }
                });
                
                // Heatmap cells
                let mut last_month = String::new();
                for (day_idx, day_data) in heatmap_data.iter().enumerate() {
                    let label = &dates[day_idx];
                    let month = if config.show_weekend_emphasis { &label[0..7] } else { "" };

                    if config.show_weekend_emphasis && month != last_month {
                        let header_rect = ui.allocate_exact_size(
                            egui::vec2(24.0 * cell_width, 25.0),
                            egui::Sense::click(),
                        ).0;
                        
                        let response = ui.interact(header_rect, ui.id().with(format!("{}_right", month)), egui::Sense::click());
                        if response.clicked() {
                            if state.collapsed_months.contains(month) {
                                state.collapsed_months.remove(month);
                            } else {
                                state.collapsed_months.insert(month.to_string());
                            }
                        }

                        let bg_color = if response.hovered() {
                            if ui.visuals().dark_mode { egui::Color32::from_gray(60) } else { egui::Color32::from_gray(210) }
                        } else {
                            if ui.visuals().dark_mode { egui::Color32::from_gray(45) } else { egui::Color32::from_gray(225) }
                        };
                        
                        ui.painter().rect_filled(header_rect, 0.0, bg_color);
                        ui.painter().line_segment(
                            [header_rect.left_bottom(), header_rect.right_bottom()], 
                            egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color)
                        );
                        
                        let sum = month_sums.get(month).cloned().unwrap_or(0.0);
                        let val_text = if config.unit == "$" {
                            format!("Total: ${:.2}", sum)
                        } else {
                            format!("Total: {:.2} {}", sum, config.unit)
                        };

                        ui.painter().text(
                            header_rect.right_center() + egui::vec2(-10.0, 0.0),
                            egui::Align2::RIGHT_CENTER,
                            val_text,
                            egui::FontId::proportional(12.0),
                            ui.visuals().text_color()
                        );
                        
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
                            let color = get_viridis_color(val, 0.0, max_val);
                            
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
                                if day_idx >= min_d && day_idx <= max_d && hour >= min_h && hour <= max_h {
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
                                    ui.label(format!(
                                        "{}, {}:00\n{}",
                                        date_label, hour, value_formatted
                                    ));
                                });
                            }
                        }
                    });
                }

                if let Some(rect) = selection_rect {
                    ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(2.0, egui::Color32::WHITE));
                }

                // Final pass: Draw weekend boundaries OVER everything
                let stroke = egui::Stroke::new(1.0, egui::Color32::from_white_alpha(120));
                let total_width = config.y_label_width + (24.0 * cell_width);
                let mut current_y = 20.0; // Starting offset for header
                
                let mut last_month = String::new();
                for (_idx, label) in dates.iter().enumerate() {
                    let month = if config.show_weekend_emphasis { &label[0..7] } else { "" };
                    
                    if config.show_weekend_emphasis && month != last_month {
                        current_y += 25.0; // Account for month header
                        last_month = month.to_string();
                    }

                    if config.show_weekend_emphasis && state.collapsed_months.contains(month) {
                        continue;
                    }

                    let date_parsed = chrono::NaiveDate::parse_from_str(label, "%Y-%m-%d").ok();
                    let (is_sat, is_sun) = if let Some(d) = date_parsed {
                        (d.weekday() == chrono::Weekday::Sat, d.weekday() == chrono::Weekday::Sun)
                    } else {
                        (label == "Saturday", label == "Sunday")
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
                        .rounding(4.0)
                        .inner_margin(6.0)
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
