use std::fmt;

// Variants are used across platform-specific modules compiled only on their
// target OS; suppress the dead_code lint that fires on other targets.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum TextBridgeError {
    /// A native OS API was available but returned an error.
    NativeApiUnavailable { reason: String },
    /// Clipboard read/write failed.
    ClipboardError(String),
    /// Keyboard simulation (enigo) failed.
    InputSimulationError(String),
    /// The OS requires an accessibility permission that has not been granted.
    /// On macOS: System Settings → Privacy & Security → Accessibility.
    AccessibilityPermissionRequired,
    /// The operation is not supported on this platform/configuration.
    NotSupported,
}

impl fmt::Display for TextBridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NativeApiUnavailable { reason } => {
                write!(f, "Native API unavailable: {reason}")
            }
            Self::ClipboardError(e) => write!(f, "Clipboard error: {e}"),
            Self::InputSimulationError(e) => write!(f, "Input simulation error: {e}"),
            Self::AccessibilityPermissionRequired => write!(
                f,
                "Accessibility permission required \
                 (macOS: System Settings → Privacy & Security → Accessibility)"
            ),
            Self::NotSupported => write!(f, "Operation not supported on this platform"),
        }
    }
}

impl std::error::Error for TextBridgeError {}
