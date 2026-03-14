//! Configuration types for Voice Type
//!
//! Contains the Config struct for user configuration.

use crate::api::constants::{GRAMMAR_MODEL_BALANCED, TRANSCRIPTION_MODEL_TURBO};
use crate::config::{
    BACKGROUND_MODE_IMAGE, DEFAULT_BACKGROUND_COLOR, DEFAULT_BACKGROUND_GRADIENT_END,
    DEFAULT_BACKGROUND_GRADIENT_START, DEFAULT_HOTKEY, DEFAULT_LANGUAGE, DEFAULT_NOISE_THRESHOLD,
    DEFAULT_SILENCE_THRESHOLD, GRAMMAR_PROFILE_BALANCED, HUD_BACKGROUND_MODE_IMAGE,
    HUD_SIDE_RIGHT, PROVIDER_GROQ, THEME_DARK, THEME_LIGHT,
};
use serde::{Deserialize, Serialize};

pub const GROQ_API_KEY_ENV_VAR: &str = "GROQ_API_KEY";

/// Application configuration
///
/// This struct holds all user-configurable settings.
/// It's serialized to JSON and stored in the config directory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// Groq API key for transcription
    pub api_key: String,

    /// Selected microphone index (None = default)
    pub mic_index: Option<usize>,

    /// Push-to-talk hotkey (e.g., "shift", "ctrl", "f9")
    pub hotkey: String,

    /// Enable accounting mode (convert words to numbers)
    pub accounting_mode: bool,

    /// Add commas to large numbers in accounting mode
    pub accounting_comma: bool,

    /// Enable casual mode (lowercase, informal punctuation)
    pub casual_mode: bool,

    /// Replace common phrases with shorthand/slang output
    #[serde(default)]
    pub shorthand_mode: bool,

    /// Final output style preset
    #[serde(default = "default_style")]
    pub style: String,

    /// Words/phrases to filter out from transcription
    pub filter_words: Vec<String>,

    /// UI theme ("dark" or "light")
    pub theme: String,

    /// Background mode ("solid", "gradient", or "image")
    #[serde(default = "default_background_mode")]
    pub background_mode: String,

    /// Base background color used by solid mode and as image fallback.
    #[serde(default = "default_background_color")]
    pub background_color: String,

    /// Gradient start color used in gradient mode.
    #[serde(default = "default_background_gradient_start")]
    pub background_gradient_start: String,

    /// Gradient end color used in gradient mode.
    #[serde(default = "default_background_gradient_end")]
    pub background_gradient_end: String,

    /// Optional override for the outer control dock color.
    #[serde(default)]
    pub panel_color: String,

    /// Optional override for inner settings section colors.
    #[serde(default)]
    pub section_color: String,

    /// Optional local image path painted behind the settings page.
    #[serde(default)]
    pub background_image_path: String,

    /// Optional override for the main meter color near the top of the bars.
    #[serde(default)]
    pub meter_color_start: String,

    /// Optional override for the lower meter accent color.
    #[serde(default)]
    pub meter_color_end: String,

    /// Auto-stop recording after silence
    pub auto_stop: bool,

    /// Seconds of silence before auto-stop
    pub silence_threshold: f32,

    /// Audio noise threshold (0.0 - 1.0)
    pub noise_threshold: f32,

    /// Language code ("auto" for auto-detect)
    pub language: String,

    /// Whisper transcription model ID on Groq.
    #[serde(default = "default_transcription_model")]
    pub transcription_model: String,

    /// Apply LLM grammar correction before typing output
    #[serde(default)]
    pub grammar_correction: bool,

    /// Grammar correction quality profile ("fast", "balanced", or "quality")
    #[serde(default = "default_grammar_profile")]
    pub grammar_profile: String,

    /// Grammar correction model ID on Groq.
    #[serde(default = "default_grammar_model")]
    pub grammar_model: String,

    /// Keep widget always on top
    pub always_on_top: bool,

    /// Enable the floating recording HUD
    #[serde(default = "default_hud_enabled")]
    pub hud_enabled: bool,

    /// Screen side used by the floating recording HUD
    #[serde(default = "default_hud_side")]
    pub hud_side: String,

    /// Show the HUD state badge such as Recording or Transcribing.
    #[serde(default = "default_hud_show_state")]
    pub hud_show_state: bool,

    /// Show the app label inside the HUD.
    #[serde(default = "default_hud_show_app_name")]
    pub hud_show_app_name: bool,

    /// Show the secondary helper text inside the HUD.
    #[serde(default = "default_hud_show_description")]
    pub hud_show_description: bool,

    /// Show the animated audio meter inside the HUD.
    #[serde(default = "default_hud_show_meter")]
    pub hud_show_meter: bool,

    /// HUD card background mode.
    #[serde(default = "default_hud_background_mode")]
    pub hud_background_mode: String,

    /// Imported HUD image or GIF path stored under the app config folder.
    #[serde(default)]
    pub hud_background_path: String,

    /// Optional HUD tint / fill color.
    #[serde(default)]
    pub hud_background_color: String,

    /// Auto-copy transcription to clipboard
    pub auto_copy: bool,

    /// Transcription provider (currently "groq")
    pub provider: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            mic_index: None,
            hotkey: DEFAULT_HOTKEY.to_string(),
            accounting_mode: false,
            accounting_comma: false,
            casual_mode: false,
            shorthand_mode: false,
            style: default_style(),
            filter_words: Vec::new(),
            theme: THEME_DARK.to_string(),
            background_mode: default_background_mode(),
            background_color: default_background_color(),
            background_gradient_start: default_background_gradient_start(),
            background_gradient_end: default_background_gradient_end(),
            panel_color: String::new(),
            section_color: String::new(),
            background_image_path: String::new(),
            meter_color_start: String::new(),
            meter_color_end: String::new(),
            auto_stop: false,
            silence_threshold: DEFAULT_SILENCE_THRESHOLD,
            noise_threshold: DEFAULT_NOISE_THRESHOLD,
            language: DEFAULT_LANGUAGE.to_string(),
            transcription_model: default_transcription_model(),
            grammar_correction: false,
            grammar_profile: default_grammar_profile(),
            grammar_model: default_grammar_model(),
            always_on_top: false,
            hud_enabled: default_hud_enabled(),
            hud_side: default_hud_side(),
            hud_show_state: default_hud_show_state(),
            hud_show_app_name: default_hud_show_app_name(),
            hud_show_description: default_hud_show_description(),
            hud_show_meter: default_hud_show_meter(),
            hud_background_mode: default_hud_background_mode(),
            hud_background_path: String::new(),
            hud_background_color: String::new(),
            auto_copy: false,
            provider: PROVIDER_GROQ.to_string(),
        }
    }
}

