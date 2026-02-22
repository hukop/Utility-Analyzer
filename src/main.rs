//! PGE Usage Analyzer
#![deny(warnings)]
//!
//! A GUI application built with eframe/egui to analyze and visualize
//! PG&E electric and natural gas usage data exported from their customer portal.

mod charts;
mod config;
mod data;
mod ui;

use anyhow::Result;
use chrono::Utc;
use config::Config;
use data::{DateRangePreset, ElectricData, GasData};
use std::path::{Path, PathBuf};
use ui::{ChartView, HeatmapMetric};

#[cfg(target_os = "windows")]
const USE_CUSTOM_WINDOW_CHROME: bool = false;
#[cfg(not(target_os = "windows"))]
const USE_CUSTOM_WINDOW_CHROME: bool = true;

struct PgeAnalyzerApp {
    config: Config,
    electric_data: Option<ElectricData>,
    gas_data: Option<GasData>,
    current_view: ChartView,
    error_message: Option<String>,
    data_dir: PathBuf,
    heatmap_state: charts::HeatmapState,
    heatmap_metric: HeatmapMetric,
    resize_state: ui::WindowResizeState,
    last_sync_pos: Option<egui::Pos2>,
    last_sync_size: Option<egui::Vec2>,
    last_sync_maximized: bool,
    #[cfg(target_os = "windows")]
    last_sync_native_titlebar_dark: Option<bool>,
    sync_timer: f32,
    zoom_state: charts::ChartZoomState,
    sidebar_collapsed: bool,
    range_preset: DateRangePreset,
    last_export_path: Option<PathBuf>,
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
        let current_view = ChartView::from_str(&config.ui.default_chart);

        Self {
            config,
            electric_data: None,
            gas_data: None,
            current_view,
            error_message: None,
            data_dir,
            heatmap_state,
            heatmap_metric: HeatmapMetric::default(),
            resize_state: ui::WindowResizeState::new(),
            last_sync_pos: None,
            last_sync_size: None,
            last_sync_maximized: false,
            #[cfg(target_os = "windows")]
            last_sync_native_titlebar_dark: None,
            sync_timer: 0.0,
            zoom_state: charts::ChartZoomState::default(),
            sidebar_collapsed: false,
            range_preset: DateRangePreset::default(),
            last_export_path: None,
        }
    }
}

