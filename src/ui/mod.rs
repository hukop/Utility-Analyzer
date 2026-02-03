pub mod styles;
pub mod components;
pub mod window;

pub use styles::*;
pub use window::{WindowResizeState, handle_window_resize, render_title_bar};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartView {
    DailyKwh,
    WeekdayHeatmap,
    DailyHeatmap,
    CostHeatmap,
    HourlyProfile,
    ExportSparklines,
    GasDaily,
}

impl ChartView {
    pub fn from_str(s: &str) -> Self {
        match s {
            "WeekdayHeatmap" => Self::WeekdayHeatmap,
            "DailyHeatmap" => Self::DailyHeatmap,
            "CostHeatmap" => Self::CostHeatmap,
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
            Self::WeekdayHeatmap => "WeekdayHeatmap",
            Self::DailyHeatmap => "DailyHeatmap",
            Self::CostHeatmap => "CostHeatmap",
            Self::HourlyProfile => "HourlyProfile",
            Self::ExportSparklines => "ExportSparklines",
            Self::GasDaily => "GasDaily",
        };
        write!(f, "{}", s)
    }
}
