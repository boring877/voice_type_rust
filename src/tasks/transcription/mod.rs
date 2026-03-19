//! Transcription task orchestration.
//!
//! Receives audio data, sends to API, post-processes text, and types output.

mod processing;
mod request;

use std::sync::Arc;
use anyhow;
use tokio::sync::{Mutex, mpsc};
use tracing::{error, info, warn};

use crate::api::transcribe;
use crate::input::type_text;
use crate::processing::apply_local_style;
use crate::types::{AppState, GuiCommand, STATUS_FILTERED, STATUS_NO_API_KEY, SharedState};
use crate::history;

use processing::process_transcription;
use request::{RequestPrepError, prepare_request};

/// Transcription task.
pub async fn transcription_task(
    app_state: Arc<Mutex<SharedState>>,
    mut transcription_rx: mpsc::Receiver<Vec<u8>>,
    gui_tx: mpsc::Sender<GuiCommand>,
) {
    info!("Transcription task started");

    while let Some(audio_bytes) = transcription_rx.recv().await {
        {
            let state = app_state.lock().await;
            if state.should_quit {
                return;
            }
        }

        let _ = gui_tx.try_send(GuiCommand::SetState(AppState::Processing));

        let prepared = match prepare_request(&app_state).await {
            Ok(prepared) => prepared,
            Err(RequestPrepError::InvalidProvider(provider_str)) => {
                error!("Invalid provider '{}'", provider_str);
                let _ = gui_tx.try_send(GuiCommand::SetStatus(format!(
                    "Invalid provider: {}",
                    provider_str
                )));
                let _ = gui_tx.try_send(GuiCommand::SetState(AppState::Error));
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                let _ = gui_tx.try_send(GuiCommand::SetState(AppState::Ready));
                continue;
            }
            Err(RequestPrepError::MissingApiKey) => {
                warn!("No API key, skipping transcription");
                let _ = gui_tx.try_send(GuiCommand::SetStatus(STATUS_NO_API_KEY.to_string()));
                let _ = gui_tx.try_send(GuiCommand::SetState(AppState::Error));
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                let _ = gui_tx.try_send(GuiCommand::SetState(AppState::Ready));
                continue;
            }
        };

        info!(
            "Sending transcription request to {}...",
            prepared.provider.name()
        );
        match transcribe(audio_bytes, &prepared.options, prepared.provider).await {
            Ok(text) => {
                info!("Transcription received: '{}'", text);

                let processed = process_transcription(&text, &app_state).await;
                if let Some(final_text) = processed {
                    let (style, leave_in_clipboard) = {
                        let state = app_state.lock().await;
                        (state.config.style.clone(), state.config.auto_copy)
                    };

                    let final_text = apply_local_style(&final_text, &style, &prepared.options.language)
                        .unwrap_or(final_text);

                    let word_count = final_text.split_whitespace().count();
                    let status = format!("{}\n\n{} words", final_text, word_count);
                    let _ = gui_tx.try_send(GuiCommand::SetStatus(status));
                    let _ = gui_tx.try_send(GuiCommand::SetState(AppState::Done));

                    let final_text_for_history = final_text.clone();
                    let type_result = tokio::task::spawn_blocking(move || {
                        type_text(&final_text, leave_in_clipboard)
                    })
                    .await;
                    if let Err(e) = type_result.unwrap_or_else(|e| Err(anyhow::anyhow!("Task join error: {}", e))) {
                        error!("Failed to type text: {}", e);
                    } else if let Err(e) = history::push(&final_text_for_history) {
                        warn!("Failed to save history entry: {}", e);
                    }
                } else {
                    info!("Text was filtered out");
                    let _ = gui_tx.try_send(GuiCommand::SetStatus(STATUS_FILTERED.to_string()));
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }
            }
            Err(e) => {
                error!("Transcription failed: {}", e);
                let _ = gui_tx.try_send(GuiCommand::SetStatus(format!("Error: {}", e)));
                let _ = gui_tx.try_send(GuiCommand::SetState(AppState::Error));
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let _ = gui_tx.try_send(GuiCommand::SetState(AppState::Ready));
    }
}
