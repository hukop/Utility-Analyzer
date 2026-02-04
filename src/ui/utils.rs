use chrono::{Datelike, NaiveDate};
use egui::{Color32, RichText};
use crate::ui::styles;

/// Utilities for UI logic and formatting.
pub struct UiUtils;

impl UiUtils {
    /// Returns true if the given date string (YYYY-MM-DD) or NaiveDate is a weekend.
    pub fn is_weekend(date: &str) -> bool {
        if date == "Saturday" || date == "Sunday" {
             return true;
        }
        if let Ok(parsed) = NaiveDate::parse_from_str(date, "%Y-%m-%d") {
            parsed.weekday() == chrono::Weekday::Sat || parsed.weekday() == chrono::Weekday::Sun
        } else {
            false
        }
    }

    pub fn is_weekend_date(date: NaiveDate) -> bool {
        date.weekday() == chrono::Weekday::Sat || date.weekday() == chrono::Weekday::Sun
    }

    /// Returns the appropriate background color for a weekend cell.
    pub fn weekend_bg(dark_mode: bool) -> Color32 {
        styles::weekend_bg(dark_mode)
    }

    /// Renders a date label with weekend styling if applicable.
    pub fn styled_date_label(date_str: &str, dark_mode: bool) -> RichText {
        let (label, color_opt) = Self::date_label_parts(date_str, dark_mode);
        let mut text = RichText::new(label);
        if let Some(color) = color_opt {
            text = text.strong().color(color);
        }
        text
    }

    /// Returns (Label, OptionalColor) for a date.
    pub fn date_label_parts(date_str: &str, dark_mode: bool) -> (String, Option<Color32>) {
        let date_parsed = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok();
        let is_weekend = date_parsed.map_or(false, |d| Self::is_weekend_date(d));

        let label = date_parsed.map_or_else(
            || date_str.to_string(),
            |d| d.format("%Y-%m-%d %a").to_string()
        );

        let color = if is_weekend { Some(styles::weekend_text(dark_mode)) } else { None };
        (label, color)
    }
}
