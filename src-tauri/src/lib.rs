mod runtime;

use base64::Engine;
use runtime::{
    RuntimeSnapshot, RuntimeState, create_tray, ensure_hud_window, init_logging, input_devices,
};
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::Manager;
use voice_type::api::test_api_key_with_model;
use voice_type::config;
use voice_type::input::{normalize_browser_key_code, normalize_browser_mouse_button};
use voice_type::types::{AppState, Config, STATUS_API_KEY_EMPTY, STATUS_API_KEY_VALID};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    app_name: String,
    version: String,
    backend: String,
}

#[tauri::command]
fn get_app_info() -> AppInfo {
    AppInfo {
        app_name: config::APP_DISPLAY_NAME.to_string(),
        version: voice_type::VERSION.to_string(),
        backend: "Rust + Tokio".to_string(),
    }
}

#[tauri::command]
fn load_config() -> Config {
    config::load()
}

#[tauri::command]
fn get_runtime_snapshot(runtime: tauri::State<'_, RuntimeState>) -> RuntimeSnapshot {
    runtime.snapshot()
}

#[tauri::command]
fn list_microphones() -> Vec<String> {
    input_devices()
}

#[tauri::command]
fn begin_hotkey_capture(runtime: tauri::State<'_, RuntimeState>) {
    runtime.begin_hotkey_capture();
}

#[tauri::command]
fn cancel_hotkey_capture(runtime: tauri::State<'_, RuntimeState>) {
    runtime.cancel_hotkey_capture();
}

#[tauri::command]
fn normalize_keyboard_capture(code: String) -> Option<runtime::HotkeyCapturePayload> {
    normalize_browser_key_code(&code).map(|binding| runtime::HotkeyCapturePayload {
        config_value: binding.config_value(),
        label: binding.label(),
    })
}

#[tauri::command]
fn normalize_mouse_capture(button: i16) -> Option<runtime::HotkeyCapturePayload> {
    normalize_browser_mouse_button(button).map(|binding| runtime::HotkeyCapturePayload {
        config_value: binding.config_value(),
        label: binding.label(),
    })
}

#[tauri::command]
fn save_config(
    app: tauri::AppHandle,
    runtime: tauri::State<'_, RuntimeState>,
    config: Config,
) -> Result<(), String> {
    runtime.save_config(&app, config).map_err(|error| {
        runtime.update_runtime_state(
            AppState::Error,
            format!("Failed to save settings: {}", error),
        );
        error.to_string()
    })
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle, runtime: tauri::State<'_, RuntimeState>) {
    runtime.set_should_quit();
    app.exit(0);
}

#[tauri::command]
fn open_external_url(url: String) -> Result<(), String> {
    let trimmed = url.trim();
    if !(trimmed.starts_with("https://") || trimmed.starts_with("http://")) {
        return Err("Only http and https URLs are allowed".to_string());
    }

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut cmd = std::process::Command::new("cmd");
        cmd.args(["/C", "start", "", trimmed]);
        cmd
    };

    #[cfg(target_os = "macos")]
    let mut command = {
        let mut cmd = std::process::Command::new("open");
        cmd.arg(trimmed);
        cmd
    };

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    let mut command = {
        let mut cmd = std::process::Command::new("xdg-open");
        cmd.arg(trimmed);
        cmd
    };

    command
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Failed to open URL: {}", error))
}

#[tauri::command]
fn import_background_image(path: String) -> Result<String, String> {
    import_background_asset(&path, "default-background")
}

#[tauri::command]
fn import_hud_background_image(path: String) -> Result<String, String> {
    import_background_asset(&path, "hud-background")
}

