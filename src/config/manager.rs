//! Configuration persistence functions
//!
//! Handles loading, saving, and accessing user preferences.

use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::config::{
    CONFIG_DIR_NAME, HUD_BACKGROUND_MODE_GLASS, HUD_BACKGROUND_MODE_IMAGE, HUD_SIDE_LEFT,
    HUD_SIDE_RIGHT,
};
use crate::types::Config;

const LEGACY_FILTER_WORDS: [&str; 3] = ["thank you", "thanks", "thank you for watching"];

/// Get the path to the config file
///
/// Returns: ~/.config/voice-type/config.json (platform-specific)
fn config_path() -> Result<PathBuf> {
    let config_dir = config_dir()?;

    Ok(config_dir.join("config.json"))
}

pub fn config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Could not find config directory")?
        .join(CONFIG_DIR_NAME);

    std::fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
    Ok(config_dir)
}

pub fn backgrounds_dir() -> Result<PathBuf> {
    let backgrounds_dir = config_dir()?.join("backgrounds");
    std::fs::create_dir_all(&backgrounds_dir).context("Failed to create backgrounds directory")?;
    Ok(backgrounds_dir)
}

/// Load configuration from disk
///
/// If no config exists, returns default configuration.
/// Logs errors but doesn't fail - falls back to defaults.
pub fn load() -> Config {
    match try_load() {
        Ok(config) => {
            tracing::info!("Config loaded successfully");
            config
        }
        Err(e) => {
            tracing::warn!("Failed to load config ({}), using defaults", e);
            Config::default()
        }
    }
}

/// Try to load config from disk
fn try_load() -> Result<Config> {
    let path = config_path()?;

    if !path.exists() {
        tracing::info!("No config file found at {:?}, using defaults", path);
        return Ok(Config::default());
    }

    let content = std::fs::read_to_string(&path).context("Failed to read config file")?;

    let mut config: Config =
        serde_json::from_str(&content).context("Failed to parse config JSON")?;
    normalize_loaded_config(&mut config);

    Ok(config)
}

/// Save configuration to disk
///
/// Creates config directory if needed.
/// Returns error if save fails (caller should show user feedback).
pub fn save(config: &Config) -> Result<()> {
    let path = config_path()?;

    let content = serde_json::to_string_pretty(config).context("Failed to serialize config")?;

    std::fs::write(&path, content).context("Failed to write config file")?;

    tracing::info!("Config saved to {:?}", path);
    Ok(())
}

/// Check if API key is configured
///
/// Returns true if the API key is non-empty.
pub fn has_api_key(config: &Config) -> bool {
    config.has_api_key()
}

fn normalize_loaded_config(config: &mut Config) {
    let uses_legacy_filter_defaults = config.filter_words.len() == LEGACY_FILTER_WORDS.len()
        && config
            .filter_words
            .iter()
            .map(|value| value.trim().to_ascii_lowercase())
            .eq(LEGACY_FILTER_WORDS.into_iter().map(str::to_string));

    if uses_legacy_filter_defaults {
        config.filter_words.clear();
    }

    if config.style.trim().is_empty() {
        config.style = "none".to_string();
    } else if config.style == "japanese_omg" {
        config.style = "japanese_emojis".to_string();
    } else if config.style == "niko" {
        config.style = "niko_style".to_string();
    }

    if config.hud_side != HUD_SIDE_LEFT && config.hud_side != HUD_SIDE_RIGHT {
        config.hud_side = HUD_SIDE_RIGHT.to_string();
    }

    if config.hud_background_mode != HUD_BACKGROUND_MODE_GLASS
        && config.hud_background_mode != HUD_BACKGROUND_MODE_IMAGE
    {
        config.hud_background_mode = HUD_BACKGROUND_MODE_IMAGE.to_string();
    }
}
