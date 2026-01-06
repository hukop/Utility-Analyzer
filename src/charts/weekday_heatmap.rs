use crate::data::ElectricData;
use egui::Ui;
use crate::charts::colormap::get_viridis_color;
use crate::charts::HeatmapState;

const WEEKDAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

pub fn render_weekday_heatmap(ui: &mut Ui, data: &ElectricData, state: &mut HeatmapState) {
    let averages = data.weekday_hour_average();
    
    // Find min/max for color scaling
    let mut min_val = f64::MAX;
    let mut max_val = f64::MIN;
    
    for weekday in 0..7 {
        for hour in 0..24 {
            let val = averages[weekday][hour];
            min_val = min_val.min(val);
            max_val = max_val.max(val);
        }
    }
    
    ui.heading("Average kWh by Weekday and Hour");
    
    // Create a custom heatmap using rectangles
    // Feature 3: Heatmaps should be wider
    let cell_width = 35.0; // Increased from 30.0
    let cell_height = 30.0;
    
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
        
        for d in min_day..=max_day {
            if d < 7 {
                for h in min_hour..=max_hour {
                    if h < 24 {
                        selection_sum += averages[d][h];
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
            // Y-axis labels (weekdays)
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0); // Remove vertical spacing
                ui.add_space(20.0); // Space for hour labels (matches X-axis header height)
                for (idx, weekday) in WEEKDAYS.iter().enumerate() {
                    let (rect, _response) = ui.allocate_exact_size(
                        egui::vec2(80.0, cell_height),
                        egui::Sense::hover(),
                    );
                    
                    // Feature 1: Mark weekends
                    let is_weekend = idx >= 5; // Saturday(5), Sunday(6)
                    let text_color = if is_weekend {
                        ui.visuals().text_color()
                    } else {
                        ui.visuals().weak_text_color()
                    };

                    if is_weekend {
                         ui.painter().rect_stroke(
                            rect.shrink(1.0),
                            0.0,
                            egui::Stroke::new(1.0, egui::Color32::from_gray(100))
                        );
                    }
                    
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        *weekday,
                        egui::FontId::proportional(14.0),
                        text_color,
                    );
                }
            });
            
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0); // Remove vertical spacing between rows
                // X-axis labels (hours)
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                    for hour in 0..24 {
                        let label = if hour % 2 == 0 {
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
                for weekday in 0..7 {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);


                        for hour in 0..24 {
                            let val = averages[weekday][hour];
                            let color = get_viridis_color(val, 0.0, max_val);
                            
                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(cell_width, cell_height),
                                egui::Sense::drag(),
                            );
                            
                             // Handle selection interactions
                            if response.drag_started() {
                                state.selection_start = Some((weekday, hour));
                                state.selection_end = Some((weekday, hour));
                                state.is_dragging = true;
                            } else if response.drag_stopped() { 
                                state.is_dragging = false;
                            }
                            
                            if state.is_dragging {
                                if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
                                    if rect.contains(pointer_pos) {
                                         state.selection_end = Some((weekday, hour));
                                    }
                                }
                            }
                            
                            if let Some(((min_d, max_d), (min_h, max_h))) = selected_indices {
                                if weekday >= min_d && weekday <= max_d && hour >= min_h && hour <= max_h {
                                    selection_rect = Some(selection_rect.map_or(rect, |r| r.union(rect)));
                                }
                            }

                            ui.painter().rect_filled(rect, 0.0, color);

                            // Feature 1: Mark weekends (border style)
                            // Saturday is index 5, Sunday is index 6
                            if weekday == 5 {
                                 let stroke = egui::Stroke::new(1.0, egui::Color32::from_white_alpha(50));
                                 // Top border for Saturday
                                 ui.painter().line_segment(
                                     [rect.left_top(), rect.right_top()], 
                                     stroke
                                 );
                            } else if weekday == 6 {
                                 let stroke = egui::Stroke::new(1.0, egui::Color32::from_white_alpha(50));
                                 // Bottom border for Sunday
                                 ui.painter().line_segment(
                                     [rect.left_bottom(), rect.right_bottom()], 
                                     stroke
                                 );
                            }

                            // Selection highlight removed (drawing single rect at end)
                            
                            if response.hovered() {
                                response.on_hover_ui(|ui| {
                                    ui.label(format!(
                                        "{}, {}:00\n{:.2} kWh",
                                        WEEKDAYS[weekday], hour, val
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
            
            egui::Area::new("weekday_heatmap_selection".into())
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


