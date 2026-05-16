//! Platform-abstracted text bridge for reading selected text and injecting text
//! at the OS cursor.
//!
//! # Public API
//! ```text
//! let bridge = text_bridge::get_bridge();
//! let selected = bridge.get_selected_text()?;  // Option<String>
//! bridge.insert_text("hello")?;
//! bridge.replace_selection("replacement")?;
//! let caps = text_bridge::get_text_bridge_capabilities();
//! ```
//!
//! # Platform paths
//!
//! | Platform | Selection reading | Insertion |
//! |----------|-------------------|-----------|
//! | Windows  | UI Automation TextPattern → clipboard fallback | Clipboard paste |
//! | macOS    | AX `AXSelectedText` → clipboard fallback | Clipboard paste |
//! | Linux X11 | PRIMARY selection → clipboard fallback | Clipboard paste |
//! | Linux Wayland | PRIMARY (best-effort) → clipboard fallback | Clipboard paste |
//!
//! Clipboard fallback is isolated in `clipboard_fallback.rs` and contains all
//! clipboard mutation + keyboard simulation logic. Platform modules call into
//! it only when native APIs are unavailable or fail.

pub mod capabilities;
pub mod clipboard_fallback;
pub mod error;

pub use capabilities::TextBridgeCapabilities;
pub use error::TextBridgeError;

// Platform-specific modules - #[cfg] only at module declarations, not inside
// business logic.
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;

use std::sync::OnceLock;

// ── Trait ─────────────────────────────────────────────────────────────────────

/// Platform abstraction for reading selected text and inserting text at the
/// OS cursor. Implementations are expected to be stateless unit structs;
/// native API objects are created fresh per call.
pub trait TextBridge: Send + Sync {
    /// Describe what this backend can do on the current system.
    fn capabilities(&self) -> TextBridgeCapabilities;

    /// Read whatever text the user currently has selected.
    ///
    /// Returns `Ok(None)` when no text is selected (the selection is empty or
    /// the OS reports no selection). Returns `Err` only when the underlying
    /// API fails in a way that prevents reading entirely.
    fn get_selected_text(&self) -> Result<Option<String>, TextBridgeError>;

    /// Insert `text` at the current cursor position (or replace the selection
    /// if any text is selected).
    fn insert_text(&self, text: &str) -> Result<(), TextBridgeError>;

        /// Replace the current selection with `text`. Falls back to `insert_text`
    /// on platforms where there is no distinct replace-selection primitive
    /// (clipboard paste naturally replaces a selection in most apps).
    #[allow(dead_code)]
    fn replace_selection(&self, text: &str) -> Result<(), TextBridgeError>;
}

// ── Global bridge instance ────────────────────────────────────────────────────

static BRIDGE: OnceLock<Box<dyn TextBridge + Send + Sync>> = OnceLock::new();

/// Return a reference to the global platform bridge.
///
/// The bridge is created once on first call. The concrete type depends on the
/// target OS and, on Linux, on the runtime display server environment.
pub fn get_bridge() -> &'static dyn TextBridge {
    BRIDGE
        .get_or_init(|| create_bridge())
        .as_ref()
}

fn create_bridge() -> Box<dyn TextBridge + Send + Sync> {
    #[cfg(target_os = "windows")]
    {
        log::info!("text_bridge: initialising Windows UI Automation bridge");
        return Box::new(windows::WindowsTextBridge);
    }

    #[cfg(target_os = "macos")]
    {
        log::info!("text_bridge: initialising macOS Accessibility bridge");
        return Box::new(macos::MacosTextBridge);
    }

    #[cfg(target_os = "linux")]
    {
        log::info!("text_bridge: initialising Linux bridge (runtime display server detection)");
        return Box::new(linux::LinuxBridge::new());
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        log::warn!("text_bridge: unsupported platform - no native bridge available");
        return Box::new(UnsupportedBridge);
    }
}

// ── Convenience wrappers ──────────────────────────────────────────────────────

/// Read selected text. Returns `Ok(None)` when nothing is selected.
pub fn get_selected_text() -> Result<Option<String>, TextBridgeError> {
    get_bridge().get_selected_text()
}

