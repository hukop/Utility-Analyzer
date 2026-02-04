use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, NaiveDateTime, Timelike, Utc};
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
    /// Pre-calculated sums of usage (kWh) per year (YYYY).
    pub yearly_kwh_sums: HashMap<String, f64>,
    /// Pre-calculated sums of cost ($) per year (YYYY).
    pub yearly_cost_sums: HashMap<String, f64>,
    /// Pre-calculated sums of export (kWh) per year (YYYY).
    pub yearly_export_sums: HashMap<String, f64>,
}

impl ElectricData {
    /// Loads electric usage data from one or more CSV strings.
    ///
    /// Expects a specific PGE export format with columns like "TYPE", "DATE",
    /// "START TIME", and "IMPORT (kWh)".
    ///
    /// If multiple CSVs are provided, data is merged. In case of overlapping
    /// timestamps, the firstOccurrence preference is given (data from earlier
    /// files in the list is preserved).
    pub fn load(csv_contents: &[String]) -> Result<Self> {
        let mut merged_data: std::collections::BTreeMap<DateTime<Utc>, ElectricDataPoint> = std::collections::BTreeMap::new();
        let mut col_indices = None;

        super::loader::for_each_record(csv_contents, |headers, record| {
            if col_indices.is_none() {
                col_indices = Some((
                    headers.iter().position(|h| h == "TYPE").context("TYPE column missing")?,
                    headers.iter().position(|h| h == "DATE").context("DATE column missing")?,
                    headers.iter().position(|h| h == "START TIME").context("START TIME column missing")?,
                    headers.iter().position(|h| h == "IMPORT (kWh)").context("IMPORT column missing")?,
                    headers.iter().position(|h| h == "TOTAL IMPORT COST"),
                    headers.iter().position(|h| h == "EXPORT (kWh)"),
                ));
            }

            let (type_i, date_i, time_i, kwh_i, cost_i, export_i) = col_indices.unwrap();

            if !record.get(type_i).map(|s| s.contains("Electric usage")).unwrap_or(false) {
                return Ok(());
            }

            let date_str = record.get(date_i).context("DATE field missing")?;
            let time_str = record.get(time_i).context("TIME field missing")?;

            let dt = NaiveDateTime::parse_from_str(&format!("{} {}", date_str, time_str), "%Y-%m-%d %H:%M")
                .or_else(|_| NaiveDateTime::parse_from_str(&format!("{} {}", date_str, time_str), "%m/%d/%Y %H:%M"))
                .with_context(|| format!("Failed to parse date/time: {} {}", date_str, time_str))?;

            let timestamp = DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc);

            if merged_data.contains_key(&timestamp) {
                return Ok(());
            }

            let kwh: f64 = record.get(kwh_i).context("kWh missing")?.parse().context("Invalid kWh")?;
            let cost = cost_i.and_then(|i| record.get(i)).map(|s| s.trim_start_matches('$').parse().unwrap_or(0.0)).unwrap_or(0.0);
            let export_kwh = export_i.and_then(|i| record.get(i)).map(|s| s.parse().unwrap_or(0.0)).unwrap_or(0.0);

            merged_data.insert(timestamp, ElectricDataPoint { timestamp, kwh, cost, export_kwh });
            Ok(())
        })?;

        let data: Vec<ElectricDataPoint> = merged_data.into_values().collect();

        let mut self_obj = Self {
            data,
            monthly_kwh_sums: HashMap::new(),
            monthly_cost_sums: HashMap::new(),
            monthly_export_sums: HashMap::new(),
            yearly_kwh_sums: HashMap::new(),
            yearly_cost_sums: HashMap::new(),
            yearly_export_sums: HashMap::new(),
        };

        self_obj.recalculate_summaries();

