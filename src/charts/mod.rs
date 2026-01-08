pub mod colormap;
pub mod profile;
pub mod daily_kwh;
pub mod gas_daily;
pub mod daily_heatmap;
pub mod cost_heatmap;
pub mod weekday_heatmap;
pub mod heatmap_base;

pub use daily_kwh::*;
pub use weekday_heatmap::*;
pub use daily_heatmap::*;
pub use cost_heatmap::*;
pub use profile::*;
pub use gas_daily::*;

use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct HeatmapState {
    pub selection_start: Option<(usize, usize)>, // (day_idx, hour)
    pub selection_end: Option<(usize, usize)>,
    pub is_dragging: bool,
    pub collapsed_months: HashSet<String>,
}
