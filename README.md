# PG&E Usage Analyzer - Rust GUI Application

A modern Windows GUI application for visualizing PG&E electric and gas usage data, built with Rust.

## Features

- **6 Interactive Charts**:
  1. Daily kWh - Line chart with 7-day rolling average
  2. Average kWh by Weekday and Hour - Heatmap visualization
  3. Daily by-hour kWh Heatmap - Detailed hourly usage patterns
  4. Daily by-hour Cost Heatmap - Cost analysis by hour
  5. Average Daily Profile - Typical hourly usage pattern
  6. Gas Daily Usage - Gas cost trends

- **Modern Windows 11-inspired UI**
- **Auto-detection of CSV files**
- **File dialog for manual CSV selection**
- **Interactive tooltips on heatmaps**
- **Smooth navigation between charts**

## Building the Application

### Prerequisites

- Rust toolchain (already installed)
- Windows 10/11

### Build Instructions

#### Option 1: Debug Build (Faster compilation)

```powershell
cd c:\proj\PGE\pge-analyzer
cargo build
```

The executable will be at: `target\debug\pge-analyzer.exe`

#### Option 2: Release Build (Optimized performance)

```powershell
cd c:\proj\PGE\pge-analyzer
cargo build --release
```

The executable will be at: `target\release\pge-analyzer.exe`

### Troubleshooting Build Issues

If you encounter "The process cannot access the file" errors (Error 32), this is typically caused by antivirus software scanning the build artifacts. Try these solutions:

1. **Add Build Exclusion to Windows Defender**:
   - Open Windows Security
   - Go to "Virus & threat protection" → "Manage settings"
   - Scroll to "Exclusions" → "Add or remove exclusions"
   - Add folder: `C:\proj\PGE\pge-analyzer\target`

2. **Temporarily Disable Real-time Protection**:
   - Open Windows Security
   - Go to "Virus & threat protection" → "Manage settings"
   - Turn off "Real-time protection" temporarily
   - Run the build
   - Re-enable protection after build completes

3. **Use Cargo Watch** (alternative):
   ```powershell
   cargo install cargo-watch
   cargo watch -x build
   ```

4. **Clean and Retry**:
   ```powershell
   cargo clean
   Start-Sleep -Seconds 5
   cargo build
   ```

## Running the Application

### From the Build Directory

```powershell
cd c:\proj\PGE\pge-analyzer
cargo run
```

### Or Run the Executable Directly

```powershell
# Debug build
.\target\debug\pge-analyzer.exe

# Release build
.\target\release\pge-analyzer.exe
```

## Usage

1. **Auto-Load Data**: The application automatically looks for CSV files matching:
   - `pge_electric*` for electric usage data
   - `pge_natural_gas*` for gas usage data

2. **Manual Load**: Use the "📂 Load Electric CSV" and "📂 Load Gas CSV" buttons in the sidebar to select files manually

3. **Navigate Charts**: Click on any chart name in the sidebar to switch views

4. **Interactive Features**:
   - Hover over heatmap cells to see detailed values
   - Scroll through large heatmaps
   - Resize the window as needed

## Project Structure

```
pge-analyzer/
├── src/
│   ├── main.rs              # Application entry point
│   ├── data/                # Data loading and processing
│   │   ├── mod.rs
│   │   ├── loader.rs        # CSV file handling
│   │   ├── electric.rs      # Electric data structures
│   │   └── gas.rs           # Gas data structures
│   ├── charts/              # Chart rendering modules
│   │   ├── mod.rs
│   │   ├── daily_kwh.rs
│   │   ├── weekday_heatmap.rs
│   │   ├── daily_heatmap.rs
│   │   ├── cost_heatmap.rs
│   │   ├── profile.rs
│   │   └── gas_daily.rs
│   └── ui/                  # UI components and styling
│       ├── mod.rs
│       └── styles.rs
└── Cargo.toml               # Dependencies and configuration
```

## Dependencies

- **eframe** & **egui**: Modern immediate-mode GUI framework
- **egui_plot**: Plotting capabilities for line charts
- **csv** & **serde**: CSV parsing and deserialization
- **chrono**: Date and time handling
- **anyhow**: Error handling
- **rfd**: Native file dialogs

## Comparison with Python Dashboard

This Rust application provides the same visualizations as `dashboard.html` but with:
- ✅ Native Windows performance
- ✅ No browser required
- ✅ Faster data loading
- ✅ Lower memory usage
- ✅ Modern desktop UI

## License

This project matches the functionality of the Python `analysis.py` script.
