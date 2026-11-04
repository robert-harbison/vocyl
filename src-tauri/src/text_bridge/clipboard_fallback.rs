//! Clipboard-based fallback for text selection reading and insertion.
//!
//! This module is the only place in the codebase that performs clipboard
//! mutation combined with keyboard simulation. Platform implementations
//! call into here only when native APIs are unavailable or fail.
//!
//! # Key design decisions
//!
//! **Fresh arboard instances everywhere**: arboard on Linux X11 spawns a
//! background thread that owns the CLIPBOARD X11 selection while a
//! `Clipboard` value is alive. Dropping the instance before sending Ctrl+C
//! ensures that thread surrenders ownership so the focused app can claim it.
//! On Windows and macOS the clipboard is message-based (no background thread),
//! so fresh instances are harmless - the behaviour is identical to reusing one.
//!
//! **No correctness sleeps**: The only sleeps here implement real protocol
//! requirements (giving the target app time to consume a paste, giving
//! arboard time to establish ownership). They are not workarounds for races.

use std::time::Duration;

use arboard::Clipboard;
use enigo::{Enigo, Settings};

use crate::text_bridge::TextBridgeError;

// ── Configuration ─────────────────────────────────────────────────────────────

/// Per-platform fallback configuration. Callers supply the key combos and
/// timing that are correct for their platform; this module contains no
/// platform-specific `#[cfg]` guards.
#[allow(dead_code)]
pub struct FallbackConfig {
    /// Send the paste shortcut (Ctrl+V / Cmd+V) to the focused application.
    pub send_paste: fn(&mut Enigo) -> Result<(), TextBridgeError>,
    /// Send the copy shortcut (Ctrl+C / Cmd+C) to the focused application.
    pub send_copy: fn(&mut Enigo) -> Result<(), TextBridgeError>,
    /// Release any modifier keys that might be held from a prior shortcut.
    pub release_modifiers: fn(&mut Enigo),
    /// Milliseconds to wait after writing to the clipboard before sending paste.
    /// Linux arboard needs this for the background thread to become the X11
    /// CLIPBOARD owner before SelectionRequest events arrive.
    pub pre_paste_ms: u64,
    /// Milliseconds to wait after sending paste, keeping our clipboard alive
    /// so the target app can complete its read.
    pub post_paste_ms: u64,
    /// Milliseconds to wait after restoring the original clipboard so arboard
    /// can re-establish ownership before the instance drops (Linux only).
    pub post_restore_ms: u64,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn new_clipboard() -> Result<Clipboard, TextBridgeError> {
    Clipboard::new().map_err(|e| TextBridgeError::ClipboardError(e.to_string()))
}

fn new_enigo() -> Result<Enigo, TextBridgeError> {
    Enigo::new(&Settings::default())
        .map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))
}

// ── Public operations ─────────────────────────────────────────────────────────

/// Insert `text` at the cursor by placing it on the clipboard and sending the
/// platform paste shortcut. Saves and restores the original clipboard content.
///
/// The clipboard instance is kept alive throughout the paste so arboard's
/// background thread can serve `SelectionRequest` events on Linux X11.
pub fn insert_via_paste(text: &str, config: &FallbackConfig) -> Result<(), TextBridgeError> {
    log::debug!("clipboard_fallback: insert_via_paste ({} chars)", text.len());

    let mut clipboard = new_clipboard()?;
    let saved = clipboard.get_text().ok();
    log::debug!(
        "clipboard_fallback: saved {} clipboard chars",
        saved.as_deref().unwrap_or("").len()
    );

    clipboard
        .set_text(text)
        .map_err(|e| TextBridgeError::ClipboardError(e.to_string()))?;

    // Release any modifier keys that may still be logically held in the focused
    // app (e.g. Alt from a PTT shortcut like Alt+A on Wayland). This must
    // happen before send_paste so Ctrl+V is not seen as Alt+Ctrl+V.
    // The pre_paste_ms sleep that follows gives the app time to process the
    // releases before the paste shortcut arrives.
    let mut enigo = new_enigo()?;
    (config.release_modifiers)(&mut enigo);

    if config.pre_paste_ms > 0 {
        std::thread::sleep(Duration::from_millis(config.pre_paste_ms));
    }

    (config.send_paste)(&mut enigo)?;
    std::thread::sleep(Duration::from_millis(config.post_paste_ms));

    if let Some(ref original) = saved {
        if !original.is_empty() {
            if let Err(e) = clipboard.set_text(original.as_str()) {
                log::warn!("clipboard_fallback: failed to restore clipboard: {e}");
            } else {
                if config.post_restore_ms > 0 {
                    std::thread::sleep(Duration::from_millis(config.post_restore_ms));
                }
                log::debug!("clipboard_fallback: clipboard restored");
            }
        }
    }

    Ok(())
}

