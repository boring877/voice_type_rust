//! Configuration module
//!
//! This module provides configuration management functionality.

// Declare submodules
mod constants;
mod manager;

// Re-export constants
pub use constants::{
    APP_DISPLAY_NAME, BACKGROUND_MODE_GRADIENT, BACKGROUND_MODE_IMAGE, BACKGROUND_MODE_SOLID,
    CONFIG_DIR_NAME, DEFAULT_BACKGROUND_COLOR, DEFAULT_BACKGROUND_GRADIENT_END,
    DEFAULT_BACKGROUND_GRADIENT_START, DEFAULT_HOTKEY, DEFAULT_LANGUAGE, DEFAULT_NOISE_THRESHOLD,
    DEFAULT_SILENCE_THRESHOLD, HUD_BACKGROUND_MODE_GLASS, HUD_BACKGROUND_MODE_IMAGE,
    HUD_SIDE_LEFT, HUD_SIDE_RIGHT, PROVIDER_GROQ, SINGLE_INSTANCE_ID, THEME_DARK, THEME_LIGHT,
};

// Re-export functions from manager
pub use manager::{backgrounds_dir, config_dir, has_api_key, load, save};
