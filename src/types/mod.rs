//! Types module for Voice Type
//!
//! This module contains all type definitions (structs and enums) used throughout the application.
//! Types are organized by domain for easy discovery and maintenance.
//!
//! ## Organization
//!
//! - `config`: Configuration types ([`Config`])
//! - `audio`: Audio recording types ([`RecordingState`])
//! - `api`: API types ([`TranscriptionOptions`], response types)
//! - `input`: Input handling types ([`HotkeyState`])
//! - `processing`: Text processing types ([`ProcessingOptions`])
//! - `gui`: Shared UI/runtime types ([`AppState`], [`GuiCommand`], and status labels)

pub mod api;
pub mod app;
pub mod audio;
pub mod config;
pub mod gui;
pub mod input;
pub mod processing;

// Re-export all public types for convenient access
pub use api::TranscriptionOptions;
pub use app::SharedState;
pub use config::Config;
pub use gui::{
    AppState, GuiCommand, STATUS_API_KEY_EMPTY, STATUS_API_KEY_VALID,
    STATUS_FAILED_TO_SAVE_SETTINGS, STATUS_FILTERED, STATUS_NO_API_KEY, STATUS_RECORDING,
    STATUS_SETTINGS_SAVED, STATUS_TESTING_API_KEY, STATUS_TRANSCRIBING,
};
pub use input::HotkeyState;
pub use processing::ProcessingOptions;
