use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

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
}

impl GasData {
    /// Loads gas usage data from one or more CSV strings.
    ///
    /// Expects a specific PGE export format with columns like "TYPE", "DATE", and "COST".
    ///
    /// If multiple CSVs are provided, data is merged. In case of overlapping
    /// dates, the firstOccurrence preference is given.
    pub fn load(csv_contents: &[String]) -> Result<Self> {
        let mut merged_data: std::collections::BTreeMap<DateTime<Utc>, GasDataPoint> = std::collections::BTreeMap::new();
        let mut col_indices = None;

        super::loader::for_each_record(csv_contents, |headers, record| {
            if col_indices.is_none() {
                col_indices = Some((
                    headers.iter().position(|h| h == "TYPE").context("TYPE column missing")?,
                    headers.iter().position(|h| h == "DATE").context("DATE column missing")?,
                    headers.iter().position(|h| h == "COST").context("COST column missing")?,
                ));
            }

            let (type_i, date_i, cost_i) = col_indices.unwrap();

            if !record.get(type_i).map(|s| s.contains("Natural gas usage")).unwrap_or(false) {
                return Ok(());
            }

            let date_str = record.get(date_i).context("DATE field missing")?;
            let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .or_else(|_| chrono::NaiveDate::parse_from_str(date_str, "%m/%d/%Y"))
                .with_context(|| format!("Failed to parse date: {}", date_str))?;

            let timestamp = DateTime::<Utc>::from_naive_utc_and_offset(
                date.and_hms_opt(0, 0, 0).unwrap(), Utc);

            if merged_data.contains_key(&timestamp) {
                return Ok(());
            }

            let cost: f64 = record.get(cost_i).context("COST missing")?.trim_start_matches('$').parse().context("Invalid cost")?;

            merged_data.insert(timestamp, GasDataPoint { date: timestamp, cost });
            Ok(())
        })?;

        let data: Vec<GasDataPoint> = merged_data.into_values().collect();
        // BTreeMap is already sorted by date

        Ok(Self { data })
    }

    /// Get daily totals (already daily, but aggregate just in case)
    pub fn daily_totals(&self) -> Vec<(DateTime<Utc>, f64)> {
        let mut daily: HashMap<String, f64> = HashMap::new();

        for point in &self.data {
            let date_key = point.date.format("%Y-%m-%d").to_string();
            *daily.entry(date_key).or_insert(0.0) += point.cost;
        }

        let mut result: Vec<_> = daily
            .into_iter()
            .filter_map(|(date, cost)| {
                chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d")
                    .ok()
                    .and_then(|d: chrono::NaiveDate| d.and_hms_opt(0, 0, 0))
                    .map(|dt: chrono::NaiveDateTime| (dt.and_utc(), cost))
            })
            .collect();

        result.sort_by_key(|(dt, _)| *dt);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_data_load() {
        let csv = "TYPE,DATE,COST\n\
                  Natural gas usage,2023-01-01,$5.50\n\
                  Natural gas usage,2023-01-02,$6.20".to_string();

        let result = GasData::load(&[csv]).unwrap();
        assert_eq!(result.data.len(), 2);
        assert_eq!(result.data[0].cost, 5.50);
        assert_eq!(result.data[1].cost, 6.20);
    }

    #[test]
    fn test_gas_data_load_alternate_date_format() {
        let csv = "TYPE,DATE,COST\n\
                  Natural gas usage,01/01/2023,$5.50".to_string();

        let result = GasData::load(&[csv]).unwrap();
        assert_eq!(result.data.len(), 1);
        assert_eq!(result.data[0].cost, 5.50);
    }
}
