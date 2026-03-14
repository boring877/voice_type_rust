//! Text post-processing helpers for transcription results.

use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::warn;

use crate::api::correct_grammar_with_options;
use crate::processing::process_text;
use crate::types::{ProcessingOptions, SharedState, TranscriptionOptions};

/// Apply local text processing based on user settings.
pub async fn process_transcription(
    text: &str,
    app_state: &Arc<Mutex<SharedState>>,
) -> Option<String> {
    let state = app_state.lock().await;

    let options = ProcessingOptions {
        accounting_mode: state.config.accounting_mode,
        accounting_comma: state.config.accounting_comma,
        casual_mode: state.config.casual_mode,
        shorthand_mode: state.config.shorthand_mode,
        capitalize_sentences: true,
        smart_quotes: false,
        filter_words: state.config.filter_words.clone(),
    };

    drop(state);
    process_text(text, &options)
}

/// Optionally run LLM grammar correction with safe fallback.
pub async fn apply_optional_grammar(
    text: String,
    enabled: bool,
    profile: &str,
    model: &str,
    options: &TranscriptionOptions,
) -> String {
    if !enabled {
        return text;
    }

    match correct_grammar_with_options(
        &text,
        &options.api_key,
        &options.language,
        profile,
        Some(model),
    )
    .await
    {
        Ok(corrected) if !corrected.is_empty() => corrected,
        Ok(_) => text,
        Err(e) => {
            warn!("Grammar correction failed, using original text: {}", e);
            text
        }
    }
}
