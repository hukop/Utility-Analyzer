use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, Timelike, Utc, Weekday};
use egui_plot::PlotPoint;
use std::collections::{BTreeMap, HashMap};

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

/// Precomputed date metadata for UI rendering.
#[derive(Debug, Clone)]
pub struct DailyDateMetadata {
    /// Date key in YYYY-MM-DD format.
    pub date_key: String,
    /// Year key in YYYY format.
    pub year_key: String,
    /// Month key in YYYY-MM format.
    pub month_key: String,
    /// Label with weekday suffix (YYYY-MM-DD Mon).
    pub display_label: String,
    /// Whether this date is Saturday or Sunday.
    pub is_weekend: bool,
    /// Whether this date is Saturday.
    pub is_saturday: bool,
    /// Whether this date is Sunday.
    pub is_sunday: bool,
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
    /// Cached daily totals for line charts.
    daily_totals_cache: Vec<(DateTime<Utc>, f64)>,
    /// Cached 7-day rolling average from daily totals.
    daily_totals_avg7_cache: Vec<(DateTime<Utc>, f64)>,
    /// Cached daily line series points for zero-allocation plotting.
    daily_points_cache: Vec<PlotPoint>,
    /// Cached 7-day average line series points for zero-allocation plotting.
    daily_avg7_points_cache: Vec<PlotPoint>,
    /// Cached x-bounds for daily charts.
    daily_bounds_cache: Option<(f64, f64)>,
    /// Cached weekday/hour averages.
    weekday_hour_average_cache: [[f64; 24]; 7],
    /// Cached weekday heatmap rows.
    weekday_hour_heatmap_cache: Vec<Vec<f64>>,
    /// Cached max value for weekday heatmap.
    weekday_hour_heatmap_max_cache: f64,
    /// Cached hourly profile averages.
    hourly_profile_cache: [f64; 24],
    /// Cached hourly export profile averages.
    hourly_export_profile_cache: [f64; 24],
    /// Cached daily/hourly kWh heatmap.
    daily_hour_heatmap_cache: (Vec<String>, Vec<Vec<f64>>),
    /// Cached row sums for daily/hourly kWh heatmap.
    daily_hour_heatmap_row_sums_cache: Vec<f64>,
    /// Cached row metadata for daily/hourly kWh heatmap.
    daily_hour_heatmap_meta_cache: Vec<DailyDateMetadata>,
    /// Cached max value for daily/hourly kWh heatmap.
    daily_hour_heatmap_max_cache: f64,
    /// Cached daily/hourly cost heatmap.
    daily_hour_cost_heatmap_cache: (Vec<String>, Vec<Vec<f64>>),
    /// Cached row sums for daily/hourly cost heatmap.
    daily_hour_cost_heatmap_row_sums_cache: Vec<f64>,
    /// Cached row metadata for daily/hourly cost heatmap.
    daily_hour_cost_heatmap_meta_cache: Vec<DailyDateMetadata>,
    /// Cached max value for daily/hourly cost heatmap.
    daily_hour_cost_heatmap_max_cache: f64,
    /// Cached daytime export sparkline data.
    daily_daytime_export_cache: (Vec<String>, Vec<Vec<f64>>, Vec<f64>),
    /// Cached max value for daytime export sparklines.
    daily_daytime_export_max_cache: f64,
    /// Cached row metadata for daytime export rows.
    daily_daytime_export_meta_cache: Vec<DailyDateMetadata>,
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
        let mut merged_data: BTreeMap<DateTime<Utc>, ElectricDataPoint> = BTreeMap::new();
        let mut col_indices = None;

