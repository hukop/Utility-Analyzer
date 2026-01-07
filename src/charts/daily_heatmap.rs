use crate::data::ElectricData;
use egui::Ui;
use crate::charts::colormap::get_viridis_color;
use crate::charts::HeatmapState;
use chrono::Datelike;

pub fn render_daily_heatmap(ui: &mut Ui, data: &ElectricData, state: &mut HeatmapState) {
    let (dates, heatmap_data) = data.daily_hour_heatmap();
    
    if dates.is_empty() {
        ui.label("No data available");
        return;
    }
    
    // Find min/max for color scaling
    let mut min_val = f64::MAX;
    let mut max_val = f64::MIN;
    
    for day_data in &heatmap_data {
        for &val in day_data {
            min_val = min_val.min(val);
            max_val = max_val.max(val);
        }
    }
    
    ui.heading("Daily kWh Heatmap: Day (rows) vs Hour (columns)");
    
    // Feature 3: Heatmaps should be wider
    let cell_width = 35.0; // Increased from 25.0
    let cell_height = 25.0;
    
    // Calculate selection sum if active
    let mut selection_sum = 0.0;
    let mut selection_count = 0;
    let mut show_selection_info = false;
    let mut selection_rect = Option::<egui::Rect>::None;
    
    // Pre-calculate selection indices
    let mut selected_indices = None;
    if let (Some((start_day, start_hour)), Some((end_day, end_hour))) = (state.selection_start, state.selection_end) {
         let (min_day, max_day) = (start_day.min(end_day), start_day.max(end_day));
         let (min_hour, max_hour) = (start_hour.min(end_hour), start_hour.max(end_hour));
         selected_indices = Some(((min_day, max_day), (min_hour, max_hour)));
         
         // Calculate sum
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

    ui.label("Click and drag to select a range to view total kWh");
    
    egui::ScrollArea::both().show(ui, |ui| {
        ui.horizontal(|ui| {
            // Y-axis labels (dates)
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                ui.add_space(20.0); // Header offset
                for date_str in &dates {
                    let (rect, _response) = ui.allocate_exact_size(
                        egui::vec2(100.0, cell_height),
                        egui::Sense::hover(),
                    );
                    
                    // Parse date for weekend check
                    let date_parsed = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok();
                    let is_weekend = if let Some(d) = date_parsed {
                        d.weekday() == chrono::Weekday::Sat || d.weekday() == chrono::Weekday::Sun
                    } else {
                        false
                    };
                    
                    let label_text = if let Some(d) = date_parsed {
                        format!("{}", d.format("%Y-%m-%d %a"))
                    } else {
                        date_str.clone()
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
                        
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.add_space(5.0);
                            ui.label(text);
                        });
                    });
                }
            });
            
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                // X-axis labels (hours)
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                    for hour in 0..24 {
                        let label = if hour % 3 == 0 {
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
                for (day_idx, day_data) in heatmap_data.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                        
                        // Check if this row is a weekend
                        let date_parsed = chrono::NaiveDate::parse_from_str(&dates[day_idx], "%Y-%m-%d").ok();

                        
                        for (hour, &val) in day_data.iter().enumerate() {
                            let color = get_viridis_color(val, 0.0, max_val);
                            
                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(cell_width, cell_height),
                                egui::Sense::drag(),
                            );
                            
                            // Handle selection interactions using global pointer position
                            // because dragging captures the first widget
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
                                    // Expand selection bounding box
                                    selection_rect = Some(selection_rect.map_or(rect, |r| r.union(rect)));
                                }
                            }

                            // Draw cell
                            ui.painter().rect_filled(rect, 0.0, color);
                            
                            // Feature 1: Mark weekends (border style)
                            if let Some(d) = date_parsed {
                                let stroke = egui::Stroke::new(1.0, egui::Color32::from_white_alpha(50));
                                if d.weekday() == chrono::Weekday::Sat {
                                     // Top border only for Saturday
                                     ui.painter().line_segment(
                                         [rect.left_top(), rect.right_top()], 
                                         stroke
                                     );
                                } else if d.weekday() == chrono::Weekday::Sun {
                                     // Bottom border only for Sunday
                                     ui.painter().line_segment(
                                         [rect.left_bottom(), rect.right_bottom()], 
                                         stroke
                                     );
                                }
                            }
                            
                            // Selection highlight removed (drawing single rect at end)
                            
                            if response.hovered() {
                                response.on_hover_ui(|ui| {
                                    ui.label(format!(
                                        "{}, {}:00\n{:.2} kWh",
                                        if let Some(d) = date_parsed {
                                            d.format("%Y-%m-%d %a").to_string()
                                        } else {
                                            dates[day_idx].clone()
                                        },
                                        hour, val
                                    ));
                                });
                            }
                        }
                    });
                }

                // Draw selection outline if active
                if let Some(rect) = selection_rect {
                    ui.painter().rect_stroke(
                        rect,
                        0.0,
                        egui::Stroke::new(2.0, egui::Color32::WHITE)
                    );
                }
            });
        });
    });

    // Draw floating summary box
    if show_selection_info {
        if let Some(rect) = selection_rect {
            let pos = rect.right_top() + egui::vec2(10.0, 0.0);
            
            egui::Area::new("daily_heatmap_selection".into())
                .fixed_pos(pos)
                .order(egui::Order::Foreground)
                .show(ui.ctx(), |ui| {
                    egui::Frame::default()
                        .fill(egui::Color32::from_black_alpha(240))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::WHITE))
                        .rounding(4.0)
                        .inner_margin(6.0) // Reduced margin
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(2.0, 2.0); // Tight spacing
                            
                            // Compact Header
                            ui.label(egui::RichText::new("SELECTION").size(10.0).color(egui::Color32::LIGHT_GRAY));
                            
                            // Large Content
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("{}", selection_count)).color(egui::Color32::WHITE).strong().size(18.0));
                                ui.label(egui::RichText::new("cells").size(12.0).color(egui::Color32::GRAY));
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("{:.2}", selection_sum)).color(egui::Color32::GREEN).strong().size(22.0));
                                ui.label(egui::RichText::new("kWh").size(14.0).color(egui::Color32::GREEN));
                            });
                        });
                });
        }
    }

    // Clear selection if clicked outside
    if ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary)) && !ui.ctx().is_using_pointer() {
        state.selection_start = None;
        state.selection_end = None;
        state.is_dragging = false; 
    }
}