impl Config {
    /// Returns true when the light theme should be used.
    pub fn uses_light_theme(&self) -> bool {
        self.theme == THEME_LIGHT
    }

    /// Resolve the effective API key from config first, then environment.
    pub fn resolved_api_key(&self) -> Option<String> {
        let configured = self.api_key.trim();
        if !configured.is_empty() {
            return Some(configured.to_string());
        }

        std::env::var(GROQ_API_KEY_ENV_VAR)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    /// Returns true when the user has configured an API key.
    pub fn has_api_key(&self) -> bool {
        self.resolved_api_key().is_some()
    }
}

fn default_grammar_profile() -> String {
    GRAMMAR_PROFILE_BALANCED.to_string()
}

fn default_style() -> String {
    "none".to_string()
}

fn default_background_mode() -> String {
    BACKGROUND_MODE_IMAGE.to_string()
}

fn default_background_color() -> String {
    DEFAULT_BACKGROUND_COLOR.to_string()
}

fn default_background_gradient_start() -> String {
    DEFAULT_BACKGROUND_GRADIENT_START.to_string()
}

fn default_background_gradient_end() -> String {
    DEFAULT_BACKGROUND_GRADIENT_END.to_string()
}

fn default_transcription_model() -> String {
    TRANSCRIPTION_MODEL_TURBO.to_string()
}

fn default_grammar_model() -> String {
    GRAMMAR_MODEL_BALANCED.to_string()
}

fn default_hud_enabled() -> bool {
    true
}

fn default_hud_side() -> String {
    HUD_SIDE_RIGHT.to_string()
}

fn default_hud_show_state() -> bool {
    true
}

fn default_hud_show_app_name() -> bool {
    true
}

fn default_hud_show_description() -> bool {
    true
}

fn default_hud_show_meter() -> bool {
    true
}

fn default_hud_background_mode() -> String {
    HUD_BACKGROUND_MODE_IMAGE.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.hotkey, "shift");
        assert!(!config.accounting_mode);
        assert!(!config.shorthand_mode);
        assert_eq!(config.style, "none");
        assert!(!config.always_on_top);
        assert!(config.hud_enabled);
        assert_eq!(config.hud_side, "right");
        assert!(config.hud_show_state);
        assert!(config.hud_show_app_name);
        assert!(config.hud_show_description);
        assert!(config.hud_show_meter);
        assert_eq!(config.hud_background_mode, "image");
        assert!(config.hud_background_path.is_empty());
        assert!(config.hud_background_color.is_empty());
        assert!(!config.auto_copy);
        assert_eq!(config.background_mode, "image");
        assert!(config.panel_color.is_empty());
        assert!(config.section_color.is_empty());
        assert!(config.background_image_path.is_empty());
        assert!(config.meter_color_start.is_empty());
        assert!(config.meter_color_end.is_empty());
    }
}
