use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, NaiveDateTime, Timelike, Utc};
use csv::ReaderBuilder;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ElectricDataPoint {
    pub timestamp: DateTime<Utc>,
    pub kwh: f64,
    pub cost: f64,
}

#[derive(Debug, Clone)]
pub struct ElectricData {
    pub data: Vec<ElectricDataPoint>,
}

impl ElectricData {
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
        
        let mut data = Vec::new();
        
        for result in reader.records() {
            let record = result?;
            
            // Get TYPE field
            let type_field = record.get(type_idx)
                .context("Missing TYPE field")?;
            
            // Filter to electric usage only
            if !type_field.contains("Electric usage") {
                continue;
            }
            
            // Parse timestamp
            let date = record.get(date_idx).context("Missing DATE field")?;
            let start_time = record.get(start_time_idx).context("Missing START TIME field")?;
            let datetime_str = format!("{} {}", date, start_time);
            let timestamp = NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M")
                .or_else(|_| NaiveDateTime::parse_from_str(&datetime_str, "%m/%d/%Y %H:%M"))
                .context("Failed to parse datetime")?
                .and_utc();
            
            // Parse kWh
            let kwh_str = record.get(kwh_idx).context("Missing kWh field")?;
            let kwh = kwh_str.parse::<f64>().unwrap_or(0.0);
            
            // Parse cost (optional)
            let cost = if let Some(idx) = cost_idx {
                record.get(idx)
                    .and_then(|c| c.replace('$', "").replace(',', "").parse::<f64>().ok())
                    .unwrap_or(0.0)
            } else {
                0.0
            };
            
            data.push(ElectricDataPoint {
                timestamp,
                kwh,
                cost,
            });
        }
        
        // Sort by timestamp
        data.sort_by_key(|d| d.timestamp);
        
        Ok(ElectricData { data })
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
    
    /// Get average kWh by weekday and hour
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
    
    /// Get hourly profile (average kWh by hour of day)
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
    
    /// Get daily by-hour heatmap data
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
    
    /// Get daily by-hour cost heatmap data
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
}
