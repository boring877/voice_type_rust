mod commands;
mod runtime;

use commands::*;
use runtime::{RuntimeState, init_logging};
use runtime::hud::ensure_hud_window;
use runtime::tray::create_tray;
use serde::Serialize;
use tauri::Manager;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    app_name: String,
    version: String,
    backend: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let runtime = match RuntimeState::initialize(&app.handle()) {
                Ok(runtime) => runtime,
                Err(error)
                    if error
                        .to_string()
                        .contains("Another instance is already running") =>
                {
                    std::process::exit(0);
                }
                Err(error) => return Err(anyhow::anyhow!(error.to_string()).into()),
            };
            let initial_snapshot = runtime.snapshot();
            app.manage(runtime);

            if let Err(e) =
                ensure_hud_window(&app.handle(), &initial_snapshot.config)
            {
                tracing::warn!("HUD window creation failed: {}", e);
            }

            let _tray = match create_tray(&app.handle()) {
                Ok(tray) => {
                    app.manage(tray);
                }
                Err(e) => {
                    tracing::warn!("Tray icon creation failed: {}", e);
                }
            };

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_title(voice_type::config::APP_DISPLAY_NAME);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            load_config,
            get_runtime_snapshot,
            list_microphones,
            begin_hotkey_capture,
            cancel_hotkey_capture,
            normalize_keyboard_capture,
            normalize_mouse_capture,
            save_config,
            test_api_key,
            import_background_image,
            import_hud_background_image,
            load_background_image_data_url,
            open_external_url,
            quit_app,
            has_configured_api_key,
            get_history,
            clear_history,
            play_beep
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            #[cfg(target_os = "windows")]
            {
                mod win_msg {
                    use std::ffi::c_int;

                    unsafe extern "system" {
                        pub fn MessageBoxA(
                            hwnd: *mut std::ffi::c_void,
                            text: *const u8,
                            caption: *const u8,
                            utype: c_int,
                        ) -> c_int;
                    }
                }

                let msg =
                    format!("Failed to start Voice Type:\n{}", e);
                let caption = std::ffi::CString::new("Voice Type Error")
                    .unwrap_or_default();
                let msg_cstr = std::ffi::CString::new(msg)
                    .unwrap_or_default();
                unsafe {
                    win_msg::MessageBoxA(
                        std::ptr::null_mut(),
                        msg_cstr.as_ptr() as *const u8,
                        caption.as_ptr() as *const u8,
                        0x10,
                    );
                }
            }
            std::process::exit(1);
        });
}
