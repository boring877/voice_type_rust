//! Grammar correction via Groq chat completions.

use anyhow::{Context, Result};
use serde_json::json;
use std::sync::OnceLock;

use crate::api::constants::{
    GRAMMAR_MODEL_BALANCED, GRAMMAR_MODEL_FAST, GRAMMAR_MODEL_QUALITY, GROQ_CHAT_URL,
    REQUEST_TIMEOUT,
};
use crate::config::{GRAMMAR_PROFILE_BALANCED, GRAMMAR_PROFILE_FAST, GRAMMAR_PROFILE_QUALITY};

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn client() -> Result<&'static reqwest::Client> {
    if let Some(existing) = HTTP_CLIENT.get() {
        return Ok(existing);
    }

    let built = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT))
        .build()
        .context("Failed to build grammar HTTP client")?;

    let _ = HTTP_CLIENT.set(built);
    HTTP_CLIENT
        .get()
        .context("Grammar HTTP client initialization failed")
}

/// Correct grammar with selectable profile and optional explicit model override.
pub async fn correct_grammar_with_options(
    text: &str,
    api_key: &str,
    language: &str,
    profile: &str,
    model_override: Option<&str>,
) -> Result<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    if api_key.is_empty() {
        anyhow::bail!("API key is required");
    }

    let language_hint = if language.is_empty() || language == "auto" {
        "same as the input text"
    } else {
        language
    };

    let (profile_model, system_prompt) = profile_settings(profile);
    let model = model_override
        .filter(|m| !m.trim().is_empty())
        .unwrap_or(profile_model);

    let user_prompt = format!(
        "Target language: {}.\nCorrect this text:\n{}",
        language_hint, trimmed
    );

    match run_grammar_request(api_key, model, system_prompt, &user_prompt).await {
        Ok(content) => Ok(content),
        Err(primary_error) => {
            if profile == GRAMMAR_PROFILE_QUALITY && model_override.is_none() {
                let (fallback_model, fallback_prompt) = profile_settings(GRAMMAR_PROFILE_BALANCED);
                return run_grammar_request(api_key, fallback_model, fallback_prompt, &user_prompt)
                    .await
                    .map_err(|fallback_error| {
                        anyhow::anyhow!(
                            "Grammar correction failed (quality then fallback): {} | {}",
                            primary_error,
                            fallback_error
                        )
                    });
            }
            Err(primary_error)
        }
    }
}

/// Validate API key by making a minimal chat-completions call with optional model override.
pub async fn test_api_key_with_model(api_key: &str, model: Option<&str>) -> Result<()> {
    let _ =
        correct_grammar_with_options("hello", api_key, "en", GRAMMAR_PROFILE_FAST, model).await?;
    Ok(())
}

fn profile_settings(profile: &str) -> (&'static str, &'static str) {
    match profile {
        GRAMMAR_PROFILE_FAST => (
            GRAMMAR_MODEL_FAST,
            "You fix only obvious grammar and punctuation issues. \
Keep wording as close as possible. \
Preserve language and meaning. \
Return only corrected text.",
        ),
        GRAMMAR_PROFILE_QUALITY => (
            GRAMMAR_MODEL_QUALITY,
            "You are a precise editor. Improve grammar, punctuation, and sentence flow for readability. \
Preserve original meaning, tone, and language. \
Do not add facts or remove key details. \
No explanations. Return only corrected text.",
        ),
        _ => (
            GRAMMAR_MODEL_BALANCED,
            "You are a strict grammar and punctuation corrector. \
Keep the original meaning, tone, and language. \
Do not add new facts. \
Do not explain anything. \
Return only the corrected text.",
        ),
    }
}

async fn run_grammar_request(
    api_key: &str,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String> {
    let request_body = json!({
        "model": model,
        "temperature": 0.0,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_prompt }
        ]
    });

    let response = client()?
        .post(GROQ_CHAT_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await
        .context("Failed to send grammar correction request")?;

    let status = response.status();
    let body = response
        .text()
        .await
        .context("Failed to read grammar response body")?;

    if !status.is_success() {
        anyhow::bail!("Grammar correction failed ({}): {}", status, body);
    }

    let value: serde_json::Value =
        serde_json::from_str(&body).context("Failed to parse grammar response JSON")?;

    let content = value
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|first| first.get("message"))
        .and_then(|msg| msg.get("content"))
        .and_then(|c| c.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .context("Grammar response did not contain corrected text")?;

    Ok(content)
}