fn import_background_asset(path: &str, file_stem: &str) -> Result<String, String> {
    let source = PathBuf::from(path.trim());
    if !source.exists() {
        return Err("Selected background image does not exist".to_string());
    }

    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .ok_or_else(|| "Background image must have a valid file extension".to_string())?;

    let allowed_extensions = ["png", "jpg", "jpeg", "webp", "bmp", "gif"];
    if !allowed_extensions.contains(&extension.as_str()) {
        return Err("Unsupported background image format".to_string());
    }

    let destination_dir = config::backgrounds_dir()
        .map_err(|error| format!("Failed to prepare backgrounds directory: {}", error))?;

    remove_old_background_assets(&destination_dir, file_stem)
        .map_err(|error| format!("Failed to replace existing background image: {}", error))?;

    let destination = destination_dir.join(format!("{}.{}", file_stem, extension));
    std::fs::copy(&source, &destination)
        .map_err(|error| format!("Failed to import background image: {}", error))?;

    destination
        .into_os_string()
        .into_string()
        .map_err(|_| "Imported background path is not valid UTF-8".to_string())
}

#[tauri::command]
fn load_background_image_data_url(path: String) -> Result<String, String> {
    let requested_path = PathBuf::from(path.trim());
    if !requested_path.exists() {
        return Err("Imported background image does not exist".to_string());
    }

    let backgrounds_dir = config::backgrounds_dir()
        .map_err(|error| format!("Failed to resolve backgrounds directory: {}", error))?;

    let canonical_requested = requested_path
        .canonicalize()
        .map_err(|error| format!("Failed to resolve imported background path: {}", error))?;
    let canonical_backgrounds = backgrounds_dir
        .canonicalize()
        .map_err(|error| format!("Failed to resolve backgrounds directory: {}", error))?;

    if !canonical_requested.starts_with(&canonical_backgrounds) {
        return Err(
            "Background image path is outside the app-controlled backgrounds folder".to_string(),
        );
    }

    let extension = canonical_requested
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .ok_or_else(|| "Imported background image must have a valid file extension".to_string())?;

    let mime = match extension.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "gif" => "image/gif",
        _ => return Err("Unsupported imported background image format".to_string()),
    };

    let bytes = std::fs::read(&canonical_requested)
        .map_err(|error| format!("Failed to read imported background image: {}", error))?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(format!("data:{};base64,{}", mime, encoded))
}

fn remove_old_background_assets(directory: &Path, file_stem: &str) -> Result<(), std::io::Error> {
    if !directory.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let matches_stem = path
            .file_stem()
            .and_then(|value| value.to_str())
            .map(|value| value == file_stem)
            .unwrap_or(false);

        if path.is_file() && matches_stem {
            std::fs::remove_file(path)?;
        }
    }

    Ok(())
}

#[tauri::command]
async fn test_api_key(
    runtime: tauri::State<'_, RuntimeState>,
    api_key: String,
    model: String,
) -> Result<String, String> {
    if api_key.trim().is_empty() {
        runtime.update_runtime_state(AppState::Error, STATUS_API_KEY_EMPTY.to_string());
        return Err(STATUS_API_KEY_EMPTY.to_string());
    }

    runtime.update_runtime_state(AppState::Processing, "Testing API key...");

    test_api_key_with_model(&api_key, Some(&model))
        .await
        .map(|_| {
            runtime.update_runtime_state(AppState::Done, STATUS_API_KEY_VALID.to_string());
            STATUS_API_KEY_VALID.to_string()
        })
        .map_err(|error| {
            runtime
                .update_runtime_state(AppState::Error, format!("API key test failed: {}", error));
            error.to_string()
        })
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
                    app.handle().exit(0);
                    std::process::exit(0);
                }
                Err(error) => return Err(anyhow::anyhow!(error.to_string()).into()),
            };
            let initial_snapshot = runtime.snapshot();
            app.manage(runtime);

            ensure_hud_window(&app.handle(), &initial_snapshot.config)
                .map_err(|error| anyhow::anyhow!(error.to_string()))?;

            let tray =
                create_tray(&app.handle()).map_err(|error| anyhow::anyhow!(error.to_string()))?;
            app.manage(tray);

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_title(config::APP_DISPLAY_NAME);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
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
            quit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
