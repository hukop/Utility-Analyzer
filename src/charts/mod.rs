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
