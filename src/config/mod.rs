//! Configuration loading and persistence.
//!
//! Configuration is stored in TOML format at `~/.config/crabchat/config.toml`.
//! If no config file exists, sensible defaults are used (including 20 built-in
//! IRC server presets and a randomly generated nickname).

pub mod model;
pub mod nickname;

use anyhow::{Context, Result};
use std::path::PathBuf;

pub use model::AppConfig;
pub use model::LoggingConfig;

/// Returns the platform-appropriate config file path
/// (`~/.config/crabchat/config.toml` on Linux/macOS).
fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("crabchat")
        .join("config.toml")
}

/// Load the application configuration from disk.
/// Returns `AppConfig::default()` if the config file does not exist.
pub fn load_config() -> Result<AppConfig> {
    let path = config_path();
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config from {}", path.display()))?;
    let config: AppConfig =
        toml::from_str(&contents).with_context(|| "Failed to parse config file")?;
    Ok(config)
}

/// Serialize and write the configuration to disk, creating parent directories
/// as needed.
#[allow(dead_code)]
pub fn save_config(config: &AppConfig) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory {}", parent.display()))?;
    }
    let contents = toml::to_string_pretty(config).with_context(|| "Failed to serialize config")?;
    std::fs::write(&path, contents)
        .with_context(|| format!("Failed to write config to {}", path.display()))?;
    Ok(())
}
