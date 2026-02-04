//! PGE Usage Analyzer
#![deny(warnings)]
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
    resize_state: ui::WindowResizeState,
    last_sync_pos: Option<egui::Pos2>,
    last_sync_size: Option<egui::Vec2>,
    last_sync_maximized: bool,
    sync_timer: f32,
}

impl Default for PgeAnalyzerApp {
    fn default() -> Self {
        let config = Config::load().unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load config, using defaults: {}", e);
            Config::default()
        });

        let data_dir = config.get_data_dir();

        let mut heatmap_state = charts::HeatmapState::default();
        if let Some(ref p) = config.ui.heatmap_palette {
            heatmap_state.palette = charts::HeatmapPalette::from_name(p);
        }

        Self {
            config: config.clone(),
            electric_data: None,
            gas_data: None,
            current_view: ChartView::from_str(&config.ui.default_chart),
            error_message: None,
            data_dir,
            heatmap_state,
            resize_state: ui::WindowResizeState::new(),
            last_sync_pos: None,
            last_sync_size: None,
            last_sync_maximized: false,
            sync_timer: 0.0,
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

        let mut heatmap_state = charts::HeatmapState::default();
        if let Some(ref p) = config.ui.heatmap_palette {
            heatmap_state.palette = charts::HeatmapPalette::from_name(p);
        }

        let mut app = Self {
            config: config.clone(),
            electric_data: None,
            gas_data: None,
            current_view: ChartView::from_str(&config.ui.default_chart),
            error_message: None,
            data_dir: config.get_data_dir(),
            heatmap_state,
            resize_state: ui::WindowResizeState::new(),
            last_sync_pos: None,
            last_sync_size: None,
            last_sync_maximized: config.window.maximized,
            sync_timer: 0.0,
        };

        // Try to auto-load data
        app.auto_load_data();

        app
    }

    fn auto_load_data(&mut self) {
        // 1. Load electric CSVs
        let electric_files = data::autodetect_csv_files(&self.data_dir, "pge_electric");
        if !electric_files.is_empty() {
            // If the user has a "last electric file" in config, we might want to prioritize it?
            // User said: "In case the date/time ranges are overlapping only take the data from the first file."
            // We'll use alphabetical order for now, or if they have a last used file, we could put it first.
            // Let's just use all matching files in the directory.
            if let Err(e) = self.load_electric_data(&electric_files[0]) {
                eprintln!("Failed to auto-load electric data: {}", e);
            }
        }

        // 2. Load gas CSVs
        let gas_files = data::autodetect_csv_files(&self.data_dir, "pge_natural_gas");
        if !gas_files.is_empty() {
            if let Err(e) = self.load_gas_data(&gas_files[0]) {
                eprintln!("Failed to auto-load gas data: {}", e);
            }
        }
    }

    fn load_electric_data(&mut self, primary_path: &Path) -> Result<()> {
        let parent_dir = primary_path.parent().unwrap_or(Path::new("."));
        let files = data::autodetect_csv_files(parent_dir, "pge_electric");

        // Ensure primary_path is first in the list if it's among the matches
        let mut sorted_files = vec![primary_path.to_path_buf()];
        for f in files {
            if f != primary_path {
                sorted_files.push(f);
            }
        }

        let mut contents = Vec::new();
        for path in sorted_files {
            if let Ok(content) = data::read_csv_with_header(&path) {
                contents.push(content);
            }
        }

        self.electric_data = Some(ElectricData::load(&contents)?);

        // Save to config
        self.config.last_electric_file = Some(primary_path.to_path_buf());
        if let Some(parent) = primary_path.parent() {
            self.config.default_data_dir = Some(parent.to_path_buf());
            self.data_dir = parent.to_path_buf();
        }
        let _ = self.config.save();

        Ok(())
    }

    fn load_gas_data(&mut self, primary_path: &Path) -> Result<()> {
        let parent_dir = primary_path.parent().unwrap_or(Path::new("."));
        let files = data::autodetect_csv_files(parent_dir, "pge_natural_gas");

        // Ensure primary_path is first in the list
        let mut sorted_files = vec![primary_path.to_path_buf()];
        for f in files {
            if f != primary_path {
                sorted_files.push(f);
            }
        }

        let mut contents = Vec::new();
        for path in sorted_files {
            if let Ok(content) = data::read_csv_with_header(&path) {
                contents.push(content);
            }
        }

        self.gas_data = Some(GasData::load(&contents)?);

        // Save to config
        self.config.last_gas_file = Some(primary_path.to_path_buf());
        if let Some(parent) = primary_path.parent() {
            self.config.default_data_dir = Some(parent.to_path_buf());
            self.data_dir = parent.to_path_buf();
        }
        let _ = self.config.save();

        Ok(())
    }

    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
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

        ui.add_space(20.0);
        ui.label(egui::RichText::new("Views:").strong().size(crate::ui::styles::SIDEBAR_SECTION_SIZE).color(ui.visuals().text_color()));

        let views = [
            (ChartView::DailyKwh, "📈 Daily Usage"),
            (ChartView::DailyHeatmap, "⚡ Daily Heatmap"),
            (ChartView::CostHeatmap, "💰 Cost Heatmap"),
            (ChartView::WeekdayHeatmap, "📊 Weekday Avg"),
            (ChartView::HourlyProfile, "🕒 Hourly Profile"),
            (ChartView::ExportSparklines, "☀ Solar Export"),
            (ChartView::GasDaily, "🔥 Gas Usage"),
        ];

        for (view, label) in views {
            if ui.selectable_label(self.current_view == view, label).clicked() {
                self.current_view = view.clone();
                self.config.ui.default_chart = view.to_string();
                let _ = self.config.save();
            }
        }

        ui.add_space(20.0);
        ui.label(egui::RichText::new("Preferences:").strong().size(crate::ui::styles::SIDEBAR_SECTION_SIZE).color(ui.visuals().text_color()));

        let mut dark_mode = self.config.ui.dark_mode.unwrap_or(false);
        if ui.checkbox(&mut dark_mode, "🌙 Dark Mode").changed() {
            self.config.ui.dark_mode = Some(dark_mode);
            let _ = self.config.save();
            ui::apply_custom_style(ui.ctx(), Some(dark_mode));
        }

        ui.add_space(8.0);
        ui.label(egui::RichText::new("Heatmap Palette:").size(12.0).color(ui.visuals().text_color().gamma_multiply(0.8)));

        egui::ComboBox::from_id_salt("palette_sel")
            .selected_text(self.heatmap_state.palette.name())
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for p in charts::HeatmapPalette::all() {
                    if ui.selectable_value(&mut self.heatmap_state.palette, *p, p.name()).clicked() {
                        self.config.ui.heatmap_palette = Some(p.name().to_string());
                        let _ = self.config.save();
                    }
                }
            });
    }

    fn render_main_content(&mut self, ui: &mut egui::Ui) {
        if let Some(ref error) = self.error_message {
            ui.colored_label(egui::Color32::RED, error);
            ui.separator();
        }

        match self.current_view {
            ChartView::DailyKwh => {
                if let Some(ref data) = self.electric_data {
                    ui.heading("Daily kWh");
                    ui::components::Card::new().show(ui, |ui| {
                        charts::render_daily_kwh(ui, data);
                    });
                } else {
                    ui.label("No electric data loaded. Please load a CSV file.");
                }
            }
            ChartView::WeekdayHeatmap => {
                if let Some(ref data) = self.electric_data {
                    ui::components::Card::new().show(ui, |ui| {
                        charts::render_weekday_heatmap(ui, data, &mut self.heatmap_state);
                    });
                } else {
                    ui.label("No electric data loaded. Please load a CSV file.");
                }
            }
            ChartView::DailyHeatmap => {
                if let Some(ref data) = self.electric_data {
                    ui::components::Card::new().show(ui, |ui| {
                        charts::render_daily_heatmap(ui, data, &mut self.heatmap_state);
                    });
                } else {
                    ui.label("No electric data loaded. Please load a CSV file.");
                }
            }
            ChartView::CostHeatmap => {
                if let Some(ref data) = self.electric_data {
                    ui::components::Card::new().show(ui, |ui| {
                        charts::render_cost_heatmap(ui, data, &mut self.heatmap_state);
                    });
                } else {
                    ui.label("No electric data loaded. Please load a CSV file.");
                }
            }
            ChartView::HourlyProfile => {
                if let Some(ref data) = self.electric_data {
                    ui.heading("Average Daily Profile (Mean kWh by Hour)");
                    ui::components::Card::new().show(ui, |ui| {
                        charts::render_hourly_profile(ui, data);
                    });
                } else {
                    ui.label("No electric data loaded. Please load a CSV file.");
                }
            }
            ChartView::ExportSparklines => {
                if let Some(ref data) = self.electric_data {
                    ui::components::Card::new().show(ui, |ui| {
                        charts::render_export_sparklines(ui, data, &mut self.heatmap_state);
                    });
                } else {
                    ui.label("No electric data loaded. Please load a CSV file.");
                }
            }
            ChartView::GasDaily => {
                if let Some(ref data) = self.gas_data {
                    ui.heading("Gas: Daily Usage (USD)");
                    ui::components::Card::new().show(ui, |ui| {
                        charts::render_gas_daily(ui, data);
                    });
                } else {
                    ui.label("No gas data loaded. Please load a CSV file.");
                }
            }
        }
    }

    fn handle_window_persistence(&mut self, ctx: &egui::Context) {
        let (pos, size, maximized) = ctx.input(|i| {
            let info = i.viewport();
            (
                info.outer_rect.map(|r| r.min),
                info.inner_rect.map(|r| r.size()),
                info.maximized.unwrap_or(false),
            )
        });

        let mut changed = false;

        if maximized != self.last_sync_maximized {
            self.last_sync_maximized = maximized;
            self.config.window.maximized = maximized;
            changed = true;
        }

        // Only track size/pos if not maximized
        if !maximized {
            if pos != self.last_sync_pos {
                if let Some(p) = pos {
                    // Filter out coordinate junk (e.g. -32000 on minimize)
                    if p.x > -10000.0 && p.y > -10000.0 {
                        self.last_sync_pos = pos;
                        self.config.window.x = Some(p.x);
                        self.config.window.y = Some(p.y);
                        changed = true;
                    }
                }
            }

            if size != self.last_sync_size {
                if let Some(s) = size {
                    if s.x > 10.0 && s.y > 10.0 {
                        self.last_sync_size = size;
                        self.config.window.width = s.x;
                        self.config.window.height = s.y;
                        changed = true;
                    }
                }
            }
        }

        if changed {
            self.sync_timer = 1.0; // Reset settle timer (1 second)
        } else if self.sync_timer > 0.0 {
            self.sync_timer -= ctx.input(|i| i.stable_dt);
            if self.sync_timer <= 0.0 {
                let _ = self.config.save();
                self.sync_timer = 0.0;
            }
            ctx.request_repaint(); // Keep updating until timer hits zero
        }
    }
}

