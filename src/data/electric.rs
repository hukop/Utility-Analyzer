use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, NaiveDateTime, Timelike, Utc};
use csv::ReaderBuilder;
use std::collections::HashMap;

/// A single data point representing electric usage at a specific time.
#[derive(Debug, Clone)]
pub struct ElectricDataPoint {
    /// The timestamp of the usage interval.
    pub timestamp: DateTime<Utc>,
    /// Kilowatt-hours imported during the interval.
    pub kwh: f64,
    /// The cost associated with this interval (if available).
    pub cost: f64,
    /// Kilowatt-hours exported (solar) during the interval.
    pub export_kwh: f64,
}

/// A collection of electric usage data points.
#[derive(Debug, Clone)]
pub struct ElectricData {
    /// The list of usage data points, usually sorted by timestamp.
    pub data: Vec<ElectricDataPoint>,
    /// Pre-calculated sums of usage (kWh) per month (YYYY-MM).
    pub monthly_kwh_sums: HashMap<String, f64>,
    /// Pre-calculated sums of cost ($) per month (YYYY-MM).
    pub monthly_cost_sums: HashMap<String, f64>,
    /// Pre-calculated sums of export (kWh) per month (YYYY-MM).
    pub monthly_export_sums: HashMap<String, f64>,
}

impl ElectricData {
    /// Loads electric usage data from a CSV string.
    ///
    /// Expects a specific PGE export format with columns like "TYPE", "DATE",
    /// "START TIME", and "IMPORT (kWh)".
    pub fn load(csv_content: &str) -> Result<Self> {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(csv_content.as_bytes());

        // Get headers to find column indices
        let headers = reader.headers()?.clone();

        // Find column indices
        let type_idx = headers.iter().position(|h| h == "TYPE")
            .context("TYPE column not found")?;
        let date_idx = headers.iter().position(|h| h == "DATE")
            .context("DATE column not found")?;
        let start_time_idx = headers.iter().position(|h| h == "START TIME")
            .context("START TIME column not found")?;
        let kwh_idx = headers.iter().position(|h| h == "IMPORT (kWh)")
            .context("IMPORT (kWh) column not found")?;
        let cost_idx = headers.iter().position(|h| h == "TOTAL IMPORT COST");
        let export_idx = headers.iter().position(|h| h == "EXPORT (kWh)");

        let mut data = Vec::new();
        let mut monthly_kwh_sums = HashMap::new();
        let mut monthly_cost_sums = HashMap::new();
        let mut monthly_export_sums = HashMap::new();

        for result in reader.records() {
            let record = result?;

            // Get TYPE field
            let type_field = record.get(type_idx)
                .context("Missing TYPE field")?;

            // Filter to electric usage only
            if !type_field.contains("Electric usage") {
                continue;
            }

            // Parse date and time
            let date_str = record.get(date_idx).context("Missing DATE field")?;
            let time_str = record.get(start_time_idx).context("Missing START TIME field")?;

            let dt = NaiveDateTime::parse_from_str(&format!("{} {}", date_str, time_str), "%Y-%m-%d %H:%M")
                .or_else(|_| NaiveDateTime::parse_from_str(&format!("{} {}", date_str, time_str), "%m/%d/%Y %H:%M"))
                .with_context(|| format!("Failed to parse date/time: {} {}", date_str, time_str))?;

            let timestamp = DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc);

            // Parse usage (kWh)
            let kwh_str = record.get(kwh_idx).context("Missing kWh field")?;
            let kwh: f64 = kwh_str.parse().context("Failed to parse kWh")?;

            // Parse cost (optional)
            let cost = if let Some(idx) = cost_idx {
                let cost_str = record.get(idx).unwrap_or("$0.00");
                cost_str.trim_start_matches('$').parse::<f64>().unwrap_or(0.0)
            } else {
                0.0
            };

            // Parse export (optional)
            let export_kwh = if let Some(idx) = export_idx {
                record.get(idx).unwrap_or("0.0").parse::<f64>().unwrap_or(0.0)
            } else {
                0.0
            };

            // Update monthly sums
            let month_key = dt.format("%Y-%m").to_string();
            *monthly_kwh_sums.entry(month_key.clone()).or_insert(0.0) += kwh;
            *monthly_cost_sums.entry(month_key.clone()).or_insert(0.0) += cost;
            *monthly_export_sums.entry(month_key).or_insert(0.0) += export_kwh;

            data.push(ElectricDataPoint {
                timestamp,
                kwh,
                cost,
                export_kwh,
            });
        }

