pub mod colormap;
pub mod profile;
pub mod daily_kwh;
pub mod gas_daily;
pub mod daily_heatmap;
pub mod cost_heatmap;
pub mod weekday_heatmap;
pub mod heatmap_base;
pub mod export_sparklines;

pub use daily_kwh::*;
pub use weekday_heatmap::*;
pub use daily_heatmap::*;
pub use cost_heatmap::*;
pub use profile::*;
pub use gas_daily::*;
pub use export_sparklines::*;

use std::collections::HashSet;
use serde::{Deserialize, Serialize};

/// Available color palettes for heatmaps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HeatmapPalette {
    #[default]
    Viridis,
    GreenYellowRed,
    GreenWhiteRed,
    YellowGreenBlue,
    Magma,
}

impl HeatmapPalette {
    pub fn all() -> &'static [Self] {
        &[
            Self::Viridis,
            Self::Magma,
            Self::GreenYellowRed,
            Self::GreenWhiteRed,
            Self::YellowGreenBlue,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Viridis => "Viridis",
            Self::Magma => "Magma",
            Self::GreenYellowRed => "Green-Yellow-Red",
            Self::GreenWhiteRed => "Green-White-Red",
            Self::YellowGreenBlue => "Yellow-Green-Blue",
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name {
            "Magma" => Self::Magma,
            "Green-Yellow-Red" => Self::GreenYellowRed,
            "Green-White-Red" => Self::GreenWhiteRed,
            "Yellow-Green-Blue" => Self::YellowGreenBlue,
            _ => Self::Viridis,
        }
    }
}

impl std::fmt::Display for HeatmapPalette {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Shared state for heatmap components, tracking selection and collapse states.
#[derive(Debug, Clone, Default)]
pub struct HeatmapState {
    /// The (day_index, hour) where the selection started.
    pub selection_start: Option<(usize, usize)>,
    /// The (day_index, hour) where the selection ended.
    pub selection_end: Option<(usize, usize)>,
    /// Whether the user is currently dragging to select.
    pub is_dragging: bool,
    /// Set of month keys (YYYY-MM) that are currently collapsed.
    pub collapsed_months: HashSet<String>,
    /// Set of year keys (YYYY) that are currently collapsed.
    pub collapsed_years: HashSet<String>,
    /// Current horizontal scroll offset for synchronizing sticky headers.
    pub scroll_offset: f32,
    /// Currently selected color palette.
    pub palette: HeatmapPalette,
}

/// Calculates a moving average over a sliding window.
///
/// # Arguments
/// * `data` - Vector of (Timestamp, Value) tuples.
/// * `window` - Size of the sliding window.
pub fn calculate_rolling_average(
    data: &[(chrono::DateTime<chrono::Utc>, f64)],
    window: usize,
) -> Vec<(chrono::DateTime<chrono::Utc>, f64)> {
    if data.len() < window {
        return data.to_vec();
    }

    let mut result = Vec::new();
    let half_window = window / 2;

    for i in 0..data.len() {
        let start = i.saturating_sub(half_window);
        let end = (i + half_window + 1).min(data.len());

        let sum: f64 = data[start..end].iter().map(|(_, v)| v).sum();
        let count = (end - start) as f64;
        let avg = sum / count;

        result.push((data[i].0, avg));
    }

    result
}

/// Shared state for zooming and panning charts
#[derive(Debug, Clone, Default)]
pub struct ChartZoomState {
    /// Stores current [min_x, max_x] visible range for each chart ID
    pub bounds: std::collections::HashMap<String, (f64, f64)>,
}

/// Renders a daily time-series chart with zoom functionality.
/// Used by Daily kWh and Gas Daily charts to share interaction logic.
pub fn render_zoomable_daily_chart(
    ui: &mut egui::Ui,
    state: &mut ChartZoomState,
    chart_id: &str,
    initial_bounds: (f64, f64),
    lines: Vec<egui_plot::Line>,
) {
    use egui_plot::{Plot, PlotBounds};
    use chrono::DateTime;

    let entry = state.bounds.entry(chart_id.to_string()).or_insert(initial_bounds);
    let (start_ref, end_ref) = entry;
    let start = *start_ref;
    let end = *end_ref;

    Plot::new(format!("{}_plot", chart_id))
        .view_aspect(2.5)
        .legend(egui_plot::Legend::default())
        .allow_zoom(true)
        .allow_drag(true)
        .allow_scroll(false)
        .include_x(start)
        .include_x(end)
        .x_axis_formatter(|x, _range| {
            let timestamp = x.value as i64;
            if let Some(dt) = DateTime::from_timestamp(timestamp, 0) {
                dt.format("%Y-%m-%d").to_string()
            } else {
                "".to_string()
            }
        })
        .label_formatter(|name, value| {
            let timestamp = value.x as i64;
            let date_str = if let Some(dt) = DateTime::from_timestamp(timestamp, 0) {
                dt.format("%Y-%m-%d").to_string()
            } else {
                "".to_string()
            };
            format!("{}: {:.2}\n{}", name, value.y, date_str)
        })
        .show(ui, |plot_ui| {
            for line in lines {
                plot_ui.line(line);
            }

            let hovered = plot_ui.response().hovered();
            let dragged = plot_ui.response().dragged();
            let double_clicked = plot_ui.response().double_clicked();
            let mut bounds_changed = false;

            if hovered {
                let mods = plot_ui.ctx().input(|i| i.modifiers);
                let scroll = plot_ui.ctx().input(|i| i.raw_scroll_delta + i.smooth_scroll_delta);

                if !mods.ctrl {
                    if scroll.y != 0.0 {
                        if let Some(pointer) = plot_ui.pointer_coordinate() {
                            let mouse_x = pointer.x;
                            let bounds = plot_ui.plot_bounds();
                            let min_x = bounds.min()[0];
                            let max_x = bounds.max()[0];
                            let min_y = bounds.min()[1];
                            let max_y = bounds.max()[1];

                            let zoom_factor = if scroll.y > 0.0 { 0.95 } else { 1.05 };

                            let new_min_x = mouse_x - (mouse_x - min_x) * zoom_factor;
                            let new_max_x = mouse_x + (max_x - mouse_x) * zoom_factor;

                            plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                                [new_min_x, min_y],
                                [new_max_x, max_y]
                            ));

                            state.bounds.insert(chart_id.to_string(), (new_min_x, new_max_x));

                            plot_ui.ctx().input_mut(|i| {
                                i.raw_scroll_delta = egui::Vec2::ZERO;
                                i.smooth_scroll_delta = egui::Vec2::ZERO;
                            });
                        }
                    }
                } else if scroll.y != 0.0 {
                     bounds_changed = true;
                }
            }

            if dragged || double_clicked {
                bounds_changed = true;
            }

            if bounds_changed {
                 let final_bounds = plot_ui.plot_bounds();
                 state.bounds.insert(chart_id.to_string(), (final_bounds.min()[0], final_bounds.max()[0]));
            }
        });
}
