use crate::data::ElectricData;
use egui::Ui;
use crate::charts::colormap::get_viridis_color;

const WEEKDAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

pub fn render_weekday_heatmap(ui: &mut Ui, data: &ElectricData) {
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
    let cell_width = 30.0;
    let cell_height = 30.0;
    
    egui::ScrollArea::both().show(ui, |ui| {
        ui.horizontal(|ui| {
            // Y-axis labels (weekdays)
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0); // Remove vertical spacing
                ui.add_space(20.0); // Space for hour labels (matches X-axis header height)
                for weekday in WEEKDAYS.iter() {
                    let (rect, _response) = ui.allocate_exact_size(
                        egui::vec2(80.0, cell_height),
                        egui::Sense::hover(),
                    );
                    
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        *weekday,
                        egui::FontId::proportional(14.0),
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
                                egui::Sense::hover(),
                            );
                            
                            ui.painter().rect_filled(rect, 0.0, color);
                            // Stroke removed for smooth look
                            
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
            });
        });
    });
}


