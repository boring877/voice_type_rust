//! Request preparation helpers for transcription.

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::api::Provider;
use crate::types::{SharedState, TranscriptionOptions};

/// Fully prepared transcription request context.
pub struct PreparedRequest {
    pub options: TranscriptionOptions,
    pub provider: Provider,
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
    let (api_key, language, transcription_model, provider_str) = {
        let state = app_state.lock().await;
        (
            state.config.resolved_api_key(),
            state.config.language.clone(),
            state.config.transcription_model.clone(),
            state.config.provider.clone(),
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
        custom_vocabulary: Vec::new(),
        transcription_model,
        provider: provider_str,
    };

    Ok(PreparedRequest {
        options,
        provider,
    })
}
