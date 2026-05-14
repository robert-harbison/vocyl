use std::time::Duration;

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::{
    constants::{OVERLAY_H, OVERLAY_W, OVERLAY_Y},
    settings::SettingsCache,
    stt::{emit_overlay_state, inject_text, start_recording, stop_and_transcribe, SttState},
    tts::{get_selected_text, start_speaking, stop_speaking, TtsState},
};

// ── Setup ─────────────────────────────────────────────────────────────────────

pub fn setup<R: tauri::Runtime>(
    app: &mut tauri::App<R>,
) -> Result<(), Box<dyn std::error::Error>> {
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

    // tao 0.35 panics inside the event loop on Linux when set_ignore_cursor_events
    // is called before the GDK window is realized (window.window() returns None and
    // tao unwraps it unconditionally). The Err return never reaches our code.
    // Click-through is best-effort on Linux anyway — skip it there entirely.
    #[cfg(not(target_os = "linux"))]
    if let Err(e) = overlay.set_ignore_cursor_events(true) {
        eprintln!("[vocyl] click-through overlay unavailable: {e}");
    }

    if let Ok(Some(monitor)) = overlay.primary_monitor() {
        let scale = monitor.scale_factor();
        let screen_w = monitor.size().width as f64 / scale;
        overlay.set_position(tauri::LogicalPosition::new(
            (screen_w - OVERLAY_W) / 2.0,
            OVERLAY_Y,
        ))?;
    }

    // Load shortcuts from saved settings (fall back to defaults)
    let (ptt_shortcut, tts_shortcut) = {
        let cache = app.state::<SettingsCache>();
        let s = cache.0.lock().unwrap();
        (s.ptt_shortcut.clone(), s.tts_shortcut.clone())
    };

    // Global shortcuts require X11 or Windows/macOS. On Wayland they are
    // silently unavailable — log the failure instead of aborting startup.
    if let Err(e) = register_shortcuts(app.handle(), &ptt_shortcut, &tts_shortcut) {
        eprintln!("[vocyl] global shortcuts unavailable (Wayland?): {e}");
    }

    Ok(())
}

// ── Shortcut registration ─────────────────────────────────────────────────────

pub fn register_shortcuts<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    ptt_shortcut: &str,
    tts_shortcut: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Remove all existing shortcuts first
    app.global_shortcut().unregister_all()?;

    // Push-to-talk
    app.global_shortcut()
        .on_shortcut(ptt_shortcut, |app, _shortcut, event| {
            handle_ptt(app, event.state);
        })?;

    // Text-to-speech toggle
    app.global_shortcut()
        .on_shortcut(tts_shortcut, |app, _shortcut, event| {
            if let ShortcutState::Pressed = event.state {
                handle_tts(app);
            }
        })?;

    Ok(())
}

#[tauri::command]
pub fn update_shortcuts<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    ptt: String,
    tts: String,
) -> Result<(), String> {
    register_shortcuts(&app, &ptt, &tts).map_err(|e| e.to_string())
}

// ── PTT handler ───────────────────────────────────────────────────────────────

fn handle_ptt<R: tauri::Runtime>(app: &tauri::AppHandle<R>, state: ShortcutState) {
    match state {
        ShortcutState::Pressed => {
            // Ignore if already recording
            if app.state::<SttState>().is_recording.load(std::sync::atomic::Ordering::SeqCst) {
                return;
            }

            // Show overlay in "waiting" state
            if let Some(overlay) = app.get_webview_window("overlay") {
                let _ = overlay.show();
                emit_overlay_state(app, "waiting");
            }

            // Get preferred input device from settings
            let device_name = app
                .state::<SettingsCache>()
                .0
                .lock()
                .unwrap()
                .input_device_id
                .clone();

            start_recording(app, device_name);
        }

        ShortcutState::Released => {
            let handle = app.clone();
            tauri::async_runtime::spawn(async move {
                emit_overlay_state(&handle, "processing");

                match stop_and_transcribe(&handle).await {
                    Ok(transcript) if !transcript.is_empty() => {
                        // Small pause so focus returns to the target window
                        tokio::time::sleep(Duration::from_millis(50)).await;

                        if let Err(e) = inject_text(&transcript) {
                            eprintln!("[vocyl] text injection failed: {e}");
                        }
                    }
                    Ok(_) => {} // empty transcript
                    Err(e) => {
                        eprintln!("[vocyl] transcription failed: {e}");
                        emit_overlay_state(&handle, "error");
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                }

                if let Some(overlay) = handle.get_webview_window("overlay") {
                    let _ = overlay.hide();
                }
            });
        }
    }
}

// ── TTS handler ───────────────────────────────────────────────────────────────

fn handle_tts<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let tts = app.state::<TtsState>();

    // Second press while playing → stop
    if tts.is_playing.load(std::sync::atomic::Ordering::SeqCst) {
        stop_speaking(app);
        return;
    }

    // Get selected text (this blocks briefly for the clipboard round-trip)
    let text = match get_selected_text() {
        Ok(t) if !t.trim().is_empty() => t.trim().to_string(),
        _ => {
            // Briefly flash "no text selected" in the overlay
            if let Some(overlay) = app.get_webview_window("overlay") {
                let _ = overlay.show();
            }
            emit_overlay_state(app, "no-text");

            let handle = app.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(Duration::from_millis(1200)).await;
                if let Some(overlay) = handle.get_webview_window("overlay") {
                    let _ = overlay.hide();
                }
            });

            return;
        }
    };

    // Show overlay in "reading" state
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.show();
    }
    emit_overlay_state(app, "reading");

    start_speaking(app, text);
}
