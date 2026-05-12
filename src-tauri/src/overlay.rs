use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::constants::{OVERLAY_H, OVERLAY_W, OVERLAY_Y, PTT_SHORTCUT};

/// Creates the transparent overlay pill and registers the push-to-talk shortcut.
pub fn setup<R: tauri::Runtime>(app: &mut tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let overlay = WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App("overlay.html".into()))
        .title("")
        .inner_size(OVERLAY_W, OVERLAY_H)
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .resizable(false)
        .focused(false)
        .build()?;

    // Pass all mouse events through so the user can still interact with
    // whatever is behind the overlay.
    overlay.set_ignore_cursor_events(true)?;

    // Centre the overlay horizontally near the top of the primary monitor.
    if let Ok(Some(monitor)) = overlay.primary_monitor() {
        let scale = monitor.scale_factor();
        let screen_w = monitor.size().width as f64 / scale;
        overlay.set_position(tauri::LogicalPosition::new(
            (screen_w - OVERLAY_W) / 2.0,
            OVERLAY_Y,
        ))?;
    }

    // Register the push-to-talk shortcut.
    let handle = app.handle().clone();
    app.global_shortcut()
        .on_shortcut(PTT_SHORTCUT, move |_app, _shortcut, event| {
            if let Some(overlay) = handle.get_webview_window("overlay") {
                match event.state {
                    ShortcutState::Pressed => {
                        let _ = overlay.show();
                        // Do NOT focus the overlay - the user is still
                        // typing/speaking into whatever was active.
                    }
                    ShortcutState::Released => {
                        let _ = overlay.hide();
                    }
                }
            }
        })?;

    Ok(())
}
