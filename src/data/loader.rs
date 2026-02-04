use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

const HEADER_PATTERN: &str = "TYPE,DATE,START TIME";

/// Reads a CSV file and strips leading metadata by searching for the header row.
///
/// The function looks for a line starting with "TYPE,DATE,START TIME" (case-insensitive)
/// and returns the content starting from that line.
pub fn read_csv_with_header(path: &Path) -> Result<String> {
    let file = File::open(path)
        .with_context(|| format!("Failed to open file: {}\nPlease check:\n• The file exists and is accessible\n• You have permission to read the file\n• The file is not currently open in another program", path.display()))?;

    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()
        .with_context(|| format!("Failed to read file: {}\nPlease check:\n• The file is a valid text file\n• The file encoding is supported (UTF-8 recommended)", path.display()))?;

    // Find header line
    let header_idx = lines
        .iter()
        .position(|line| line.trim().to_uppercase().starts_with(HEADER_PATTERN))
        .context(format!(
            "Could not find valid PG&E header in file: {}\n\nExpected header containing: '{}'\n\nPlease ensure:\n• This is a valid PG&E export file\n• The file contains the required header row\n• The file format matches PG&E's standard export",
            path.display(), HEADER_PATTERN
        ))?;

    // Return CSV content from header onwards
    Ok(lines[header_idx..].join("\n"))
}

/// Automatically finds all CSV files in the given directory that match the pattern.
pub fn autodetect_csv_files(dir: &Path, pattern: &str) -> Vec<PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut matches: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| {
                        // Must contain pattern, end with .csv, and not be a lock file
                        s.contains(pattern)
                            && s.to_lowercase().ends_with(".csv")
                            && !s.starts_with(".~lock.")
                    })
                    .unwrap_or(false)
            })
            .collect();

        matches.sort();
        matches
    } else {
        Vec::new()
    }
}

/// Opens a native file dialog to let the user select a CSV file.
pub fn select_csv_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("CSV Files", &["csv"])
        .pick_file()
}
