use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use csv::ReaderBuilder;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GasDataPoint {
    pub date: DateTime<Utc>,
    pub cost: f64,
}

#[derive(Debug, Clone)]
pub struct GasData {
    pub data: Vec<GasDataPoint>,
}

impl GasData {
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
        let cost_idx = headers.iter().position(|h| h == "COST")
            .context("COST column not found")?;
        
        let mut data = Vec::new();
        
        for result in reader.records() {
            let record = result?;
            
            // Get TYPE field
            let type_field = record.get(type_idx)
                .context("Missing TYPE field")?;
            
            // Filter to natural gas usage only
            if !type_field.contains("Natural gas usage") {
                continue;
            }
            
            // Parse date
            let date_str = record.get(date_idx).context("Missing DATE field")?;
            let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .or_else(|_| NaiveDate::parse_from_str(date_str, "%m/%d/%Y"))
                .context("Failed to parse date")?
                .and_hms_opt(0, 0, 0)
                .context("Failed to create datetime")?
                .and_utc();
            
            // Parse cost
            let cost_str = record.get(cost_idx).context("Missing COST field")?;
            let cost = cost_str
                .replace('$', "")
                .replace(',', "")
                .parse::<f64>()
                .unwrap_or(0.0);
            
            data.push(GasDataPoint { date, cost });
        }
        
        // Sort by date
        data.sort_by_key(|d| d.date);
        
        Ok(GasData { data })
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
                NaiveDate::parse_from_str(&date, "%Y-%m-%d")
                    .ok()
                    .and_then(|d| d.and_hms_opt(0, 0, 0))
                    .map(|dt| (dt.and_utc(), cost))
            })
            .collect();
        
        result.sort_by_key(|(dt, _)| *dt);
        result
    }
}
