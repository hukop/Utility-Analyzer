use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default directory to look for CSV files
    pub default_data_dir: Option<PathBuf>,
    /// Last used electric CSV file
    pub last_electric_file: Option<PathBuf>,
    /// Last used gas CSV file
    pub last_gas_file: Option<PathBuf>,
    /// Window settings
    pub window: WindowConfig,
    /// UI preferences
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Window width
    pub width: f32,
    /// Window height
    pub height: f32,
    /// Window x position
    pub x: Option<f32>,
    /// Window y position
    pub y: Option<f32>,
    /// Whether to start maximized
    pub maximized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Default chart view to show on startup
    pub default_chart: String,
    /// Whether to use dark mode by default
    pub dark_mode: Option<bool>,
    /// Font size multiplier
    pub font_scale: f32,
    /// Heatmap color palette
    pub heatmap_palette: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_data_dir: None,
            last_electric_file: None,
            last_gas_file: None,
            window: WindowConfig {
                width: 1400.0,
                height: 900.0,
                x: None,
                y: None,
                maximized: false,
            },
            ui: UiConfig {
                default_chart: "DailyKwh".to_string(),
                dark_mode: None,
                font_scale: 1.0,
                heatmap_palette: Some("Viridis".to_string()),
            },
        }
    }
}

impl Config {
    /// Load configuration from file, creating default if it doesn't exist
    pub fn load() -> Result<Self> {
        let config_path = get_config_path();

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).with_context(|| {
                format!("Failed to read config file: {}", config_path.display())
            })?;

            toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {}", config_path.display()))
        } else {
            // Create default config
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path();

        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory: {}", parent.display())
            })?;
        }

        let content = toml::to_string_pretty(self).context("Failed to serialize configuration")?;

        std::fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

        Ok(())
    }

    /// Get the effective data directory (config default or current directory)
    pub fn get_data_dir(&self) -> PathBuf {
        self.default_data_dir
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }
}

/// Get the path to the configuration file
pub fn get_config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("pge-analyzer");

    config_dir.join("config.toml")
}
