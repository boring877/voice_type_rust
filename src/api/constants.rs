//! API-related constants

/// Default model for Groq Whisper API.
pub const DEFAULT_TRANSCRIPTION_MODEL: &str = "whisper-large-v3";

pub const WHISPER_MODEL: &str = DEFAULT_TRANSCRIPTION_MODEL;

/// Groq API base URL
///
/// OpenAI-compatible audio transcription endpoint.
pub const GROQ_API_URL: &str = "https://api.groq.com/openai/v1/audio/transcriptions";

/// Groq models endpoint. Listing models is an authenticated, inference-free
/// request, so it is suitable for validating credentials.
pub const GROQ_MODELS_URL: &str = "https://api.groq.com/openai/v1/models";

/// Request timeout in seconds
///
/// Applies to the whole HTTP request (connect + upload + response).
pub const REQUEST_TIMEOUT: u64 = 30;
