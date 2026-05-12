use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

/// Opens the settings window. If it is already open, focuses it instead.
pub fn toggle_settings<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_focus();
    } else {
        let _ = open_settings_window(app);
    }
}

/// Creates the settings window. Caller is responsible for not calling this
/// when the window already exists (use `toggle_settings` instead).
pub fn open_settings_window<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("Vocyl")
        .inner_size(800.0, 600.0)
        .center()
        .resizable(false)
        .maximizable(false)
        .skip_taskbar(true)
        .build()?;
    let _ = window.set_focus();
    Ok(())
}
