use std::sync::{Arc, Mutex as StdMutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::sync::{Mutex, mpsc};
use super::{
    apply_gui_command, HotkeyCapturePayload, HOTKEY_CAPTURE_EVENT, RuntimeSnapshot, RUNTIME_EVENT,
};
use super::hud::sync_hud_window;
use super::tray::update_tray_tooltip;
use voice_type::tasks::{audio_recording_task, transcription_task};
use voice_type::types::{GuiCommand, HotkeyState, SharedState};

pub(crate) fn spawn_audio_level_bridge(
    mut audio_level_rx: mpsc::Receiver<f32>,
    gui_tx: mpsc::Sender<GuiCommand>,
) {
    thread::spawn(move || {
        let frame_interval = Duration::from_millis(33);
        let mut last_emit = Instant::now()
            .checked_sub(frame_interval)
            .unwrap_or_else(Instant::now);

        while let Some(level) = audio_level_rx.blocking_recv() {
            let mut latest_level = level.clamp(0.0, 1.0);
            while let Ok(next_level) = audio_level_rx.try_recv() {
                latest_level = next_level.clamp(0.0, 1.0);
            }

            if last_emit.elapsed() < frame_interval {
                continue;
            }

            let _ = gui_tx.try_send(GuiCommand::SetLevel(latest_level));
            last_emit = Instant::now();
        }
    });
}

pub(crate) fn spawn_audio_task(
    shared_state: Arc<Mutex<SharedState>>,
    hotkey_state: Arc<HotkeyState>,
    audio_level_tx: mpsc::Sender<f32>,
    transcription_tx: mpsc::Sender<Vec<u8>>,
    gui_tx: mpsc::Sender<GuiCommand>,
    runtime: Arc<tokio::runtime::Runtime>,
) {
    thread::spawn(move || {
        runtime.block_on(audio_recording_task(
            shared_state,
            hotkey_state,
            audio_level_tx,
            transcription_tx,
            gui_tx,
        ));
    });
}

pub(crate) fn spawn_gui_bridge(
    app: AppHandle,
    snapshot: Arc<StdMutex<RuntimeSnapshot>>,
    mut gui_rx: mpsc::Receiver<GuiCommand>,
) {
    thread::spawn(move || {
        while let Some(cmd) = gui_rx.blocking_recv() {
            let next_snapshot = {
                let mut current = snapshot
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                apply_gui_command(&mut current, cmd);
                current.clone()
            };

            update_tray_tooltip(&app, &next_snapshot);
            sync_hud_window(&app, &next_snapshot);
            let _ = app.emit(RUNTIME_EVENT, next_snapshot);
        }
    });
}

pub(crate) fn spawn_hotkey_capture_bridge(
    app: AppHandle,
    shared_state: Arc<Mutex<SharedState>>,
    hotkey_state: Arc<HotkeyState>,
) {
    thread::spawn(move || {
        loop {
            {
                let state = shared_state.blocking_lock();
                if state.should_quit {
                    return;
                }
            }

            if hotkey_state.is_capturing() {
                hotkey_state.arm_capture();

                if let Some(binding) = hotkey_state.take_captured_binding() {
                    let payload = HotkeyCapturePayload {
                        config_value: binding.config_value(),
                        label: binding.label(),
                    };
                    let _ = app.emit(HOTKEY_CAPTURE_EVENT, payload);
                }
            }

            thread::sleep(Duration::from_millis(50));
        }
    });
}

pub(crate) fn spawn_transcription_task(
    shared_state: Arc<Mutex<SharedState>>,
    transcription_rx: mpsc::Receiver<Vec<u8>>,
    gui_tx: mpsc::Sender<GuiCommand>,
    runtime: Arc<tokio::runtime::Runtime>,
) {
    thread::spawn(move || {
        runtime.block_on(transcription_task(shared_state, transcription_rx, gui_tx));
    });
}
