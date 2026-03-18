use anyhow::{Context, Result};
use serde::Serialize;
use single_instance::SingleInstance;
use std::sync::{Arc, Mutex as StdMutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::menu::MenuBuilder;
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder, Wry,
};
use tokio::sync::{Mutex, mpsc};
use tracing::{info, warn};
use voice_type::audio;
use voice_type::config::{self, APP_DISPLAY_NAME, SINGLE_INSTANCE_ID};
use voice_type::input::{describe_hotkey, parse_hotkey, start_listener};
use voice_type::tasks::{audio_recording_task, transcription_task};
use voice_type::types::{
    AppState, Config, GuiCommand, HotkeyState, STATUS_RECORDING, STATUS_TRANSCRIBING, SharedState,
};

pub const HOTKEY_CAPTURE_EVENT: &str = "voice-type://hotkey-captured";
pub const RUNTIME_EVENT: &str = "voice-type://runtime";

const MENU_OPEN: &str = "open";
const MENU_QUIT: &str = "quit";
const TRAY_ID: &str = "main";
const HUD_WINDOW: &str = "hud";
const HUD_WIDTH: f64 = 264.0;
const HUD_HEIGHT: f64 = 96.0;
const HUD_MARGIN: i32 = 18;

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

        spawn_audio_task(
            Arc::clone(&shared_state),
            Arc::clone(&hotkey_state),
            audio_level_tx,
            transcription_tx,
            gui_tx.clone(),
        );
        spawn_transcription_task(Arc::clone(&shared_state), transcription_rx, gui_tx.clone());
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

pub fn create_tray(app: &AppHandle) -> Result<TrayIcon<Wry>> {
    let menu = MenuBuilder::new(app)
        .text(MENU_OPEN, "Open Voice Type")
        .separator()
        .text(MENU_QUIT, "Quit")
        .build()
        .context("Failed to build tray menu")?;

    let icon = app
        .default_window_icon()
        .cloned()
        .context("Missing default window icon for tray")?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .menu(&menu)
        .tooltip(APP_DISPLAY_NAME)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            MENU_OPEN => {
                let _ = show_main_window(app);
            }
            MENU_QUIT => {
                quit_runtime(app);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = show_main_window(&tray.app_handle());
            }
        })
        .build(app)
        .context("Failed to create tray icon")
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

fn apply_gui_command(snapshot: &mut RuntimeSnapshot, cmd: GuiCommand) {
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

fn quit_runtime(app: &AppHandle) {
    if let Some(runtime) = app.try_state::<RuntimeState>() {
        runtime.set_should_quit();
    }
    app.exit(0);
}

fn show_main_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.show()?;
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
    Ok(())
}

fn spawn_audio_level_bridge(
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

fn spawn_audio_task(
    shared_state: Arc<Mutex<SharedState>>,
    hotkey_state: Arc<HotkeyState>,
    audio_level_tx: mpsc::Sender<f32>,
    transcription_tx: mpsc::Sender<Vec<u8>>,
    gui_tx: mpsc::Sender<GuiCommand>,
) {
    thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("failed to create audio runtime");
        runtime.block_on(audio_recording_task(
            shared_state,
            hotkey_state,
            audio_level_tx,
            transcription_tx,
            gui_tx,
        ));
    });
}

fn spawn_gui_bridge(
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

pub fn ensure_hud_window(app: &AppHandle, config: &Config) -> Result<()> {
    if app.get_webview_window(HUD_WINDOW).is_some() {
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(app, HUD_WINDOW, WebviewUrl::App("index.html".into()))
        .title(APP_DISPLAY_NAME)
        .inner_size(HUD_WIDTH, HUD_HEIGHT)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .closable(false)
        .visible(false)
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .focusable(false)
        .skip_taskbar(true)
        .shadow(false)
        .build()
        .context("Failed to build HUD webview window")?;

    let _ = apply_hud_window_layout(&window, config);
    let _ = window.set_ignore_cursor_events(true);
    let _ = position_hud_window(app, &window, config);

    Ok(())
}

fn sync_hud_window(app: &AppHandle, snapshot: &RuntimeSnapshot) {
    let Some(window) = app.get_webview_window(HUD_WINDOW) else {
        return;
    };

    if snapshot.config.hud_enabled
        && matches!(snapshot.app_state.as_str(), "recording" | "processing")
    {
        let visible = window.is_visible().unwrap_or(false);
        if !visible {
            let _ = position_hud_window(app, &window, &snapshot.config);
            let _ = window.show();
        }
    } else if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    }
}

fn position_hud_window(
    app: &AppHandle,
    window: &WebviewWindow,
    config: &Config,
) -> tauri::Result<()> {
    let monitor = app
        .get_webview_window("main")
        .and_then(|main_window| main_window.current_monitor().ok().flatten())
        .or(window.current_monitor()?)
        .or(window.primary_monitor()?);

    let Some(monitor) = monitor else {
        return Ok(());
    };

    let work_area = monitor.work_area();
    let size = window.inner_size()?;
    let x = if config.hud_side == "left" {
        work_area.position.x + HUD_MARGIN
    } else {
        work_area.position.x + work_area.size.width as i32 - size.width as i32 - HUD_MARGIN
    };
    let y = work_area.position.y + work_area.size.height as i32 - size.height as i32 - HUD_MARGIN;

    window.set_position(PhysicalPosition::new(x, y))
}

fn apply_hud_window_layout(window: &WebviewWindow, config: &Config) -> tauri::Result<()> {
    let (width, height) = hud_window_size(config);
    window.set_size(LogicalSize::new(width, height))
}

fn hud_window_size(config: &Config) -> (f64, f64) {
    let show_topline = config.hud_show_state || config.hud_show_app_name;

    let mut height = 50.0;
    if show_topline {
        height += 14.0;
    }
    if config.hud_show_description {
        height += 12.0;
    }
    if config.hud_show_meter {
        height += 30.0;
    }

    let width = if config.hud_show_meter {
        264.0
    } else if config.hud_show_description {
        236.0
    } else if show_topline {
        212.0
    } else {
        188.0
    };

    (width, height)
}

fn spawn_hotkey_capture_bridge(
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

fn spawn_transcription_task(
    shared_state: Arc<Mutex<SharedState>>,
    transcription_rx: mpsc::Receiver<Vec<u8>>,
    gui_tx: mpsc::Sender<GuiCommand>,
) {
    thread::spawn(move || {
        let runtime =
            tokio::runtime::Runtime::new().expect("failed to create transcription runtime");
        runtime.block_on(transcription_task(shared_state, transcription_rx, gui_tx));
    });
}

fn tray_tooltip(snapshot: &RuntimeSnapshot) -> String {
    match snapshot.app_state.as_str() {
        "recording" => format!("{} - Recording...", APP_DISPLAY_NAME),
        "processing" => format!("{} - Transcribing...", APP_DISPLAY_NAME),
        "done" | "error" => format!(
            "{} - {}",
            APP_DISPLAY_NAME,
            first_line(&snapshot.status_text)
        ),
        _ => format!(
            "{} - Hold {} to speak",
            APP_DISPLAY_NAME, snapshot.hotkey_label
        ),
    }
}

fn update_tray_tooltip(app: &AppHandle, snapshot: &RuntimeSnapshot) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(tray_tooltip(snapshot)));
    }
}

fn first_line(text: &str) -> String {
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
