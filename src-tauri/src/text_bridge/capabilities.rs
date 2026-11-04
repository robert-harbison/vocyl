use serde::Serialize;

/// Describes what the active text bridge backend can do on this system.
/// Exposed to the frontend via the `get_text_bridge_capabilities` Tauri command
/// so the UI can explain degraded support (e.g. "Accessibility permission required").
#[derive(Debug, Clone, Serialize)]
pub struct TextBridgeCapabilities {
    /// Selected text can be read without touching the clipboard.
    pub can_read_selection_native: bool,
    /// Text can be inserted at cursor without clipboard mutation.
    pub can_insert_native: bool,
    /// Selected text can be replaced without clipboard mutation.
    pub can_replace_selection_native: bool,
    /// Some operations fall back to clipboard read/write + key simulation.
    pub uses_clipboard_fallback: bool,
    /// The OS requires an explicit permission grant before native APIs work.
    /// Currently true on macOS when Accessibility access has not been granted.
    pub requires_accessibility_permission: bool,
    /// The session runs on Wayland where global selection access is restricted
    /// by the compositor security model. Support is best-effort.
    pub wayland_limited: bool,
    /// Human-readable name of the active backend, e.g. "windows-uia", "macos-ax",
    /// "linux-x11", "linux-wayland".
    pub backend_name: &'static str,
}
