//! X11 text bridge — PRIMARY selection primary, clipboard fallback.
//!
//! On X11 any text the user highlights is automatically placed in the PRIMARY
//! selection. Reading PRIMARY requires no Ctrl+C and avoids the arboard
//! CLIPBOARD background-thread ownership race entirely.
//!
//! Fall-back chain for `get_selected_text`:
//! 1. arboard `LinuxClipboardKind::Primary` (native, no subprocess).
//! 2. `xsel --output --primary` (common X11 tool).
//! 3. `xclip -out -selection primary` (alternative X11 tool).
//! 4. `wl-paste --primary --no-newline` (XWayland sessions where the
//!    compositor supports zwp-primary-selection).
//! 5. Clipboard + Ctrl+C fallback (last resort).

use arboard::{GetExtLinux, LinuxClipboardKind};
use enigo::{Direction, Enigo, Key, Keyboard};

use crate::text_bridge::{
    TextBridge, TextBridgeCapabilities, TextBridgeError,
    clipboard_fallback::{FallbackConfig, get_selection_via_copy, insert_via_paste},
};

// ── PRIMARY selection ─────────────────────────────────────────────────────────

pub(super) fn try_get_primary_selection() -> Option<String> {
    try_primary_via_arboard().or_else(try_primary_via_subprocess)
}

fn try_primary_via_arboard() -> Option<String> {
    let mut clipboard = arboard::Clipboard::new().ok()?;
    match clipboard.get().clipboard(LinuxClipboardKind::Primary).text() {
        Ok(text) if !text.is_empty() => {
            log::debug!("X11 PRIMARY via arboard: {} chars", text.len());
            Some(text)
        }
        Ok(_) => None,
        Err(e) => {
            log::debug!("arboard PRIMARY unavailable: {e}");
            None
        }
    }
}

fn try_primary_via_subprocess() -> Option<String> {
    // xsel is most common on pure X11 sessions; try it before wl-paste.
    run_cmd("xsel", &["--output", "--primary"])
        .or_else(|| run_cmd("xclip", &["-out", "-selection", "primary"]))
        // wl-paste works on XWayland sessions with zwp-primary-selection.
        .or_else(|| run_cmd("wl-paste", &["--primary", "--no-newline"]))
}

fn run_cmd(cmd: &str, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new(cmd)
        .args(args)
        .output()
        .ok()?;
    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout).into_owned();
        if !text.is_empty() {
            log::debug!("{cmd}: PRIMARY {} chars", text.len());
            return Some(text);
        }
    }
    None
}

// ── Key simulation helpers ────────────────────────────────────────────────────

fn x11_send_paste(enigo: &mut Enigo) -> Result<(), TextBridgeError> {
    enigo.key(Key::Control, Direction::Press).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    enigo.key(Key::Control, Direction::Release).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    Ok(())
}

fn x11_send_copy(enigo: &mut Enigo) -> Result<(), TextBridgeError> {
    enigo.key(Key::Control, Direction::Press).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    enigo.key(Key::Unicode('c'), Direction::Click).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    enigo.key(Key::Control, Direction::Release).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    Ok(())
}

fn x11_release_modifiers(enigo: &mut Enigo) {
    enigo.key(Key::Shift, Direction::Release).ok();
    enigo.key(Key::Alt, Direction::Release).ok();
}

fn x11_fallback_config() -> FallbackConfig {
    FallbackConfig {
        send_paste: x11_send_paste,
        send_copy: x11_send_copy,
        release_modifiers: x11_release_modifiers,
        // arboard needs its background thread alive before SelectionRequest
        // events arrive (pre-paste). Post-paste keeps ownership while the
        // target reads. Post-restore lets arboard re-establish before drop.
        pre_paste_ms: 100,
        post_paste_ms: 250,
        post_restore_ms: 50,
    }
}

// ── TextBridge implementation ─────────────────────────────────────────────────

pub struct X11Bridge;

unsafe impl Send for X11Bridge {}
unsafe impl Sync for X11Bridge {}

impl TextBridge for X11Bridge {
    fn capabilities(&self) -> TextBridgeCapabilities {
        TextBridgeCapabilities {
            can_read_selection_native: true, // PRIMARY selection, no Ctrl+C needed
            can_insert_native: false,
            can_replace_selection_native: false,
            uses_clipboard_fallback: true,
            requires_accessibility_permission: false,
            wayland_limited: false,
            backend_name: "linux-x11",
        }
    }

    fn get_selected_text(&self) -> Result<Option<String>, TextBridgeError> {
        if let Some(text) = try_get_primary_selection() {
            log::debug!("X11: PRIMARY selection {} chars", text.len());
            return Ok(Some(text));
        }
        log::debug!("X11: PRIMARY empty, falling back to Ctrl+C clipboard copy");
        get_selection_via_copy(&x11_fallback_config())
    }

    fn insert_text(&self, text: &str) -> Result<(), TextBridgeError> {
        insert_via_paste(text, &x11_fallback_config())
    }

    fn replace_selection(&self, text: &str) -> Result<(), TextBridgeError> {
        insert_via_paste(text, &x11_fallback_config())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text_bridge::TextBridge;

    #[test]
    fn capabilities_are_correct() {
        let bridge = X11Bridge;
        let caps = bridge.capabilities();
        assert_eq!(caps.backend_name, "linux-x11");
        assert!(caps.can_read_selection_native);
        assert!(!caps.wayland_limited);
        assert!(!caps.requires_accessibility_permission);
    }
}
