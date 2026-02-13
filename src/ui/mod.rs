pub mod styles;
pub mod components;
pub mod window;
pub mod utils;

pub use styles::*;
pub use utils::UiUtils;
pub use window::{WindowResizeState, handle_window_resize, render_title_bar};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartView {
    DailyKwh,
    WeekdayHeatmap,
    DailyHeatmap,
    HourlyProfile,
    ExportSparklines,
    GasDaily,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeatmapMetric {
    #[default]
    Energy,
    Cost,
}

impl HeatmapMetric {
    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Energy => "Energy",
            Self::Cost => "Cost",
        }
    }

    #[allow(dead_code)]
    pub fn toggle(&self) -> Self {
        match self {
            Self::Energy => Self::Cost,
            Self::Cost => Self::Energy,
        }
    }
}

impl ChartView {
    pub fn from_str(s: &str) -> Self {
        match s {
            "WeekdayHeatmap" => Self::WeekdayHeatmap,
            "DailyHeatmap" => Self::DailyHeatmap,
            "HourlyProfile" => Self::HourlyProfile,
            "ExportSparklines" => Self::ExportSparklines,
            "GasDaily" => Self::GasDaily,
            _ => Self::DailyKwh, // Default fallback
        }
    }
}

impl std::fmt::Display for ChartView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::DailyKwh => "DailyKwh",
            Self::DailyHeatmap => "DailyHeatmap",
            Self::HourlyProfile => "HourlyProfile",
            Self::ExportSparklines => "ExportSparklines",
            Self::GasDaily => "GasDaily",
            Self::WeekdayHeatmap => "WeekdayHeatmap",
        };
        write!(f, "{}", s)
    }
}