        Ok(self_obj)
    }

    /// Recalculates all monthly and yearly sums based on existing data.
    fn recalculate_summaries(&mut self) {
        self.monthly_kwh_sums.clear();
        self.monthly_cost_sums.clear();
        self.monthly_export_sums.clear();
        self.yearly_kwh_sums.clear();
        self.yearly_cost_sums.clear();
        self.yearly_export_sums.clear();

        for point in &self.data {
            let dt = point.timestamp.naive_utc();
            let month_key = dt.format("%Y-%m").to_string();
            let year_key = dt.format("%Y").to_string();

            *self.monthly_kwh_sums.entry(month_key.clone()).or_insert(0.0) += point.kwh;
            *self.monthly_cost_sums.entry(month_key.clone()).or_insert(0.0) += point.cost;
            *self.monthly_export_sums.entry(month_key).or_insert(0.0) += point.export_kwh;

            *self.yearly_kwh_sums.entry(year_key.clone()).or_insert(0.0) += point.kwh;
            *self.yearly_cost_sums.entry(year_key.clone()).or_insert(0.0) += point.cost;
            *self.yearly_export_sums.entry(year_key).or_insert(0.0) += point.export_kwh;
        }
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

    /// Returns average export kWh grouped by hour of day (0-23).
    pub fn hourly_export_profile(&self) -> [f64; 24] {
        let mut totals = [0.0; 24];
        let mut counts = [0u32; 24];

        for point in &self.data {
            let hour = point.timestamp.hour() as usize;
            totals[hour] += point.export_kwh;
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
            if (6..=18).contains(&hour) {
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
                  Electric usage,2023-01-01,01:00,2.0,$0.40".to_string();

        let result = ElectricData::load(&[csv]).unwrap();
        assert_eq!(result.data.len(), 2);
        assert_eq!(result.data[0].kwh, 1.5);
        assert_eq!(result.data[0].cost, 0.30);
    }

    #[test]
    fn test_electric_data_load_alternate_date_format() {
        let csv = "TYPE,DATE,START TIME,IMPORT (kWh),TOTAL IMPORT COST\n\
                  Electric usage,01/01/2023,00:00,1.5,$0.30".to_string();

        let result = ElectricData::load(&[csv]).unwrap();
        assert_eq!(result.data.len(), 1);
        assert_eq!(result.data[0].kwh, 1.5);
    }

    #[test]
    fn test_electric_monthly_sums() {
        let csv = "TYPE,DATE,START TIME,IMPORT (kWh),TOTAL IMPORT COST\n\
                  Electric usage,2023-01-01,00:00,1.0,$0.20\n\
                  Electric usage,2023-01-02,00:00,2.0,$0.40\n\
                  Electric usage,2023-02-01,00:00,3.0,$0.60".to_string();

        let result = ElectricData::load(&[csv]).unwrap();

        let jan_kwh = result.monthly_kwh_sums.get("2023-01").unwrap();
        let jan_cost = result.monthly_cost_sums.get("2023-01").unwrap();
        let feb_kwh = result.monthly_kwh_sums.get("2023-02").unwrap();
        let feb_cost = result.monthly_cost_sums.get("2023-02").unwrap();

        assert!((jan_kwh - 3.0).abs() < 1e-10);
        assert!((jan_cost - 0.60).abs() < 1e-10);
        assert!((feb_kwh - 3.0).abs() < 1e-10);
        assert!((feb_cost - 0.60).abs() < 1e-10);
    }

    #[test]
    fn test_electric_data_merge() {
        let csv1 = "TYPE,DATE,START TIME,IMPORT (kWh),TOTAL IMPORT COST\n\
                   Electric usage,2023-01-01,00:00,1.0,$0.20".to_string();
        let csv2 = "TYPE,DATE,START TIME,IMPORT (kWh),TOTAL IMPORT COST\n\
                   Electric usage,2023-01-01,00:00,5.0,$1.00\n\
                   Electric usage,2023-01-01,01:00,2.0,$0.40".to_string();

        let result = ElectricData::load(&[csv1, csv2]).unwrap();

        // Should have 2 points, and the first point should be from csv1 (1.0 kWh)
        assert_eq!(result.data.len(), 2);
        assert_eq!(result.data[0].kwh, 1.0);
        assert_eq!(result.data[1].kwh, 2.0);
    }
}
