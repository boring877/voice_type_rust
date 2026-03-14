//! Speech-to-text API clients
//!
//! Supports multiple transcription providers:
//! - Groq (Whisper API) - default
//!
//! Uses a provider trait pattern for extensibility.

// Declare submodules
pub(crate) mod constants;
mod grammar;
mod groq;
mod provider;

// Constants are used internally, not re-exported

// Re-export provider API
pub use crate::types::api::Provider;
pub use grammar::correct_grammar_with_options;
pub use grammar::test_api_key_with_model;
pub use provider::transcribe;
