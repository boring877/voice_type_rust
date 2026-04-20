use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub release_url: String,
    pub update_available: bool,
}

#[tauri::command]
pub async fn check_for_updates() -> Result<Option<UpdateInfo>, String> {
    let current = voice_type::VERSION;

    let response = reqwest::Client::new()
        .get("https://api.github.com/repos/boring877/voice_type_rust/releases/latest")
        .header("User-Agent", "Voice-Type-Desktop")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Update check failed: {}", e))?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse update response: {}", e))?;

    let tag = body["tag_name"].as_str().unwrap_or("");
    let latest = tag.trim_start_matches('v');

    if latest.is_empty() {
        return Ok(None);
    }

    let release_url = body["html_url"]
        .as_str()
        .unwrap_or("https://github.com/boring877/voice_type_rust/releases")
        .to_string();

    if latest == current {
        return Ok(None);
    }

    let latest_major: u32 = latest.split('.').next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let current_major: u32 = current.split('.').next().and_then(|s| s.parse().ok()).unwrap_or(0);

    if latest_major < current_major {
        return Ok(None);
    }

    Ok(Some(UpdateInfo {
        current_version: current.to_string(),
        latest_version: latest.to_string(),
        release_url,
        update_available: true,
    }))
}
