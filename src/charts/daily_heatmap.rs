use crate::data::ElectricData;
use egui::Ui;
use crate::charts::colormap::get_viridis_color;

pub fn render_daily_heatmap(ui: &mut Ui, data: &ElectricData) {
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
    
    let cell_width = 25.0;
    let cell_height = 25.0; // Square cells match Plotly better
    
    egui::ScrollArea::both().show(ui, |ui| {
        ui.horizontal(|ui| {
            // Y-axis labels (dates)
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0); // Remove vertical spacing
                ui.add_space(20.0); // Space for hour labels (matches X-axis header height)
                for date in &dates {
                    let (rect, _response) = ui.allocate_exact_size(
                        egui::vec2(100.0, cell_height),
                        egui::Sense::hover(),
                    );
                    
                    ui.painter().text(
                        rect.left_center() + egui::vec2(5.0, 0.0), // Left align with padding
                        egui::Align2::LEFT_CENTER,
                        date,
                        egui::FontId::proportional(12.0), // Smaller font for dates
                        ui.visuals().text_color(),
                    );
                }
            });
            
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0); // Remove vertical spacing between rows
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
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0); // Remove horizontal spacing
                        for (hour, &val) in day_data.iter().enumerate() {
                            let color = get_viridis_color(val, 0.0, max_val);
                            
                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(cell_width, cell_height),
                                egui::Sense::hover(),
                            );
                            
                            ui.painter().rect_filled(rect, 0.0, color);
                            // Stroke removed for smooth look
                            
                            if response.hovered() {
                                response.on_hover_ui(|ui| {
                                    ui.label(format!(
                                        "{}, {}:00\n{:.2} kWh",
                                        dates[day_idx], hour, val
                                    ));
                                });
                            }
                        }
                    });
                }
            });
        });
    });
}


