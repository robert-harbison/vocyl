//! Wayland text bridge — best-effort PRIMARY selection, clipboard fallback.
//!
//! # Limitations (Wayland security model)
//!
//! Wayland's security model intentionally prevents global input capture and
//! global clipboard/selection access without compositor cooperation:
//!
//! - **PRIMARY selection**: Only available if the compositor implements the
//!   `zwp-primary-selection-unstable-v1` protocol (GNOME, KDE) *and* the
//!   requesting process is the "active" client. Not universally supported.
//!
//! - **Global key simulation**: `enigo` falls back to `libei` or `uinput`
//!   on Wayland. These require either compositor support or elevated privileges.
//!   Ctrl+C/Ctrl+V simulation may not work in all applications.
//!
//! - **Result**: `wayland_limited = true` in capabilities. The UI should
//!   communicate that Wayland support is best-effort and some features may
//!   not function in all applications or compositor configurations.
//!
//! # Fall-back chain for `get_selected_text`
//! 1. arboard `LinuxClipboardKind::Primary` (works on compositors with
//!    `zwp-primary-selection-unstable-v1`).
//! 2. `wl-paste --primary --no-newline` (requires `wl-clipboard` package;
//!    optional, clearly logged when used).
//! 3. `xsel` / `xclip` (may work on XWayland sessions).
//! 4. Clipboard + Ctrl+C fallback (last resort, may not work on native Wayland
//!    depending on compositor and focused application).

use arboard::{GetExtLinux, LinuxClipboardKind};
use enigo::{Direction, Enigo, Key, Keyboard};

use crate::text_bridge::{
    TextBridge, TextBridgeCapabilities, TextBridgeError,
    clipboard_fallback::{FallbackConfig, get_selection_via_copy, insert_via_paste},
};

// ── PRIMARY selection (best-effort) ──────────────────────────────────────────

fn try_get_primary_selection() -> Option<String> {
    try_primary_via_arboard()
        .or_else(try_primary_via_subprocess)
}

fn try_primary_via_arboard() -> Option<String> {
    let mut clipboard = arboard::Clipboard::new().ok()?;
    match clipboard.get().clipboard(LinuxClipboardKind::Primary).text() {
        Ok(text) if !text.is_empty() => {
            log::debug!("Wayland PRIMARY via arboard: {} chars", text.len());
            Some(text)
        }
        Ok(_) => None,
        Err(e) => {
            log::debug!("Wayland: arboard PRIMARY unavailable ({e})");
            None
        }
    }
}

fn try_primary_via_subprocess() -> Option<String> {
    // wl-paste is the native Wayland clipboard tool; try it first.
    run_cmd("wl-paste", &["--primary", "--no-newline"])
        // xsel/xclip may work on XWayland.
        .or_else(|| run_cmd("xsel", &["--output", "--primary"]))
        .or_else(|| run_cmd("xclip", &["-out", "-selection", "primary"]))
}

fn run_cmd(cmd: &str, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new(cmd)
        .args(args)
        .output()
        .ok()?;
    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout).into_owned();
        if !text.is_empty() {
            log::debug!("{cmd}: Wayland PRIMARY {} chars", text.len());
            return Some(text);
        }
    }
    None
}

// ── Key simulation helpers ────────────────────────────────────────────────────

fn wayland_send_paste(enigo: &mut Enigo) -> Result<(), TextBridgeError> {
    enigo.key(Key::Control, Direction::Press).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    enigo.key(Key::Control, Direction::Release).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    Ok(())
}

fn wayland_send_copy(enigo: &mut Enigo) -> Result<(), TextBridgeError> {
    enigo.key(Key::Control, Direction::Press).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    enigo.key(Key::Unicode('c'), Direction::Click).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    enigo.key(Key::Control, Direction::Release).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    Ok(())
}

fn wayland_release_modifiers(enigo: &mut Enigo) {
    enigo.key(Key::Shift, Direction::Release).ok();
    enigo.key(Key::Alt, Direction::Release).ok();
}

fn wayland_fallback_config() -> FallbackConfig {
    FallbackConfig {
        send_paste: wayland_send_paste,
        send_copy: wayland_send_copy,
        release_modifiers: wayland_release_modifiers,
        pre_paste_ms: 100,
        post_paste_ms: 250,
        post_restore_ms: 50,
    }
}

// ── TextBridge implementation ─────────────────────────────────────────────────

pub struct WaylandBridge;

unsafe impl Send for WaylandBridge {}
unsafe impl Sync for WaylandBridge {}

impl TextBridge for WaylandBridge {
    fn capabilities(&self) -> TextBridgeCapabilities {
        TextBridgeCapabilities {
            // PRIMARY selection may work on compositors with zwp-primary-selection,
            // but it is not universally available and is compositor-dependent.
            can_read_selection_native: false,
            can_insert_native: false,
            can_replace_selection_native: false,
            uses_clipboard_fallback: true,
            requires_accessibility_permission: false,
            wayland_limited: true,
            backend_name: "linux-wayland",
        }
    }

    fn get_selected_text(&self) -> Result<Option<String>, TextBridgeError> {
        if let Some(text) = try_get_primary_selection() {
            log::debug!("Wayland: PRIMARY selection {} chars", text.len());
            return Ok(Some(text));
        }
        log::debug!(
            "Wayland: PRIMARY selection unavailable (compositor may not support \
             zwp-primary-selection-unstable-v1); falling back to Ctrl+C clipboard copy"
        );
        get_selection_via_copy(&wayland_fallback_config())
    }

    fn insert_text(&self, text: &str) -> Result<(), TextBridgeError> {
        insert_via_paste(text, &wayland_fallback_config())
    }

    fn replace_selection(&self, text: &str) -> Result<(), TextBridgeError> {
        insert_via_paste(text, &wayland_fallback_config())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text_bridge::TextBridge;

    #[test]
    fn capabilities_report_wayland_limited() {
        let bridge = WaylandBridge;
        let caps = bridge.capabilities();
        assert_eq!(caps.backend_name, "linux-wayland");
        assert!(caps.wayland_limited);
        // Native read is best-effort on Wayland; capability is false by design.
        assert!(!caps.can_read_selection_native);
    }
}
