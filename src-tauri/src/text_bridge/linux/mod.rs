//! Linux text bridge - runtime X11/Wayland detection.
//!
//! The display server is detected at runtime (not compile time) by inspecting
//! environment variables:
//! - `WAYLAND_DISPLAY` set → Wayland session (even if `DISPLAY` is also set,
//!   which is common on XWayland).
//! - `DISPLAY` set, `WAYLAND_DISPLAY` absent → X11 session.
//! - Neither set → unknown; falls back to Wayland bridge (more conservative).

pub mod wayland;
pub mod x11;

use crate::text_bridge::{TextBridge, TextBridgeCapabilities, TextBridgeError};

// Re-export the concrete bridge types so the factory can be called from tests.
pub use wayland::WaylandBridge;
pub use x11::X11Bridge;

// ── Display server detection ──────────────────────────────────────────────────

enum DisplayServer {
    X11,
    Wayland,
    Unknown,
}

fn detect_display_server() -> DisplayServer {
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        return DisplayServer::Wayland;
    }
    if std::env::var("DISPLAY").is_ok() {
        return DisplayServer::X11;
    }
    DisplayServer::Unknown
}

// ── Factory ───────────────────────────────────────────────────────────────────

/// Create the appropriate Linux bridge for the current display server session.
pub fn new_linux_bridge() -> Box<dyn TextBridge + Send + Sync> {
    match detect_display_server() {
        DisplayServer::X11 => {
            log::info!("text_bridge: Linux X11 session detected");
            Box::new(X11Bridge)
        }
        DisplayServer::Wayland => {
            log::info!("text_bridge: Linux Wayland session detected (support is best-effort)");
            Box::new(WaylandBridge)
        }
        DisplayServer::Unknown => {
            log::warn!(
                "text_bridge: no DISPLAY or WAYLAND_DISPLAY - using Wayland bridge as default"
            );
            Box::new(WaylandBridge)
        }
    }
}

// ── Dispatcher (keeps TextBridge impl in one place) ───────────────────────────

/// A Linux bridge that dispatches to the concrete X11 or Wayland bridge
/// selected at construction time. Stored in the global `OnceLock`.
pub struct LinuxBridge(Box<dyn TextBridge + Send + Sync>);

impl LinuxBridge {
    pub fn new() -> Self {
        Self(new_linux_bridge())
    }
}

impl TextBridge for LinuxBridge {
    fn capabilities(&self) -> TextBridgeCapabilities {
        self.0.capabilities()
    }
    fn get_selected_text(&self) -> Result<Option<String>, TextBridgeError> {
        self.0.get_selected_text()
    }
    fn insert_text(&self, text: &str) -> Result<(), TextBridgeError> {
        self.0.insert_text(text)
    }
    fn replace_selection(&self, text: &str) -> Result<(), TextBridgeError> {
        self.0.replace_selection(text)
    }
}

unsafe impl Send for LinuxBridge {}
unsafe impl Sync for LinuxBridge {}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn x11_bridge_capabilities_sane() {
        let caps = X11Bridge.capabilities();
        assert_eq!(caps.backend_name, "linux-x11");
        assert!(!caps.wayland_limited);
    }

    #[test]
    fn wayland_bridge_capabilities_sane() {
        let caps = WaylandBridge.capabilities();
        assert_eq!(caps.backend_name, "linux-wayland");
        assert!(caps.wayland_limited);
    }
}
