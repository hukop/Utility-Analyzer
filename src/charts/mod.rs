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
}
