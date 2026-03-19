use crate::AppInfo;
use crate::runtime::{RuntimeSnapshot, RuntimeState};
use voice_type::api::test_api_key as validate_api_key;
use voice_type::config;
use voice_type::types::{AppState, Config, STATUS_API_KEY_EMPTY, STATUS_API_KEY_VALID};

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        app_name: config::APP_DISPLAY_NAME.to_string(),
        version: voice_type::VERSION.to_string(),
        backend: "Rust + Tokio".to_string(),
    }
}

#[tauri::command]
pub fn load_config() -> Config {
    config::load()
}

#[tauri::command]
pub fn get_runtime_snapshot(runtime: tauri::State<'_, RuntimeState>) -> RuntimeSnapshot {
    runtime.snapshot()
}

#[tauri::command]
pub fn save_config(
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
pub fn has_configured_api_key(runtime: tauri::State<'_, RuntimeState>) -> bool {
    let config = &runtime.snapshot().config;
    config::has_api_key(config)
}

#[tauri::command]
pub fn quit_app(app: tauri::AppHandle, runtime: tauri::State<'_, RuntimeState>) {
    runtime.set_should_quit();
    app.exit(0);
}

#[tauri::command]
pub async fn test_api_key(
    runtime: tauri::State<'_, RuntimeState>,
    api_key: String,
) -> Result<String, String> {
    if api_key.trim().is_empty() {
        runtime.update_runtime_state(AppState::Error, STATUS_API_KEY_EMPTY.to_string());
        return Err(STATUS_API_KEY_EMPTY.to_string());
    }

    runtime.update_runtime_state(AppState::Processing, "Testing API key...");

    validate_api_key(&api_key)
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
