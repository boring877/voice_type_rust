//! API-related constants

/// Default balanced model for speed/accuracy on Groq Whisper API.
pub const TRANSCRIPTION_MODEL_TURBO: &str = "whisper-large-v3-turbo";

/// Lower-latency fallback with potential quality tradeoff.
pub const TRANSCRIPTION_MODEL_DISTIL_EN: &str = "distil-whisper-large-v3-en";

/// Backwards-compatible aliases for the transcription provider implementation.
pub const WHISPER_MODEL: &str = TRANSCRIPTION_MODEL_TURBO;
pub const WHISPER_MODEL_LITE: &str = TRANSCRIPTION_MODEL_DISTIL_EN;

/// Groq API base URL
///
/// OpenAI-compatible audio transcription endpoint.
pub const GROQ_API_URL: &str = "https://api.groq.com/openai/v1/audio/transcriptions";

/// Groq OpenAI-compatible chat completions endpoint.
pub const GROQ_CHAT_URL: &str = "https://api.groq.com/openai/v1/chat/completions";

/// Grammar model options.
pub const GRAMMAR_MODEL_LLAMA: &str = "llama-3.3-70b-versatile";

/// Fast grammar model default.
pub const GRAMMAR_MODEL_FAST: &str = GRAMMAR_MODEL_LLAMA;

/// Balanced grammar model default.
pub const GRAMMAR_MODEL_BALANCED: &str = GRAMMAR_MODEL_LLAMA;

/// Quality grammar model (higher quality, slower/costlier).
pub const GRAMMAR_MODEL_QUALITY: &str = GRAMMAR_MODEL_LLAMA;

/// Request timeout in seconds
///
/// Applies to the whole HTTP request (connect + upload + response).
pub const REQUEST_TIMEOUT: u64 = 30;
