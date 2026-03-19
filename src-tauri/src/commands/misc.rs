#[tauri::command]
pub fn open_external_url(url: String) -> Result<(), String> {
    let trimmed = url.trim();
    if !(trimmed.starts_with("https://") || trimmed.starts_with("http://")) {
        return Err("Only http and https URLs are allowed".to_string());
    }

    open::that(trimmed)
        .map(|_| ())
        .map_err(|error| format!("Failed to open URL: {}", error))
}

#[tauri::command]
pub fn get_history() -> Vec<voice_type::history::HistoryEntry> {
    voice_type::history::load()
}

#[tauri::command]
pub fn clear_history() -> Result<(), String> {
    voice_type::history::clear().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn play_beep(frequency: u32, duration_ms: u32) -> Result<(), String> {
    std::thread::spawn(move || {
        #[cfg(target_os = "windows")]
        {
            mod win_beep {
                use std::ffi::c_int;
                unsafe extern "system" {
                    pub fn Beep(dwFreq: u32, dwDuration: u32) -> c_int;
                }
            }
            unsafe {
                win_beep::Beep(frequency, duration_ms);
            }
        }
    });
    Ok(())
}
