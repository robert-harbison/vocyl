mod audio;
mod constants;
mod models;
mod overlay;
mod settings;
mod stt;
mod tts;
mod tray;
mod windows;
mod app;

use std::sync::Mutex;
use tauri::Manager;

use app::exit_app;
use overlay::update_shortcuts;
use settings::{load_settings, save_settings, settings_exist, SettingsCache};
use stt::SttState;
use tts::TtsState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            // Settings
            settings_exist,
            load_settings,
            save_settings,
            // Shortcuts
            update_shortcuts,
            // Audio devices
            audio::list_input_devices,
            audio::list_output_devices,
            // Model management
            models::list_downloaded_files,
            models::download_file,
            models::delete_file,
            models::is_piper_installed,
            models::is_whisper_installed,
            models::is_sherpa_tts_installed,
            models::is_kokoro_installed,
            models::download_piper,
            models::download_whisper,
            models::download_sherpa_tts,
            models::download_kokoro_package,
            // App
            exit_app,
        ])
        .setup(|app| {
            // Load settings into the in-memory cache (used by shortcut handlers)
            let cached = settings::load_settings_internal(app.handle())
                .unwrap_or_default();
            app.manage(SettingsCache(Mutex::new(cached)));

            // STT / TTS runtime state
            app.manage(SttState::new());
            app.manage(TtsState::new());

            tray::setup(app)?;
            overlay::setup(app)?;

            // Auto-show settings on first run
            let config_dir = app.path().app_config_dir()?;
            if !config_dir.join("settings.json").exists() {
                windows::open_settings_window(app.handle())?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
