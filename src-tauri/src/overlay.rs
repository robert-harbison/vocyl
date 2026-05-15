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
    log::info!("Setting up overlay window");
    let builder = WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App("overlay.html".into()))
        .title("")
        .inner_size(OVERLAY_W, OVERLAY_H)
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .resizable(false);
    // focused(false) triggers a tao 0.35 panic on Linux: when the overlay is
    // first shown via show(), a focus event fires before the GDK window is
    // realized, causing unwrap() on None in tao's event_loop.rs:457. Same
    // root cause as the set_ignore_cursor_events guard below.
    #[cfg(not(target_os = "linux"))]
    let builder = builder.focused(false);
    let overlay = builder.build()?;

    #[cfg(not(target_os = "linux"))]
    if let Err(e) = overlay.set_ignore_cursor_events(true) {
        log::warn!("Click-through overlay unavailable: {}", e);
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

    log::info!(
        "Registering shortcuts — PTT='{}', TTS='{}'",
        ptt_shortcut,
        tts_shortcut
    );
    if let Err(e) = register_shortcuts(app.handle(), &ptt_shortcut, &tts_shortcut) {
        log::warn!("Global shortcuts unavailable (Wayland?): {}", e);
    }

    log::info!("Overlay setup complete");
    Ok(())
}

// ── Shortcut registration ─────────────────────────────────────────────────────

pub fn register_shortcuts<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    ptt_shortcut: &str,
    tts_shortcut: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!(
        "Registering global shortcuts — PTT='{}', TTS='{}'",
        ptt_shortcut,
        tts_shortcut
    );
    app.global_shortcut().unregister_all()?;

    app.global_shortcut()
        .on_shortcut(ptt_shortcut, |app, _shortcut, event| {
            handle_ptt(app, event.state);
        })?;

    app.global_shortcut()
        .on_shortcut(tts_shortcut, |app, _shortcut, event| {
            if let ShortcutState::Pressed = event.state {
                handle_tts(app);
            }
        })?;

    log::info!("Global shortcuts registered successfully");
    Ok(())
}

#[tauri::command]
pub fn update_shortcuts<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    ptt: String,
    tts: String,
) -> Result<(), String> {
    log::info!("Updating shortcuts — PTT='{}', TTS='{}'", ptt, tts);
    register_shortcuts(&app, &ptt, &tts).map_err(|e| e.to_string())
}

// ── PTT handler ───────────────────────────────────────────────────────────────

fn handle_ptt<R: tauri::Runtime>(app: &tauri::AppHandle<R>, state: ShortcutState) {
    match state {
        ShortcutState::Pressed => {
            if app.state::<SttState>().is_recording.load(std::sync::atomic::Ordering::SeqCst) {
                log::debug!("PTT: already recording, ignoring press");
                return;
            }

            log::info!("PTT: key pressed — starting push-to-talk");
            if let Some(overlay) = app.get_webview_window("overlay") {
                let _ = overlay.show();
                emit_overlay_state(app, "waiting");
            }

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
            log::info!("PTT: key released — transcribing");
            let handle = app.clone();
            tauri::async_runtime::spawn(async move {
                emit_overlay_state(&handle, "processing");

                match stop_and_transcribe(&handle).await {
                    Ok(transcript) if !transcript.is_empty() => {
                        log::info!("PTT: transcript received, injecting text");

                        // Hide the overlay BEFORE injecting so the OS returns focus
                        // to the user's previous application. If we skip this the
                        // Ctrl+V / Cmd+V keystroke lands in the overlay window.
                        if let Some(overlay) = handle.get_webview_window("overlay") {
                            let _ = overlay.hide();
                        }

                        // Give the window manager enough time to actually shift focus.
                        // Linux in particular needs a generous margin here.
                        tokio::time::sleep(Duration::from_millis(200)).await;

                        if let Err(e) = inject_text(&transcript) {
                            log::error!("PTT: text injection failed: {}", e);
                        }
                        return; // overlay already hidden above
                    }
                    Ok(_) => {
                        log::info!("PTT: empty transcript, nothing to inject");
                    }
                    Err(e) => {
                        log::error!("PTT: transcription failed: {}", e);
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

    if tts.is_playing.load(std::sync::atomic::Ordering::SeqCst) {
        log::info!("TTS: shortcut pressed while playing — stopping");
        stop_speaking(app);
        return;
    }

    log::info!("TTS: shortcut pressed — capturing selected text");

    // `get_selected_text` blocks for ~350 ms (modifier-release pause + clipboard
    // settle wait).  Running it on the global-shortcut callback thread would stall
    // the entire shortcut subsystem, so we hand it off to a blocking thread.
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let text_result = tokio::task::spawn_blocking(get_selected_text).await;

        let text = match text_result {
            Ok(Ok(t)) if !t.trim().is_empty() => {
                log::info!("TTS: got selected text ({} chars)", t.trim().len());
                t.trim().to_string()
            }
            Ok(Ok(_)) => {
                log::info!("TTS: no text selected");
                if let Some(overlay) = app_handle.get_webview_window("overlay") {
                    let _ = overlay.show();
                }
                emit_overlay_state(&app_handle, "no-text");

                let handle = app_handle.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(1200)).await;
                    if let Some(overlay) = handle.get_webview_window("overlay") {
                        let _ = overlay.hide();
                    }
                });

                return;
            }
            Ok(Err(e)) => {
                log::error!("TTS: failed to get selected text: {}", e);
                return;
            }
            Err(e) => {
                log::error!("TTS: get_selected_text task panicked: {}", e);
                return;
            }
        };

        if let Some(overlay) = app_handle.get_webview_window("overlay") {
            let _ = overlay.show();
        }
        emit_overlay_state(&app_handle, "reading");

        start_speaking(&app_handle, text);
    });
}
