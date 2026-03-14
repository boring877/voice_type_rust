//! UI runtime types for Voice Type.
//!
//! These types are shared between the Rust backend tasks and the desktop shell.

use crate::types::config::Config;

pub const STATUS_RECORDING: &str = "Recording...";
pub const STATUS_TRANSCRIBING: &str = "Transcribing...";
pub const STATUS_SETTINGS_SAVED: &str = "Settings saved";
pub const STATUS_FAILED_TO_SAVE_SETTINGS: &str = "Failed to save settings";
pub const STATUS_TESTING_API_KEY: &str = "Testing API key...";
pub const STATUS_API_KEY_VALID: &str = "API key is valid";
pub const STATUS_API_KEY_EMPTY: &str = "API key is empty";
pub const STATUS_NO_API_KEY: &str = "No API Key";
pub const STATUS_FILTERED: &str = "Filtered";

/// Current state of the application UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    /// Ready and waiting for input
    Ready,
    /// Recording audio
    Recording,
    /// Processing transcription
    Processing,
    /// Transcription complete
    Done,
    /// Error occurred
    Error,
}

/// Commands to control the GUI from the main app
#[derive(Debug, Clone)]
pub enum GuiCommand {
    /// Update application state
    SetState(AppState),
    /// Update status text
    SetStatus(String),
    /// Update audio level
    SetLevel(f32),
    /// Update config
    UpdateConfig(Config),
}
