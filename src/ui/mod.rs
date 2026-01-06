pub mod styles;

pub use styles::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartView {
    DailyKwh,
    WeekdayHeatmap,
    DailyHeatmap,
    CostHeatmap,
    HourlyProfile,
    GasDaily,
}

impl ChartView {
    pub fn all() -> Vec<Self> {
        vec![
            Self::DailyKwh,
            Self::WeekdayHeatmap,
            Self::DailyHeatmap,
            Self::CostHeatmap,
            Self::HourlyProfile,
            Self::GasDaily,
        ]
    }
    
    pub fn name(&self) -> &str {
        match self {
            Self::DailyKwh => "Daily kWh",
            Self::WeekdayHeatmap => "Average kWh by Weekday and Hour",
            Self::DailyHeatmap => "Daily by-hour kWh Heatmap",
            Self::CostHeatmap => "Daily by-hour Cost Heatmap",
            Self::HourlyProfile => "Average Daily Profile",
            Self::GasDaily => "Gas: Daily Usage (USD)",
        }
    }
}
