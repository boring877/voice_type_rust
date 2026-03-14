//! Transcription provider trait and implementations
//!
//! Supports multiple speech-to-text providers:
//! - Groq (Whisper API) - default
//!
//! More providers can be added here.

// `anyhow` provides ergonomic error handling (`Result`, `context`, `bail!`).
use anyhow::Result;
// `async_trait` allows async functions inside traits.
use async_trait::async_trait;

use crate::api::groq::GroqProvider;
use crate::types::api::{Provider, TranscriptionOptions};

/// Trait for transcription providers
#[async_trait]
pub trait TranscriptionProvider: Send + Sync {
    /// Transcribe audio to text
    ///
    /// # Arguments
    /// * `audio_bytes` - WAV-encoded audio data
    /// * `options` - Transcription options
    ///
    /// # Returns
    /// Transcribed text
    async fn transcribe(
        &self,
        audio_bytes: Vec<u8>,
        options: &TranscriptionOptions,
    ) -> Result<String>;
}

/// Transcribe audio using the specified provider.
///
/// This is the API-layer dispatch point. It selects the concrete provider
/// implementation and forwards the request through the shared trait contract.
pub async fn transcribe(
    audio_bytes: Vec<u8>,
    options: &TranscriptionOptions,
    provider: Provider,
) -> Result<String> {
    // Provider factory/dispatch. Add future providers here.
    let provider_impl: Box<dyn TranscriptionProvider> = match provider {
        Provider::Groq => Box::new(GroqProvider::new()?),
    };
    provider_impl.transcribe(audio_bytes, options).await
}
