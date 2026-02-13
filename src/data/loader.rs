use anyhow::{Context, Result};
use csv::ReaderBuilder;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

const HEADER_PATTERN: &str = "TYPE,DATE,START TIME";

fn is_header_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed
        .get(..HEADER_PATTERN.len())
        .map(|prefix| prefix.eq_ignore_ascii_case(HEADER_PATTERN))
        .unwrap_or(false)
}

/// Reads a CSV file and strips leading metadata by searching for the header row.
///
/// The function looks for a line starting with "TYPE,DATE,START TIME" (case-insensitive)
/// and returns the content starting from that line.
pub fn read_csv_with_header(path: &Path) -> Result<String> {
    let file = File::open(path).with_context(|| {
        format!(
            "Failed to open file: {}\nPlease check:\n- The file exists and is accessible\n- You have permission to read the file\n- The file is not currently open in another program",
            path.display()
        )
    })?;

    let reader = BufReader::new(file);
    let mut found_header = false;
    let mut output = String::new();

    for line_result in reader.lines() {
        let line = line_result.with_context(|| {
            format!(
                "Failed to read file: {}\nPlease check:\n- The file is a valid text file\n- The file encoding is supported (UTF-8 recommended)",
                path.display()
            )
        })?;

        if !found_header {
            found_header = is_header_line(&line);
        }

        if found_header {
            if !output.is_empty() {
                output.push('\n');
            }
            output.push_str(&line);
        }
    }

    if !found_header {
        anyhow::bail!(
            "Could not find valid PG&E header in file: {}\n\nExpected header containing: '{}'\n\nPlease ensure:\n- This is a valid PG&E export file\n- The file contains the required header row\n- The file format matches PG&E's standard export",
            path.display(),
            HEADER_PATTERN
        );
    }

    Ok(output)
}

/// Automatically finds all CSV files in the given directory that match the pattern.
pub fn autodetect_csv_files(dir: &Path, pattern: &str) -> Vec<PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut matches: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                let file_name = p.file_name().and_then(|n| n.to_str());
                let is_csv = p
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("csv"))
                    .unwrap_or(false);

                file_name
                    .map(|s| s.contains(pattern) && is_csv && !s.starts_with(".~lock."))
                    .unwrap_or(false)
            })
            .collect();

        matches.sort();
        matches
    } else {
        Vec::new()
    }
}

/// Iterates over all records in multiple CSV strings.
///
/// Handler receives (headers, record).
pub fn for_each_record<F>(csv_contents: &[String], mut handler: F) -> Result<()>
where
    F: FnMut(&csv::StringRecord, &csv::StringRecord) -> Result<()>,
{
    for content in csv_contents {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(content.as_bytes());

        let headers = reader.headers()?.clone();
        for result in reader.records() {
            let record = result?;
            handler(&headers, &record)?;
        }
    }
    Ok(())
}

/// Opens a native file dialog to let the user select a CSV file.
pub fn select_csv_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("CSV Files", &["csv"])
        .pick_file()
}
