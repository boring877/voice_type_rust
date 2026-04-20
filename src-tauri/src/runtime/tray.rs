use anyhow::{Context, Result};
use super::{first_line, RuntimeState, RuntimeSnapshot};
use tauri::menu::MenuBuilder;
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};
use voice_type::config::APP_DISPLAY_NAME;

pub(crate) const MENU_OPEN: &str = "open";
pub(crate) const MENU_QUIT: &str = "quit";
pub(crate) const MENU_VERSION: &str = "version";
pub(crate) const TRAY_ID: &str = "main";

pub(crate) fn create_tray(app: &AppHandle) -> Result<TrayIcon<Wry>> {
    let menu = MenuBuilder::new(app)
        .text(MENU_OPEN, "Open Voice Type")
        .separator()
        .text(MENU_VERSION, format!("Voice Type v{}", voice_type::VERSION))
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

pub(crate) fn quit_runtime(app: &AppHandle) {
    if let Some(runtime) = app.try_state::<RuntimeState>() {
        runtime.set_should_quit();
    }
    app.exit(0);
}

pub(crate) fn show_main_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.show()?;
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
    Ok(())
}

pub(crate) fn tray_tooltip(snapshot: &RuntimeSnapshot) -> String {
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

pub(crate) fn update_tray_tooltip(app: &AppHandle, snapshot: &RuntimeSnapshot) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(tray_tooltip(snapshot)));
    }
}
