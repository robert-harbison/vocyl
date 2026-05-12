use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

// The push-to-talk shortcut. Change this to whatever key you prefer.
// Valid modifiers: Ctrl, Shift, Alt, Super/Meta, CmdOrCtrl
const PTT_SHORTCUT: &str = "CmdOrCtrl+Shift+Space";

/// Width/height of the floating overlay pill (logical pixels).
const OVERLAY_W: f64 = 280.0;
const OVERLAY_H: f64 = 64.0;
/// Distance from the top of the screen (logical pixels).
const OVERLAY_Y: f64 = 24.0;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // ── Tray context menu ────────────────────────────────────────────
            let settings_item =
                MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit Vocyl", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_item, &quit_item])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Vocyl")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "settings" => toggle_settings(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // Left-click toggles the settings window
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_settings(tray.app_handle());
                    }
                })
                .build(app)?;

            // ── Overlay window ───────────────────────────────────────────────
            // Created programmatically so we can position it on the primary monitor.
            let overlay = WebviewWindowBuilder::new(
                app,
                "overlay",
                WebviewUrl::App("overlay.html".into()),
            )
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

            // Pass all mouse events through the overlay so the user can still
            // interact with whatever is behind it.
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

            // ── Global push-to-talk shortcut ─────────────────────────────────
            let handle = app.handle().clone();
            app.global_shortcut()
                .on_shortcut(PTT_SHORTCUT, move |_app, _shortcut, event| {
                    if let Some(overlay) = handle.get_webview_window("overlay") {
                        match event.state {
                            ShortcutState::Pressed => {
                                let _ = overlay.show();
                                // Do NOT focus the overlay — the user is still
                                // typing/speaking into whatever was active.
                            }
                            ShortcutState::Released => {
                                let _ = overlay.hide();
                            }
                        }
                    }
                })?;

            Ok(())
        })
        // Closing the settings window hides it instead of quitting the app.
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn toggle_settings(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

