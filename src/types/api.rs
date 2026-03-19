//! API types for Voice Type
//!
//! Contains types related to Groq API transcription.

use serde::Deserialize;

use crate::api::constants::TRANSCRIPTION_MODEL_TURBO;
use crate::config::{DEFAULT_LANGUAGE, PROVIDER_GROQ};

/// Transcription request options
#[derive(Debug, Clone)]
pub struct TranscriptionOptions {
    /// API key for authentication
    pub api_key: String,
    /// Language code (e.g., "en", "auto" for auto-detect)
    pub language: String,
    /// Custom vocabulary to improve recognition
    pub custom_vocabulary: Vec<String>,
    /// Explicit transcription model id
    pub transcription_model: String,
    /// Which provider to use (stored for reference, actual provider passed separately)
    #[allow(dead_code)]
    pub provider: String,
}

impl Default for TranscriptionOptions {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            language: DEFAULT_LANGUAGE.to_string(),
            custom_vocabulary: Vec::new(),
            transcription_model: TRANSCRIPTION_MODEL_TURBO.to_string(),
            provider: PROVIDER_GROQ.to_string(),
        }
    }
}

/// Transcription response from API
#[derive(Debug, Deserialize)]
pub struct TranscriptionResponse {
    /// The transcribed text
    pub text: String,
}

/// Supported transcription providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Provider {
    /// Groq Whisper API (default)
    #[default]
    Groq,
}

impl Provider {
    /// Display name for UI/logging.
    pub fn name(&self) -> &'static str {
        match self {
            Provider::Groq => "Groq",
        }
    }
}

impl std::str::FromStr for Provider {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            PROVIDER_GROQ => Ok(Provider::Groq),
            _ => anyhow::bail!("Unknown provider: {}", s),
        }
    }
}
