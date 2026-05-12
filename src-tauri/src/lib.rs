mod constants;
mod overlay;
mod settings;
mod tray;
mod windows;
mod app;

use tauri::Manager;

use app::exit_app;
use settings::{load_settings, save_settings, settings_exist};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            settings_exist,
            load_settings,
            save_settings,
            exit_app,
        ])
        .setup(|app| {
            tray::setup(app)?;
            overlay::setup(app)?;

            // Auto-show settings on first run (no config file yet).
            let config_dir = app.path().app_config_dir()?;
            if !config_dir.join("settings.json").exists() {
                windows::open_settings_window(app.handle())?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
