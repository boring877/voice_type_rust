//! Speech-to-text API clients
//!
//! Supports multiple transcription providers:
//! - Groq (Whisper API) - default
//!
//! Uses a provider trait pattern for extensibility.

// Declare submodules
pub(crate) mod constants;
mod groq;
mod provider;

// Constants are used internally, not re-exported

// Re-export provider API
pub use crate::types::api::Provider;
pub use groq::{rewrite_with_llm, test_api_key};
pub use provider::transcribe;