        super::loader::for_each_record(csv_contents, |headers, record| {
            if col_indices.is_none() {
                col_indices = Some((
                    headers
                        .iter()
                        .position(|h| h == "TYPE")
                        .context("TYPE column missing")?,
                    headers
                        .iter()
                        .position(|h| h == "DATE")
                        .context("DATE column missing")?,
                    headers
                        .iter()
                        .position(|h| h == "START TIME")
                        .context("START TIME column missing")?,
                    headers
                        .iter()
                        .position(|h| h == "IMPORT (kWh)")
                        .context("IMPORT column missing")?,
                    headers.iter().position(|h| h == "TOTAL IMPORT COST"),
                    headers.iter().position(|h| h == "EXPORT (kWh)"),
                ));
            }

            let (type_i, date_i, time_i, kwh_i, cost_i, export_i) = col_indices.unwrap();

            if !record
                .get(type_i)
                .map(|s| s.contains("Electric usage"))
                .unwrap_or(false)
            {
                return Ok(());
            }

            let date_str = record.get(date_i).context("DATE field missing")?;
            let time_str = record.get(time_i).context("TIME field missing")?;

            let dt = NaiveDateTime::parse_from_str(
                &format!("{} {}", date_str, time_str),
                "%Y-%m-%d %H:%M",
            )
            .or_else(|_| {
                NaiveDateTime::parse_from_str(
                    &format!("{} {}", date_str, time_str),
                    "%m/%d/%Y %H:%M",
                )
            })
            .with_context(|| format!("Failed to parse date/time: {} {}", date_str, time_str))?;

            let timestamp = DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc);

            match merged_data.entry(timestamp) {
                std::collections::btree_map::Entry::Occupied(_) => return Ok(()),
                std::collections::btree_map::Entry::Vacant(slot) => {
                    let kwh: f64 = record
                        .get(kwh_i)
                        .context("kWh missing")?
                        .parse()
                        .context("Invalid kWh")?;
                    let cost = cost_i
                        .and_then(|i| record.get(i))
                        .map(|s| s.trim_start_matches('$').parse().unwrap_or(0.0))
                        .unwrap_or(0.0);
                    let export_kwh = export_i
                        .and_then(|i| record.get(i))
                        .map(|s| s.parse().unwrap_or(0.0))
                        .unwrap_or(0.0);

                    slot.insert(ElectricDataPoint {
                        timestamp,
                        kwh,
                        cost,
                        export_kwh,
                    });
                }
            }

            Ok(())
        })?;

        let data: Vec<ElectricDataPoint> = merged_data.into_values().collect();

        let mut this = Self {
            data,
            monthly_kwh_sums: HashMap::new(),
            monthly_cost_sums: HashMap::new(),
            monthly_export_sums: HashMap::new(),
            yearly_kwh_sums: HashMap::new(),
            yearly_cost_sums: HashMap::new(),
            yearly_export_sums: HashMap::new(),
            daily_totals_cache: Vec::new(),
            daily_totals_avg7_cache: Vec::new(),
            daily_points_cache: Vec::new(),
            daily_avg7_points_cache: Vec::new(),
            daily_bounds_cache: None,
            weekday_hour_average_cache: [[0.0; 24]; 7],
            weekday_hour_heatmap_cache: Vec::new(),
            weekday_hour_heatmap_max_cache: 0.0,
            hourly_profile_cache: [0.0; 24],
            hourly_export_profile_cache: [0.0; 24],
            daily_hour_heatmap_cache: (Vec::new(), Vec::new()),
            daily_hour_heatmap_row_sums_cache: Vec::new(),
            daily_hour_heatmap_meta_cache: Vec::new(),
            daily_hour_heatmap_max_cache: 0.0,
            daily_hour_cost_heatmap_cache: (Vec::new(), Vec::new()),
            daily_hour_cost_heatmap_row_sums_cache: Vec::new(),
            daily_hour_cost_heatmap_meta_cache: Vec::new(),
            daily_hour_cost_heatmap_max_cache: 0.0,
            daily_daytime_export_cache: (Vec::new(), Vec::new(), Vec::new()),
            daily_daytime_export_max_cache: 0.0,
            daily_daytime_export_meta_cache: Vec::new(),
        };

        this.recalculate_summaries();
        this.recalculate_caches();

        Ok(this)
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

            *self
                .monthly_kwh_sums
                .entry(month_key.clone())
                .or_insert(0.0) += point.kwh;
            *self
                .monthly_cost_sums
                .entry(month_key.clone())
                .or_insert(0.0) += point.cost;
            *self.monthly_export_sums.entry(month_key).or_insert(0.0) += point.export_kwh;

            *self.yearly_kwh_sums.entry(year_key.clone()).or_insert(0.0) += point.kwh;
            *self.yearly_cost_sums.entry(year_key.clone()).or_insert(0.0) += point.cost;
            *self.yearly_export_sums.entry(year_key).or_insert(0.0) += point.export_kwh;
        }
    }

    fn recalculate_caches(&mut self) {
        self.daily_totals_cache = Self::compute_daily_totals_from_points(&self.data);
        self.daily_totals_avg7_cache = Self::compute_rolling_average(&self.daily_totals_cache, 7);
        self.daily_points_cache = self
            .daily_totals_cache
            .iter()
            .map(|(dt, val)| PlotPoint::new(dt.timestamp() as f64, *val))
            .collect();
        self.daily_avg7_points_cache = self
            .daily_totals_avg7_cache
            .iter()
            .map(|(dt, val)| PlotPoint::new(dt.timestamp() as f64, *val))
            .collect();

        self.daily_bounds_cache = self
            .daily_totals_cache
            .first()
            .zip(self.daily_totals_cache.last())
            .map(|((start, _), (end, _))| (start.timestamp() as f64, end.timestamp() as f64));

        self.weekday_hour_average_cache = Self::compute_weekday_hour_average(&self.data);
        self.weekday_hour_heatmap_cache = self
            .weekday_hour_average_cache
            .iter()
            .map(|day| day.to_vec())
            .collect();
        self.weekday_hour_heatmap_max_cache = self
            .weekday_hour_average_cache
            .iter()
            .flat_map(|day| day.iter())
            .copied()
            .fold(0.0_f64, f64::max);

        self.hourly_profile_cache = Self::compute_hourly_profile(&self.data);
        self.hourly_export_profile_cache = Self::compute_hourly_export_profile(&self.data);

        let (dates, heatmap, sums, max_val, meta) = Self::compute_daily_hour_heatmap(&self.data);
        self.daily_hour_heatmap_cache = (dates, heatmap);
        self.daily_hour_heatmap_row_sums_cache = sums;
        self.daily_hour_heatmap_max_cache = max_val;
        self.daily_hour_heatmap_meta_cache = meta;

        let (dates, heatmap, sums, max_val, meta) =
            Self::compute_daily_hour_cost_heatmap(&self.data);
        self.daily_hour_cost_heatmap_cache = (dates, heatmap);
        self.daily_hour_cost_heatmap_row_sums_cache = sums;
        self.daily_hour_cost_heatmap_max_cache = max_val;
        self.daily_hour_cost_heatmap_meta_cache = meta;

        let (dates, rows, sums, max_val, meta) =
            Self::compute_daily_daytime_export_data(&self.data);
        self.daily_daytime_export_cache = (dates, rows, sums);
        self.daily_daytime_export_max_cache = max_val;
        self.daily_daytime_export_meta_cache = meta;
    }

    fn build_daily_metadata(date: NaiveDate) -> DailyDateMetadata {
        let weekday = date.weekday();
        let is_saturday = weekday == Weekday::Sat;
        let is_sunday = weekday == Weekday::Sun;
        let date_key = date.format("%Y-%m-%d").to_string();

        DailyDateMetadata {
            date_key: date_key.clone(),
            year_key: date.format("%Y").to_string(),
            month_key: date.format("%Y-%m").to_string(),
            display_label: format!("{} {}", date_key, date.format("%a")),
            is_weekend: is_saturday || is_sunday,
            is_saturday,
            is_sunday,
        }
    }

    fn compute_daily_totals_from_points(data: &[ElectricDataPoint]) -> Vec<(DateTime<Utc>, f64)> {
        let mut daily: BTreeMap<NaiveDate, f64> = BTreeMap::new();

        for point in data {
            *daily.entry(point.timestamp.date_naive()).or_insert(0.0) += point.kwh;
        }

        daily
            .into_iter()
            .filter_map(|(date, kwh)| {
                date.and_hms_opt(0, 0, 0)
                    .map(|dt| (DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc), kwh))
            })
            .collect()
    }

    fn compute_rolling_average(
        data: &[(DateTime<Utc>, f64)],
        window: usize,
    ) -> Vec<(DateTime<Utc>, f64)> {
        if data.len() < window {
            return data.to_vec();
        }

        let mut result = Vec::with_capacity(data.len());
        let half_window = window / 2;

        for i in 0..data.len() {
            let start = i.saturating_sub(half_window);
            let end = (i + half_window + 1).min(data.len());
            let sum: f64 = data[start..end].iter().map(|(_, v)| v).sum();
            let count = (end - start) as f64;
            result.push((data[i].0, sum / count));
        }

        result
    }

    fn compute_weekday_hour_average(data: &[ElectricDataPoint]) -> [[f64; 24]; 7] {
        let mut totals = [[0.0; 24]; 7];
        let mut counts = [[0u32; 24]; 7];

        for point in data {
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

    fn compute_hourly_profile(data: &[ElectricDataPoint]) -> [f64; 24] {
        let mut totals = [0.0; 24];
        let mut counts = [0u32; 24];

        for point in data {
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

    fn compute_hourly_export_profile(data: &[ElectricDataPoint]) -> [f64; 24] {
        let mut totals = [0.0; 24];
        let mut counts = [0u32; 24];

        for point in data {
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

    fn compute_daily_hour_heatmap(
        data: &[ElectricDataPoint],
    ) -> (
        Vec<String>,
        Vec<Vec<f64>>,
        Vec<f64>,
        f64,
        Vec<DailyDateMetadata>,
    ) {
        let mut daily_data: BTreeMap<NaiveDate, [f64; 24]> = BTreeMap::new();

        for point in data {
            let date = point.timestamp.date_naive();
            let hour = point.timestamp.hour() as usize;
            let day_data = daily_data.entry(date).or_insert([0.0; 24]);
            day_data[hour] += point.kwh;
        }

        let mut dates = Vec::with_capacity(daily_data.len());
        let mut rows = Vec::with_capacity(daily_data.len());
        let mut row_sums = Vec::with_capacity(daily_data.len());
        let mut max_val = 0.0_f64;
        let mut meta = Vec::with_capacity(daily_data.len());

        for (date, row) in daily_data {
            let date_key = date.format("%Y-%m-%d").to_string();
            let values = row.to_vec();
            let row_sum: f64 = values.iter().sum();
            let row_max = values.iter().copied().fold(0.0_f64, f64::max);
            max_val = max_val.max(row_max);

            dates.push(date_key);
            rows.push(values);
            row_sums.push(row_sum);
            meta.push(Self::build_daily_metadata(date));
        }

        (dates, rows, row_sums, max_val, meta)
    }

    fn compute_daily_hour_cost_heatmap(
        data: &[ElectricDataPoint],
    ) -> (
        Vec<String>,
        Vec<Vec<f64>>,
        Vec<f64>,
        f64,
        Vec<DailyDateMetadata>,
    ) {
        let mut daily_data: BTreeMap<NaiveDate, [f64; 24]> = BTreeMap::new();

        for point in data {
            let date = point.timestamp.date_naive();
            let hour = point.timestamp.hour() as usize;
            let day_data = daily_data.entry(date).or_insert([0.0; 24]);
            day_data[hour] += point.cost;
        }

        let mut dates = Vec::with_capacity(daily_data.len());
        let mut rows = Vec::with_capacity(daily_data.len());
        let mut row_sums = Vec::with_capacity(daily_data.len());
        let mut max_val = 0.0_f64;
        let mut meta = Vec::with_capacity(daily_data.len());

        for (date, row) in daily_data {
            let date_key = date.format("%Y-%m-%d").to_string();
            let values = row.to_vec();
            let row_sum: f64 = values.iter().sum();
            let row_max = values.iter().copied().fold(0.0_f64, f64::max);
            max_val = max_val.max(row_max);

            dates.push(date_key);
            rows.push(values);
            row_sums.push(row_sum);
            meta.push(Self::build_daily_metadata(date));
        }

        (dates, rows, row_sums, max_val, meta)
    }

    fn compute_daily_daytime_export_data(
        data: &[ElectricDataPoint],
    ) -> (
        Vec<String>,
        Vec<Vec<f64>>,
        Vec<f64>,
        f64,
        Vec<DailyDateMetadata>,
    ) {
        let mut daily_data: BTreeMap<NaiveDate, [f64; 13]> = BTreeMap::new();

        for point in data {
            let hour = point.timestamp.hour() as usize;
            if (6..=18).contains(&hour) {
                let date = point.timestamp.date_naive();
                let day_data = daily_data.entry(date).or_insert([0.0; 13]);
                day_data[hour - 6] += point.export_kwh;
            }
        }

        let mut dates = Vec::with_capacity(daily_data.len());
        let mut rows = Vec::with_capacity(daily_data.len());
        let mut sums = Vec::with_capacity(daily_data.len());
        let mut max_val = 0.0_f64;
        let mut meta = Vec::with_capacity(daily_data.len());

        for (date, row) in daily_data {
            let date_key = date.format("%Y-%m-%d").to_string();
            let values = row.to_vec();
            let day_sum: f64 = values.iter().sum();
            let row_max = values.iter().copied().fold(0.0_f64, f64::max);
            max_val = max_val.max(row_max);

            dates.push(date_key);
            rows.push(values);
            sums.push(day_sum);
            meta.push(Self::build_daily_metadata(date));
        }

        (dates, rows, sums, max_val, meta)
    }

    /// Returns plot-ready points for daily charts without per-frame allocation.
    pub fn daily_plot_points_cached(&self) -> &[PlotPoint] {
        &self.daily_points_cache
    }

    /// Returns plot-ready 7-day average points without per-frame allocation.
    pub fn daily_avg7_plot_points_cached(&self) -> &[PlotPoint] {
        &self.daily_avg7_points_cache
    }

    /// Returns x-bounds for daily charts.
    pub fn daily_chart_bounds(&self) -> Option<(f64, f64)> {
        self.daily_bounds_cache
    }

    /// Returns cached weekday/hour heatmap rows without per-frame allocations.
    pub fn weekday_hour_heatmap_cached(&self) -> &[Vec<f64>] {
        &self.weekday_hour_heatmap_cache
    }

    /// Returns cached weekday/hour heatmap max value.
    pub fn weekday_hour_heatmap_max_cached(&self) -> f64 {
        self.weekday_hour_heatmap_max_cache
    }

    /// Returns cached hourly profile without recomputation.
    pub fn hourly_profile_cached(&self) -> &[f64; 24] {
        &self.hourly_profile_cache
    }

    /// Returns cached hourly export profile without recomputation.
    pub fn hourly_export_profile_cached(&self) -> &[f64; 24] {
        &self.hourly_export_profile_cache
    }

    /// Returns cached kWh heatmap without per-frame allocations.
    pub fn daily_hour_heatmap_cached(&self) -> (&[String], &[Vec<f64>]) {
        (
            &self.daily_hour_heatmap_cache.0,
            &self.daily_hour_heatmap_cache.1,
        )
    }

    /// Returns cached kWh heatmap row sums.
    pub fn daily_hour_heatmap_row_sums_cached(&self) -> &[f64] {
        &self.daily_hour_heatmap_row_sums_cache
    }

    /// Returns cached metadata for kWh heatmap rows.
    pub fn daily_hour_heatmap_meta_cached(&self) -> &[DailyDateMetadata] {
        &self.daily_hour_heatmap_meta_cache
    }

    /// Returns cached cost heatmap without per-frame allocations.
    pub fn daily_hour_cost_heatmap_cached(&self) -> (&[String], &[Vec<f64>]) {
        (
            &self.daily_hour_cost_heatmap_cache.0,
            &self.daily_hour_cost_heatmap_cache.1,
        )
    }

    /// Returns cached cost heatmap row sums.
    pub fn daily_hour_cost_heatmap_row_sums_cached(&self) -> &[f64] {
        &self.daily_hour_cost_heatmap_row_sums_cache
    }

    /// Returns cached metadata for cost heatmap rows.
    pub fn daily_hour_cost_heatmap_meta_cached(&self) -> &[DailyDateMetadata] {
        &self.daily_hour_cost_heatmap_meta_cache
    }

    /// Returns cached daytime export sparkline data without per-frame allocations.
    pub fn daily_daytime_export_data_cached(
        &self,
    ) -> (&[String], &[Vec<f64>], &[f64], f64, &[DailyDateMetadata]) {
        (
            &self.daily_daytime_export_cache.0,
            &self.daily_daytime_export_cache.1,
            &self.daily_daytime_export_cache.2,
            self.daily_daytime_export_max_cache,
            &self.daily_daytime_export_meta_cache,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_electric_data_load() {
        let csv = "TYPE,DATE,START TIME,IMPORT (kWh),TOTAL IMPORT COST\n\
                  Electric usage,2023-01-01,00:00,1.5,$0.30\n\
                  Electric usage,2023-01-01,01:00,2.0,$0.40"
            .to_string();

        let result = ElectricData::load(&[csv]).unwrap();
        assert_eq!(result.data.len(), 2);
        assert_eq!(result.data[0].kwh, 1.5);
        assert_eq!(result.data[0].cost, 0.30);
    }

    #[test]
    fn test_electric_data_load_alternate_date_format() {
        let csv = "TYPE,DATE,START TIME,IMPORT (kWh),TOTAL IMPORT COST\n\
                  Electric usage,01/01/2023,00:00,1.5,$0.30"
            .to_string();

        let result = ElectricData::load(&[csv]).unwrap();
        assert_eq!(result.data.len(), 1);
        assert_eq!(result.data[0].kwh, 1.5);
    }

    #[test]
    fn test_electric_monthly_sums() {
        let csv = "TYPE,DATE,START TIME,IMPORT (kWh),TOTAL IMPORT COST\n\
                  Electric usage,2023-01-01,00:00,1.0,$0.20\n\
                  Electric usage,2023-01-02,00:00,2.0,$0.40\n\
                  Electric usage,2023-02-01,00:00,3.0,$0.60"
            .to_string();

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
                   Electric usage,2023-01-01,00:00,1.0,$0.20"
            .to_string();
        let csv2 = "TYPE,DATE,START TIME,IMPORT (kWh),TOTAL IMPORT COST\n\
                   Electric usage,2023-01-01,00:00,5.0,$1.00\n\
                   Electric usage,2023-01-01,01:00,2.0,$0.40"
            .to_string();

        let result = ElectricData::load(&[csv1, csv2]).unwrap();

        // Should have 2 points, and the first point should be from csv1 (1.0 kWh)
        assert_eq!(result.data.len(), 2);
        assert_eq!(result.data[0].kwh, 1.0);
        assert_eq!(result.data[1].kwh, 2.0);
    }
}
