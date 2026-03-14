//! Request preparation helpers for transcription.

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::api::Provider;
use crate::types::{SharedState, TranscriptionOptions};

/// Fully prepared transcription request context.
pub struct PreparedRequest {
    pub options: TranscriptionOptions,
    pub provider: Provider,
    pub grammar_correction: bool,
    pub grammar_profile: String,
    pub grammar_model: String,
}

/// Errors that can happen before sending a transcription request.
pub enum RequestPrepError {
    InvalidProvider(String),
    MissingApiKey,
}

/// Read config from shared state and build a validated request context.
pub async fn prepare_request(
    app_state: &Arc<Mutex<SharedState>>,
) -> Result<PreparedRequest, RequestPrepError> {
    let (
        api_key,
        language,
        transcription_model,
        provider_str,
        grammar_correction,
        grammar_profile,
        grammar_model,
    ) = {
        let state = app_state.lock().await;
        (
            state.config.resolved_api_key(),
            state.config.language.clone(),
            state.config.transcription_model.clone(),
            state.config.provider.clone(),
            state.config.grammar_correction,
            state.config.grammar_profile.clone(),
            state.config.grammar_model.clone(),
        )
    };

    let provider = provider_str
        .parse::<Provider>()
        .map_err(|_| RequestPrepError::InvalidProvider(provider_str.clone()))?;

    let Some(api_key) = api_key else {
        return Err(RequestPrepError::MissingApiKey);
    };

    let options = TranscriptionOptions {
        api_key,
        language,
        custom_vocabulary: Vec::new(), // TODO: Add to config
        transcription_model,
        use_lite_model: false,
        provider: provider_str,
    };

    Ok(PreparedRequest {
        options,
        provider,
        grammar_correction,
        grammar_profile,
        grammar_model,
    })
}
