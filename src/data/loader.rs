use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

const HEADER_PATTERN: &str = "TYPE,DATE,START TIME";

/// Read CSV file with auto-detection of header line
pub fn read_csv_with_header(path: &Path) -> Result<String> {
    let file = File::open(path)
        .with_context(|| format!("Failed to open file: {}", path.display()))?;
    
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
    
    // Find header line
    let header_idx = lines
        .iter()
        .position(|line| line.trim().to_uppercase().starts_with(HEADER_PATTERN))
        .context("Could not find header row")?;
    
    // Return CSV content from header onwards
    Ok(lines[header_idx..].join("\n"))
}

/// Auto-detect CSV file by pattern in the given directory
pub fn autodetect_csv(dir: &Path, pattern: &str) -> Option<PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut matches: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.contains(pattern))
                    .unwrap_or(false)
            })
            .collect();
        
        matches.sort();
        matches.into_iter().next()
    } else {
        None
    }
}

/// Open file dialog to select a CSV file
pub fn select_csv_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("CSV Files", &["csv"])
        .pick_file()
}
