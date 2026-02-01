//! PGE Usage Analyzer
//!
//! A GUI application built with eframe/egui to analyze and visualize
//! PG&E electric and natural gas usage data exported from their customer portal.

mod data;
mod charts;
mod ui;
mod config;

use anyhow::Result;
use data::{ElectricData, GasData};
use std::path::{Path, PathBuf};
use ui::ChartView;
use config::Config;

struct PgeAnalyzerApp {
    config: Config,
    electric_data: Option<ElectricData>,
    gas_data: Option<GasData>,
    current_view: ChartView,
    error_message: Option<String>,
    data_dir: PathBuf,
    heatmap_state: charts::HeatmapState,
    first_frame: bool,
    resize_state: ui::WindowResizeState,
}

impl Default for PgeAnalyzerApp {
    fn default() -> Self {
        let config = Config::load().unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load config, using defaults: {}", e);
            Config::default()
        });

        let data_dir = config.get_data_dir();

        Self {
            config: config.clone(),
            electric_data: None,
            gas_data: None,
            current_view: ChartView::from_str(&config.ui.default_chart),
            error_message: None,
            data_dir,
            heatmap_state: charts::HeatmapState::default(),
            first_frame: true,
            resize_state: ui::WindowResizeState::new(),
        }
    }
}

impl PgeAnalyzerApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let config = Config::load().unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load config, using defaults: {}", e);
            Config::default()
        });

        ui::apply_custom_style(&cc.egui_ctx, config.ui.dark_mode);

        let mut app = Self {
            config: config.clone(),
            electric_data: None,
            gas_data: None,
            current_view: ChartView::from_str(&config.ui.default_chart),
            error_message: None,
            data_dir: config.get_data_dir(),
            heatmap_state: charts::HeatmapState::default(),
            first_frame: true,
            resize_state: ui::WindowResizeState::new(),
        };

        // Try to auto-load data
        app.auto_load_data();

        app
    }

    fn auto_load_data(&mut self) {
        // 1. Try last used electric CSV from config
        let electric_path = self.config.last_electric_file.clone()
            .or_else(|| data::autodetect_csv(&self.data_dir, "pge_electric"));

        if let Some(path) = electric_path {
            if let Err(e) = self.load_electric_data(&path) {
                // If it fails, maybe it was deleted or moved, don't show error if it was just an auto-load
                // but if it was in config, maybe we should clear it?
                // For now, just log and continue to gas.
                eprintln!("Failed to auto-load electric data from {:?}: {}", path, e);
            }
        }

        // 2. Try last used gas CSV from config
        let gas_path = self.config.last_gas_file.clone()
            .or_else(|| data::autodetect_csv(&self.data_dir, "pge_natural_gas"));

        if let Some(path) = gas_path {
            if let Err(e) = self.load_gas_data(&path) {
                eprintln!("Failed to auto-load gas data from {:?}: {}", path, e);
            }
        }
    }

    fn load_electric_data(&mut self, path: &Path) -> Result<()> {
        let csv_content = data::read_csv_with_header(path)?;
        self.electric_data = Some(ElectricData::load(&csv_content)?);

        // Save to config
        self.config.last_electric_file = Some(path.to_path_buf());
        if let Some(parent) = path.parent() {
            self.config.default_data_dir = Some(parent.to_path_buf());
            self.data_dir = parent.to_path_buf();
        }
        let _ = self.config.save();

        Ok(())
    }

    fn load_gas_data(&mut self, path: &Path) -> Result<()> {
        let csv_content = data::read_csv_with_header(path)?;
        self.gas_data = Some(GasData::load(&csv_content)?);

        // Save to config
        self.config.last_gas_file = Some(path.to_path_buf());
        if let Some(parent) = path.parent() {
            self.config.default_data_dir = Some(parent.to_path_buf());
            self.data_dir = parent.to_path_buf();
        }
        let _ = self.config.save();

        Ok(())
    }

    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        //ui.heading("PG&E Usage Analyzer");
        //ui.separator();

        // File loading buttons
        ui.label(egui::RichText::new("Data Files:").strong().size(crate::ui::styles::SIDEBAR_SECTION_SIZE).color(ui.visuals().text_color()));

        if ui.button("📂 Load Electric CSV").clicked() {
            if let Some(path) = data::select_csv_file() {
                match self.load_electric_data(&path) {
                    Ok(_) => {
                        self.error_message = None;
                    }
                    Err(e) => {
                        self.error_message = Some(format!("⚠️ Failed to Load Electric Data\n\n{}", e));
                    }
                }
            } else {
                self.error_message = Some("ℹ️ No file selected\n\nPlease select a PG&E electric usage CSV file to continue.".to_string());
            }
        }

        if ui.button("📂 Load Gas CSV").clicked() {
            if let Some(path) = data::select_csv_file() {
                match self.load_gas_data(&path) {
                    Ok(_) => {
                        self.error_message = None;
                    }
                    Err(e) => {
                        self.error_message = Some(format!("⚠️ Failed to Load Gas Data\n\n{}", e));
                    }
                }
            } else {
                self.error_message = Some("ℹ️ No file selected\n\nPlease select a PG&E gas usage CSV file to continue.".to_string());
            }
        }

        ui.separator();

        // Data status
        ui.label(egui::RichText::new("Status:").strong().size(crate::ui::styles::SIDEBAR_SECTION_SIZE).color(ui.visuals().text_color()));
        if self.electric_data.is_some() {
            ui.label(egui::RichText::new("✔ Electric data loaded")
                .monospace()
                .color(ui::styles::status_green()));
        } else {
            ui.label(egui::RichText::new("× No electric data")
                .monospace()
                .color(ui::styles::status_red()));
        }

        if self.gas_data.is_some() {
            ui.label(egui::RichText::new("✔ Gas data loaded")
                .monospace()
                .color(ui::styles::status_green()));
        } else {
            ui.label(egui::RichText::new("× No gas data")
                .monospace()
                .color(ui::styles::status_red()));
        }

        ui.separator();

        // Chart selection
        ui.label(egui::RichText::new("Charts:").strong().size(crate::ui::styles::SIDEBAR_SECTION_SIZE).color(ui.visuals().text_color()));

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
                    self.config.ui.default_chart = view.to_string();
                    let _ = self.config.save();
                }
            });
        }

        ui.separator();

        // Theme Toggle
        ui.label(egui::RichText::new("Appearance:").strong().size(crate::ui::styles::SIDEBAR_SECTION_SIZE).color(ui.visuals().text_color()));

        let mut dark_mode = self.config.ui.dark_mode.unwrap_or(false);
        if ui.checkbox(&mut dark_mode, "🌙 Dark Mode").changed() {
            self.config.ui.dark_mode = Some(dark_mode);
            let _ = self.config.save();
            ui::apply_custom_style(ui.ctx(), Some(dark_mode));
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
            ChartView::ExportSparklines => {
                if let Some(ref data) = self.electric_data {
                    charts::render_export_sparklines(ui, data, &mut self.heatmap_state);
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

    fn handle_input(&mut self, ctx: &egui::Context) {
        // Handle global zoom with Ctrl + Mouse Wheel
        let zoom_delta = ctx.input(|i| i.zoom_delta());
        if zoom_delta != 1.0 {
            ctx.set_pixels_per_point(ctx.pixels_per_point() * zoom_delta);
        }

        // Handle navigation and other shortcuts
        ctx.input(|i| {
            if i.key_pressed(egui::Key::ArrowUp) {
                let all_views = ChartView::all();
                if let Some(pos) = all_views.iter().position(|&v| v == self.current_view) {
                    let new_pos = if pos == 0 { all_views.len() - 1 } else { pos - 1 };
                    self.current_view = all_views[new_pos];
                    self.config.ui.default_chart = self.current_view.to_string();
                    let _ = self.config.save();
                }
            }
            if i.key_pressed(egui::Key::ArrowDown) {
                let all_views = ChartView::all();
                if let Some(pos) = all_views.iter().position(|&v| v == self.current_view) {
                    let new_pos = (pos + 1) % all_views.len();
                    self.current_view = all_views[new_pos];
                    self.config.ui.default_chart = self.current_view.to_string();
                    let _ = self.config.save();
                }
            }
            if i.key_pressed(egui::Key::Escape) {
                self.heatmap_state.selection_start = None;
                self.heatmap_state.selection_end = None;
            }
        });
    }
}

impl eframe::App for PgeAnalyzerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle frameless window resize (must be first)
        ui::handle_window_resize(ctx, &mut self.resize_state);

        if self.first_frame {
            ui::apply_custom_style(ctx, self.config.ui.dark_mode);
            self.first_frame = false;
        }

        // Responsive scaling: adjust UI scale based on window WIDTH only
        // Reference: 1400 physical pixels width = 1.0 scale
        // Height is ignored since charts scroll vertically
        let current_ppp = ctx.pixels_per_point();
        let screen_rect = ctx.screen_rect();

        // Convert logical pixels to physical pixels (stable reference)
        let physical_width = screen_rect.width() * current_ppp;

        // Calculate scale factor based on physical width only
        let auto_scale = physical_width / 1400.0;

        // Clamp scale between 0.6 and 2.0 to avoid extremes
        let base_scale = auto_scale.clamp(0.6, 2.0);

        // Apply user's font_scale preference on top
        let final_scale = base_scale * self.config.ui.font_scale;

        // Only update if scale changed significantly (avoid constant redraws)
        if (final_scale - current_ppp).abs() > 0.05 {
            ctx.set_pixels_per_point(final_scale);
        }

        // Save window size every frame
        let available = ctx.available_rect();
        self.config.window.width = available.width() as f32;
        self.config.window.height = available.height() as f32;
        let _ = self.config.save();

        self.handle_input(ctx);

        // Custom title bar for frameless window
        ui::render_title_bar(ctx, "PG&E Usage Analyzer");

        egui::SidePanel::left("sidebar")
            .min_width(200.0)
            .show(ctx, |ui| {
                self.render_sidebar(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui::components::Card::new().show(ui, |ui| {
                    self.render_main_content(ui);
                });
            });
        });

        // Apply resize cursor at end of frame (overrides UI cursors)
        self.resize_state.apply_cursor(ctx);
    }
}

#[cfg(target_os = "windows")]
#[no_mangle]
pub extern "system" fn WinMain(
    _hinstance: *mut std::ffi::c_void,
    _hprevinstance: *mut std::ffi::c_void,
    _lpstrcmd: *mut u16,
    _ncmdshow: i32,
) -> i32 {
    match run_app() {
        Ok(_) => 0,
        Err(e) => {
            // In a GUI app, we can't easily show errors, so just return error code
            eprintln!("Application error: {}", e);
            1
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    run_app()
}

fn run_app() -> Result<(), eframe::Error> {
    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load config, using defaults: {}", e);
        Config::default()
    });

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([config.window.width, config.window.height])
        .with_title("PG&E Usage Analyzer")
        .with_decorations(false)  // Frameless window - custom title bar
        .with_min_inner_size([400.0, 300.0]);

    // Restore maximized state
    if config.window.maximized {
        viewport = viewport.with_maximized(true);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "PG&E Usage Analyzer",
        options,
        Box::new(|cc| Ok(Box::new(PgeAnalyzerApp::new(cc)))),
    )
}
