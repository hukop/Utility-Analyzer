use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use egui_plot::PlotPoint;
use std::collections::BTreeMap;

/// A single data point representing gas usage cost on a specific date.
#[derive(Debug, Clone)]
pub struct GasDataPoint {
    /// The date of the gas usage.
    pub date: DateTime<Utc>,
    /// The cost associated with this date.
    pub cost: f64,
}

/// A collection of gas usage data points.
#[derive(Debug, Clone)]
pub struct GasData {
    /// The list of gas usage data points, usually sorted by date.
    pub data: Vec<GasDataPoint>,
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
}

impl GasData {
    /// Loads gas usage data from one or more CSV strings.
    ///
    /// Expects a specific PGE export format with columns like "TYPE", "DATE", and "COST".
    ///
    /// If multiple CSVs are provided, data is merged. In case of overlapping
    /// dates, the firstOccurrence preference is given.
    pub fn load(csv_contents: &[String]) -> Result<Self> {
        let mut merged_data: BTreeMap<DateTime<Utc>, GasDataPoint> = BTreeMap::new();
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
                        .position(|h| h == "COST")
                        .context("COST column missing")?,
                ));
            }

            let (type_i, date_i, cost_i) = col_indices.unwrap();

            if !record
                .get(type_i)
                .map(|s| s.contains("Natural gas usage"))
                .unwrap_or(false)
            {
                return Ok(());
            }

            let date_str = record.get(date_i).context("DATE field missing")?;
            let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .or_else(|_| NaiveDate::parse_from_str(date_str, "%m/%d/%Y"))
                .with_context(|| format!("Failed to parse date: {}", date_str))?;

            let timestamp =
                DateTime::<Utc>::from_naive_utc_and_offset(date.and_hms_opt(0, 0, 0).unwrap(), Utc);

            match merged_data.entry(timestamp) {
                std::collections::btree_map::Entry::Occupied(_) => return Ok(()),
                std::collections::btree_map::Entry::Vacant(slot) => {
                    let cost: f64 = record
                        .get(cost_i)
                        .context("COST missing")?
                        .trim_start_matches('$')
                        .parse()
                        .context("Invalid cost")?;

                    slot.insert(GasDataPoint {
                        date: timestamp,
                        cost,
                    });
                }
            }

            Ok(())
        })?;

        let data: Vec<GasDataPoint> = merged_data.into_values().collect();

        let mut this = Self {
            data,
            daily_totals_cache: Vec::new(),
            daily_totals_avg7_cache: Vec::new(),
            daily_points_cache: Vec::new(),
            daily_avg7_points_cache: Vec::new(),
            daily_bounds_cache: None,
        };
        this.recalculate_caches();

        Ok(this)
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
    }

    fn compute_daily_totals_from_points(data: &[GasDataPoint]) -> Vec<(DateTime<Utc>, f64)> {
        let mut daily: BTreeMap<NaiveDate, f64> = BTreeMap::new();

        for point in data {
            *daily.entry(point.date.date_naive()).or_insert(0.0) += point.cost;
        }

        daily
            .into_iter()
            .filter_map(|(date, cost)| {
                date.and_hms_opt(0, 0, 0)
                    .map(|dt| (DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc), cost))
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

    /// Returns plot-ready points and 7-day average points filtered by the preset range.
    pub fn daily_plot_points_filtered(&self, preset: crate::data::DateRangePreset) -> (&[PlotPoint], &[PlotPoint], Option<(f64, f64)>) {
        if self.daily_totals_cache.is_empty() {
            return (&[], &[], None);
        }

        let latest = self.daily_totals_cache.last().unwrap().0.date_naive();
        if let Some(start_date) = preset.start_date(latest) {
            let start_idx = self
                .daily_totals_cache
                .iter()
                .position(|(dt, _)| dt.date_naive() >= start_date)
                .unwrap_or(0);

            let bounds = if start_idx < self.daily_totals_cache.len() {
                Some((
                    self.daily_totals_cache[start_idx].0.timestamp() as f64,
                    self.daily_totals_cache.last().unwrap().0.timestamp() as f64,
                ))
            } else {
                None
            };

            (&self.daily_points_cache[start_idx..], &self.daily_avg7_points_cache[start_idx..], bounds)
        } else {
            (&self.daily_points_cache, &self.daily_avg7_points_cache, self.daily_bounds_cache)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_data_load() {
        let csv = "TYPE,DATE,COST\n\
                  Natural gas usage,2023-01-01,$5.50\n\
                  Natural gas usage,2023-01-02,$6.20"
            .to_string();

        let result = GasData::load(&[csv]).unwrap();
        assert_eq!(result.data.len(), 2);
        assert_eq!(result.data[0].cost, 5.50);
        assert_eq!(result.data[1].cost, 6.20);
    }

    #[test]
    fn test_gas_data_load_alternate_date_format() {
        let csv = "TYPE,DATE,COST\n\
                  Natural gas usage,01/01/2023,$5.50"
            .to_string();

        let result = GasData::load(&[csv]).unwrap();
        assert_eq!(result.data.len(), 1);
        assert_eq!(result.data[0].cost, 5.50);
    }
}