impl eframe::App for PgeAnalyzerApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0] // Transparent
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Enforce theme on startup or if egui resets it (e.g. system theme change)
        if ctx.style().visuals.window_fill != egui::Color32::TRANSPARENT {
            ui::apply_custom_style(ctx, self.config.ui.dark_mode);
        }

        // 1. Paint the main window background manually on the background layer
        let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
        let rounding = if is_maximized { 0.0 } else { ui::styles::WINDOW_ROUNDING };
        let bg_color = ui::actual_window_background(ctx);

        ctx.layer_painter(egui::LayerId::background()).rect_filled(
            ctx.screen_rect(),
            egui::Rounding::same(rounding),
            bg_color,
        );

        // 2. Add a border stroke if not maximized
        if !is_maximized {
            ctx.layer_painter(egui::LayerId::background()).rect_stroke(
                ctx.screen_rect(),
                egui::Rounding::same(rounding),
                ctx.style().visuals.widgets.noninteractive.bg_stroke,
            );
        }

        ui::handle_window_resize(ctx, &mut self.resize_state);

        // --- Custom Panels (Transparent) ---
        ui::render_title_bar(ctx, "PG&E Usage Analyzer");

        egui::SidePanel::left("sidebar_panel")
            .frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT))
            .resizable(false)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.add_space(20.0); // Space for top rounding
                egui::Frame::none()
                    .inner_margin(egui::Margin::symmetric(10.0, 0.0))
                    .show(ui, |ui| {
                        self.render_sidebar(ui);
                    });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none()
                .fill(egui::Color32::TRANSPARENT)
                .inner_margin(egui::Margin {
                    left: 10.0,
                    right: 10.0,
                    top: 0.0,
                    bottom: 10.0,
                }))
            .show(ctx, |ui| {
                ui.add_space(10.0);
                self.render_main_content(ui);
            });

        self.resize_state.apply_cursor(ctx);
        self.handle_window_persistence(ctx);
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
        .with_decorations(false)
        .with_transparent(true)
        .with_min_inner_size([400.0, 300.0]);

    if let (Some(x), Some(y)) = (config.window.x, config.window.y) {
        viewport = viewport.with_position([x, y]);
    }

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
