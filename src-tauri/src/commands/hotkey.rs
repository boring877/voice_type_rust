use crate::runtime::{HotkeyCapturePayload, RuntimeState};
use voice_type::input::{normalize_browser_key_code, normalize_browser_mouse_button};

#[tauri::command]
pub fn begin_hotkey_capture(runtime: tauri::State<'_, RuntimeState>) {
    runtime.begin_hotkey_capture();
}

#[tauri::command]
pub fn cancel_hotkey_capture(runtime: tauri::State<'_, RuntimeState>) {
    runtime.cancel_hotkey_capture();
}

#[tauri::command]
pub fn normalize_keyboard_capture(code: String) -> Option<HotkeyCapturePayload> {
    normalize_browser_key_code(&code).map(|binding| HotkeyCapturePayload {
        config_value: binding.config_value(),
        label: binding.label(),
    })
}

#[tauri::command]
pub fn normalize_mouse_capture(button: i16) -> Option<HotkeyCapturePayload> {
    normalize_browser_mouse_button(button).map(|binding| HotkeyCapturePayload {
        config_value: binding.config_value(),
        label: binding.label(),
    })
}

#[tauri::command]
pub fn list_microphones() -> Vec<String> {
    crate::runtime::input_devices()
}
