use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

/// Opens the settings window. If it is already open, brings it to the front.
pub fn toggle_settings<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        // On Linux, overlay show/hide cycles can leave the window in a hidden
        // state (the handle still exists in Tauri's registry but the window is
        // withdrawn). show() must come before set_focus() to make it visible.
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        if let Err(e) = open_settings_window(app) {
            eprintln!("[vocyl] failed to open settings window: {e}");
        }
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
