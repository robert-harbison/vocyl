use std::path::PathBuf;
use std::sync::Mutex;

use tauri::Manager;

use crate::constants::{
    DEFAULT_PTT_SHORTCUT, DEFAULT_TTS_SHORTCUT, SETTINGS_SCHEMA_VERSION,
};

// ── Defaults ──────────────────────────────────────────────────────────────────

fn default_schema_version() -> u32 {
    SETTINGS_SCHEMA_VERSION
}
fn default_ptt_shortcut() -> String {
    DEFAULT_PTT_SHORTCUT.to_string()
}
fn default_tts_shortcut() -> String {
    DEFAULT_TTS_SHORTCUT.to_string()
}
fn default_tts_voice() -> Option<String> {
    Some("af".to_string())
}

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct VocylSettings {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,

    pub input_device_id: Option<String>,
    pub output_device_id: Option<String>,

    #[serde(default = "default_ptt_shortcut")]
    pub ptt_shortcut: String,

    #[serde(default = "default_tts_shortcut")]
    pub tts_shortcut: String,

    #[serde(default = "default_tts_voice")]
    pub tts_voice: Option<String>,

    pub active_stt_model: Option<String>,
}

impl Default for VocylSettings {
    fn default() -> Self {
        Self {
            schema_version: SETTINGS_SCHEMA_VERSION,
            input_device_id: None,
            output_device_id: None,
            ptt_shortcut: default_ptt_shortcut(),
            tts_shortcut: default_tts_shortcut(),
            tts_voice: default_tts_voice(),
            active_stt_model: None,
        }
    }
}

/// In-memory settings cache updated on every save so shortcut handlers can
/// read settings without hitting the filesystem.
pub struct SettingsCache(pub Mutex<VocylSettings>);

// ── Internal helpers ──────────────────────────────────────────────────────────

pub fn settings_path<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}

pub fn load_settings_internal<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<VocylSettings, String> {
    let path = settings_path(app)?;
    log::debug!("Loading settings from {}", path.display());
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let settings = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    log::info!("Settings loaded successfully from {}", path.display());
    Ok(settings)
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn settings_exist<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> bool {
    let exists = settings_path(&app).map(|p| p.exists()).unwrap_or(false);
    log::debug!("settings_exist → {}", exists);
    exists
}

#[tauri::command]
pub fn load_settings<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<VocylSettings, String> {
    log::info!("load_settings invoked");
    load_settings_internal(&app)
}

#[tauri::command]
pub fn save_settings<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    settings: VocylSettings,
) -> Result<(), String> {
    let path = settings_path(&app)?;
    log::info!("Saving settings to {}", path.display());
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    log::info!("Settings saved successfully");

    // Update in-memory cache
    if let Some(cache) = app.try_state::<SettingsCache>() {
        *cache.0.lock().unwrap() = settings;
    }

    Ok(())
}
