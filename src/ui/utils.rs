use chrono::{Datelike, NaiveDate};

/// Utilities for UI logic and formatting.
pub struct UiUtils;

impl UiUtils {
    /// Returns true if the given date string (YYYY-MM-DD) is a weekend.
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
}
