mod data;
mod charts;
mod ui;

use anyhow::Result;
use data::{ElectricData, GasData};
use std::path::PathBuf;
use ui::ChartView;

struct PgeAnalyzerApp {
    electric_data: Option<ElectricData>,
    gas_data: Option<GasData>,
    current_view: ChartView,
    error_message: Option<String>,
    data_dir: PathBuf,
    heatmap_state: charts::HeatmapState,
}

impl Default for PgeAnalyzerApp {
    fn default() -> Self {
        let data_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        
        Self {
            electric_data: None,
            gas_data: None,
            current_view: ChartView::DailyKwh,
            error_message: None,
            data_dir,
            heatmap_state: charts::HeatmapState::default(),
        }
    }
}

impl PgeAnalyzerApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        ui::apply_custom_style(&cc.egui_ctx);
        
        let mut app = Self::default();
        
        // Try to auto-load data
        app.auto_load_data();
        
        app
    }
    
    fn auto_load_data(&mut self) {
        // Try to auto-detect electric CSV
        if let Some(electric_path) = data::autodetect_csv(&self.data_dir, "pge_electric") {
            match self.load_electric_data(&electric_path) {
                Ok(_) => {
                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(format!("Error loading electric data: {}", e));
                }
            }
        }
        
        // Try to auto-detect gas CSV
        if let Some(gas_path) = data::autodetect_csv(&self.data_dir, "pge_natural_gas") {
            match self.load_gas_data(&gas_path) {
                Ok(_) => {
                    if self.error_message.is_none() {
                        self.error_message = None;
                    }
                }
                Err(e) => {
                    let msg = format!("Error loading gas data: {}", e);
                    self.error_message = Some(
                        self.error_message
                            .as_ref()
                            .map(|m| format!("{}\n{}", m, msg))
                            .unwrap_or(msg),
                    );
                }
            }
        }
    }
    
    fn load_electric_data(&mut self, path: &PathBuf) -> Result<()> {
        let csv_content = data::read_csv_with_header(path)?;
        self.electric_data = Some(ElectricData::load(&csv_content)?);
        Ok(())
    }
    
    fn load_gas_data(&mut self, path: &PathBuf) -> Result<()> {
        let csv_content = data::read_csv_with_header(path)?;
        self.gas_data = Some(GasData::load(&csv_content)?);
        Ok(())
    }
    
    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.heading("PG&E Usage Analyzer");
        ui.separator();
        
        // File loading buttons
        ui.label("Data Files:");
        
        if ui.button("📂 Load Electric CSV").clicked() {
            if let Some(path) = data::select_csv_file() {
                match self.load_electric_data(&path) {
                    Ok(_) => {
                        self.error_message = None;
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Error: {}", e));
                    }
                }
            }
        }
        
        if ui.button("📂 Load Gas CSV").clicked() {
            if let Some(path) = data::select_csv_file() {
                match self.load_gas_data(&path) {
                    Ok(_) => {
                        self.error_message = None;
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Error: {}", e));
                    }
                }
            }
        }
        
        ui.separator();
        
        // Data status
        ui.label("Status:");
        if self.electric_data.is_some() {
            ui.label("✓ Electric data loaded");
        } else {
            ui.label("✗ No electric data");
        }
        
        if self.gas_data.is_some() {
            ui.label("✓ Gas data loaded");
        } else {
            ui.label("✗ No gas data");
        }
        
        ui.separator();
        
        // Chart selection
        ui.label("Charts:");
        
        for view in ChartView::all() {
            let is_selected = self.current_view == view;
            
            // Disable gas chart if no gas data
            let enabled = if view == ChartView::GasDaily {
                self.gas_data.is_some()
            } else {
                self.electric_data.is_some()
            };
            
            ui.add_enabled_ui(enabled, |ui| {
                let mut text = egui::RichText::new(view.name());
                if is_selected {
                    text = text.color(egui::Color32::WHITE);
                }
                if ui.selectable_label(is_selected, text).clicked() {
                    self.current_view = view;
                }
            });
        }
    }
    
    fn render_main_content(&mut self, ui: &mut egui::Ui) {
        // Show error message if any
        if let Some(ref error) = self.error_message {
            ui.colored_label(egui::Color32::RED, error);
            ui.separator();
        }
        
        // Render current chart
        match self.current_view {
            ChartView::DailyKwh => {
                if let Some(ref data) = self.electric_data {
                    ui.heading("Daily kWh");
                    charts::render_daily_kwh(ui, data);
                } else {
                    ui.label("No electric data loaded. Please load a CSV file.");
                }
            }
            ChartView::WeekdayHeatmap => {
                if let Some(ref data) = self.electric_data {
                    charts::render_weekday_heatmap(ui, data, &mut self.heatmap_state);
                } else {
                    ui.label("No electric data loaded. Please load a CSV file.");
                }
            }
            ChartView::DailyHeatmap => {
                if let Some(ref data) = self.electric_data {
                    charts::render_daily_heatmap(ui, data, &mut self.heatmap_state);
                } else {
                    ui.label("No electric data loaded. Please load a CSV file.");
                }
            }
            ChartView::CostHeatmap => {
                if let Some(ref data) = self.electric_data {
                    charts::render_cost_heatmap(ui, data, &mut self.heatmap_state);
                } else {
                    ui.label("No electric data loaded. Please load a CSV file.");
                }
            }
            ChartView::HourlyProfile => {
                if let Some(ref data) = self.electric_data {
                    ui.heading("Average Daily Profile (Mean kWh by Hour)");
                    charts::render_hourly_profile(ui, data);
                } else {
                    ui.label("No electric data loaded. Please load a CSV file.");
                }
            }
            ChartView::GasDaily => {
                if let Some(ref data) = self.gas_data {
                    ui.heading("Gas: Daily Usage (USD)");
                    charts::render_gas_daily(ui, data);
                } else {
                    ui.label("No gas data loaded. Please load a CSV file.");
                }
            }
        }
    }
}

impl eframe::App for PgeAnalyzerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle global zoom with Ctrl + Mouse Wheel
        let zoom_delta = ctx.input(|i| i.zoom_delta());
        if zoom_delta != 1.0 {
            ctx.set_pixels_per_point(ctx.pixels_per_point() * zoom_delta);
        }

        egui::SidePanel::left("sidebar")
            .min_width(200.0)
            .show(ctx, |ui| {
                self.render_sidebar(ui);
            });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                self.render_main_content(ui);
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_title("PG&E Usage Analyzer"),
        ..Default::default()
    };
    
    eframe::run_native(
        "PG&E Usage Analyzer",
        options,
        Box::new(|cc| Ok(Box::new(PgeAnalyzerApp::new(cc)))),
    )
}
