//! Groq provider implementation.

// `anyhow` provides ergonomic error handling (`Result`, `context`, `bail!`).
use anyhow::{Context, Result};
// `async_trait` allows async functions inside traits.
use async_trait::async_trait;
// `reqwest` is the HTTP client used to call the transcription API.
use reqwest::multipart::{Form, Part};
use std::sync::OnceLock;

    use crate::api::constants::{GROQ_API_URL, GROQ_CHAT_URL, REQUEST_TIMEOUT, WHISPER_MODEL};
use crate::api::provider::TranscriptionProvider;
use crate::types::api::TranscriptionOptions;

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn client() -> Result<&'static reqwest::Client> {
    if let Some(existing) = HTTP_CLIENT.get() {
        return Ok(existing);
    }

    let built = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT))
        .build()
        .context("Failed to build HTTP client")?;

    let _ = HTTP_CLIENT.set(built);
    HTTP_CLIENT
        .get()
        .context("HTTP client initialization failed")
}

/// Groq Whisper API provider.
pub struct GroqProvider;

#[async_trait]
impl TranscriptionProvider for GroqProvider {
    async fn transcribe(
        &self,
        audio_bytes: Vec<u8>,
        options: &TranscriptionOptions,
    ) -> Result<String> {
        let c = client()?;
        transcribe_with_client(c, audio_bytes, options).await
    }
}

/// Core Groq transcription routine using an injected/reusable HTTP client.
///
/// Keeping the client as an argument makes it explicit that network transport
/// is reusable and separate from request-specific data (audio/options).
async fn transcribe_with_client(
    client: &reqwest::Client,
    audio_bytes: Vec<u8>,
    options: &TranscriptionOptions,
) -> Result<String> {
    // Fast-fail before building request payload.
    if options.api_key.is_empty() {
        anyhow::bail!("API key is required");
    }

    // Build the uploaded WAV file part for multipart/form-data.
    let file_part = Part::bytes(audio_bytes)
        .file_name("recording.wav")
        .mime_str("audio/wav")
        .context("Failed to create file part")?;

    let model = if !options.transcription_model.trim().is_empty() {
        options.transcription_model.as_str()
    } else {
        WHISPER_MODEL
    };

    // Start multipart form with required fields.
    let mut form = Form::new()
        .part("file", file_part)
        .text("model", model.to_string());

    // Optional language override; "auto" means auto-detect.
    if options.language != "auto" && !options.language.is_empty() {
        form = form.text("language", options.language.clone());
    }

    // Optional prompt to bias recognition toward specific vocabulary.
    if !options.custom_vocabulary.is_empty() {
        let prompt = options.custom_vocabulary.join(", ");
        form = form.text("prompt", prompt);
    }

    tracing::info!("Sending transcription request to Groq...");
    let start = std::time::Instant::now();

    // Send request with bearer auth and multipart body.
    let response = client
        .post(GROQ_API_URL)
        .header("Authorization", format!("Bearer {}", options.api_key))
        .multipart(form)
        .send()
        .await
        .context("Failed to send transcription request")?;

    let elapsed = start.elapsed();
    tracing::info!("API response received in {:?}", elapsed);

    // Read status and full body once, then parse according to status.
    let status = response.status();
    let body = response
        .text()
        .await
        .context("Failed to read response body")?;

    if status.is_success() {
        // Success shape: { "text": "..." }
        let result: crate::types::api::TranscriptionResponse =
            serde_json::from_str(&body).context("Failed to parse transcription response")?;

        // Log metadata only (no transcript content) for privacy.
        let char_count = result.text.chars().count();
        tracing::info!("Transcription success ({} chars)", char_count);
        Ok(result.text.trim().to_string())
    } else {
        // Error shape may vary by provider/proxy, parse best-effort JSON.
        let error: serde_json::Value =
            serde_json::from_str(&body).unwrap_or_else(|_| serde_json::json!({ "message": body }));
        let error_message = extract_error_message(&error, &body);

        tracing::error!("API error ({}): {}", status, error_message);
        anyhow::bail!("Transcription failed: {}", error_message);
    }
}

/// Extract a user-facing error message from API error JSON.
///
/// Supports both flat (`message`) and nested (`error.message`) formats.
fn extract_error_message(error: &serde_json::Value, raw_body: &str) -> String {
    // Most common flat error shape.
    if let Some(message) = error.get("message").and_then(|m| m.as_str()) {
        return message.to_string();
    }

    // OpenAI-compatible nested error shape.
    if let Some(message) = error
        .get("error")
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str())
    {
        return message.to_string();
    }

    // Fallback to raw response for unknown formats.
    raw_body.to_string()
}

/// Validate an API key by sending a minimal chat-completions request.
pub async fn test_api_key(api_key: &str) -> Result<()> {
    let c = client()?;
    let body = serde_json::json!({
        "model": "llama-3.3-70b-versatile",
        "messages": [{ "role": "user", "content": "hi" }],
        "max_tokens": 1
    });

    let response = c
        .post(GROQ_CHAT_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .context("Failed to send API key test request")?;

    if response.status().is_success() {
        Ok(())
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("API key validation failed ({}): {}", status, body)
    }
}

fn style_system_prompt(style: &str) -> &'static str {
    match style {
        "linkedin" => "You rewrite casual speech into the style of a LinkedIn influencer post. \
            Use corporate buzzwords, frame everything as a professional achievement or lesson, \
            add reflective openers and engagement-seeking closers. Keep the same meaning. \
            Output ONLY the rewritten text, nothing else.",
        "lawyer" => "You rewrite casual speech into formal legalese, as if spoken by a lawyer. \
            Use legal terminology, formal phrasing, and authoritative tone. \
            Keep the same meaning. \
            Output ONLY the rewritten text, nothing else.",
        _ => "Rewrite the following text. Output ONLY the rewritten text, nothing else.",
    }
}

/// Rewrite text using an LLM in the specified style.
pub async fn rewrite_with_llm(api_key: &str, text: &str, style: &str) -> Result<String> {
    let c = client()?;
    let system_prompt = style_system_prompt(style);

    let body = serde_json::json!({
        "model": "llama-3.3-70b-versatile",
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": text }
        ],
        "max_tokens": 512,
        "temperature": 0.8
    });

    let response = c
        .post(GROQ_CHAT_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .context("Failed to send style rewrite request")?;

    let status = response.status();
    let body = response
        .text()
        .await
        .context("Failed to read style rewrite response")?;

    if status.is_success() {
        let parsed: serde_json::Value =
            serde_json::from_str(&body).context("Failed to parse style rewrite response")?;
        let rewritten = parsed
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or(text);
        Ok(rewritten.trim().to_string())
    } else {
        let error: serde_json::Value =
            serde_json::from_str(&body).unwrap_or_else(|_| serde_json::json!({ "message": body }));
        let error_message = extract_error_message(&error, &body);
        tracing::warn!("Style rewrite failed, using original text: {}", error_message);
        Ok(text.to_string())
    }
}
