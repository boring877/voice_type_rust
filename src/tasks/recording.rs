//! Audio recording task
//!
//! Monitors hotkey state and records audio when pressed.

use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tracing::{error, info, warn};

use crate::audio::{self, wav_duration_seconds};
use crate::input::is_recording;
use crate::types::{AppState, GuiCommand, HotkeyState, SharedState};

/// Audio recording task
///
/// Monitors the hotkey state and records audio when pressed.
/// Sends audio data to the transcription task when released.
pub async fn audio_recording_task(
    app_state: Arc<Mutex<SharedState>>,
    hotkey_state: Arc<HotkeyState>,
    level_tx: mpsc::Sender<f32>,
    transcription_tx: mpsc::Sender<Vec<u8>>,
    gui_tx: mpsc::Sender<GuiCommand>,
) {
    info!("Audio recording task started");

    loop {
        // Check if should quit
        {
            let state = app_state.lock().await;
            if state.should_quit {
                return;
            }
        }

        // Wait for hotkey press
        while !is_recording(&hotkey_state) {
            // Check quit periodically
            {
                let state = app_state.lock().await;
                if state.should_quit {
                    return;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        info!("Hotkey pressed, starting recording...");

        // Update UI state
        let _ = gui_tx.try_send(GuiCommand::SetState(AppState::Recording));

        // Record audio while hotkey is held
        let mic_index = {
            let state = app_state.lock().await;
            state.config.mic_index
        };
        let level_tx_clone = level_tx.clone();

        // Clone hotkey_state for the blocking task
        let hotkey_state_clone = Arc::clone(&hotkey_state);

        let audio_result = tokio::task::spawn_blocking(move || {
            audio::record_while(
                mic_index,
                || is_recording(&hotkey_state_clone),
                level_tx_clone,
            )
        })
        .await;

        match audio_result {
            Ok(Ok(audio_bytes)) => {
                let duration = wav_duration_seconds(&audio_bytes);
                info!("Recording complete: {:.1}s", duration);

                // Check if recording is long enough
                if audio::is_valid_duration(&audio_bytes) {
                    // Send to transcription task
                    if let Err(e) = transcription_tx.send(audio_bytes).await {
                        error!("Failed to send audio to transcription: {}", e);
                    }
                } else {
                    warn!("Recording too short, discarding");

                    // Update UI to show error
                    let _ = gui_tx.try_send(GuiCommand::SetStatus("Too short".to_string()));
                    let _ = gui_tx.try_send(GuiCommand::SetState(AppState::Error));

                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    let _ = gui_tx.try_send(GuiCommand::SetState(AppState::Ready));
                }
            }
            Ok(Err(e)) => {
                error!("Recording error: {}", e);

                let _ = gui_tx.try_send(GuiCommand::SetStatus(format!("Error: {}", e)));
                let _ = gui_tx.try_send(GuiCommand::SetState(AppState::Error));

                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                let _ = gui_tx.try_send(GuiCommand::SetState(AppState::Ready));
            }
            Err(e) => {
                error!("Recording task panicked: {}", e);

                let _ = gui_tx
                    .try_send(GuiCommand::SetStatus(format!("Recording crashed: {}", e)));
                let _ = gui_tx.try_send(GuiCommand::SetState(AppState::Error));

                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                let _ = gui_tx.try_send(GuiCommand::SetState(AppState::Ready));
            }
        }

        // Small delay before next recording
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
