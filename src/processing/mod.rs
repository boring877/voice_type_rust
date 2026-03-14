//! Text processing module
//!
//! Handles post-processing of transcribed text:
//! - Accounting mode (words, numbers)
//! - Casual mode (lowercase, informal)
//! - Filter words removal
//! - Text normalization

// Re-export ProcessingOptions from types module
pub use crate::types::processing::ProcessingOptions;

// Declare submodules
mod processor;

// Re-export functions from processor
pub use processor::{apply_style_preset, process_text};
