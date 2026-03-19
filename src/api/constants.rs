//! API-related constants

/// Default model for Groq Whisper API.
pub const TRANSCRIPTION_MODEL_TURBO: &str = "whisper-large-v3-turbo";

pub const WHISPER_MODEL: &str = TRANSCRIPTION_MODEL_TURBO;

/// Groq API base URL
///
/// OpenAI-compatible audio transcription endpoint.
pub const GROQ_API_URL: &str = "https://api.groq.com/openai/v1/audio/transcriptions";

/// Groq OpenAI-compatible chat completions endpoint.
pub const GROQ_CHAT_URL: &str = "https://api.groq.com/openai/v1/chat/completions";

/// Request timeout in seconds
///
/// Applies to the whole HTTP request (connect + upload + response).
pub const REQUEST_TIMEOUT: u64 = 30;