impl PgeAnalyzerApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();
        ui::apply_custom_style(&cc.egui_ctx, app.config.ui.dark_mode);

        // Re-read maximized state for viewport
        app.last_sync_maximized = app.config.window.maximized;

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

    fn load_electric_data(&mut self, path: &Path) -> Result<()> {
        let contents = self.read_merged_csvs(path, "pge_electric")?;
        self.electric_data = Some(ElectricData::load(&contents)?);
        self.save_data_paths(path, true);
        Ok(())
    }

    fn load_gas_data(&mut self, path: &Path) -> Result<()> {
        let contents = self.read_merged_csvs(path, "pge_natural_gas")?;
        self.gas_data = Some(GasData::load(&contents)?);
        self.save_data_paths(path, false);
        Ok(())
    }

    /// Reads metadata-stripped contents for a primary file and all siblings matching the pattern.
    fn read_merged_csvs(&self, primary_path: &Path, pattern: &str) -> Result<Vec<String>> {
        let parent_dir = primary_path.parent().unwrap_or(Path::new("."));
        let files = data::autodetect_csv_files(parent_dir, pattern);

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
        Ok(contents)
    }

    /// Updates configuration with last used file and data directory.
    fn save_data_paths(&mut self, path: &Path, is_electric: bool) {
        if is_electric {
            self.config.last_electric_file = Some(path.to_path_buf());
        } else {
            self.config.last_gas_file = Some(path.to_path_buf());
        }

        if let Some(parent) = path.parent() {
            self.config.default_data_dir = Some(parent.to_path_buf());
            self.data_dir = parent.to_path_buf();
        }
        let _ = self.config.save();
    }

    fn pick_and_load_electric(&mut self) {
        if let Some(path) = data::select_csv_file() {
            match self.load_electric_data(&path) {
                Ok(_) => {
                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to load electric data: {}", e));
                }
            }
        } else {
            self.error_message =
                Some("No file selected. Please choose an electric usage CSV file.".to_string());
        }
    }

    fn pick_and_load_gas(&mut self) {
        if let Some(path) = data::select_csv_file() {
            match self.load_gas_data(&path) {
                Ok(_) => {
                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to load gas data: {}", e));
                }
            }
        } else {
            self.error_message =
                Some("No file selected. Please choose a gas usage CSV file.".to_string());
        }
    }

    fn set_view(&mut self, view: ChartView) {
        self.current_view = view;
        self.config.ui.default_chart = view.to_string();
        let _ = self.config.save();
    }

    fn view_icon(view: ChartView) -> (&'static str, egui::Color32, &'static str) {
        match view {
            ChartView::DailyKwh => ("📈", egui::Color32::from_rgb(80, 180, 255), "Daily Usage"),
            ChartView::DailyHeatmap => {
                ("⚡", egui::Color32::from_rgb(255, 210, 80), "Daily Heatmap")
            }
            ChartView::WeekdayHeatmap => {
                ("📊", egui::Color32::from_rgb(120, 220, 170), "Weekday Avg")
            }
            ChartView::HourlyProfile => (
                "🕒",
                egui::Color32::from_rgb(190, 170, 255),
                "Hourly Profile",
            ),
            ChartView::ExportSparklines => {
                ("☀", egui::Color32::from_rgb(255, 190, 70), "Solar Export")
            }
            ChartView::GasDaily => ("🔥", egui::Color32::from_rgb(255, 120, 90), "Gas Usage"),
        }
    }

    fn is_range_enabled_view(&self) -> bool {
        matches!(
            self.current_view,
            ChartView::DailyHeatmap | ChartView::ExportSparklines
        )
    }

    fn visible_range_label(&self) -> Option<String> {
        let data = self.electric_data.as_ref()?;
        match self.current_view {
            ChartView::DailyHeatmap => {
                if self.heatmap_metric == HeatmapMetric::Cost {
                    let (_, _, _, meta) = data.daily_hour_cost_heatmap_filtered(self.range_preset);
                    meta.first()
                        .zip(meta.last())
                        .map(|(start, end)| format!("{} to {}", start.date_key, end.date_key))
                } else {
                    let (_, _, _, meta) = data.daily_hour_heatmap_filtered(self.range_preset);
                    meta.first()
                        .zip(meta.last())
                        .map(|(start, end)| format!("{} to {}", start.date_key, end.date_key))
                }
            }
            ChartView::ExportSparklines => {
                let (_, _, _, _, meta) = data.daily_daytime_export_data_filtered(self.range_preset);
                meta.first()
                    .zip(meta.last())
                    .map(|(start, end)| format!("{} to {}", start.date_key, end.date_key))
            }
            _ => None,
        }
    }

    fn export_visible_daily_csv(&self) -> Result<PathBuf> {
        let Some(data) = self.electric_data.as_ref() else {
            anyhow::bail!("No electric data loaded");
        };

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let preset = self.range_preset.label().to_lowercase();

        let (filename, csv) = match self.current_view {
            ChartView::DailyHeatmap if self.heatmap_metric == HeatmapMetric::Cost => {
                let (_, _, sums, meta) = data.daily_hour_cost_heatmap_filtered(self.range_preset);
                let mut content = String::from("date,total_cost_usd\n");
                for (m, sum) in meta.iter().zip(sums.iter().copied()) {
                    content.push_str(&format!("{},{:.4}\n", m.date_key, sum));
                }
                (
                    format!("pge_daily_cost_{}_{}.csv", preset, timestamp),
                    content,
                )
            }
            ChartView::ExportSparklines => {
                let (_, _, sums, _, meta) =
                    data.daily_daytime_export_data_filtered(self.range_preset);
                let mut content = String::from("date,daytime_export_kwh\n");
                for (m, sum) in meta.iter().zip(sums.iter().copied()) {
                    content.push_str(&format!("{},{:.4}\n", m.date_key, sum));
                }
                (format!("pge_export_{}_{}.csv", preset, timestamp), content)
            }
            _ => {
                let mut content = String::from("date,total_kwh\n");
                for (m, sum) in data.daily_totals_filtered(self.range_preset) {
                    content.push_str(&format!("{},{:.4}\n", m.date_key, sum));
                }
                (
                    format!("pge_daily_kwh_{}_{}.csv", preset, timestamp),
                    content,
                )
            }
        };

        let output_path = self.data_dir.join(filename);
        std::fs::write(&output_path, csv)?;
        Ok(output_path)
    }

    fn render_command_bar(&mut self, ui: &mut egui::Ui) {
        let frame_fill = ui.visuals().widgets.noninteractive.bg_fill;
        let frame_stroke = ui.visuals().widgets.noninteractive.bg_stroke;

        egui::Frame::NONE
            .fill(frame_fill)
            .stroke(frame_stroke)
            .corner_radius(egui::CornerRadius::same(6))
            .inner_margin(egui::Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let collapse_label = if self.sidebar_collapsed {
                        "Expand Sidebar"
                    } else {
                        "Collapse Sidebar"
                    };
                    if ui.small_button(collapse_label).clicked() {
                        self.sidebar_collapsed = !self.sidebar_collapsed;
                    }

                    ui.separator();
                    ui.label(
                        egui::RichText::new("Quick Range")
                            .size(crate::ui::styles::BODY_FONT_SIZE)
                            .strong(),
                    );

                    let range_enabled =
                        self.is_range_enabled_view() && self.electric_data.is_some();
                    ui.add_enabled_ui(range_enabled, |ui| {
                        if self.config.ui.modern_ui {
                            let options: Vec<_> = DateRangePreset::all()
                                .iter()
                                .map(|p| (*p, p.label()))
                                .collect();
                            ui::components::render_segmented_control(ui, &mut self.range_preset, &options);
                        } else {
                            for preset in DateRangePreset::all() {
                                ui.selectable_value(&mut self.range_preset, *preset, preset.label());
                            }
                        }
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let export_enabled =
                            self.electric_data.is_some() && self.is_range_enabled_view();
                        if ui
                            .add_enabled(export_enabled, egui::Button::new("Export Visible CSV"))
                            .clicked()
                        {
                            match self.export_visible_daily_csv() {
                                Ok(path) => {
                                    self.last_export_path = Some(path);
                                    self.error_message = None;
                                }
                                Err(e) => {
                                    self.error_message = Some(format!("Export failed: {}", e));
                                }
                            }
                        }

                        if let Some(path) = &self.last_export_path {
                            ui.label(
                                egui::RichText::new(format!("Last export: {}", path.display()))
                                    .size(11.0)
                                    .color(ui.visuals().text_color().gamma_multiply(0.7)),
                            );
                        }

                        if let Some(label) = self.visible_range_label() {
                            ui.separator();
                            ui.label(
                                egui::RichText::new(label)
                                    .size(11.0)
                                    .color(ui.visuals().text_color().gamma_multiply(0.8)),
                            );
                        }
                    });
                });
            });
    }

    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        let toggle = if self.sidebar_collapsed { ">>" } else { "<<" };
        if ui.small_button(toggle).clicked() {
            self.sidebar_collapsed = !self.sidebar_collapsed;
        }
        ui.add_space(6.0);

        if self.sidebar_collapsed {
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("⚡")
                            .size(18.0)
                            .color(egui::Color32::from_rgb(255, 210, 80)),
                    )
                    .min_size(egui::vec2(28.0, 24.0)),
                )
                .on_hover_text("Load electric CSV")
                .clicked()
            {
                self.pick_and_load_electric();
            }
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("🔥")
                            .size(18.0)
                            .color(egui::Color32::from_rgb(255, 120, 90)),
                    )
                    .min_size(egui::vec2(28.0, 24.0)),
                )
                .on_hover_text("Load gas CSV")
                .clicked()
            {
                self.pick_and_load_gas();
            }

            ui.add_space(10.0);
            let compact_views = [
                ChartView::DailyKwh,
                ChartView::DailyHeatmap,
                ChartView::WeekdayHeatmap,
                ChartView::HourlyProfile,
                ChartView::ExportSparklines,
                ChartView::GasDaily,
            ];

            for view in compact_views {
                let (icon, color, tip) = Self::view_icon(view);
                let response = ui.add(
                    egui::Button::new(egui::RichText::new(icon).size(18.0).color(color))
                        .selected(self.current_view == view)
                        .min_size(egui::vec2(28.0, 24.0)),
                );
                if response.clicked() {
                    self.set_view(view);
                }
                response.on_hover_text(tip);
            }

            ui.add_space(10.0);
            let mut dark_mode = self.config.ui.dark_mode.unwrap_or(false);
            if ui
                .checkbox(&mut dark_mode, "")
                .on_hover_text("Dark mode")
                .changed()
            {
                self.config.ui.dark_mode = Some(dark_mode);
                let _ = self.config.save();
                ui::apply_custom_style(ui.ctx(), Some(dark_mode));
            }
            return;
        }

        let modern = self.config.ui.modern_ui;

        ui.label(
            egui::RichText::new("Data Files")
                .strong()
                .size(crate::ui::styles::SIDEBAR_SECTION_SIZE)
                .color(ui.visuals().text_color()),
        );

        if ui::components::render_sidebar_item(
            ui,
            false,
            "⚡",
            egui::Color32::from_rgb(255, 210, 80),
            "Load Electric CSV",
            modern,
        ) {
            self.pick_and_load_electric();
        }

        if ui::components::render_sidebar_item(
            ui,
            false,
            "🔥",
            egui::Color32::from_rgb(255, 120, 90),
            "Load Gas CSV",
            modern,
        ) {
            self.pick_and_load_gas();
        }

        ui.add_space(16.0);
        ui.label(
            egui::RichText::new("Views")
                .strong()
                .size(crate::ui::styles::SIDEBAR_SECTION_SIZE)
                .color(ui.visuals().text_color()),
        );

        let views = [
            ChartView::DailyKwh,
            ChartView::DailyHeatmap,
            ChartView::WeekdayHeatmap,
            ChartView::HourlyProfile,
            ChartView::ExportSparklines,
            ChartView::GasDaily,
        ];

        for view in views {
            let (icon, color, label) = Self::view_icon(view);
            if ui::components::render_sidebar_item(
                ui,
                self.current_view == view,
                icon,
                color,
                label,
                modern,
            ) {
                self.set_view(view);
            }
        }

        ui.add_space(16.0);
        ui.label(
            egui::RichText::new("Preferences")
                .strong()
                .size(crate::ui::styles::SIDEBAR_SECTION_SIZE)
                .color(ui.visuals().text_color()),
        );

        let mut dark_mode = self.config.ui.dark_mode.unwrap_or(false);
        if ui
            .checkbox(
                &mut dark_mode,
                egui::RichText::new("🌙 Dark Mode")
                    .size(13.0)
                    .color(egui::Color32::from_rgb(170, 190, 255)),
            )
            .changed()
        {
            self.config.ui.dark_mode = Some(dark_mode);
            let _ = self.config.save();
            ui::apply_custom_style(ui.ctx(), Some(dark_mode));
        }

        if ui
            .checkbox(
                &mut self.config.ui.modern_ui,
                egui::RichText::new("✨ Modern UI").size(13.0),
            )
            .changed()
        {
            let _ = self.config.save();
        }

        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("Heatmap Palette")
                .size(12.0)
                .color(ui.visuals().text_color().gamma_multiply(0.8)),
        );

        egui::ComboBox::from_id_salt("palette_sel")
            .selected_text(self.heatmap_state.palette.name())
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for p in charts::HeatmapPalette::all() {
                    if ui
                        .selectable_value(&mut self.heatmap_state.palette, *p, p.name())
                        .clicked()
                    {
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
                    ui::components::Card::new().show(ui, self.config.ui.modern_ui, |ui| {
                        charts::render_daily_kwh(ui, data, &mut self.zoom_state);
                    });
                } else {
                    ui.label("No electric data loaded. Please load a CSV file.");
                }
            }
            ChartView::WeekdayHeatmap => {
                if let Some(ref data) = self.electric_data {
                    ui::components::Card::new().show(ui, self.config.ui.modern_ui, |ui| {
                        charts::render_weekday_heatmap(
                            ui,
                            data,
                            &mut self.heatmap_state,
                            self.config.ui.modern_ui,
                        );
                    });
                } else {
                    ui.label("No electric data loaded. Please load a CSV file.");
                }
            }
            ChartView::DailyHeatmap => {
                if let Some(data) = self.electric_data.as_ref() {
                    ui::components::Card::new().show(ui, self.config.ui.modern_ui, |ui| {
                        charts::render_daily_heatmap_with_toggle(
                            ui,
                            data,
                            &mut self.heatmap_state,
                            &mut self.heatmap_metric,
                            self.range_preset,
                            self.config.ui.modern_ui,
                        );
                    });
                } else {
                    ui.label("No electric data loaded. Please load a CSV file.");
                }
            }
            ChartView::HourlyProfile => {
                if let Some(ref data) = self.electric_data {
                    ui.heading("Average Daily Profile (Mean kWh by Hour)");
                    ui::components::Card::new().show(ui, self.config.ui.modern_ui, |ui| {
                        charts::render_hourly_profile(ui, data);
                    });
                } else {
                    ui.label("No electric data loaded. Please load a CSV file.");
                }
            }
            ChartView::ExportSparklines => {
                if let Some(ref data) = self.electric_data {
                    ui::components::Card::new().show(ui, self.config.ui.modern_ui, |ui| {
                        charts::render_export_sparklines(
                            ui,
                            data,
                            &mut self.heatmap_state,
                            self.range_preset,
                            self.config.ui.modern_ui,
                        );
                    });
                } else {
                    ui.label("No electric data loaded. Please load a CSV file.");
                }
            }
            ChartView::GasDaily => {
                if let Some(ref data) = self.gas_data {
                    ui.heading("Gas: Daily Usage (USD)");
                    ui::components::Card::new().show(ui, self.config.ui.modern_ui, |ui| {
                        charts::render_gas_daily(ui, data, &mut self.zoom_state);
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

    #[cfg(target_os = "windows")]
    fn apply_native_titlebar_colors(&self, frame: &eframe::Frame, dark_mode: bool) {
        use raw_window_handle::{HasWindowHandle as _, RawWindowHandle};
        use windows::Win32::Foundation::HWND;
        use windows::Win32::Graphics::Dwm::{
            DwmSetWindowAttribute, DWMWA_CAPTION_COLOR, DWMWA_TEXT_COLOR,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
        };

        let Ok(window_handle) = frame.window_handle() else {
            return;
        };

        let RawWindowHandle::Win32(win32) = window_handle.as_raw() else {
            return;
        };

        let hwnd = HWND(win32.hwnd.get() as *mut core::ffi::c_void);

        let caption_color = if dark_mode {
            egui::Color32::from_rgb(32, 32, 32)
        } else {
            ui::styles::window_bg()
        };
        let caption_colorref = u32::from(caption_color.r())
            | (u32::from(caption_color.g()) << 8)
            | (u32::from(caption_color.b()) << 16);

        let text_color = if dark_mode {
            egui::Color32::from_rgb(255, 255, 255)
        } else {
            egui::Color32::from_rgb(0, 0, 0)
        };
        let text_colorref = u32::from(text_color.r())
            | (u32::from(text_color.g()) << 8)
            | (u32::from(text_color.b()) << 16);

        let immersive_dark: i32 = if dark_mode { 1 } else { 0 };

        // Best effort: unsupported attributes are ignored on older Windows builds.
        unsafe {
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE,
                (&immersive_dark as *const i32).cast(),
                std::mem::size_of::<i32>() as u32,
            );
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_CAPTION_COLOR,
                (&caption_colorref as *const u32).cast(),
                std::mem::size_of::<u32>() as u32,
            );
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_TEXT_COLOR,
                (&text_colorref as *const u32).cast(),
                std::mem::size_of::<u32>() as u32,
            );
        }
    }

    fn sync_native_titlebar_theme(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        #[cfg(target_os = "windows")]
        {
            if !USE_CUSTOM_WINDOW_CHROME {
                let wants_dark = self.config.ui.dark_mode.unwrap_or(false);
                if self.last_sync_native_titlebar_dark != Some(wants_dark) {
                    let theme = if wants_dark {
                        egui::SystemTheme::Dark
                    } else {
                        egui::SystemTheme::Light
                    };
                    ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(theme));
                    self.apply_native_titlebar_colors(frame, wants_dark);
                    self.last_sync_native_titlebar_dark = Some(wants_dark);
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = ctx;
            let _ = frame;
        }
    }
}

impl eframe::App for PgeAnalyzerApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        let bg = if self.config.ui.dark_mode.unwrap_or(false) {
            egui::Color32::from_rgb(32, 32, 32)
        } else {
            ui::styles::window_bg()
        };
        bg.to_normalized_gamma_f32()
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Enforce theme on startup or if egui resets it (e.g. system theme change)
        if ctx.style().visuals.window_fill != egui::Color32::TRANSPARENT {
            ui::apply_custom_style(ctx, self.config.ui.dark_mode);
        }
        self.sync_native_titlebar_theme(ctx, frame);

        if USE_CUSTOM_WINDOW_CHROME {
            // Update resize state before painting so we can adjust background rendering
            // while the OS is actively resizing the frameless window.
            ui::handle_window_resize(ctx, &mut self.resize_state);

            // 1. Paint the main window background manually on the background layer
            let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
            let rounding = if is_maximized || self.resize_state.is_resizing() {
                0.0
            } else {
                ui::styles::WINDOW_ROUNDING
            };
            let bg_color = ui::actual_window_background(ctx);

            ctx.layer_painter(egui::LayerId::background()).rect_filled(
                ctx.viewport_rect(),
                egui::CornerRadius::same(rounding as u8),
                bg_color,
            );

            // 2. Border stroke removed - custom title bar handles visual boundaries
        }

        // --- Custom Panels (Transparent) ---
        if USE_CUSTOM_WINDOW_CHROME {
            ui::render_title_bar(ctx, "PG&E Usage Analyzer");
        }

        let sidebar_width = if self.sidebar_collapsed { 44.0 } else { 180.0 };
        egui::SidePanel::left("sidebar_panel")
            .frame(egui::Frame::NONE.fill(egui::Color32::TRANSPARENT))
            .resizable(false)
            .default_width(sidebar_width)
            .min_width(sidebar_width)
            .max_width(sidebar_width)
            .show(ctx, |ui| {
                if USE_CUSTOM_WINDOW_CHROME {
                    ui.add_space(20.0); // Space for top rounding
                }
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(10, 0))
                    .show(ui, |ui| {
                        self.render_sidebar(ui);
                    });
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(egui::Color32::TRANSPARENT)
                    .inner_margin(egui::Margin {
                        left: 10,
                        right: 10,
                        top: 0,
                        bottom: 10,
                    }),
            )
            .show(ctx, |ui| {
                if USE_CUSTOM_WINDOW_CHROME {
                    ui.add_space(10.0);
                }
                self.render_command_bar(ui);
                ui.add_space(8.0);
                self.render_main_content(ui);
            });

        if USE_CUSTOM_WINDOW_CHROME {
            self.resize_state.apply_cursor(ctx);
        }
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
        .with_decorations(!USE_CUSTOM_WINDOW_CHROME)
        .with_transparent(false)
        .with_has_shadow(USE_CUSTOM_WINDOW_CHROME)
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
