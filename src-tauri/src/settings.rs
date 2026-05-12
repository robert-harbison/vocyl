use std::path::PathBuf;

use tauri::Manager;

use crate::constants::SETTINGS_SCHEMA_VERSION;

// ── Default helpers (required by serde) ───────────────────────────────────────

fn default_schema_version() -> u32 {
    SETTINGS_SCHEMA_VERSION
}

fn default_test_property() -> String {
    String::new()
}

// ── Type ──────────────────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct VocylSettings {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default = "default_test_property")]
    pub test_property: String,
}

impl Default for VocylSettings {
    fn default() -> Self {
        Self {
            schema_version: SETTINGS_SCHEMA_VERSION,
            test_property: default_test_property(),
        }
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn settings_path<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn settings_exist<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> bool {
    settings_path(&app).map(|p| p.exists()).unwrap_or(false)
}

#[tauri::command]
pub fn load_settings<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<VocylSettings, String> {
    let path = settings_path(&app)?;
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_settings<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    settings: VocylSettings,
) -> Result<(), String> {
    let path = settings_path(&app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}