        // Sort data by timestamp
        data.sort_by_key(|p| p.timestamp);

        Ok(Self { data, monthly_kwh_sums, monthly_cost_sums, monthly_export_sums })
    }

    /// Get daily totals
    pub fn daily_totals(&self) -> Vec<(DateTime<Utc>, f64)> {
        let mut daily: HashMap<String, f64> = HashMap::new();

        for point in &self.data {
            let date_key = point.timestamp.format("%Y-%m-%d").to_string();
            *daily.entry(date_key).or_insert(0.0) += point.kwh;
        }

        let mut result: Vec<_> = daily
            .into_iter()
            .filter_map(|(date, kwh)| {
                NaiveDateTime::parse_from_str(&format!("{} 00:00", date), "%Y-%m-%d %H:%M")
                    .ok()
                    .map(|dt| (dt.and_utc(), kwh))
            })
            .collect();

        result.sort_by_key(|(dt, _)| *dt);
        result
    }

    /// Returns average kWh grouped by weekday and hour of day.
    pub fn weekday_hour_average(&self) -> [[f64; 24]; 7] {
        let mut totals = [[0.0; 24]; 7];
        let mut counts = [[0u32; 24]; 7];

        for point in &self.data {
            let weekday = point.timestamp.weekday().num_days_from_monday() as usize;
            let hour = point.timestamp.hour() as usize;

            totals[weekday][hour] += point.kwh;
            counts[weekday][hour] += 1;
        }

        let mut averages = [[0.0; 24]; 7];
        for weekday in 0..7 {
            for hour in 0..24 {
                if counts[weekday][hour] > 0 {
                    averages[weekday][hour] = totals[weekday][hour] / counts[weekday][hour] as f64;
                }
            }
        }

        averages
    }

    /// Returns average kWh grouped by hour of day (0-23).
    pub fn hourly_profile(&self) -> [f64; 24] {
        let mut totals = [0.0; 24];
        let mut counts = [0u32; 24];

        for point in &self.data {
            let hour = point.timestamp.hour() as usize;
            totals[hour] += point.kwh;
            counts[hour] += 1;
        }

        let mut profile = [0.0; 24];
        for hour in 0..24 {
            if counts[hour] > 0 {
                profile[hour] = totals[hour] / counts[hour] as f64;
            }
        }

        profile
    }

    /// Returns a list of dates and a 2D matrix of kWh by [day][hour].
    pub fn daily_hour_heatmap(&self) -> (Vec<String>, Vec<Vec<f64>>) {
        let mut daily_data: HashMap<String, [f64; 24]> = HashMap::new();

        for point in &self.data {
            let date_key = point.timestamp.format("%Y-%m-%d").to_string();
            let hour = point.timestamp.hour() as usize;

            let day_data = daily_data.entry(date_key).or_insert([0.0; 24]);
            day_data[hour] += point.kwh;
        }

        let mut dates: Vec<String> = daily_data.keys().cloned().collect();
        dates.sort();

        let data: Vec<Vec<f64>> = dates
            .iter()
            .map(|date| daily_data[date].to_vec())
            .collect();

        (dates, data)
    }

    /// Returns a list of dates and a 2D matrix of cost by [day][hour].
    pub fn daily_hour_cost_heatmap(&self) -> (Vec<String>, Vec<Vec<f64>>) {
        let mut daily_data: HashMap<String, [f64; 24]> = HashMap::new();

        for point in &self.data {
            let date_key = point.timestamp.format("%Y-%m-%d").to_string();
            let hour = point.timestamp.hour() as usize;

            let day_data = daily_data.entry(date_key).or_insert([0.0; 24]);
            day_data[hour] += point.cost;
        }

        let mut dates: Vec<String> = daily_data.keys().cloned().collect();
        dates.sort();

        let data: Vec<Vec<f64>> = dates
            .iter()
            .map(|date| daily_data[date].to_vec())
            .collect();

        (dates, data)
    }

    /// Returns daytime (6-18) export data for sparkline charts.
    /// Returns (dates, export_data_per_day, daily_sums) where each day has 13 values (hours 6-18).
    pub fn daily_daytime_export_data(&self) -> (Vec<String>, Vec<Vec<f64>>, Vec<f64>) {
        let mut daily_data: HashMap<String, [f64; 13]> = HashMap::new();

        for point in &self.data {
            let hour = point.timestamp.hour() as usize;
            // Only include hours 6-18 (indices 0-12 in our 13-element array)
            if hour >= 6 && hour <= 18 {
                let date_key = point.timestamp.format("%Y-%m-%d").to_string();
                let day_data = daily_data.entry(date_key).or_insert([0.0; 13]);
                day_data[hour - 6] += point.export_kwh;
            }
        }

        let mut dates: Vec<String> = daily_data.keys().cloned().collect();
        dates.sort();

        let data: Vec<Vec<f64>> = dates
            .iter()
            .map(|date| daily_data[date].to_vec())
            .collect();

        let sums: Vec<f64> = data
            .iter()
            .map(|day| day.iter().sum())
            .collect();

        (dates, data, sums)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_electric_data_load() {
        let csv = "TYPE,DATE,START TIME,IMPORT (kWh),TOTAL IMPORT COST\n\
                  Electric usage,2023-01-01,00:00,1.5,$0.30\n\
                  Electric usage,2023-01-01,01:00,2.0,$0.40";

        let result = ElectricData::load(csv).unwrap();
        assert_eq!(result.data.len(), 2);
        assert_eq!(result.data[0].kwh, 1.5);
        assert_eq!(result.data[0].cost, 0.30);
    }

    #[test]
    fn test_electric_data_load_alternate_date_format() {
        let csv = "TYPE,DATE,START TIME,IMPORT (kWh),TOTAL IMPORT COST\n\
                  Electric usage,01/01/2023,00:00,1.5,$0.30";

        let result = ElectricData::load(csv).unwrap();
        assert_eq!(result.data.len(), 1);
        assert_eq!(result.data[0].kwh, 1.5);
    }

    #[test]
    fn test_electric_monthly_sums() {
        let csv = "TYPE,DATE,START TIME,IMPORT (kWh),TOTAL IMPORT COST\n\
                  Electric usage,2023-01-01,00:00,1.0,$0.20\n\
                  Electric usage,2023-01-02,00:00,2.0,$0.40\n\
                  Electric usage,2023-02-01,00:00,3.0,$0.60";

        let result = ElectricData::load(csv).unwrap();

        let jan_kwh = result.monthly_kwh_sums.get("2023-01").unwrap();
        let jan_cost = result.monthly_cost_sums.get("2023-01").unwrap();
        let feb_kwh = result.monthly_kwh_sums.get("2023-02").unwrap();
        let feb_cost = result.monthly_cost_sums.get("2023-02").unwrap();

        assert!((jan_kwh - 3.0).abs() < 1e-10);
        assert!((jan_cost - 0.60).abs() < 1e-10);
        assert!((feb_kwh - 3.0).abs() < 1e-10);
        assert!((feb_cost - 0.60).abs() < 1e-10);
    }
}