/// Read the user's selected text by clearing the clipboard, sending the copy
/// shortcut, and reading the result.
///
/// Returns `None` if nothing was selected (clipboard stayed empty after copy).
/// Returns `Err` only if the clipboard or input simulation itself fails.
///
/// The write-owning clipboard instance is dropped *before* the copy shortcut
/// is sent. On Linux X11 this is required so arboard's background thread
/// relinquishes CLIPBOARD ownership; on Windows/macOS it is a no-op but
/// harmless. The subsequent read uses a fresh instance that has no stale
/// ownership state and queries the actual current clipboard owner.
#[allow(dead_code)]
pub fn get_selection_via_copy(
    config: &FallbackConfig,
) -> Result<Option<String>, TextBridgeError> {
    log::debug!("clipboard_fallback: get_selection_via_copy");

    let saved = {
        let mut cb = new_clipboard()?;
        let saved = cb.get_text().ok();
        cb.set_text("")
            .map_err(|e| TextBridgeError::ClipboardError(e.to_string()))?;
        saved
        // cb drops here - arboard background thread surrenders X11 CLIPBOARD
        // ownership before the copy shortcut fires.
    };

    let mut enigo = new_enigo()?;
    (config.release_modifiers)(&mut enigo);
    std::thread::sleep(Duration::from_millis(50));
    (config.send_copy)(&mut enigo)?;
    std::thread::sleep(Duration::from_millis(200));

    // Fresh instance: no stale background-thread state; queries the actual owner.
    let selected = new_clipboard()?.get_text().unwrap_or_default();
    log::debug!("clipboard_fallback: copy returned {} chars", selected.len());

    if let Some(old) = saved {
        if !old.is_empty() {
            if let Ok(mut cb) = new_clipboard() {
                cb.set_text(old).ok();
            }
        }
    }

    Ok(if selected.is_empty() { None } else { Some(selected) })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn noop_paste(_: &mut Enigo) -> Result<(), TextBridgeError> {
        Ok(())
    }
    fn noop_copy(_: &mut Enigo) -> Result<(), TextBridgeError> {
        Ok(())
    }
    fn noop_release(_: &mut Enigo) {}

    fn failing_paste(_: &mut Enigo) -> Result<(), TextBridgeError> {
        Err(TextBridgeError::InputSimulationError("simulated failure".into()))
    }

    fn test_config() -> FallbackConfig {
        FallbackConfig {
            send_paste: noop_paste,
            send_copy: noop_copy,
            release_modifiers: noop_release,
            pre_paste_ms: 0,
            post_paste_ms: 0,
            post_restore_ms: 0,
        }
    }

    #[test]
    fn error_display_is_human_readable() {
        let e = TextBridgeError::ClipboardError("disk full".into());
        assert!(e.to_string().contains("Clipboard error"));
        assert!(e.to_string().contains("disk full"));

        let e = TextBridgeError::NativeApiUnavailable { reason: "COM not initialized".into() };
        assert!(e.to_string().contains("Native API unavailable"));

        let e = TextBridgeError::AccessibilityPermissionRequired;
        assert!(e.to_string().contains("Accessibility permission"));
    }

    #[test]
    fn error_is_clone() {
        let e = TextBridgeError::InputSimulationError("x".into());
        let _ = e.clone();
    }

    #[test]
    fn fallback_config_send_paste_propagates_error() {
        let cfg = FallbackConfig {
            send_paste: failing_paste,
            ..test_config()
        };
        // We can't easily call insert_via_paste without a real clipboard in CI,
        // but we can verify the fn pointer is callable and returns the expected Err.
        let mut enigo = Enigo::new(&Settings::default()).expect("enigo should init in test");
        let result = (cfg.send_paste)(&mut enigo);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TextBridgeError::InputSimulationError(_)));
    }
}
