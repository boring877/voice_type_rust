use anyhow::{Context, Result};
use super::RuntimeSnapshot;
use tauri::{AppHandle, LogicalSize, Manager, PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use voice_type::config::APP_DISPLAY_NAME;
use voice_type::types::Config;

pub(crate) const HUD_WINDOW: &str = "hud";
pub(crate) const HUD_WIDTH: f64 = 264.0;
pub(crate) const HUD_HEIGHT: f64 = 96.0;
pub(crate) const HUD_MARGIN: i32 = 18;

pub(crate) fn ensure_hud_window(app: &AppHandle, config: &Config) -> Result<()> {
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

pub(crate) fn sync_hud_window(app: &AppHandle, snapshot: &RuntimeSnapshot) {
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

pub(crate) fn position_hud_window(
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

pub(crate) fn apply_hud_window_layout(window: &WebviewWindow, config: &Config) -> tauri::Result<()> {
    let (width, height) = hud_window_size(config);
    window.set_size(LogicalSize::new(width, height))
}

pub(crate) fn hud_window_size(config: &Config) -> (f64, f64) {
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
