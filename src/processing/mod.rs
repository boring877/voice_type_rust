//! Text processing module
//!
//! Handles post-processing of transcribed text:
//! - Accounting mode (words, numbers)
//! - Casual mode (lowercase, informal)
//! - Filter words removal
//! - Text normalization
//! - Style presets (local emoji styles + LLM-based rewrite styles)

pub use crate::types::processing::ProcessingOptions;

mod numbers;
mod processor;
mod style;

pub use numbers::{convert_numbers_to_digits, format_number_commas};
pub use processor::process_text;
pub use style::apply_local_style;
