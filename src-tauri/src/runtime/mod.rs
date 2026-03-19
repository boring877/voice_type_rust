use anyhow::{Context, Result};
use serde::Serialize;
use single_instance::SingleInstance;
use std::sync::{Arc, Mutex as StdMutex};
use tauri::{AppHandle, Manager};
use tokio::sync::{Mutex, mpsc};
use tracing::{info, warn};
use voice_type::audio;
use voice_type::config::{self, APP_DISPLAY_NAME, SINGLE_INSTANCE_ID};
use voice_type::input::{describe_hotkey, parse_hotkey, start_listener};
use voice_type::types::{
    AppState, Config, GuiCommand, HotkeyState, STATUS_RECORDING, STATUS_TRANSCRIBING, SharedState,
};

pub(crate) mod bridges;
pub(crate) mod hud;
pub(crate) mod tray;

use bridges::*;
use hud::*;
use tray::*;

pub const HOTKEY_CAPTURE_EVENT: &str = "voice-type://hotkey-captured";
pub const RUNTIME_EVENT: &str = "voice-type://runtime";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSnapshot {
    pub app_state: String,
    pub audio_level: f32,
    pub config: Config,
    pub hotkey_label: String,
    pub status_text: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyCapturePayload {
    pub config_value: String,
    pub label: String,
}

pub struct RuntimeState {
    snapshot: Arc<StdMutex<RuntimeSnapshot>>,
    shared_state: Arc<Mutex<SharedState>>,
    hotkey_state: Arc<HotkeyState>,
    gui_tx: mpsc::Sender<GuiCommand>,
    _instance: Option<SingleInstance>,
    _runtime: Arc<tokio::runtime::Runtime>,
}

impl RuntimeState {
    pub fn initialize(app: &AppHandle) -> Result<Self> {
        let instance = match SingleInstance::new(SINGLE_INSTANCE_ID) {
            Ok(inst) => Some(inst),
            Err(e) => {
                tracing::warn!(
                    "Single instance check failed, continuing without lock: {}",
                    e
                );
                None
            }
        };

        if let Some(ref inst) = instance {
            if !inst.is_single() {
                anyhow::bail!("Another instance is already running");
            }
        }

        let config = config::load();
        let hotkey_state = start_listener(&config.hotkey)
            .with_context(|| format!("Failed to start hotkey listener for {}", config.hotkey))?;
        let shared_state = Arc::new(Mutex::new(SharedState::new(config.clone())));
        let snapshot = Arc::new(StdMutex::new(RuntimeSnapshot::new(config.clone())));

        let (gui_tx, gui_rx) = mpsc::channel(64);
        let (audio_level_tx, audio_level_rx) = mpsc::channel(128);
        let (transcription_tx, transcription_rx) = mpsc::channel(4);

        let runtime = Arc::new(
            tokio::runtime::Runtime::new().expect("failed to create shared tokio runtime"),
        );

        spawn_audio_task(
            Arc::clone(&shared_state),
            Arc::clone(&hotkey_state),
            audio_level_tx,
            transcription_tx,
            gui_tx.clone(),
            Arc::clone(&runtime),
        );
        spawn_transcription_task(
            Arc::clone(&shared_state),
            transcription_rx,
            gui_tx.clone(),
            Arc::clone(&runtime),
        );
        spawn_audio_level_bridge(audio_level_rx, gui_tx.clone());
        spawn_gui_bridge(app.clone(), Arc::clone(&snapshot), gui_rx);
        spawn_hotkey_capture_bridge(
            app.clone(),
            Arc::clone(&shared_state),
            Arc::clone(&hotkey_state),
        );

        if let Some(window) = app.get_webview_window("main") {
            let _ = window.set_title(APP_DISPLAY_NAME);
            let _ = window.set_always_on_top(config.always_on_top);
        }

        update_tray_tooltip(app, &snapshot.lock().unwrap().clone());

        Ok(Self {
            snapshot,
            shared_state,
            hotkey_state,
            gui_tx,
            _instance: instance,
            _runtime: runtime,
        })
    }

    pub fn begin_hotkey_capture(&self) {
        self.hotkey_state.begin_capture();
    }

    pub fn cancel_hotkey_capture(&self) {
        self.hotkey_state.cancel_capture();
    }

    pub fn save_config(&self, app: &AppHandle, config: Config) -> Result<()> {
        let binding = parse_hotkey(&config.hotkey)
            .with_context(|| format!("Invalid hotkey configuration: {}", config.hotkey))?;

        config::save(&config).context("Failed to save configuration")?;

        {
            let mut state = self.shared_state.blocking_lock();
            state.config = config.clone();
        }

        self.hotkey_state.cancel_capture();
        self.hotkey_state.set_target_binding(binding);
        self.gui_tx
            .try_send(GuiCommand::UpdateConfig(config.clone()))
            .ok();
        self.gui_tx
            .try_send(GuiCommand::SetStatus("Settings saved".to_string()))
            .ok();
        self.gui_tx
            .try_send(GuiCommand::SetState(AppState::Done))
            .ok();

        if let Some(window) = app.get_webview_window("main") {
            let _ = window.set_always_on_top(config.always_on_top);
        }

        if let Some(window) = app.get_webview_window(HUD_WINDOW) {
            let _ = apply_hud_window_layout(&window, &config);
            let _ = position_hud_window(app, &window, &config);
        }

        Ok(())
    }

    pub fn set_should_quit(&self) {
        let mut state = self.shared_state.blocking_lock();
        state.should_quit = true;
    }

    pub fn snapshot(&self) -> RuntimeSnapshot {
        self.snapshot
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub fn update_runtime_state(&self, state: AppState, status_text: impl Into<String>) {
        self.gui_tx
            .try_send(GuiCommand::SetStatus(status_text.into()))
            .ok();
        self.gui_tx.try_send(GuiCommand::SetState(state)).ok();
    }
}

impl RuntimeSnapshot {
    fn new(config: Config) -> Self {
        Self {
            app_state: app_state_label(AppState::Ready).to_string(),
            audio_level: 0.0,
            hotkey_label: describe_hotkey(&config.hotkey),
            status_text: hold_to_speak_text(&config.hotkey),
            config,
        }
    }
}

fn app_state_label(state: AppState) -> &'static str {
    match state {
        AppState::Ready => "ready",
        AppState::Recording => "recording",
        AppState::Processing => "processing",
        AppState::Done => "done",
        AppState::Error => "error",
    }
}

pub(crate) fn apply_gui_command(snapshot: &mut RuntimeSnapshot, cmd: GuiCommand) {
    match cmd {
        GuiCommand::SetState(state) => {
            snapshot.app_state = app_state_label(state).to_string();
            if state != AppState::Recording {
                snapshot.audio_level = 0.0;
            }

            snapshot.status_text = match state {
                AppState::Ready => hold_to_speak_text(&snapshot.config.hotkey),
                AppState::Recording => STATUS_RECORDING.to_string(),
                AppState::Processing => STATUS_TRANSCRIBING.to_string(),
                AppState::Done | AppState::Error => snapshot.status_text.clone(),
            };
        }
        GuiCommand::SetStatus(text) => {
            snapshot.status_text = text;
        }
        GuiCommand::SetLevel(level) => {
            snapshot.audio_level = level;
        }
        GuiCommand::UpdateConfig(config) => {
            snapshot.hotkey_label = describe_hotkey(&config.hotkey);
            snapshot.config = config;
            if snapshot.app_state == "ready" {
                snapshot.status_text = hold_to_speak_text(&snapshot.config.hotkey);
            }
        }
    }
}

fn hold_to_speak_text(hotkey: &str) -> String {
    format!("Hold {} to speak...", describe_hotkey(hotkey))
}

pub(crate) fn first_line(text: &str) -> String {
    text.lines()
        .next()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            if line.chars().count() > 72 {
                let truncated: String = line.chars().take(69).collect();
                format!("{}...", truncated)
            } else {
                line.to_string()
            }
        })
        .unwrap_or_else(|| "Ready".to_string())
}

pub fn init_logging() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("voice_type=info,voice_type_tauri=info")
        .try_init();
    info!("{} Tauri shell starting", APP_DISPLAY_NAME);
}

pub fn input_devices() -> Vec<String> {
    match audio::list_input_devices() {
        Ok(devices) => devices,
        Err(error) => {
            warn!("Failed to list input devices: {}", error);
            Vec::new()
        }
    }
}