/// Insert `text` at the cursor / replace current selection.
pub fn insert_text(text: &str) -> Result<(), TextBridgeError> {
    get_bridge().insert_text(text)
}

/// Replace the current selection with `text`.
#[allow(dead_code)]
pub fn replace_selection(text: &str) -> Result<(), TextBridgeError> {
    get_bridge().replace_selection(text)
}

/// Return capability information for the active bridge (used by the UI to
/// explain degraded support).
pub fn get_text_bridge_capabilities() -> TextBridgeCapabilities {
    get_bridge().capabilities()
}

// ── Fallback for unsupported platforms ───────────────────────────────────────

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
struct UnsupportedBridge;

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
impl TextBridge for UnsupportedBridge {
    fn capabilities(&self) -> TextBridgeCapabilities {
        TextBridgeCapabilities {
            can_read_selection_native: false,
            can_insert_native: false,
            can_replace_selection_native: false,
            uses_clipboard_fallback: false,
            requires_accessibility_permission: false,
            wayland_limited: false,
            backend_name: "unsupported",
        }
    }
    fn get_selected_text(&self) -> Result<Option<String>, TextBridgeError> {
        Err(TextBridgeError::NotSupported)
    }
    fn insert_text(&self, _text: &str) -> Result<(), TextBridgeError> {
        Err(TextBridgeError::NotSupported)
    }
    fn replace_selection(&self, _text: &str) -> Result<(), TextBridgeError> {
        Err(TextBridgeError::NotSupported)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Mock bridge for unit testing callers of TextBridge.
    pub struct MockTextBridge {
        pub selection: Option<String>,
        pub insert_result: Result<(), TextBridgeError>,
    }

    impl MockTextBridge {
        pub fn with_selection(text: impl Into<String>) -> Self {
            Self {
                selection: Some(text.into()),
                insert_result: Ok(()),
            }
        }

        pub fn empty() -> Self {
            Self {
                selection: None,
                insert_result: Ok(()),
            }
        }

        pub fn failing(err: TextBridgeError) -> Self {
            Self {
                selection: None,
                insert_result: Err(err),
            }
        }
    }

    impl TextBridge for MockTextBridge {
        fn capabilities(&self) -> TextBridgeCapabilities {
            TextBridgeCapabilities {
                can_read_selection_native: true,
                can_insert_native: true,
                can_replace_selection_native: true,
                uses_clipboard_fallback: false,
                requires_accessibility_permission: false,
                wayland_limited: false,
                backend_name: "mock",
            }
        }

        fn get_selected_text(&self) -> Result<Option<String>, TextBridgeError> {
            Ok(self.selection.clone())
        }

        fn insert_text(&self, _text: &str) -> Result<(), TextBridgeError> {
            self.insert_result.clone()
        }

        fn replace_selection(&self, text: &str) -> Result<(), TextBridgeError> {
            self.insert_text(text)
        }
    }

    // Tests for MockTextBridge itself (validates that the trait contract works)

    #[test]
    fn mock_returns_selection() {
        let bridge = MockTextBridge::with_selection("hello world");
        let result = bridge.get_selected_text().unwrap();
        assert_eq!(result.as_deref(), Some("hello world"));
    }

    #[test]
    fn mock_returns_none_when_empty() {
        let bridge = MockTextBridge::empty();
        assert!(bridge.get_selected_text().unwrap().is_none());
    }

    #[test]
    fn mock_propagates_insert_error() {
        let bridge =
            MockTextBridge::failing(TextBridgeError::ClipboardError("test error".into()));
        let result = bridge.insert_text("some text");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("test error"));
    }

    #[test]
    fn mock_capabilities_are_sane() {
        let bridge = MockTextBridge::empty();
        let caps = bridge.capabilities();
        assert_eq!(caps.backend_name, "mock");
        assert!(!caps.wayland_limited);
    }

    #[test]
    fn text_bridge_error_is_display() {
        let err = TextBridgeError::NotSupported;
        assert!(!err.to_string().is_empty());

        let err = TextBridgeError::AccessibilityPermissionRequired;
        assert!(err.to_string().contains("Accessibility"));
    }
}
