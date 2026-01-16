# Agent Guide — pge-analyzer

Purpose: help an assistant (e.g., Claude Sonnet) quickly understand, run, and modify this repository.

## Quick commands
- Build (debug): `cargo build`
- Build (release): `cargo build --release`
- Run (dev): `cargo run`
- Run executable (Windows): `target\\debug\\pge-analyzer.exe` or `target\\release\\pge-analyzer.exe`

## Key files (quick map)
- `Cargo.toml` — crate and dependencies
- `src/main.rs` — application entry, `PgeAnalyzerApp`, sidebar and chart dispatch
- `src/config.rs` — `Config` type and `Config::load()` / config location helpers
- `src/data/loader.rs` — `read_csv_with_header()`, `autodetect_csv()` and `select_csv_file()`
- `src/data/electric.rs` — `ElectricData::load()` and data structures
- `src/data/gas.rs` — `GasData::load()` and data structures
- `src/charts/` — chart rendering modules

## Configuration & where it lives
- Config is implemented in `src/config.rs`. The app calls `Config::load()` on startup; if missing it writes defaults to the platform config directory (see `config.example.toml`).

## Data handling & autodetect rules
- Autodetection searches the data directory for filenames containing `pge_electric` (electric) and `pge_natural_gas` (gas) (see `autodetect_csv()` in `src/data/loader.rs`).
- The CSV reader first strips file metadata by searching for a header row that starts with `TYPE,DATE,START TIME` (case-insensitive) using `read_csv_with_header()`.

## Sample CSV excerpts (included)
- `data/samples/electric_excerpt.csv` — minimal synthetic electric export that matches header expectations and shows `IMPORT (kWh)` and `TOTAL IMPORT COST` columns.
- `data/samples/gas_excerpt.csv` — minimal synthetic gas export matching `TYPE,DATE,COST` header.

Place these files via the GUI Load buttons or rely on `autodetect_csv()` if you copy them to the configured data dir.

## Helpful sample prompts for an assistant
- "Where does the app look for CSV files?" — point to `src/data/loader.rs::autodetect_csv` and `Config::get_data_dir()` in `src/config.rs`.
- "Add a new chart module for monthly totals" — update `src/charts/`, add a variant to the chart enum in `src/ui/mod.rs` and handle it in `src/main.rs` chart dispatch.
- "How can I change autodetect patterns?" — modify the substrings used by `autodetect_csv()` calls in `src/main.rs` (search for `pge_electric` / `pge_natural_gas`).

## Agent tasks checklist
- Run the app locally against `data/samples/*` to confirm parsing.
- Add or modify chart rendering in `src/charts/` and update `ChartView` in `src/ui/mod.rs`.
- Make autodetect patterns configurable via `config.toml` (if requested).

## Clarifying questions to ask the repo owner
1. Preferred default data directory (APPDATA, Documents, repo-local)?
2. Should autodetect patterns be configurable via `config.toml`?
3. Do you want sample CSVs included in the repo long-term or only in `docs/`?
4. CI expectations (build only, clippy/format checks, unit tests)?
5. Preferred release behavior on Windows (GUI-only, or keep console)?

---
Files: `data/samples/electric_excerpt.csv`, `data/samples/gas_excerpt.csv`
