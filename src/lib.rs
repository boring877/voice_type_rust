//! Voice Type - Rust Implementation
//!
//! A fast, reliable speech-to-text application using Groq Whisper API.
//!
//! ## Architecture
//!
//! The application is organized into focused modules:
//!
//! - `types`: All type definitions (structs and enums)
//! - `config`: Configuration management (API keys, preferences)
//! - `audio`: Audio recording using cpal
//! - `api`: Groq API client for transcription
//! - `input`: Global hotkeys and text typing simulation
//! - `processing`: Text post-processing (accounting mode, filtering)
//! - `tasks`: Background recording and transcription orchestration
//!
//! ## Example Usage
//!
//! ```ignore
//! use voice_type::config::{load, Config};
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = load();
//!     println!("Hotkey: {}", config.hotkey);
//! }
//! ```

// Type definitions (must be first for dependency order)
pub mod types;

// Re-export public modules
pub mod api;
pub mod audio;
pub mod config;
pub mod input;
pub mod processing;
pub mod tasks;

/// Version of the application
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
