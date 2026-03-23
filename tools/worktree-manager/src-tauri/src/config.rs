use serde::{Deserialize, Serialize};
use std::fs;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AppConfig {
    pub repositories: Vec<String>,
}

fn config_path(app_handle: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))?;
    Ok(data_dir.join("config.json"))
}

pub fn load_config(app_handle: &tauri::AppHandle) -> AppConfig {
    let path = match config_path(app_handle) {
        Ok(p) => p,
        Err(_) => return AppConfig::default(),
    };

    if !path.exists() {
        return AppConfig::default();
    }

    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => AppConfig::default(),
    }
}

pub fn save_config(app_handle: &tauri::AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = config_path(app_handle)?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {e}"))?;
    }

    let json =
        serde_json::to_string_pretty(config).map_err(|e| format!("Failed to serialize config: {e}"))?;

    fs::write(&path, json).map_err(|e| format!("Failed to write config: {e}"))?;

    Ok(())
}

// --- Tauri Commands ---

#[tauri::command]
pub fn list_repositories(app_handle: tauri::AppHandle) -> Vec<String> {
    load_config(&app_handle).repositories
}

#[tauri::command]
pub fn add_repository(app_handle: tauri::AppHandle, path: String) -> Result<(), String> {
    let mut config = load_config(&app_handle);
    if !config.repositories.contains(&path) {
        config.repositories.push(path);
        save_config(&app_handle, &config)?;
    }
    Ok(())
}

#[tauri::command]
pub fn remove_repository(app_handle: tauri::AppHandle, path: String) -> Result<(), String> {
    let mut config = load_config(&app_handle);
    config.repositories.retain(|r| r != &path);
    save_config(&app_handle, &config)?;
    Ok(())
}
