//! Text post-processing helpers for transcription results.

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::processing::process_text;
use crate::types::{ProcessingOptions, SharedState};

/// Apply local text processing based on user settings.
pub async fn process_transcription(
    text: &str,
    app_state: &Arc<Mutex<SharedState>>,
) -> Option<String> {
    let state = app_state.lock().await;

    let language = state.config.language.trim().to_ascii_lowercase();
    let is_english = language.is_empty() || language == "auto" || language.starts_with("en");

    let options = ProcessingOptions {
        accounting_mode: state.config.accounting_mode,
        accounting_comma: state.config.accounting_comma,
        casual_mode: state.config.casual_mode,
        shorthand_mode: state.config.shorthand_mode,
        capitalize_sentences: is_english,
        smart_quotes: is_english,
        filter_words: state.config.filter_words.clone(),
    };

    drop(state);
    process_text(text, &options)
}
